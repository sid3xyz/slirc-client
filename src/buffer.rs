/// Represents a single buffer (channel, query, or system)
#[derive(Default)]
pub struct Buffer {
    pub messages: Vec<(String, String, String)>, // (timestamp, sender, text)
    pub users: Vec<String>,
    pub topic: String,
    pub unread: usize,
    pub has_mention: bool,
}
