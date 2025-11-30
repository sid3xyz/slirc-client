//! Backend event processing (incoming IRC messages, user lists, topics, etc.).

use chrono::Local;
use crossbeam_channel::Receiver;
use std::collections::{HashMap, HashSet};

use crate::buffer::{ChannelBuffer, MessageType, RenderedMessage};
use crate::logging::Logger;
use crate::protocol::{GuiEvent, UserInfo};

/// Process all pending events from the backend.
pub fn process_events(
    event_rx: &Receiver<GuiEvent>,
    is_connected: &mut bool,
    buffers: &mut HashMap<String, ChannelBuffer>,
    buffers_order: &mut Vec<String>,
    active_buffer: &mut String,
    nickname_input: &mut String,
    system_log: &mut Vec<String>,
    expanded_networks: &mut HashSet<String>,
    status_messages: &mut Vec<(String, std::time::Instant)>,
    server_input: &str,
    font_fallback: &Option<String>,
    logger: &Option<Logger>,
) {
    // Drain all pending events from the backend
    while let Ok(event) = event_rx.try_recv() {
        match event {
            GuiEvent::Connected => {
                *is_connected = true;
                let ts = Local::now().format("%H:%M:%S").to_string();
                system_log.push(format!("[{}] ✓ Connected and registered!", ts));
                // Expand the server in the network list and show a status toast
                expanded_networks.insert(server_input.to_string());
                status_messages.push((
                    format!("Connected to {}", server_input),
                    std::time::Instant::now(),
                ));
            }

            GuiEvent::Disconnected(reason) => {
                *is_connected = false;
                let ts = Local::now().format("%H:%M:%S").to_string();
                system_log.push(format!("[{}] ✗ Disconnected: {}", ts, reason));
                status_messages.push(("Disconnected".into(), std::time::Instant::now()));
            }

            GuiEvent::Error(msg) => {
                let ts = Local::now().format("%H:%M:%S").to_string();
                system_log.push(format!("[{}] ⚠ Error: {}", ts, msg));
                status_messages.push((format!("Error: {}", msg), std::time::Instant::now()));
            }
            
            GuiEvent::NickChanged { old, new } => {
                // Update user lists in all buffers where the old nick existed
                for (buffer_name, buffer) in buffers.iter_mut() {
                    if buffer.users.iter().any(|u| u.nick == old) {
                        for user in buffer.users.iter_mut() {
                            if user.nick == old {
                                user.nick = new.clone();
                            }
                        }
                        let ts = Local::now().format("%H:%M:%S").to_string();
                        let nick_msg = RenderedMessage::new(
                            ts.clone(),
                            "*".into(),
                            format!("{} is now known as {}", old, new),
                        )
                        .with_type(MessageType::NickChange);
                        let is_active = *buffer_name == *active_buffer;
                        buffer.add_message(nick_msg, is_active, false);
                    }
                }
                // Update the UI nickname field when the server acknowledges it
                *nickname_input = new.clone();
                let ts = Local::now().format("%H:%M:%S").to_string();
                system_log.push(format!("[{}] Nick changed to {} (was: {})", ts, new, old));
            }

            GuiEvent::RawMessage(msg) => {
                let ts = Local::now().format("%H:%M:%S").to_string();
                system_log.push(format!("[{}] {}", ts, msg));
                // Keep log from growing too large
                if system_log.len() > 500 {
                    system_log.remove(0);
                }
            }

            GuiEvent::MessageReceived {
                target,
                sender,
                text,
            } => {
                // If it's a PM, the target is the sender (for display)
                let buffer_name = if target.starts_with('#') || target.starts_with('&') {
                    target.clone()
                } else {
                    // Private message - use sender as buffer name
                    sender.clone()
                };

                let ts = Local::now().format("%H:%M:%S").to_string();
                let mention = text.contains(nickname_input.as_str());
                let is_own_msg = sender == *nickname_input;
                let active = active_buffer.clone();
                let buffer = ensure_buffer(buffers, buffers_order, &buffer_name);
                let is_active = active == buffer_name;
                let msg_type = if text.starts_with("\x01ACTION ") && text.ends_with('\x01') {
                    MessageType::Action
                } else if sender.starts_with('-') && sender.ends_with('-') {
                    // Backend marks NOTICE messages with -<sender>- as a sender string
                    MessageType::Notice
                } else {
                    MessageType::Normal
                };
                let msg = RenderedMessage::new(ts.clone(), sender.clone(), text.clone())
                    .with_type(msg_type);
                buffer.add_message(msg, is_active || is_own_msg, mention);
                
                // Log to file (non-blocking)
                if let Some(logger) = logger {
                    logger.log(crate::logging::LogEntry {
                        network: server_input.to_string(),
                        channel: buffer_name.clone(),
                        timestamp: ts.clone(),
                        nick: sender.clone(),
                        message: text.clone(),
                    });
                }
                
                // Keep user list updated if a new nick speaks
                if (buffer_name.starts_with('#') || buffer_name.starts_with('&'))
                    && !buffer.users.iter().any(|u| u.nick == sender)
                {
                    buffer.users.push(UserInfo {
                        nick: sender.clone(),
                        prefix: None,
                    });
                    crate::ui::sort_users(&mut buffer.users[..]);
                }
                // Unread/highlight handled by ChannelBuffer::add_message
            }

            GuiEvent::JoinedChannel(channel) => {
                let ts = Local::now().format("%H:%M:%S").to_string();
                system_log.push(format!("[{}] ✓ Joined {}", ts, channel));
                status_messages.push((format!("Joined {}", channel), std::time::Instant::now()));
                let buffer = ensure_buffer(buffers, buffers_order, &channel);
                buffer.clear_unread();
                buffer.has_highlight = false;
                *active_buffer = channel;
            }

            GuiEvent::PartedChannel(channel) => {
                let ts = Local::now().format("%H:%M:%S").to_string();
                system_log.push(format!("[{}] ← Left {}", ts, channel));
                status_messages.push((format!("Left {}", channel), std::time::Instant::now()));
                buffers.remove(&channel);
                buffers_order.retain(|b| *b != channel);
                if *active_buffer == channel {
                    *active_buffer = "System".into();
                }
            }

            GuiEvent::UserJoined { channel, nick } => {
                let is_active = *active_buffer == channel;
                let buffer = ensure_buffer(buffers, buffers_order, &channel);
                let ts = Local::now().format("%H:%M:%S").to_string();
                let join_msg =
                    RenderedMessage::new(ts.clone(), "→".into(), format!("{} joined", nick))
                        .with_type(MessageType::Join);
                buffer.add_message(join_msg, is_active, false);
                if !buffer.users.iter().any(|u| u.nick == nick) {
                    buffer.users.push(UserInfo {
                        nick: nick.clone(),
                        prefix: None,
                    });
                    crate::ui::sort_users(&mut buffer.users[..]);
                }
                // Unread handled by add_message
            }

            GuiEvent::UserParted {
                channel,
                nick,
                message,
            } => {
                let is_active = *active_buffer == channel;
                let buffer = ensure_buffer(buffers, buffers_order, &channel);
                let msg = message.map(|m| format!(" ({})", m)).unwrap_or_default();
                let ts = Local::now().format("%H:%M:%S").to_string();
                let part_msg = RenderedMessage::new(
                    ts.clone(),
                    "←".into(),
                    format!("{} left{}", nick, msg),
                )
                .with_type(MessageType::Part);
                buffer.add_message(part_msg, is_active, false);
                buffer.users.retain(|u| u.nick != nick);
                // Unread handled by add_message
            }

            GuiEvent::UserQuit { nick, message } => {
                // Remove the user from all channels and add quit message
                let active = active_buffer.clone();
                let msg = message.map(|m| format!(" ({})", m)).unwrap_or_default();
                let ts = Local::now().format("%H:%M:%S").to_string();

                for (channel_name, buffer) in buffers.iter_mut() {
                    if buffer.users.iter().any(|u| u.nick == nick) {
                        let quit_msg = RenderedMessage::new(
                            ts.clone(),
                            "⇐".into(),
                            format!("{} quit{}", nick, msg),
                        )
                        .with_type(MessageType::Quit);
                        let is_active = active == *channel_name;
                        buffer.add_message(quit_msg, is_active, false);
                        buffer.users.retain(|u| u.nick != nick);
                    }
                }
            }

            GuiEvent::Motd(line) => {
                let ts = Local::now().format("%H:%M:%S").to_string();
                // Clean up MOTD line formatting a bit for readability
                let cleaned = clean_motd_line(&line, font_fallback);
                if cleaned.is_empty() {
                    system_log.push(format!("[{}] MOTD:", ts));
                } else {
                    system_log.push(format!("[{}] MOTD: {}", ts, cleaned));
                }
            }

            GuiEvent::Topic { channel, topic } => {
                let active = active_buffer.clone();
                let buffer = ensure_buffer(buffers, buffers_order, &channel);
                buffer.topic = topic.clone();
                let ts = Local::now().format("%H:%M:%S").to_string();
                let topic_msg =
                    RenderedMessage::new(ts.clone(), "*".into(), format!("Topic: {}", topic))
                        .with_type(MessageType::Topic);
                buffer.add_message(topic_msg, active == channel, false);
                // Unread handled by add_message
            }

            GuiEvent::Names { channel, names } => {
                let buffer = ensure_buffer(buffers, buffers_order, &channel);
                buffer.users = names;
                crate::ui::sort_users(&mut buffer.users[..]);
            }
            
            GuiEvent::UserMode {
                channel,
                nick,
                prefix,
                added,
            } => {
                let buffer = ensure_buffer(buffers, buffers_order, &channel);
                // Find the user and update the prefix; if the user isn't present,
                // add them (some servers may send MODE before a NAMES refresh).
                if let Some(user) = buffer.users.iter_mut().find(|u| u.nick == nick) {
                    if added {
                        user.prefix = prefix;
                    } else if user.prefix == prefix {
                        user.prefix = None;
                    }
                } else if added {
                    buffer.users.push(UserInfo {
                        nick: nick.clone(),
                        prefix,
                    });
                }
                crate::ui::sort_users(&mut buffer.users[..]);
            }
        }
    }
}

/// Ensure a buffer exists for the given channel/PM name.
fn ensure_buffer<'a>(
    buffers: &'a mut HashMap<String, ChannelBuffer>,
    buffers_order: &mut Vec<String>,
    name: &str,
) -> &'a mut ChannelBuffer {
    if !buffers.contains_key(name) {
        buffers.insert(name.to_string(), ChannelBuffer::new());
        // keep insertion order
        if !buffers_order.contains(&name.to_string()) {
            buffers_order.push(name.to_string());
        }
    }
    // Safe unwrap: we just ensured the key exists above
    buffers.get_mut(name).expect("Buffer should exist after insertion")
}

/// Clean MOTD line formatting.
pub fn clean_motd_line(line: &str, font_fallback: &Option<String>) -> String {
    let mut s = line.trim_start();
    if let Some(rest) = s.strip_prefix(":- ") {
        s = rest.trim_start();
    } else if let Some(rest) = s.strip_prefix(":-") {
        s = rest.trim_start();
    } else if let Some(rest) = s.strip_prefix("- ") {
        s = rest.trim_start();
    } else if s == "-" {
        s = "";
    }
    let mut s2 = s.to_string();
    if font_fallback.is_none() {
        s2 = s2
            .replace(['═', '─'], "-")
            .replace(['│', '║'], "|")
            .replace(['┌', '┐', '└', '┘'], "+");
    }
    s2
}
