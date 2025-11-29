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
mod events;
mod protocol;
mod ui;

use app::SlircApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "SLIRC - IRC Client",
        options,
        Box::new(|cc| Ok(Box::new(SlircApp::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use crate::app::SlircApp;
    use crate::buffer::ChannelBuffer;
    use crate::config::{DEFAULT_CHANNEL, DEFAULT_SERVER};
    use crate::protocol::{BackendAction, GuiEvent};
    use crossbeam_channel::unbounded;
    use std::collections::{HashMap, HashSet};

    /// Helper to create a test SlircApp instance
    fn create_test_app() -> (
        SlircApp,
        crossbeam_channel::Sender<GuiEvent>,
        crossbeam_channel::Receiver<BackendAction>,
    ) {
        let (action_tx, action_rx) = unbounded::<BackendAction>();
        let (event_tx, event_rx) = unbounded::<GuiEvent>();
        // Pre-populate System buffer
        let mut buffers = HashMap::new();
        buffers.insert("System".to_string(), ChannelBuffer::new());

        let app = SlircApp {
            server_input: DEFAULT_SERVER.into(),
            nickname_input: "tester".into(),
            is_connected: false,
            action_tx,
            event_rx,
            buffers,
            buffers_order: vec!["System".into()],
            active_buffer: "System".into(),
            channel_input: DEFAULT_CHANNEL.into(),
            message_input: String::new(),
            system_log: Vec::new(),
            history: Vec::new(),
            history_pos: None,
            history_saved_input: None,
            context_menu_visible: false,
            context_menu_target: None,
            open_windows: HashSet::new(),
            completions: Vec::new(),
            completion_index: None,
            completion_prefix: None,
            completion_target_channel: false,
            last_input_text: String::new(),
            theme: String::from("dark"),
            font_fallback: None,
            topic_editor_open: None,
            networks: Vec::new(),
            network_manager_open: false,
            editing_network: None,
            network_form: crate::ui::dialogs::NetworkForm::default(),
            show_channel_list: true,
            show_user_list: true,
            expanded_networks: HashSet::new(),
            show_help_dialog: false,
            nick_change_dialog_open: false,
            nick_change_input: String::new(),
            status_messages: Vec::new(),
        };
        (app, event_tx, action_rx)
    }

    #[test]
    fn test_clean_motd() {
        let font_fallback = None;
        assert_eq!(crate::events::clean_motd_line("-", &font_fallback), "");
        assert_eq!(crate::events::clean_motd_line(":-", &font_fallback), "");
        assert_eq!(crate::events::clean_motd_line(":- ", &font_fallback), "");
        assert_eq!(crate::events::clean_motd_line(":- Hello world", &font_fallback), "Hello world");
        assert_eq!(crate::events::clean_motd_line("- ═════════", &font_fallback), "---------"); // replaced since no font fallback
        assert_eq!(crate::events::clean_motd_line("Hello", &font_fallback), "Hello");
        assert_eq!(crate::events::clean_motd_line(" - Hello", &font_fallback), "Hello");
    }

    #[test]
    fn test_motd_processed_in_system_log() {
        let (mut app, event_tx, _) = create_test_app();

        let motd_line = String::from(":- Welcome to the server");
        let _ = event_tx.send(GuiEvent::Motd(motd_line));

        app.process_events();
        assert!(app
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
        assert!(app.buffers.contains_key("#test"));
        let buf = app.buffers.get("#test").unwrap();
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
        app.buffers.insert("#test".into(), buf);

        let _ = event_tx.send(GuiEvent::UserMode {
            channel: "#test".into(),
            nick: "alice".into(),
            prefix: Some('@'),
            added: true,
        });
        app.process_events();
        let b = app.buffers.get("#test").unwrap();
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
        let b2 = app.buffers.get("#test").unwrap();
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
        let b = app.buffers.get("#test").unwrap();
        assert_eq!(b.topic, "New Topic");
    }

    #[test]
    fn test_whois_command_sends_action() {
        let (mut app, _, action_rx) = create_test_app();
        app.is_connected = true;

        // Set the message input to a whois command and ensure the action is sent
        app.message_input = String::from("/whois someuser");
        assert!(crate::commands::handle_user_command(
            &app.message_input,
            &app.active_buffer,
            &app.buffers,
            &app.action_tx,
            &mut app.system_log,
            &mut app.nickname_input,
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
        let buf = app.buffers.get("-server-").unwrap();
        assert!(!buf.messages.is_empty());
        assert_eq!(buf.messages.last().unwrap().msg_type, MessageType::Notice);
    }

    #[test]
    fn test_status_messages_on_connect() {
        let (mut app, event_tx, _) = create_test_app();
        let _ = event_tx.send(GuiEvent::Connected);
        app.process_events();
        assert!(!app.status_messages.is_empty());
        assert!(app
            .status_messages
            .last()
            .unwrap()
            .0
            .contains("Connected to"));
    }

    #[test]
    fn test_topic_command_set_and_show() {
        let (mut app, _, action_rx) = create_test_app();
        app.is_connected = true;
        app.active_buffer = "#test".into();
        app.buffers.insert("#test".into(), ChannelBuffer::new());

        // Set the message input to a topic change command and ensure the action is sent
        app.message_input = String::from("/topic New Topic");
        assert!(crate::commands::handle_user_command(
            &app.message_input,
            &app.active_buffer,
            &app.buffers,
            &app.action_tx,
            &mut app.system_log,
            &mut app.nickname_input,
        ));
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::SetTopic { channel, topic } => {
                assert_eq!(channel, "#test");
                assert_eq!(topic, "New Topic");
            }
            _ => panic!("Expected SetTopic action"),
        }

        // Now test that /topic with no args displays the topic in system_log
        app.buffers.get_mut("#test").unwrap().topic = "Displayed Topic".into();
        app.message_input = String::from("/topic");
        assert!(crate::commands::handle_user_command(
            &app.message_input,
            &app.active_buffer,
            &app.buffers,
            &app.action_tx,
            &mut app.system_log,
            &mut app.nickname_input,
        ));
        assert!(app.system_log.iter().any(|l| l.contains("Displayed Topic")));
    }

    #[test]
    fn test_kick_command_sends_action() {
        let (mut app, _, action_rx) = create_test_app();
        app.is_connected = true;
        app.active_buffer = "#test".into();
        app.buffers.insert("#test".into(), ChannelBuffer::new());

        app.message_input = String::from("/kick alice Spamming");
        assert!(crate::commands::handle_user_command(
            &app.message_input,
            &app.active_buffer,
            &app.buffers,
            &app.action_tx,
            &mut app.system_log,
            &mut app.nickname_input,
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
        app.is_connected = true;
        app.active_buffer = "#test".into();
        app.buffers.insert("#test".into(), ChannelBuffer::new());

        app.message_input = String::from("/me waves hello");
        assert!(crate::commands::handle_user_command(
            &app.message_input,
            &app.active_buffer,
            &app.buffers,
            &app.action_tx,
            &mut app.system_log,
            &mut app.nickname_input,
        ));
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::SendMessage { target, text } => {
                assert_eq!(target, "#test");
                assert_eq!(text, "\x01ACTION waves hello\x01");
            }
            _ => panic!("Expected SendMessage action with CTCP ACTION"),
        }
    }
}
