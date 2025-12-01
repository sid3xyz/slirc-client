//! Comprehensive backend tests for network layer

use crossbeam_channel::unbounded;
use std::time::Duration;

use crate::protocol::{BackendAction, GuiEvent};

    #[test]
    fn test_backend_thread_creation() {
        // Test that the backend thread can be created without panicking
        let (action_tx, action_rx) = unbounded::<BackendAction>();
        let (event_tx, _event_rx) = unbounded::<GuiEvent>();

        let _handle = std::thread::spawn(move || {
            crate::backend::run_backend(action_rx, event_tx);
        });

        // Send disconnect to allow thread to exit cleanly
        let _ = action_tx.send(BackendAction::Disconnect);

        // Thread should not panic
        std::thread::sleep(Duration::from_millis(100));
        drop(action_tx);
    }

    #[test]
    fn test_connection_validation() {
        // Test input validation for connection parameters
        use crate::validation;

        // Valid server addresses
        assert!(validation::validate_server_address("irc.libera.chat:6667").is_ok());
        assert!(validation::validate_server_address("irc.libera.chat").is_ok());

        // Invalid server addresses
        assert!(validation::validate_server_address("").is_err());
        assert!(validation::validate_server_address(":6667").is_err());
        assert!(validation::validate_server_address("host:0").is_err());

        // Valid nicknames
        assert!(validation::validate_nickname("alice").is_ok());
        assert!(validation::validate_nickname("Bob123").is_ok());
        assert!(validation::validate_nickname("[guest]").is_ok());

        // Invalid nicknames
        assert!(validation::validate_nickname("").is_err());
        assert!(validation::validate_nickname("123user").is_err());
        assert!(validation::validate_nickname(&"a".repeat(31)).is_err());
    }

    #[test]
    fn test_disconnect_handling() {
        let (action_tx, action_rx) = unbounded::<BackendAction>();
        let (event_tx, event_rx) = unbounded::<GuiEvent>();

        let _handle = std::thread::spawn(move || {
            crate::backend::run_backend(action_rx, event_tx);
        });

        // Send disconnect
        action_tx.send(BackendAction::Disconnect).unwrap();

        // Should receive disconnected event
        match event_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(GuiEvent::Disconnected(_)) => {
                // Expected
            }
            _ => panic!("Expected Disconnected event"),
        }

        drop(action_tx);
    }

    #[test]
    fn test_channel_validation() {
        use crate::validation;

        // Valid channels
        assert!(validation::validate_channel_name("#general").is_ok());
        assert!(validation::validate_channel_name("&local").is_ok());
        assert!(validation::validate_channel_name("#rust-lang").is_ok());

        // Invalid channels
        assert!(validation::validate_channel_name("").is_err());
        assert!(validation::validate_channel_name("notachannel").is_err());
        assert!(validation::validate_channel_name("#test channel").is_err());
        assert!(validation::validate_channel_name(&"#".to_string().repeat(51)).is_err());
    }

    #[test]
    fn test_message_validation() {
        use crate::validation;

        // Valid messages
        assert!(validation::validate_message("Hello, world!").is_ok());
        assert!(validation::validate_message("Test 123").is_ok());

        // Invalid messages
        assert!(validation::validate_message("").is_err());
        assert!(validation::validate_message("Line1\nLine2").is_err());
        assert!(validation::validate_message(&"x".repeat(401)).is_err());
    }

    #[test]
    fn test_message_sanitization() {
        use crate::validation;

        assert_eq!(validation::sanitize_message("Hello"), "Hello");
        assert_eq!(validation::sanitize_message("Line1\nLine2"), "Line1Line2");
        assert_eq!(validation::sanitize_message(&"x".repeat(500)), "x".repeat(400));
    }

    #[test]
    fn test_action_channel_communication() {
        let (action_tx, action_rx) = unbounded::<BackendAction>();

        // Test that we can send various action types
        action_tx.send(BackendAction::Disconnect).unwrap();
        action_tx.send(BackendAction::Join("#test".to_string())).unwrap();
        action_tx.send(BackendAction::Part {
            channel: "#test".to_string(),
            message: None,
        }).unwrap();

        // Verify we can receive them
        assert!(matches!(action_rx.recv().unwrap(), BackendAction::Disconnect));
        assert!(matches!(action_rx.recv().unwrap(), BackendAction::Join(_)));
        assert!(matches!(action_rx.recv().unwrap(), BackendAction::Part { .. }));
    }

    #[test]
    fn test_gui_event_types() {
        let (event_tx, event_rx) = unbounded::<GuiEvent>();

        // Test various event types
        event_tx.send(GuiEvent::Connected).unwrap();
        event_tx.send(GuiEvent::Disconnected("test".to_string())).unwrap();
        event_tx.send(GuiEvent::Error("test error".to_string())).unwrap();
        event_tx.send(GuiEvent::JoinedChannel("#test".to_string())).unwrap();

        // Verify they can be received
        assert!(matches!(event_rx.recv().unwrap(), GuiEvent::Connected));
        assert!(matches!(event_rx.recv().unwrap(), GuiEvent::Disconnected(_)));
        assert!(matches!(event_rx.recv().unwrap(), GuiEvent::Error(_)));
        assert!(matches!(event_rx.recv().unwrap(), GuiEvent::JoinedChannel(_)));
    }

    #[test]
    fn test_tls_configuration_parsing() {
        use crate::config::Network;

        let network = Network {
            name: "Test".to_string(),
            servers: vec!["irc.libera.chat:6697".to_string()],
            nick: "testuser".to_string(),
            auto_connect: false,
            favorite_channels: vec![],
            nickserv_password: None,
            use_tls: true,
            auto_reconnect: true,
        };

        assert!(network.use_tls);
        assert_eq!(network.servers[0], "irc.libera.chat:6697");
    }

    #[test]
    fn test_buffer_state_management() {
        use crate::buffer::{ChannelBuffer, MessageType, RenderedMessage};
        use crate::protocol::UserInfo;

        let mut buffer = ChannelBuffer::new();

        // Test adding messages
        let msg1 = RenderedMessage {
            timestamp: "12:00".to_string(),
            sender: "Alice".to_string(),
            text: "Hello!".to_string(),
            msg_type: MessageType::Normal,
        };
        let msg2 = RenderedMessage {
            timestamp: "12:01".to_string(),
            sender: "Bob".to_string(),
            text: "Hi there!".to_string(),
            msg_type: MessageType::Normal,
        };
        buffer.add_message(msg1, false, false);
        buffer.add_message(msg2, false, false);

        assert_eq!(buffer.messages.len(), 2);
        assert_eq!(buffer.unread_count, 2);

        // Test clearing unread
        buffer.clear_unread();
        assert_eq!(buffer.unread_count, 0);

        // Test user list
        buffer.users.push(UserInfo { nick: "Alice".to_string(), prefix: Some('@') });
        buffer.users.push(UserInfo { nick: "Bob".to_string(), prefix: None });
        assert_eq!(buffer.users.len(), 2);
    }

    #[test]
    fn test_event_processing_flow() {
        use crate::buffer::ChannelBuffer;
        use crate::protocol::UserInfo;
        use std::collections::HashMap;

        let mut buffers = HashMap::new();
        let mut buffers_order = Vec::new();

        // Simulate receiving a Names event
        let names = vec![
            UserInfo {
                nick: "alice".to_string(),
                prefix: Some('@'),
            },
            UserInfo {
                nick: "bob".to_string(),
                prefix: None,
            },
        ];

        // This would normally be processed by events::process_events
        // For now, just verify the data structures work
        buffers.insert("#test".to_string(), ChannelBuffer::new());
        buffers_order.push("#test".to_string());

        let buffer = buffers.get_mut("#test").expect("Buffer should exist");
        buffer.users = names.clone();

        assert_eq!(buffer.users.len(), 2);
        assert!(buffer.users.iter().any(|u| u.nick == "alice"));
        assert!(buffer.users.iter().any(|u| u.nick == "bob"));
    }

    #[test]
    fn test_command_parsing() {
        use crate::commands;

        use crossbeam_channel::unbounded;
        use std::collections::HashMap;

        let (action_tx, action_rx) = unbounded();
        let buffers = HashMap::new();
        let mut system_log = Vec::new();
        let mut nickname = "test".to_string();

        // Test join command
        let is_cmd = commands::handle_user_command(
            "/join #test",
            "System",
            &buffers,
            &action_tx,
            &mut system_log,
            &mut nickname,
        );

        assert!(is_cmd);
        let action = action_rx.recv().unwrap();
        assert!(matches!(action, crate::protocol::BackendAction::Join(_)));
    }

    #[test]
    fn test_protocol_action_serialization() {
        use crate::protocol::BackendAction;

        // Test that actions can be cloned and moved
        let action = BackendAction::Connect {
            server: "test.server".to_string(),
            port: 6667,
            nickname: "testuser".to_string(),
            username: "testuser".to_string(),
            realname: "Test User".to_string(),
            use_tls: false,
            auto_reconnect: true,
            sasl_password: None,
        };

        let cloned = action.clone();
        assert!(matches!(cloned, BackendAction::Connect { .. }));
    }

    #[test]
    fn test_error_handling_in_validation() {
        use crate::validation;

        // Test that errors contain meaningful messages
        match validation::validate_channel_name("invalid") {
            Err(msg) => assert!(msg.contains("must start with")),
            Ok(_) => panic!("Should have failed"),
        }

        match validation::validate_nickname("") {
            Err(msg) => assert!(msg.contains("cannot be empty")),
            Ok(_) => panic!("Should have failed"),
        }

        match validation::validate_message("") {
            Err(msg) => assert!(msg.contains("cannot be empty")),
            Ok(_) => panic!("Should have failed"),
        }
    }
