//! IRC command handling (/join, /part, /msg, etc.).

use crossbeam_channel::Sender;

use crate::protocol::BackendAction;

/// Handle user commands starting with '/'.
/// Returns true if the input was a command (and should be cleared), false otherwise.
pub fn handle_user_command(
    message_input: &str,
    active_buffer: &str,
    buffers: &std::collections::HashMap<String, crate::buffer::ChannelBuffer>,
    action_tx: &Sender<BackendAction>,
    system_log: &mut Vec<String>,
    nickname_input: &mut String,
) -> bool {
    let s = message_input.trim();
    if !s.starts_with('/') {
        return false;
    }

    // Remove leading '/'
    let cmdline = s[1..].trim();
    let mut parts = cmdline.split_whitespace();
    let cmd = parts.next().unwrap_or("").to_lowercase();
    
    match cmd.as_str() {
        "join" | "j" => {
            if let Some(chan) = parts.next() {
                let channel = if chan.starts_with('#') || chan.starts_with('&') {
                    chan.to_string()
                } else {
                    format!("#{}", chan)
                };
                let _ = action_tx.send(BackendAction::Join(channel));
            } else {
                system_log.push("Usage: /join <channel>".into());
            }
        }
        "part" | "p" => {
            if let Some(chan) = parts.next() {
                let channel = if chan.starts_with('#') || chan.starts_with('&') {
                    chan.to_string()
                } else {
                    format!("#{}", chan)
                };
                let reason = parts.collect::<Vec<_>>().join(" ");
                let _ = action_tx.send(BackendAction::Part {
                    channel,
                    message: if reason.is_empty() {
                        None
                    } else {
                        Some(reason)
                    },
                });
            } else {
                // If no channel was provided, part the active buffer if it's a channel
                if active_buffer.starts_with('#') || active_buffer.starts_with('&') {
                    let channel = active_buffer.to_string();
                    let reason = parts.collect::<Vec<_>>().join(" ");
                    let _ = action_tx.send(BackendAction::Part {
                        channel,
                        message: if reason.is_empty() {
                            None
                        } else {
                            Some(reason)
                        },
                    });
                } else {
                    system_log.push("Usage: /part <channel>".into());
                }
            }
        }
        "msg" | "privmsg" => {
            if let Some(target) = parts.next() {
                let text = parts.collect::<Vec<_>>().join(" ");
                if text.is_empty() {
                    system_log.push("Usage: /msg <target> <message>".into());
                } else {
                    let target = target.to_string();
                    let _ = action_tx.send(BackendAction::SendMessage { target, text });
                }
            } else {
                system_log.push("Usage: /msg <target> <message>".into());
            }
        }
        "me" => {
            let text = parts.collect::<Vec<_>>().join(" ");
            if text.is_empty() {
                system_log.push("Usage: /me <action>".into());
            } else {
                // Use ACTION CTCP encoding
                let action_text = format!("\x01ACTION {}\x01", text);
                // Send to active buffer
                if active_buffer != "System" {
                    let target = active_buffer.to_string();
                    let _ = action_tx.send(BackendAction::SendMessage {
                        target,
                        text: action_text,
                    });
                } else {
                    system_log.push("/me can only be used in a channel or PM".into());
                }
            }
        }
        "whois" | "w" => {
            if let Some(target) = parts.next() {
                let _ = action_tx.send(BackendAction::Whois(target.to_string()));
            } else {
                system_log.push("Usage: /whois <nick>".into());
            }
        }
        "topic" | "t" => {
            // If no argument provided, show current topic for active buffer
            let new_topic = parts.collect::<Vec<_>>().join(" ");
            if active_buffer.starts_with('#') || active_buffer.starts_with('&') {
                if new_topic.is_empty() {
                    if let Some(buffer) = buffers.get(active_buffer) {
                        if buffer.topic.is_empty() {
                            system_log.push(format!("No topic set for {}", active_buffer));
                        } else {
                            system_log.push(format!(
                                "Topic for {}: {}",
                                active_buffer, buffer.topic
                            ));
                        }
                    }
                } else {
                    let _ = action_tx.send(BackendAction::SetTopic {
                        channel: active_buffer.to_string(),
                        topic: new_topic,
                    });
                }
            } else {
                system_log.push("/topic can only be used in a channel".into());
            }
        }
        "kick" | "k" => {
            if let Some(nick) = parts.next() {
                let reason = parts.collect::<Vec<_>>().join(" ");
                if active_buffer.starts_with('#') || active_buffer.starts_with('&') {
                    let _ = action_tx.send(BackendAction::Kick {
                        channel: active_buffer.to_string(),
                        nick: nick.to_string(),
                        reason: if reason.is_empty() {
                            None
                        } else {
                            Some(reason)
                        },
                    });
                } else {
                    system_log.push("/kick can only be used in a channel".into());
                }
            } else {
                system_log.push("Usage: /kick <nick> [reason]".into());
            }
        }
        "nick" => {
            if let Some(newnick) = parts.next() {
                // Update locally and send to server
                *nickname_input = newnick.to_string();
                let _ = action_tx.send(BackendAction::Nick(newnick.to_string()));
            } else {
                system_log.push("Usage: /nick <newnick>".into());
            }
        }
        "quit" | "exit" => {
            let reason = parts.collect::<Vec<_>>().join(" ");
            let _ = action_tx.send(BackendAction::Quit(if reason.is_empty() {
                None
            } else {
                Some(reason)
            }));
        }
        "help" => {
            system_log.push("Supported commands: /join, /part, /msg, /me, /nick, /quit, /whois, /topic, /kick".into());
        }
        unknown => {
            system_log.push(format!("Unknown command: /{}", unknown));
        }
    }
    true
}
