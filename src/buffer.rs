use crate::protocol::UserInfo;

/// Maximum messages to keep in a buffer before trimming
const MAX_BUFFER_MESSAGES: usize = 2000;
/// Number of oldest messages to remove when trimming
const BUFFER_TRIM_COUNT: usize = 500;

/// Represents a rendered message with timestamp, sender info, and styled text
#[derive(Clone, Debug)]
pub struct RenderedMessage {
    pub timestamp: String,
    pub sender: String,
    pub text: String,
    /// Message type for special rendering
    pub msg_type: MessageType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageType {
    Normal,
    Action, // /me messages
    Join,
    Part,
    Quit,
    NickChange,
    Topic,
    Notice,
}

impl RenderedMessage {
    pub fn new(timestamp: String, sender: String, text: String) -> Self {
        Self {
            timestamp,
            sender,
            text,
            msg_type: MessageType::Normal,
        }
    }

    pub fn with_type(mut self, msg_type: MessageType) -> Self {
        self.msg_type = msg_type;
        self
    }
}

/// Represents a single buffer (channel, query, or system)
#[derive(Default, Clone)]
pub struct ChannelBuffer {
    pub messages: Vec<RenderedMessage>,
    /// Users in this buffer with prefix/mode character (if any)
    pub users: Vec<UserInfo>,
    /// Topic of the channel
    pub topic: String,
    /// Number of unread messages
    pub unread_count: usize,
    /// Whether there is a highlight/mention in unread messages
    pub has_highlight: bool,
    /// Channel modes (e.g., "mtn" for +m+t+n)
    pub channel_modes: String,
    /// Whether notifications are muted for this channel
    pub notifications_muted: bool,
    /// List of pinned message IDs (indices in messages vec)
    pub pinned_messages: Vec<usize>,
}

impl ChannelBuffer {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            users: Vec::new(),
            topic: String::new(),
            unread_count: 0,
            has_highlight: false,
            channel_modes: String::new(),
            notifications_muted: false,
            pinned_messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, msg: RenderedMessage, is_active: bool, is_highlight: bool) {
        self.messages.push(msg);
        if !is_active {
            self.unread_count += 1;
            if is_highlight {
                self.has_highlight = true;
            }
        }
        // Trim old messages if buffer gets too large
        if self.messages.len() > MAX_BUFFER_MESSAGES {
            self.messages.drain(0..BUFFER_TRIM_COUNT);
        }
    }

    pub fn clear_unread(&mut self) {
        self.unread_count = 0;
        self.has_highlight = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_message_unread_and_trim() {
        let mut buf = ChannelBuffer::new();
        // Add unread messages
        for i in 0..10 {
            buf.add_message(
                RenderedMessage::new(format!("10:{:02}", i), "alice".into(), format!("msg{}", i)),
                false,
                false,
            );
        }
        assert_eq!(buf.unread_count, 10);

        // Trim by adding many messages until we exceed MAX_BUFFER_MESSAGES
        for i in 0..(MAX_BUFFER_MESSAGES + 10) {
            buf.add_message(
                RenderedMessage::new(format!("11:{:02}", i), "bob".into(), "X".into()),
                true,
                false,
            );
        }
        // Size should not blow up beyond MAX_BUFFER_MESSAGES
        assert!(buf.messages.len() <= MAX_BUFFER_MESSAGES);
    }

    #[test]
    fn test_clear_unread() {
        let mut buf = ChannelBuffer::new();
        buf.add_message(
            RenderedMessage::new("12:00".into(), "a".into(), "hello".into()),
            false,
            false,
        );
        assert_eq!(buf.unread_count, 1);
        buf.clear_unread();
        assert_eq!(buf.unread_count, 0);
        assert!(!buf.has_highlight);
    }
}
