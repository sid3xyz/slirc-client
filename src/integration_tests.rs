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
        let users = vec![
            UserInfo {
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
            },
        ];

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
}
