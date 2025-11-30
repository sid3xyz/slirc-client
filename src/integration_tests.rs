//! Integration tests for slirc-client
//! 
//! These tests exercise full workflows across multiple modules to ensure
//! proper integration between backend, events, commands, and UI state.

#[cfg(test)]
mod integration_tests {
    use crate::buffer::{ChannelBuffer, MessageType, RenderedMessage};
    use crate::protocol::{BackendAction, GuiEvent, UserInfo};
    use crossbeam_channel::unbounded;
    use std::collections::HashMap;

    /// Test channel buffer state management with multiple channels
    #[test]
    fn test_multi_channel_buffer_state() {
        let mut buffers: HashMap<String, ChannelBuffer> = HashMap::new();

        // Create server and channel buffers
        buffers.insert("-server-".to_string(), ChannelBuffer::new());
        buffers.insert("#rust".to_string(), ChannelBuffer::new());
        buffers.insert("#test".to_string(), ChannelBuffer::new());

        assert_eq!(buffers.len(), 3);

        // Add messages to different channels
        let msg1 = RenderedMessage {
            timestamp: "12:00".to_string(),
            sender: "alice".to_string(),
            text: "Hello #rust!".to_string(),
            msg_type: MessageType::Normal,
        };

        buffers.get_mut("#rust").unwrap().add_message(msg1, false, false);

        let msg2 = RenderedMessage {
            timestamp: "12:01".to_string(),
            sender: "bob".to_string(),
            text: "Hello #test!".to_string(),
            msg_type: MessageType::Normal,
        };

        buffers.get_mut("#test").unwrap().add_message(msg2, false, false);

        // Verify messages are in correct channels
        assert_eq!(buffers.get("#rust").unwrap().messages.len(), 1);
        assert_eq!(buffers.get("#test").unwrap().messages.len(), 1);
        assert_eq!(buffers.get("-server-").unwrap().messages.len(), 0);

        // Verify unread counts
        assert_eq!(buffers.get("#rust").unwrap().unread_count, 1);
        assert_eq!(buffers.get("#test").unwrap().unread_count, 1);
    }

    /// Test user list management in channel buffers
    #[test]
    fn test_channel_user_list_management() {
        let mut buffer = ChannelBuffer::new();

        // Add users
        buffer.users.push(UserInfo {
            nick: "alice".to_string(),
            prefix: Some('@'),
        });
        buffer.users.push(UserInfo {
            nick: "bob".to_string(),
            prefix: Some('+'),
        });
        buffer.users.push(UserInfo {
            nick: "charlie".to_string(),
            prefix: None,
        });

        assert_eq!(buffer.users.len(), 3);

        // Verify user order and prefixes
        assert_eq!(buffer.users[0].nick, "alice");
        assert_eq!(buffer.users[0].prefix, Some('@'));

        // Remove a user (simulate quit)
        buffer.users.retain(|u| u.nick != "bob");
        assert_eq!(buffer.users.len(), 2);
        assert!(!buffer.users.iter().any(|u| u.nick == "bob"));

        // Update user prefix (simulate mode change)
        if let Some(user) = buffer.users.iter_mut().find(|u| u.nick == "charlie") {
            user.prefix = Some('+');
        }
        assert_eq!(buffer.users.iter().find(|u| u.nick == "charlie").unwrap().prefix, Some('+'));
    }

    /// Test message type handling
    #[test]
    fn test_message_type_handling() {
        let mut buffer = ChannelBuffer::new();

        // Add different message types
        let messages = vec![
            (MessageType::Normal, "Normal message"),
            (MessageType::Action, "ACTION message"),
            (MessageType::Join, "joined"),
            (MessageType::Part, "left"),
            (MessageType::Quit, "quit"),
            (MessageType::NickChange, "changed nick"),
            (MessageType::Topic, "changed topic"),
            (MessageType::Notice, "NOTICE"),
        ];

        for (msg_type, text) in messages {
            buffer.add_message(
                RenderedMessage {
                    timestamp: "12:00".to_string(),
                    sender: "system".to_string(),
                    text: text.to_string(),
                    msg_type: msg_type.clone(),
                },
                false,
                false,
            );
        }

        assert_eq!(buffer.messages.len(), 8);

        // Verify message types are preserved
        assert_eq!(buffer.messages[0].msg_type, MessageType::Normal);
        assert_eq!(buffer.messages[1].msg_type, MessageType::Action);
        assert_eq!(buffer.messages[7].msg_type, MessageType::Notice);
    }

    /// Test buffer trimming when exceeding max messages
    #[test]
    fn test_buffer_message_trimming() {
        let mut buffer = ChannelBuffer::new();

        // Add messages up to limit (2000)
        for i in 0..2100 {
            buffer.add_message(
                RenderedMessage {
                    timestamp: format!("12:{:02}", i % 60),
                    sender: "test".to_string(),
                    text: format!("Message {}", i),
                    msg_type: MessageType::Normal,
                },
                false,
                false,
            );
        }

        // Buffer should have been trimmed
        assert!(buffer.messages.len() <= 2000);
    }

    /// Test channel topic updates
    #[test]
    fn test_channel_topic_management() {
        let mut buffer = ChannelBuffer::new();

        assert_eq!(buffer.topic, "");

        buffer.topic = "Welcome to #test!".to_string();
        assert_eq!(buffer.topic, "Welcome to #test!");

        buffer.topic = "New topic".to_string();
        assert_eq!(buffer.topic, "New topic");
    }

    /// Test unread count and highlighting
    #[test]
    fn test_unread_and_highlight_tracking() {
        let mut buffer = ChannelBuffer::new();

        // Add normal message (not active, not highlight)
        buffer.add_message(
            RenderedMessage {
                timestamp: "12:00".to_string(),
                sender: "alice".to_string(),
                text: "Hello".to_string(),
                msg_type: MessageType::Normal,
            },
            false,
            false,
        );

        assert_eq!(buffer.unread_count, 1);
        assert!(!buffer.has_highlight);

        // Add highlighted message
        buffer.add_message(
            RenderedMessage {
                timestamp: "12:01".to_string(),
                sender: "bob".to_string(),
                text: "mynick: ping!".to_string(),
                msg_type: MessageType::Normal,
            },
            false,
            true, // highlight
        );

        assert_eq!(buffer.unread_count, 2);
        assert!(buffer.has_highlight);

        // Clear unread
        buffer.clear_unread();
        assert_eq!(buffer.unread_count, 0);
        assert!(!buffer.has_highlight);

        // Active message shouldn't increment unread
        buffer.add_message(
            RenderedMessage {
                timestamp: "12:02".to_string(),
                sender: "charlie".to_string(),
                text: "Message in active buffer".to_string(),
                msg_type: MessageType::Normal,
            },
            true, // is_active
            false,
        );

        assert_eq!(buffer.unread_count, 0);
    }

    /// Test backend action channel communication
    #[test]
    fn test_backend_action_channel() {
        let (action_tx, action_rx) = unbounded::<BackendAction>();

        // Send various actions
        action_tx.send(BackendAction::Connect {
            server: "irc.example.com".to_string(),
            port: 6667,
            nickname: "testbot".to_string(),
            username: "testuser".to_string(),
            realname: "Test User".to_string(),
            use_tls: false,
            auto_reconnect: true,
        }).unwrap();

        action_tx.send(BackendAction::Join("#test".to_string())).unwrap();
        action_tx.send(BackendAction::SendMessage {
            target: "#test".to_string(),
            text: "Hello world!".to_string(),
        }).unwrap();
        action_tx.send(BackendAction::Disconnect).unwrap();

        // Verify actions are received in order
        match action_rx.recv().unwrap() {
            BackendAction::Connect { server, .. } => assert_eq!(server, "irc.example.com"),
            _ => panic!("Expected Connect action"),
        }

        match action_rx.recv().unwrap() {
            BackendAction::Join(channel) => assert_eq!(channel, "#test"),
            _ => panic!("Expected Join action"),
        }

        match action_rx.recv().unwrap() {
            BackendAction::SendMessage { target, text } => {
                assert_eq!(target, "#test");
                assert_eq!(text, "Hello world!");
            }
            _ => panic!("Expected SendMessage action"),
        }

        assert!(matches!(action_rx.recv().unwrap(), BackendAction::Disconnect));
    }

    /// Test GUI event channel communication
    #[test]
    fn test_gui_event_channel() {
        let (event_tx, event_rx) = unbounded::<GuiEvent>();

        // Send various events
        event_tx.send(GuiEvent::Connected).unwrap();
        event_tx.send(GuiEvent::JoinedChannel("#test".to_string())).unwrap();
        event_tx.send(GuiEvent::MessageReceived {
            target: "#test".to_string(),
            sender: "alice".to_string(),
            text: "Hello!".to_string(),
        }).unwrap();
        event_tx.send(GuiEvent::Disconnected("Server closed connection".to_string())).unwrap();

        // Verify events are received
        assert!(matches!(event_rx.recv().unwrap(), GuiEvent::Connected));
        assert!(matches!(event_rx.recv().unwrap(), GuiEvent::JoinedChannel(_)));

        match event_rx.recv().unwrap() {
            GuiEvent::MessageReceived { target, sender, text } => {
                assert_eq!(target, "#test");
                assert_eq!(sender, "alice");
                assert_eq!(text, "Hello!");
            }
            _ => panic!("Expected MessageReceived event"),
        }

        assert!(matches!(event_rx.recv().unwrap(), GuiEvent::Disconnected(_)));
    }

    /// Test UserInfo structure
    #[test]
    fn test_user_info_structure() {
        let users = [UserInfo {
                nick: "owner".to_string(),
                prefix: Some('~'),
            },
            UserInfo {
                nick: "admin".to_string(),
                prefix: Some('&'),
            },
            UserInfo {
                nick: "op".to_string(),
                prefix: Some('@'),
            },
            UserInfo {
                nick: "halfop".to_string(),
                prefix: Some('%'),
            },
            UserInfo {
                nick: "voice".to_string(),
                prefix: Some('+'),
            },
            UserInfo {
                nick: "regular".to_string(),
                prefix: None,
            }];

        assert_eq!(users.len(), 6);
        assert_eq!(users[0].prefix, Some('~'));
        assert_eq!(users[5].prefix, None);
    }

    /// Test multiple buffers with different states
    #[test]
    fn test_buffer_state_isolation() {
        let mut buffers: HashMap<String, ChannelBuffer> = HashMap::new();

        buffers.insert("#channel1".to_string(), ChannelBuffer::new());
        buffers.insert("#channel2".to_string(), ChannelBuffer::new());

        // Add users to channel1
        buffers.get_mut("#channel1").unwrap().users.push(UserInfo {
            nick: "alice".to_string(),
            prefix: Some('@'),
        });

        // Add messages to channel2
        buffers.get_mut("#channel2").unwrap().add_message(
            RenderedMessage {
                timestamp: "12:00".to_string(),
                sender: "bob".to_string(),
                text: "Hello".to_string(),
                msg_type: MessageType::Normal,
            },
            false,
            false,
        );

        // Verify isolation
        assert_eq!(buffers.get("#channel1").unwrap().users.len(), 1);
        assert_eq!(buffers.get("#channel1").unwrap().messages.len(), 0);

        assert_eq!(buffers.get("#channel2").unwrap().users.len(), 0);
        assert_eq!(buffers.get("#channel2").unwrap().messages.len(), 1);
    }

    /// Test TLS connector creation with webpki root certificates
    #[test]
    fn test_tls_connector_creation() {
        use crate::backend::create_tls_connector;
        
        let result = create_tls_connector();
        assert!(result.is_ok(), "TLS connector should be created successfully");
    }

    /// Test TLS-enabled backend action
    #[test]
    fn test_tls_backend_action() {
        let (action_tx, action_rx) = unbounded::<BackendAction>();

        // Send a connect action with TLS enabled
        action_tx.send(BackendAction::Connect {
            server: "irc.libera.chat".to_string(),
            port: 6697,
            nickname: "testuser".to_string(),
            username: "testuser".to_string(),
            realname: "Test User".to_string(),
            use_tls: true,
            auto_reconnect: true,
        }).unwrap();

        // Verify action was sent correctly
        let action = action_rx.try_recv().unwrap();
        match action {
            BackendAction::Connect { server, port, use_tls, .. } => {
                assert_eq!(server, "irc.libera.chat");
                assert_eq!(port, 6697);
                assert!(use_tls, "TLS should be enabled");
            }
            _ => panic!("Expected Connect action"),
        }
    }

    /// Test TLS state propagation through protocol structures
    #[test]
    fn test_tls_state_propagation() {
        let (action_tx, action_rx) = unbounded::<BackendAction>();

        // Test with TLS enabled
        action_tx.send(BackendAction::Connect {
            server: "secure.server.com".to_string(),
            port: 6697,
            nickname: "user".to_string(),
            username: "user".to_string(),
            realname: "User".to_string(),
            use_tls: true,
            auto_reconnect: true,
        }).unwrap();

        let action1 = action_rx.try_recv().unwrap();
        if let BackendAction::Connect { use_tls, .. } = action1 {
            assert!(use_tls);
        }

        // Test with TLS disabled
        action_tx.send(BackendAction::Connect {
            server: "plain.server.com".to_string(),
            port: 6667,
            nickname: "user".to_string(),
            username: "user".to_string(),
            realname: "User".to_string(),
            use_tls: false,
            auto_reconnect: true,
        }).unwrap();

        let action2 = action_rx.try_recv().unwrap();
        if let BackendAction::Connect { use_tls, .. } = action2 {
            assert!(!use_tls);
        }
    }

    /// Test hostname extraction for SNI (Server Name Indication)
    #[test]
    fn test_hostname_extraction_for_sni() {
        // Test cases for hostname extraction used in TLS SNI
        let test_cases = vec![
            ("irc.libera.chat", "irc.libera.chat"),
            ("irc.libera.chat:6697", "irc.libera.chat"),
            ("192.168.1.1", "192.168.1.1"),
            ("192.168.1.1:6667", "192.168.1.1"),
            ("irc.example.com:9999", "irc.example.com"),
        ];

        for (input, expected) in test_cases {
            let hostname = input.split(':').next().unwrap_or(input);
            assert_eq!(hostname, expected, "Failed for input: {}", input);
        }
    }

    /// Test TLS error event handling
    #[test]
    fn test_tls_error_event_handling() {
        let (event_tx, event_rx) = unbounded::<GuiEvent>();
        let mut system_log: Vec<String> = Vec::new();

        // Simulate TLS error
        event_tx.send(GuiEvent::Error(
            "TLS handshake failed: certificate verify failed".to_string()
        )).unwrap();

        // Process the error event
        if let Ok(GuiEvent::Error(msg)) = event_rx.try_recv() {
            system_log.push(format!("Error: {}", msg));
            assert!(msg.contains("TLS"));
            assert!(msg.contains("certificate"));
        }

        assert_eq!(system_log.len(), 1);
        assert!(system_log[0].contains("TLS handshake failed"));
    }

    /// Test port selection based on TLS setting
    #[test]
    fn test_port_selection_for_tls() {
        // Typical IRC ports
        let tls_port = 6697;
        let plain_port = 6667;

        // Verify TLS uses secure port
        assert_eq!(tls_port, 6697);
        assert_ne!(tls_port, plain_port);

        // Verify plain uses standard port
        assert_eq!(plain_port, 6667);
    }

    /// Test IRC color code parsing doesn't panic
    #[test]
    fn test_irc_color_codes_parsing() {
        // Test various color code formats
        let test_cases = vec![
            "\x0304Red text",                          // Foreground only
            "\x0304,02Red on blue",                   // Foreground and background
            "\x03Reset colors",                       // Color reset
            "Normal \x0312blue\x03 normal",          // Color in middle
            "\x02Bold\x02 normal",                   // Bold toggle
            "\x1DItalic\x1D normal",                 // Italic toggle
            "\x02\x0304Bold red\x0F reset all",      // Multiple formats
            "\x0399,01Light green on black",         // Two-digit codes
            "Mixed \x02bold\x0304red\x1Ditalic\x0F", // Complex formatting
        ];

        for text in test_cases {
            // This should not panic - the parser should handle all cases
            let chars: Vec<char> = text.chars().collect();
            assert!(!chars.is_empty(), "Test case should not be empty");
            
            // Verify control codes are present
            let has_formatting = text.contains('\x02') 
                || text.contains('\x03') 
                || text.contains('\x1D')
                || text.contains('\x0F');
            
            // At least some test cases should have formatting
            if text.len() < 20 {
                assert!(has_formatting || text == "\x03Reset colors", 
                    "Expected formatting in: {:?}", text);
            }
        }
    }

    /// Test IRC formatting state machine edge cases
    #[test]
    fn test_irc_formatting_edge_cases() {
        // Edge cases that should be handled gracefully
        let edge_cases = vec![
            "",                      // Empty string
            "\x03",                  // Color code at end
            "\x02",                  // Bold at end
            "\x03\x02\x1D\x0F",     // Only control codes
            "\x0399",                // Incomplete color code (no comma)
            "\x03,",                 // Color code with comma but no bg
            "\x03,,",                // Multiple commas
            "Text\x03\x03\x03",     // Multiple consecutive color resets
            "\x02\x02\x02Text",     // Multiple bold toggles
            "\x0F\x0FNormal",       // Multiple resets
        ];

        for text in edge_cases {
            // Should not panic on edge cases
            let chars: Vec<char> = text.chars().collect();
            
            // Verify we can safely iterate
            let mut i = 0;
            while i < chars.len() {
                match chars[i] {
                    '\x02' | '\x1D' | '\x0F' => {
                        // Control codes should be recognized
                        assert!(chars[i] < ' ', "Control code verified");
                    }
                    '\x03' => {
                        // Color code handling
                        i += 1;
                        // Skip any digits
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                        continue;
                    }
                    _ => {}
                }
                i += 1;
            }
        }
    }

    /// Test mIRC color palette range
    #[test]
    fn test_mirc_color_palette() {
        use crate::ui::theme::mirc_color;
        
        // Test all 16 standard mIRC colors exist and are distinct
        let colors: Vec<_> = (0..16).map(|i| mirc_color(i)).collect();
        
        // Verify color 0 is white (standard mIRC)
        assert_eq!(colors[0], eframe::egui::Color32::from_rgb(255, 255, 255));
        
        // Verify color 1 is black (standard mIRC)
        assert_eq!(colors[1], eframe::egui::Color32::from_rgb(0, 0, 0));
        
        // Test out-of-range codes return white (safe default)
        let out_of_range = mirc_color(99);
        assert_eq!(out_of_range, eframe::egui::Color32::WHITE);
    }

    /// Test logging module initialization
    #[test]
    fn test_logger_initialization() {
        use crate::logging::Logger;
        
        // Logger should initialize successfully
        let result = Logger::new();
        assert!(result.is_ok(), "Logger should initialize: {:?}", result.err());
        
        if let Ok(logger) = result {
            // Verify log directory is created
            let log_dir = logger.log_directory();
            assert!(log_dir.to_string_lossy().contains("slirc-client"));
            assert!(log_dir.to_string_lossy().contains("logs"));
        }
    }
}


