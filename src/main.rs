//! SLIRC Client - An IRC client built with egui and slirc-proto
//!
//! Architecture:
//! - Main thread: runs the egui UI
//! - Backend thread: runs a Tokio runtime for async network I/O
//! - Communication via crossbeam channels (lock-free, sync-safe)

mod protocol;
mod buffer;
mod config;
mod backend;
mod app;

use eframe::egui;
use app::SlircApp;

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
    use super::*;
    use crate::app::SlircApp;
    use crate::protocol::{BackendAction, GuiEvent};
    use crate::buffer::Buffer;
    use crate::config::{DEFAULT_SERVER, DEFAULT_CHANNEL};
    use crossbeam_channel::unbounded;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_clean_motd() {
        let (action_tx, _action_rx) = unbounded::<BackendAction>();
        let (event_tx, event_rx) = unbounded::<GuiEvent>();
        let mut app = SlircApp {
            server_input: DEFAULT_SERVER.into(),
            nickname_input: "tester".into(),
            is_connected: false,
            action_tx,
            event_rx,
            buffers: HashMap::new(),
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
        };
        assert_eq!(app.clean_motd_line("-"), "");
        assert_eq!(app.clean_motd_line(":-"), "");
        assert_eq!(app.clean_motd_line(":- "), "");
        assert_eq!(app.clean_motd_line(":- Hello world"), "Hello world");
        assert_eq!(app.clean_motd_line("- ═════════"), "---------"); // replaced since no font fallback
        assert_eq!(app.clean_motd_line("Hello"), "Hello");
        assert_eq!(app.clean_motd_line(" - Hello"), "Hello");
    }

    #[test]
    fn test_motd_processed_in_system_log() {
        let (action_tx, _action_rx) = unbounded::<BackendAction>();
        let (event_tx, event_rx) = unbounded::<GuiEvent>();
        let mut app = SlircApp {
            server_input: DEFAULT_SERVER.into(),
            nickname_input: "tester".into(),
            is_connected: false,
            action_tx,
            event_rx,
            buffers: HashMap::new(),
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
        };
        app.buffers.insert("System".into(), Buffer::default());

        let motd_line = String::from(":- Welcome to the server");
        let _ = event_tx.send(GuiEvent::Motd(motd_line));
        
        app.process_events();
        assert!(app.system_log.iter().any(|l| l.contains("MOTD: Welcome to the server")));
    }

    #[test]
    fn test_names_event_populates_users() {
        let (action_tx, _action_rx) = unbounded::<BackendAction>();
        let (event_tx, event_rx) = unbounded::<GuiEvent>();
        let mut app = SlircApp {
            server_input: DEFAULT_SERVER.into(),
            nickname_input: "tester".into(),
            is_connected: false,
            action_tx,
            event_rx,
            buffers: HashMap::new(),
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
        };
        app.buffers.insert("System".into(), Buffer::default());

        let names = vec![
            crate::protocol::UserInfo { nick: "admin".into(), prefix: Some('@') },
            crate::protocol::UserInfo { nick: "foo".into(), prefix: None },
            crate::protocol::UserInfo { nick: "bar".into(), prefix: Some('+') },
        ];
        let _ = event_tx.send(GuiEvent::Names { channel: "#test".into(), names });
        app.process_events();
        // Buffer should be created and populated
        assert!(app.buffers.contains_key("#test"));
        let buf = app.buffers.get("#test").unwrap();
        assert_eq!(buf.users.len(), 3);
        assert!(buf.users.iter().any(|u| u.nick == "admin" && u.prefix == Some('@')));
        assert!(buf.users.iter().any(|u| u.nick == "bar" && u.prefix == Some('+')));
    }

    #[test]
    fn test_user_mode_event_updates_prefix() {
        use crate::protocol::UserInfo as PUser;
        let (action_tx, _action_rx) = unbounded::<BackendAction>();
        let (event_tx, event_rx) = unbounded::<GuiEvent>();
        let mut app = SlircApp {
            server_input: DEFAULT_SERVER.into(),
            nickname_input: "tester".into(),
            is_connected: false,
            action_tx,
            event_rx,
            buffers: HashMap::new(),
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
        };
        app.buffers.insert("System".into(), Buffer::default());
        // Create a channel buffer with one user
        let mut buf = Buffer::default();
        buf.users.push(PUser { nick: "alice".into(), prefix: None });
        app.buffers.insert("#test".into(), buf);

        let _ = event_tx.send(GuiEvent::UserMode { channel: "#test".into(), nick: "alice".into(), prefix: Some('@'), added: true });
        app.process_events();
        let b = app.buffers.get("#test").unwrap();
        assert!(b.users.iter().any(|u| u.nick == "alice" && u.prefix == Some('@')));

        // Now remove the op
        let _ = event_tx.send(GuiEvent::UserMode { channel: "#test".into(), nick: "alice".into(), prefix: Some('@'), added: false });
        app.process_events();
        let b2 = app.buffers.get("#test").unwrap();
        assert!(b2.users.iter().any(|u| u.nick == "alice" && u.prefix.is_none()));
    }

    #[test]
    fn test_topic_event_updates_buffer_topic() {
        let (action_tx, _action_rx) = unbounded::<BackendAction>();
        let (event_tx, event_rx) = unbounded::<GuiEvent>();
        let mut app = SlircApp {
            server_input: DEFAULT_SERVER.into(),
            nickname_input: "tester".into(),
            is_connected: false,
            action_tx,
            event_rx,
            buffers: HashMap::new(),
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
        };
        app.buffers.insert("System".into(), Buffer::default());
        let _ = event_tx.send(GuiEvent::Topic { channel: "#test".into(), topic: "New Topic".into() });
        app.process_events();
        let b = app.buffers.get("#test").unwrap();
        assert_eq!(b.topic, "New Topic");
    }
}
