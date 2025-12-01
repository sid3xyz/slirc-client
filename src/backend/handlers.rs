//! IRC message routing and event generation
//!
//! This module handles the conversion of incoming IRC messages to GUI events.
//! It's extracted from the main backend loop to improve maintainability.

use crossbeam_channel::Sender;
use slirc_proto::mode::{ChannelMode, Mode};
use slirc_proto::{Command, Message, Prefix};

use crate::protocol::{GuiEvent, UserInfo};

/// Route an IRC message to appropriate GUI event handlers
///
/// This function processes incoming IRC messages and generates corresponding
/// GUI events. It handles all standard IRC commands (PRIVMSG, JOIN, PART, etc.)
/// and numeric replies.
///
/// # Arguments
/// * `msg` - The IRC message to process
/// * `current_nick` - The client's current nickname (for self-detection)
/// * `event_tx` - Channel sender for dispatching GUI events
///
/// # Returns
/// `Some(new_nick)` if the message was a NICK change affecting us, otherwise `None`
pub fn route_message(
    msg: &Message,
    current_nick: &str,
    event_tx: &Sender<GuiEvent>,
) -> Option<String> {
    match &msg.command {
        // RPL_ISUPPORT (005) - Server capabilities
        Command::Response(code, args) if code.code() == 5 => {
            // Parse ISUPPORT tokens using slirc-proto
            let params: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let isupport = slirc_proto::Isupport::parse_params(&params);

            // Extract useful info and send to UI
            let _ = event_tx.send(GuiEvent::ServerInfo {
                network: isupport.network().map(|s| s.to_string()),
                casemapping: isupport.casemapping().map(|s| s.to_string()),
            });
            None
        }

        // RPL_TOPIC (332)
        Command::Response(code, args) if code.code() == 332 => {
            if args.len() >= 3 {
                let _ = event_tx.send(GuiEvent::Topic {
                    channel: args[1].clone(),
                    topic: args[2].clone(),
                });
            }
            None
        }

        // RPL_NAMREPLY (353)
        Command::Response(code, args) if code.code() == 353 => {
            if args.len() >= 4 {
                let channel = args[2].clone();
                let mut names: Vec<UserInfo> = Vec::new();
                for s in args[3].split_whitespace() {
                    let mut chars = s.chars();
                    let prefix = chars
                        .next()
                        .filter(|c| matches!(c, '@' | '+' | '%' | '&' | '~'));
                    let nick = if prefix.is_some() {
                        chars.as_str().to_string()
                    } else {
                        s.to_string()
                    };
                    names.push(UserInfo { nick, prefix });
                }
                let _ = event_tx.send(GuiEvent::Names { channel, names });
            }
            None
        }

        // RPL_LIST (322) - channel list item
        Command::Response(code, args) if code.code() == 322 => {
            if args.len() >= 4 {
                let channel = args[1].clone();
                let user_count = args[2].parse::<usize>().unwrap_or(0);
                let topic = args[3].clone();
                let _ = event_tx.send(GuiEvent::ChannelListItem {
                    channel,
                    user_count,
                    topic,
                });
            }
            None
        }

        // RPL_LISTEND (323) - end of channel list
        Command::Response(code, _) if code.code() == 323 => {
            let _ = event_tx.send(GuiEvent::ChannelListEnd);
            None
        }

        // RPL_MOTD (372) and RPL_MOTDSTART (375)
        Command::Response(code, args) if code.code() == 372 || code.code() == 375 => {
            if let Some(text) = args.last() {
                let _ = event_tx.send(GuiEvent::Motd(text.clone()));
            }
            None
        }

        // PRIVMSG
        Command::PRIVMSG(target, text) => {
            let sender = msg.source_nickname().unwrap_or("unknown").to_string();
            let _ = event_tx.send(GuiEvent::MessageReceived {
                target: target.clone(),
                sender,
                text: text.clone(),
            });
            None
        }

        // NOTICE
        Command::NOTICE(target, text) => {
            let sender = msg.source_nickname().unwrap_or("server").to_string();
            let _ = event_tx.send(GuiEvent::MessageReceived {
                target: target.clone(),
                sender: format!("-{}-", sender),
                text: text.clone(),
            });
            None
        }

        // JOIN
        Command::JOIN(channel, _, _) => {
            let nick = msg.source_nickname().unwrap_or("").to_string();
            if nick == current_nick {
                let _ = event_tx.send(GuiEvent::JoinedChannel(channel.clone()));
            } else {
                let _ = event_tx.send(GuiEvent::UserJoined {
                    channel: channel.clone(),
                    nick,
                });
            }
            None
        }

        // PART
        Command::PART(channel, message) => {
            let nick = msg.source_nickname().unwrap_or("").to_string();
            if nick == current_nick {
                let _ = event_tx.send(GuiEvent::PartedChannel(channel.clone()));
            } else {
                let _ = event_tx.send(GuiEvent::UserParted {
                    channel: channel.clone(),
                    nick,
                    message: message.clone(),
                });
            }
            None
        }

        // NICK change (someone changed their nick)
        Command::NICK(newnick) => {
            let oldnick = msg.source_nickname().unwrap_or("").to_string();
            let _ = event_tx.send(GuiEvent::NickChanged {
                old: oldnick.clone(),
                new: newnick.clone(),
            });

            // Return new nick if it affects us (caller will update current_nick)
            if oldnick == current_nick {
                Some(newnick.clone())
            } else {
                None
            }
        }

        // QUIT - user left the server
        Command::QUIT(message) => {
            if let Some(Prefix::Nickname(nick, _, _)) = &msg.prefix {
                let _ = event_tx.send(GuiEvent::UserQuit {
                    nick: nick.to_string(),
                    message: message.clone(),
                });
            }
            None
        }

        // ERROR from server
        Command::ERROR(message) => {
            let _ = event_tx.send(GuiEvent::Error(message.clone()));
            None
        }

        // Channel mode changes (e.g. +o/-o): update UI user prefixes and channel modes
        Command::ChannelMODE(channel, modes) => {
            let set_by = msg.source_nickname().unwrap_or("server").to_string();
            let mut channel_mode_changes = String::new();

            for m in modes {
                match m {
                    Mode::Plus(ChannelMode::Oper, Some(nick)) => {
                        let _ = event_tx.send(GuiEvent::UserMode {
                            channel: channel.clone(),
                            nick: nick.clone(),
                            prefix: Some('@'),
                            added: true,
                        });
                    }
                    Mode::Minus(ChannelMode::Oper, Some(nick)) => {
                        let _ = event_tx.send(GuiEvent::UserMode {
                            channel: channel.clone(),
                            nick: nick.clone(),
                            prefix: Some('@'),
                            added: false,
                        });
                    }
                    Mode::Plus(ChannelMode::Voice, Some(nick)) => {
                        let _ = event_tx.send(GuiEvent::UserMode {
                            channel: channel.clone(),
                            nick: nick.clone(),
                            prefix: Some('+'),
                            added: true,
                        });
                    }
                    Mode::Minus(ChannelMode::Voice, Some(nick)) => {
                        let _ = event_tx.send(GuiEvent::UserMode {
                            channel: channel.clone(),
                            nick: nick.clone(),
                            prefix: Some('+'),
                            added: false,
                        });
                    }
                    Mode::Plus(ChannelMode::Halfop, Some(nick)) => {
                        let _ = event_tx.send(GuiEvent::UserMode {
                            channel: channel.clone(),
                            nick: nick.clone(),
                            prefix: Some('%'),
                            added: true,
                        });
                    }
                    Mode::Minus(ChannelMode::Halfop, Some(nick)) => {
                        let _ = event_tx.send(GuiEvent::UserMode {
                            channel: channel.clone(),
                            nick: nick.clone(),
                            prefix: Some('%'),
                            added: false,
                        });
                    }
                    Mode::Plus(ChannelMode::Admin, Some(nick)) => {
                        let _ = event_tx.send(GuiEvent::UserMode {
                            channel: channel.clone(),
                            nick: nick.clone(),
                            prefix: Some('&'),
                            added: true,
                        });
                    }
                    Mode::Minus(ChannelMode::Admin, Some(nick)) => {
                        let _ = event_tx.send(GuiEvent::UserMode {
                            channel: channel.clone(),
                            nick: nick.clone(),
                            prefix: Some('&'),
                            added: false,
                        });
                    }
                    Mode::Plus(ChannelMode::Founder, Some(nick)) => {
                        let _ = event_tx.send(GuiEvent::UserMode {
                            channel: channel.clone(),
                            nick: nick.clone(),
                            prefix: Some('~'),
                            added: true,
                        });
                    }
                    Mode::Minus(ChannelMode::Founder, Some(nick)) => {
                        let _ = event_tx.send(GuiEvent::UserMode {
                            channel: channel.clone(),
                            nick: nick.clone(),
                            prefix: Some('~'),
                            added: false,
                        });
                    }
                    // Channel-level modes (no nick arg) - track for topic bar
                    Mode::Plus(mode, None) => {
                        if let Some(c) = channel_mode_char(mode) {
                            channel_mode_changes.push('+');
                            channel_mode_changes.push(c);
                        }
                    }
                    Mode::Minus(mode, None) => {
                        if let Some(c) = channel_mode_char(mode) {
                            channel_mode_changes.push('-');
                            channel_mode_changes.push(c);
                        }
                    }
                    _ => {}
                }
            }

            // Emit channel mode event if any channel-level modes changed
            if !channel_mode_changes.is_empty() {
                let _ = event_tx.send(GuiEvent::ChannelMode {
                    channel: channel.clone(),
                    modes: channel_mode_changes,
                    set_by,
                });
            }

            None
        }

        // For other messages, no specific handling needed
        _ => None,
    }
}

/// Convert a ChannelMode to its single-character representation for display
fn channel_mode_char(mode: &ChannelMode) -> Option<char> {
    match mode {
        ChannelMode::Moderated => Some('m'),
        ChannelMode::ProtectedTopic => Some('t'),
        ChannelMode::NoExternalMessages => Some('n'),
        ChannelMode::Secret => Some('s'),
        ChannelMode::InviteOnly => Some('i'),
        ChannelMode::RegisteredOnly => Some('r'),
        ChannelMode::Key => Some('k'),
        ChannelMode::Limit => Some('l'),
        // User prefix modes and list modes are handled separately
        ChannelMode::Oper | ChannelMode::Voice | ChannelMode::Halfop
        | ChannelMode::Admin | ChannelMode::Founder | ChannelMode::Ban
        | ChannelMode::Exception | ChannelMode::InviteException | ChannelMode::Quiet => None,
        // Unknown modes - try to emit them anyway if they have a char
        ChannelMode::Unknown(c) => Some(*c),
        // Catch-all for any new modes added to the non-exhaustive enum
        _ => None,
    }
}
