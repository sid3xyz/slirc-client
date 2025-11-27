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
    /// Raw server message for the system log
    RawMessage(String),
    /// MOTD line
    Motd(String),
    /// Topic for a channel
    Topic { channel: String, topic: String },
    /// Names list for a channel
    Names { channel: String, names: Vec<String> },
    /// Notification that the nick changed locally
    NickChanged { old: String, new: String },
}
