use crossbeam_channel::{Receiver, Sender};
use slirc_proto::mode::{ChannelMode, Mode};
use slirc_proto::{Command, Message, Prefix, Transport};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::time::timeout;

use crate::protocol::{BackendAction, GuiEvent, UserInfo};

pub fn run_backend(action_rx: Receiver<BackendAction>, event_tx: Sender<GuiEvent>) {
    // Create a Tokio runtime for this thread
    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    rt.block_on(async move {
        let mut transport: Option<Transport> = None;
        let mut current_nick = String::new();

        loop {
            // Check for actions from the UI (non-blocking)
            while let Ok(action) = action_rx.try_recv() {
                match action {
                    BackendAction::Connect {
                        server,
                        port,
                        nickname,
                        username,
                        realname,
                    } => {
                        current_nick = nickname.clone();

                        // Try to connect
                        let addr = format!("{}:{}", server, port);
                        let _ = event_tx
                            .send(GuiEvent::RawMessage(format!("Connecting to {}...", addr)));

                        match TcpStream::connect(&addr).await {
                            Ok(stream) => {
                                let mut t = match Transport::tcp(stream) {
                                    Ok(t) => t,
                                    Err(e) => {
                                        let _ = event_tx.send(GuiEvent::Error(format!(
                                            "Failed to create transport: {}",
                                            e
                                        )));
                                        continue;
                                    }
                                };

                                // Send NICK
                                let nick_msg = Message::nick(&nickname);
                                if let Err(e) = t.write_message(&nick_msg).await {
                                    let _ = event_tx.send(GuiEvent::Error(format!(
                                        "Failed to send NICK: {}",
                                        e
                                    )));
                                    continue;
                                }
                                // let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", nick_msg)));

                                // Send USER
                                let user_msg = Message::user(&username, &realname);
                                if let Err(e) = t.write_message(&user_msg).await {
                                    let _ = event_tx.send(GuiEvent::Error(format!(
                                        "Failed to send USER: {}",
                                        e
                                    )));
                                    continue;
                                }
                                // let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", user_msg)));

                                transport = Some(t);
                            }
                            Err(e) => {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Connection failed: {}", e)));
                            }
                        }
                    }

                    BackendAction::Disconnect => {
                        if let Some(ref mut t) = transport {
                            let quit_msg = Message::quit_with_message("Leaving");
                            let _ = t.write_message(&quit_msg).await;
                        }
                        transport = None;
                        let _ = event_tx.send(GuiEvent::Disconnected("User disconnected".into()));
                    }

                    BackendAction::Join(channel) => {
                        if let Some(ref mut t) = transport {
                            let join_msg = Message::join(&channel);
                            // let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", join_msg)));
                            if let Err(e) = t.write_message(&join_msg).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to join: {}", e)));
                            }
                        }
                    }

                    BackendAction::Part { channel, message } => {
                        if let Some(ref mut t) = transport {
                            let part_msg = if let Some(msg) = message {
                                Message::part_with_message(&channel, &msg)
                            } else {
                                Message::part(&channel)
                            };
                            // let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", part_msg)));
                            if let Err(e) = t.write_message(&part_msg).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to part: {}", e)));
                            }
                        }
                    }
                    BackendAction::Nick(newnick) => {
                        if let Some(ref mut t) = transport {
                            let nick_msg = Message::nick(&newnick);
                            // let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", nick_msg)));
                            if let Err(e) = t.write_message(&nick_msg).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to send NICK: {}", e)));
                            } else {
                                let old_nick = current_nick.clone();
                                current_nick = newnick.clone();
                                let _ = event_tx.send(GuiEvent::NickChanged {
                                    old: old_nick,
                                    new: newnick.clone(),
                                });
                            }
                        } else {
                            let _ = event_tx.send(GuiEvent::Error("Not connected".into()));
                        }
                    }
                    BackendAction::Whois(target) => {
                        if let Some(ref mut t) = transport {
                            let whois = Message::from(slirc_proto::command::Command::WHOIS(
                                None,
                                target.clone(),
                            ));
                            if let Err(e) = t.write_message(&whois).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to send WHOIS: {}", e)));
                            }
                        } else {
                            let _ = event_tx.send(GuiEvent::Error("Not connected".into()));
                        }
                    }
                    BackendAction::SetTopic { channel, topic } => {
                        if let Some(ref mut t) = transport {
                            let topic_cmd = Message::from(slirc_proto::command::Command::TOPIC(
                                channel.clone(),
                                Some(topic.clone()),
                            ));
                            if let Err(e) = t.write_message(&topic_cmd).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to set topic: {}", e)));
                            }
                        }
                    }
                    BackendAction::Kick {
                        channel,
                        nick,
                        reason,
                    } => {
                        if let Some(ref mut t) = transport {
                            let kick_msg = if let Some(r) = reason {
                                Message::kick_with_reason(&channel, &nick, &r)
                            } else {
                                Message::kick(&channel, &nick)
                            };
                            if let Err(e) = t.write_message(&kick_msg).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to kick: {}", e)));
                            }
                        }
                    }
                    BackendAction::SetUserMode {
                        channel,
                        nick,
                        mode,
                    } => {
                        if let Some(ref mut t) = transport {
                            // Create a raw MODE command with args: MODE <channel> <mode> <nick>
                            if let Ok(mode_msg) =
                                Message::new(None, "MODE", vec![&channel, &mode, &nick])
                            {
                                if let Err(e) = t.write_message(&mode_msg).await {
                                    let _ = event_tx.send(GuiEvent::Error(format!(
                                        "Failed to set user mode: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }
                    BackendAction::Quit(reason) => {
                        if let Some(ref mut t) = transport {
                            let quit_msg = if let Some(r) = reason {
                                Message::quit_with_message(&r)
                            } else {
                                Message::quit()
                            };
                            // let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", quit_msg)));
                            let _ = t.write_message(&quit_msg).await;
                        }
                        transport = None;
                        let _ = event_tx.send(GuiEvent::Disconnected("User quit".into()));
                    }

                    BackendAction::SendMessage { target, text } => {
                        if let Some(ref mut t) = transport {
                            let privmsg = Message::privmsg(&target, &text);
                            // let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", privmsg)));
                            if let Err(e) = t.write_message(&privmsg).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to send: {}", e)));
                            } else {
                                // Echo our own message to the UI
                                let _ = event_tx.send(GuiEvent::MessageReceived {
                                    target: target.clone(),
                                    sender: current_nick.clone(),
                                    text,
                                });
                            }
                        }
                    }
                }
            }

            // Read from the network (with short timeout so we can check for actions)
            if let Some(ref mut t) = transport {
                match timeout(Duration::from_millis(50), t.read_message()).await {
                    Ok(Ok(Some(message))) => {
                        // We do NOT send RawMessage for everything anymore to keep the log clean.
                        // Only send it if we don't handle the message or if it's an error.

                        match &message.command {
                            // PING -> PONG
                            Command::PING(server, _) => {
                                let pong = Message::pong(server);
                                let _ = t.write_message(&pong).await;
                                // let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", pong)));
                            }

                            // RPL_WELCOME (001) - Registration complete
                            Command::Response(code, _) if code.code() == 1 => {
                                let _ = event_tx.send(GuiEvent::Connected);
                            }

                            // RPL_TOPIC (332)
                            Command::Response(code, args) if code.code() == 332 => {
                                if args.len() >= 3 {
                                    let _ = event_tx.send(GuiEvent::Topic {
                                        channel: args[1].clone(),
                                        topic: args[2].clone(),
                                    });
                                }
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
                            }

                            // RPL_MOTD (372) and RPL_MOTDSTART (375)
                            Command::Response(code, args)
                                if code.code() == 372 || code.code() == 375 =>
                            {
                                if let Some(text) = args.last() {
                                    let _ = event_tx.send(GuiEvent::Motd(text.clone()));
                                }
                            }

                            // PRIVMSG
                            Command::PRIVMSG(target, text) => {
                                let sender =
                                    message.source_nickname().unwrap_or("unknown").to_string();
                                let _ = event_tx.send(GuiEvent::MessageReceived {
                                    target: target.clone(),
                                    sender,
                                    text: text.clone(),
                                });
                            }

                            // NOTICE
                            Command::NOTICE(target, text) => {
                                let sender =
                                    message.source_nickname().unwrap_or("server").to_string();
                                let _ = event_tx.send(GuiEvent::MessageReceived {
                                    target: target.clone(),
                                    sender: format!("-{}-", sender),
                                    text: text.clone(),
                                });
                            }

                            // JOIN
                            Command::JOIN(channel, _, _) => {
                                let nick = message.source_nickname().unwrap_or("").to_string();
                                if nick == current_nick {
                                    let _ = event_tx.send(GuiEvent::JoinedChannel(channel.clone()));
                                } else {
                                    let _ = event_tx.send(GuiEvent::UserJoined {
                                        channel: channel.clone(),
                                        nick,
                                    });
                                }
                            }

                            // PART
                            Command::PART(channel, msg) => {
                                let nick = message.source_nickname().unwrap_or("").to_string();
                                if nick == current_nick {
                                    let _ = event_tx.send(GuiEvent::PartedChannel(channel.clone()));
                                } else {
                                    let _ = event_tx.send(GuiEvent::UserParted {
                                        channel: channel.clone(),
                                        nick,
                                        message: msg.clone(),
                                    });
                                }
                            }

                            // NICK change (someone changed their nick)
                            Command::NICK(newnick) => {
                                let oldnick = message.source_nickname().unwrap_or("").to_string();
                                // Update internal state if it was our nick
                                if oldnick == current_nick {
                                    current_nick = newnick.clone();
                                }
                                let _ = event_tx.send(GuiEvent::NickChanged {
                                    old: oldnick.clone(),
                                    new: newnick.clone(),
                                });
                            }

                            // QUIT - user left the server
                            Command::QUIT(msg) => {
                                if let Some(Prefix::Nickname(nick, _, _)) = &message.prefix {
                                    let _ = event_tx.send(GuiEvent::UserQuit {
                                        nick: nick.to_string(),
                                        message: msg.clone(),
                                    });
                                }
                            }

                            // ERROR from server
                            Command::ERROR(msg) => {
                                let _ = event_tx.send(GuiEvent::Error(msg.clone()));
                            }

                            // Channel mode changes (e.g. +o/-o): update UI user prefixes
                            Command::ChannelMODE(channel, modes) => {
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
                                        _ => {}
                                    }
                                }
                            }

                            // For other messages, we might want to log them if they are interesting
                            _ => {
                                // Optional: Log unhandled messages to system log for debugging
                                // let _ = event_tx.send(GuiEvent::RawMessage(format!("← {}", message)));
                            }
                        }
                    }
                    Ok(Ok(None)) => {
                        // Connection closed
                        transport = None;
                        let _ = event_tx
                            .send(GuiEvent::Disconnected("Connection closed by server".into()));
                    }
                    Ok(Err(e)) => {
                        let _ = event_tx.send(GuiEvent::Error(format!("Read error: {:?}", e)));
                        transport = None;
                        let _ = event_tx.send(GuiEvent::Disconnected("Read error".into()));
                    }
                    Err(_) => {
                        // Timeout - this is normal, just loop
                    }
                }
            } else {
                // No connection, sleep a bit to avoid busy-looping
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    });
}
