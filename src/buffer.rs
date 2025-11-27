/// Represents a single buffer (channel, query, or system)
#[derive(Default, Clone)]
pub struct Buffer {
    pub messages: Vec<(String, String, String)>, // (timestamp, sender, text)
    /// Users in this buffer. We keep the prefix/mode character (if any) so the
    /// UI can display operators, voiced users, etc.
    pub users: Vec<crate::protocol::UserInfo>,
    pub topic: String,
    pub unread: usize,
    pub has_mention: bool,
}
