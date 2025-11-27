/// Actions sent from the UI to the Backend
#[derive(Debug, Clone)]
pub enum BackendAction {
    /// Connect to an IRC server
    Connect {
        server: String,
        port: u16,
        nickname: String,
        username: String,
        realname: String,
    },
    /// Disconnect from the server
    Disconnect,
    /// Join a channel
    Join(String),
    /// Part (leave) a channel
    Part { channel: String, message: Option<String> },
    /// Change nick
    Nick(String),
    /// Quit the server
    Quit(Option<String>),
    /// Send a message to a target (channel or user)
    SendMessage { target: String, text: String },
}

/// Events sent from the Backend to the UI
#[derive(Debug, Clone)]
pub enum GuiEvent {
    /// Successfully connected and registered
    Connected,
    /// Disconnected from server
    Disconnected(String),
    /// Connection error
    Error(String),
    /// A message was received (for any target)
    MessageReceived {
        target: String,
        sender: String,
        text: String,
    },
    /// We successfully joined a channel
    JoinedChannel(String),
    /// We left a channel
    PartedChannel(String),
    /// Someone joined a channel we're in
    UserJoined { channel: String, nick: String },
    /// Someone left a channel we're in
    UserParted { channel: String, nick: String, message: Option<String> },
    /// A user mode was changed in a channel (e.g. +o/-o) — used to update
    /// the nickname prefix in the UI.
    UserMode { channel: String, nick: String, prefix: Option<char>, added: bool },
    /// Raw server message for the system log
    RawMessage(String),
    /// MOTD line
    Motd(String),
    /// Topic for a channel
    Topic { channel: String, topic: String },
    /// Names list for a channel. Each name contains any mode prefix that was
    /// included in the NAMES reply (e.g. `@`, `+`, `%`, `~`, `&`).
    Names { channel: String, names: Vec<UserInfo> },
    /// Notification that the nick changed locally
    NickChanged { old: String, new: String },
}

/// Represents a nick and any prefix/mode that is associated with it in a
/// NAMES reply (e.g. `@` for ops, `+` for voice). This is intended to be a
/// lightweight representation used by both the backend and the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInfo {
    pub nick: String,
    /// A single-character prefix if present (e.g. '@', '+', '%', '&', '~'), or
    /// `None` for regular users.
    pub prefix: Option<char>,
}
