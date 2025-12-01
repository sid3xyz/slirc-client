//! SLIRC Client - An IRC client built with egui and slirc-proto
//!
//! Architecture:
//! - Main thread: runs the egui UI
//! - Backend thread: runs a Tokio runtime for async network I/O
//! - Communication via crossbeam channels (lock-free, sync-safe)

mod app;
mod backend;
mod buffer;
mod commands;
mod config;
mod dialog_manager;
mod events;
mod fonts;
mod input_state;
mod logging;
mod protocol;
mod state;
mod ui;
mod validation;

#[cfg(test)]
mod backend_tests;

#[cfg(test)]
mod integration_tests;

use app::SlircApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])  // Modern default size
            .with_min_inner_size([800.0, 600.0]),  // Minimum for 2.5-column layout
        ..Default::default()
    };

    eframe::run_native(
        "SLIRC - IRC Client",
        options,
        Box::new(|cc| {
            // Setup modern fonts (Inter + JetBrains Mono)
            cc.egui_ctx.set_fonts(fonts::setup_fonts());

            // Setup modern text styles
            cc.egui_ctx.all_styles_mut(|style| {
                style.text_styles = ui::theme::configure_text_styles();
            });

            Ok(Box::new(SlircApp::new(cc)))
        }),
    )
}

#[cfg(test)]
mod tests {
    use crate::app::SlircApp;
    use crate::buffer::ChannelBuffer;
    use crate::config::DEFAULT_SERVER;
    use crate::protocol::{BackendAction, GuiEvent, UserInfo};
    use crate::state::ClientState;
    use crossbeam_channel::unbounded;
    use std::collections::HashSet;

    /// Helper to create a test SlircApp instance
    fn create_test_app() -> (
        SlircApp,
        crossbeam_channel::Sender<GuiEvent>,
        crossbeam_channel::Receiver<BackendAction>,
    ) {
        let (action_tx, action_rx) = unbounded::<BackendAction>();
        let (event_tx, event_rx) = unbounded::<GuiEvent>();

        // Create state with pre-populated System buffer
        let mut state = ClientState::new();
        state.logger = None; // No logger in tests

        let app = SlircApp {
            state,
            connection: crate::config::ConnectionConfig {
                server: DEFAULT_SERVER.into(),
                nickname: "tester".into(),
                use_tls: false,
            },
            action_tx,
            event_rx,
            input: crate::input_state::InputState::new(),
            context_menu_visible: false,
            context_menu_target: None,
            open_windows: HashSet::new(),
            theme: String::from("dark"),
            show_channel_list: true,
            show_user_list: true,
            quick_switcher: crate::ui::quick_switcher::QuickSwitcher::default(),
            // Dialogs - managed centrally by DialogManager
            dialogs: crate::dialog_manager::DialogManager::new(),
            // Keyboard shortcuts
            shortcuts: crate::ui::shortcuts::ShortcutRegistry::new(),
            show_shortcuts_help: false,
        };
        (app, event_tx, action_rx)
    }

    #[test]
    fn test_clean_motd() {
        assert_eq!(crate::events::clean_motd_line("-"), "");
        assert_eq!(crate::events::clean_motd_line(":-"), "");
        assert_eq!(crate::events::clean_motd_line(":- "), "");
        assert_eq!(crate::events::clean_motd_line(":- Hello world"), "Hello world");
        assert_eq!(crate::events::clean_motd_line("- ═════════"), "═════════"); // preserved with bundled fonts
        assert_eq!(crate::events::clean_motd_line("Hello"), "Hello");
        assert_eq!(crate::events::clean_motd_line(" - Hello"), "Hello");
    }

    #[test]
    fn test_motd_processed_in_system_log() {
        let (mut app, event_tx, _) = create_test_app();

        let motd_line = String::from(":- Welcome to the server");
        let _ = event_tx.send(GuiEvent::Motd(motd_line));

        app.process_events();
        assert!(app
            .state
            .system_log
            .iter()
            .any(|l| l.contains("MOTD: Welcome to the server")));
    }

    #[test]
    fn test_names_event_populates_users() {
        let (mut app, event_tx, _) = create_test_app();

        let names = vec![
            crate::protocol::UserInfo {
                nick: "admin".into(),
                prefix: Some('@'),
            },
            crate::protocol::UserInfo {
                nick: "foo".into(),
                prefix: None,
            },
            crate::protocol::UserInfo {
                nick: "bar".into(),
                prefix: Some('+'),
            },
        ];
        let _ = event_tx.send(GuiEvent::Names {
            channel: "#test".into(),
            names,
        });
        app.process_events();
        // Buffer should be created and populated
        assert!(app.state.buffers.contains_key("#test"));
        let buf = app.state.buffers.get("#test").unwrap();
        assert_eq!(buf.users.len(), 3);
        assert!(buf
            .users
            .iter()
            .any(|u| u.nick == "admin" && u.prefix == Some('@')));
        assert!(buf
            .users
            .iter()
            .any(|u| u.nick == "bar" && u.prefix == Some('+')));
    }

    #[test]
    fn test_user_mode_event_updates_prefix() {
        use crate::protocol::UserInfo as PUser;
        let (mut app, event_tx, _) = create_test_app();
        // Create a channel buffer with one user
        let mut buf = ChannelBuffer::new();
        buf.users.push(PUser {
            nick: "alice".into(),
            prefix: None,
        });
        app.state.buffers.insert("#test".into(), buf);

        let _ = event_tx.send(GuiEvent::UserMode {
            channel: "#test".into(),
            nick: "alice".into(),
            prefix: Some('@'),
            added: true,
        });
        app.process_events();
        let b = app.state.buffers.get("#test").unwrap();
        assert!(b
            .users
            .iter()
            .any(|u| u.nick == "alice" && u.prefix == Some('@')));

        // Now remove the op
        let _ = event_tx.send(GuiEvent::UserMode {
            channel: "#test".into(),
            nick: "alice".into(),
            prefix: Some('@'),
            added: false,
        });
        app.process_events();
        let b2 = app.state.buffers.get("#test").unwrap();
        assert!(b2
            .users
            .iter()
            .any(|u| u.nick == "alice" && u.prefix.is_none()));
    }

    #[test]
    fn test_topic_event_updates_buffer_topic() {
        let (mut app, event_tx, _) = create_test_app();
        let _ = event_tx.send(GuiEvent::Topic {
            channel: "#test".into(),
            topic: "New Topic".into(),
        });
        app.process_events();
        let b = app.state.buffers.get("#test").unwrap();
        assert_eq!(b.topic, "New Topic");
    }

    #[test]
    fn test_whois_command_sends_action() {
        let (mut app, _, action_rx) = create_test_app();
        app.state.is_connected = true;

        // Set the message input to a whois command and ensure the action is sent
        app.input.message_input = String::from("/whois someuser");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::Whois(nick) => assert_eq!(nick, "someuser"),
            _ => panic!("Expected Whois action"),
        }
    }

    #[test]
    fn test_notice_message_type() {
        use crate::buffer::MessageType;
        let (mut app, event_tx, _) = create_test_app();
        let _ = event_tx.send(GuiEvent::MessageReceived {
            target: "System".into(),
            sender: "-server-".into(),
            text: "This is a notice".into(),
        });
        app.process_events();
        let buf = app.state.buffers.get("-server-").unwrap();
        assert!(!buf.messages.is_empty());
        assert_eq!(buf.messages.last().unwrap().msg_type, MessageType::Notice);
    }

    #[test]
    fn test_status_messages_on_connect() {
        let (mut app, event_tx, _) = create_test_app();
        let _ = event_tx.send(GuiEvent::Connected);
        app.process_events();
        assert!(!app.state.status_messages.is_empty());
        assert!(app
            .state
            .status_messages
            .last()
            .unwrap()
            .0
            .contains("Connected to"));
    }

    #[test]
    fn test_topic_command_set_and_show() {
        let (mut app, _, action_rx) = create_test_app();
        app.state.is_connected = true;
        app.state.active_buffer = "#test".into();
        app.state.buffers.insert("#test".into(), ChannelBuffer::new());

        // Set the message input to a topic change command and ensure the action is sent
        app.input.message_input = String::from("/topic hello world");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::SetTopic { channel, topic } => {
                assert_eq!(channel, "#test");
                assert_eq!(topic, "hello world");
            }
            _ => panic!("Expected SetTopic action"),
        }

        // Now test that /topic with no args displays the topic in system_log
        app.state.buffers.get_mut("#test").unwrap().topic = "Displayed Topic".into();
        app.input.message_input = String::from("/topic");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        assert!(app.state.system_log.iter().any(|l| l.contains("Displayed Topic")));
    }

    #[test]
    fn test_kick_command_sends_action() {
        let (mut app, _, action_rx) = create_test_app();
        app.state.is_connected = true;
        app.state.active_buffer = "#test".into();
        app.state.buffers.insert("#test".into(), ChannelBuffer::new());

        app.input.message_input = String::from("/kick alice Spamming");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::Kick {
                channel,
                nick,
                reason,
            } => {
                assert_eq!(channel, "#test");
                assert_eq!(nick, "alice");
                assert_eq!(reason.unwrap(), "Spamming");
            }
            _ => panic!("Expected Kick action"),
        }
    }

    #[test]
    fn test_me_command_sends_action_ctcp() {
        let (mut app, _, action_rx) = create_test_app();
        app.state.is_connected = true;
        app.state.active_buffer = "#test".into();
        app.state.buffers.insert("#test".into(), ChannelBuffer::new());

        app.input.message_input = String::from("/me does something");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::SendMessage { target, text } => {
                assert_eq!(target, "#test");
                assert_eq!(text, "\x01ACTION does something\x01");
            }
            _ => panic!("Expected SendMessage action with CTCP ACTION"),
        }
    }

    #[test]
    fn test_nick_command_sends_action() {
        let (mut app, _, action_rx) = create_test_app();
        app.state.is_connected = true;
        app.connection.nickname = "oldnick".into();

        app.input.message_input = String::from("/nick newnick");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        assert_eq!(app.connection.nickname, "newnick");
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::Nick(nick) => {
                assert_eq!(nick, "newnick");
            }
            _ => panic!("Expected Nick action"),
        }
    }

    #[test]
    fn test_quit_command_sends_action() {
        let (mut app, _, action_rx) = create_test_app();
        app.state.is_connected = true;

        app.input.message_input = String::from("/quit Goodbye everyone");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::Quit(reason) => {
                assert_eq!(reason, Some("Goodbye everyone".to_string()));
            }
            _ => panic!("Expected Quit action"),
        }
    }

    #[test]
    fn test_quit_command_without_reason() {
        let (mut app, _, action_rx) = create_test_app();
        app.state.is_connected = true;

        app.input.message_input = String::from("/quit");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::Quit(reason) => {
                assert_eq!(reason, None);
            }
            _ => panic!("Expected Quit action"),
        }
    }

    #[test]
    fn test_help_command_shows_usage() {
        let (mut app, _, _) = create_test_app();
        let original_log_size = app.state.system_log.len();

        app.input.message_input = String::from("/help");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        assert!(app.state.system_log.len() > original_log_size);
        assert!(app.state.system_log.last().unwrap().contains("Supported commands"));
    }

    #[test]
    fn test_unknown_command_logs_error() {
        let (mut app, _, _) = create_test_app();
        let original_log_size = app.state.system_log.len();

        app.input.message_input = String::from("/foobar");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        assert!(app.state.system_log.len() > original_log_size);
        assert!(app.state.system_log.last().unwrap().contains("Unknown command"));
        assert!(app.state.system_log.last().unwrap().contains("foobar"));
    }

    #[test]
    fn test_msg_command_without_message() {
        let (mut app, _, _) = create_test_app();
        let original_log_size = app.state.system_log.len();

        app.input.message_input = String::from("/msg alice");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        assert!(app.state.system_log.len() > original_log_size);
        assert!(app.state.system_log.last().unwrap().contains("Usage"));
    }

    #[test]
    fn test_part_without_args_parts_active_channel() {
        let (mut app, _, action_rx) = create_test_app();
        app.state.is_connected = true;
        app.state.active_buffer = "#test".into();
        app.state.buffers.insert("#test".into(), ChannelBuffer::new());

        app.input.message_input = String::from("/part");
        assert!(crate::commands::handle_user_command(
            &app.input.message_input,
            &app.state.active_buffer,
            &app.state.buffers,
            &app.action_tx,
            &mut app.state.system_log,
            &mut app.connection.nickname,
        ));
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::Part { channel, message } => {
                assert_eq!(channel, "#test");
                assert_eq!(message, None);
            }
            _ => panic!("Expected Part action"),
        }
    }

    #[test]
    fn test_user_joined_event() {
        let (mut app, event_tx, _) = create_test_app();
        app.state.is_connected = true;
        app.state.active_buffer = "#test".into();
        app.state.buffers.insert("#test".into(), ChannelBuffer::new());

        event_tx
            .send(GuiEvent::UserJoined {
                channel: "#test".to_string(),
                nick: "alice".to_string(),
            })
            .unwrap();

        app.process_events();

        let buffer = app.state.buffers.get("#test").unwrap();
        assert!(buffer.users.iter().any(|u| u.nick == "alice"));
        assert!(buffer.messages.iter().any(|m| m.text.contains("alice joined")));
    }

    #[test]
    fn test_user_parted_event() {
        let (mut app, event_tx, _) = create_test_app();
        app.state.is_connected = true;
        app.state.active_buffer = "#test".into();
        let mut buffer = ChannelBuffer::new();
        buffer.users.push(UserInfo {
            nick: "alice".to_string(),
            prefix: None,
        });
        app.state.buffers.insert("#test".into(), buffer);

        event_tx
            .send(GuiEvent::UserParted {
                channel: "#test".to_string(),
                nick: "alice".to_string(),
                message: Some("Goodbye".to_string()),
            })
            .unwrap();

        app.process_events();

        let buffer = app.state.buffers.get("#test").unwrap();
        assert!(!buffer.users.iter().any(|u| u.nick == "alice"));
        assert!(buffer.messages.iter().any(|m| m.text.contains("alice left")));
        assert!(buffer.messages.iter().any(|m| m.text.contains("Goodbye")));
    }

    #[test]
    fn test_user_quit_event() {
        let (mut app, event_tx, _) = create_test_app();
        app.state.is_connected = true;
        app.state.active_buffer = "#test".into();
        let mut buffer = ChannelBuffer::new();
        buffer.users.push(UserInfo {
            nick: "bob".to_string(),
            prefix: None,
        });
        app.state.buffers.insert("#test".into(), buffer);

        event_tx
            .send(GuiEvent::UserQuit {
                nick: "bob".to_string(),
                message: Some("Connection reset".to_string()),
            })
            .unwrap();

        app.process_events();

        let buffer = app.state.buffers.get("#test").unwrap();
        assert!(!buffer.users.iter().any(|u| u.nick == "bob"));
        assert!(buffer.messages.iter().any(|m| m.text.contains("bob quit")));
        assert!(buffer.messages.iter().any(|m| m.text.contains("Connection reset")));
    }

    #[test]
    fn test_nick_changed_event() {
        let (mut app, event_tx, _) = create_test_app();
        app.state.is_connected = true;
        app.connection.nickname = "alice".into();
        app.state.active_buffer = "#test".into();
        let mut buffer = ChannelBuffer::new();
        buffer.users.push(UserInfo {
            nick: "alice".to_string(),
            prefix: None,
        });
        app.state.buffers.insert("#test".into(), buffer);

        event_tx
            .send(GuiEvent::NickChanged {
                old: "alice".to_string(),
                new: "alice_away".to_string(),
            })
            .unwrap();

        app.process_events();

        assert_eq!(app.connection.nickname, "alice_away");
        let buffer = app.state.buffers.get("#test").unwrap();
        assert!(buffer.users.iter().any(|u| u.nick == "alice_away"));
        assert!(!buffer.users.iter().any(|u| u.nick == "alice"));
        assert!(buffer.messages.iter().any(|m| m.text.contains("alice is now known as alice_away")));
    }

    #[test]
    fn test_connected_event() {
        let (mut app, event_tx, _) = create_test_app();
        app.state.is_connected = false;
        app.connection.server = "irc.example.com".into();
        // Set state.server_name to simulate connection initiation
        app.state.server_name = "irc.example.com".into();

        event_tx.send(GuiEvent::Connected).unwrap();

        app.process_events();

        assert!(app.state.is_connected);
        assert!(app.state.expanded_networks.contains("irc.example.com"));
        assert!(app.state.system_log.iter().any(|m| m.contains("Connected")));
        assert!(!app.state.status_messages.is_empty());
    }

    #[test]
    fn test_disconnected_event() {
        let (mut app, event_tx, _) = create_test_app();
        app.state.is_connected = true;

        event_tx
            .send(GuiEvent::Disconnected("Connection lost".to_string()))
            .unwrap();

        app.process_events();

        assert!(!app.state.is_connected);
        assert!(app.state.system_log.iter().any(|m| m.contains("Disconnected")));
        assert!(app.state.system_log.iter().any(|m| m.contains("Connection lost")));
    }

    #[test]
    fn test_error_event() {
        let (mut app, event_tx, _) = create_test_app();
        let original_log_size = app.state.system_log.len();

        event_tx
            .send(GuiEvent::Error("Test error message".to_string()))
            .unwrap();

        app.process_events();

        assert!(app.state.system_log.len() > original_log_size);
        assert!(app.state.system_log.iter().any(|m| m.contains("Error")));
        assert!(app.state.system_log.iter().any(|m| m.contains("Test error message")));
        assert!(!app.state.status_messages.is_empty());
    }

    #[test]
    fn test_raw_message_event() {
        let (mut app, event_tx, _) = create_test_app();
        let original_log_size = app.state.system_log.len();

        event_tx
            .send(GuiEvent::RawMessage("PING :server".to_string()))
            .unwrap();

        app.process_events();

        assert!(app.state.system_log.len() > original_log_size);
        assert!(app.state.system_log.iter().any(|m| m.contains("PING :server")));
    }

    #[test]
    fn test_joined_channel_event() {
        let (mut app, event_tx, _) = create_test_app();
        app.state.is_connected = true;
        app.state.active_buffer = "System".into();

        event_tx
            .send(GuiEvent::JoinedChannel("#newchan".to_string()))
            .unwrap();

        app.process_events();

        assert_eq!(app.state.active_buffer, "#newchan");
        assert!(app.state.buffers.contains_key("#newchan"));
        assert!(app.state.system_log.iter().any(|m| m.contains("Joined #newchan")));
    }

    #[test]
    fn test_parted_channel_event() {
        let (mut app, event_tx, _) = create_test_app();
        app.state.is_connected = true;
        app.state.active_buffer = "#test".into();
        app.state.buffers.insert("#test".into(), ChannelBuffer::new());
        app.state.buffers_order.push("#test".into());

        event_tx
            .send(GuiEvent::PartedChannel("#test".to_string()))
            .unwrap();

        app.process_events();

        assert!(!app.state.buffers.contains_key("#test"));
        assert!(!app.state.buffers_order.contains(&"#test".to_string()));
        assert_eq!(app.state.active_buffer, "System");
        assert!(app.state.system_log.iter().any(|m| m.contains("Left #test")));
    }

    #[test]
    fn test_message_received_creates_pm_buffer() {
        let (mut app, event_tx, _) = create_test_app();
        app.state.is_connected = true;
        app.connection.nickname = "me".into();

        // PM from alice
        event_tx
            .send(GuiEvent::MessageReceived {
                target: "me".to_string(),
                sender: "alice".to_string(),
                text: "Hello there!".to_string(),
            })
            .unwrap();

        app.process_events();

        assert!(app.state.buffers.contains_key("alice"));
        let buffer = app.state.buffers.get("alice").unwrap();
        assert!(buffer.messages.iter().any(|m| m.text.contains("Hello there!")));
    }
}

