//! SLIRC Client - An IRC client built with egui and slirc-proto
//!
//! Architecture:
//! - Main thread: runs the egui UI
//! - Backend thread: runs a Tokio runtime for async network I/O
//! - Communication via crossbeam channels (lock-free, sync-safe)

use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use chrono::Local;

use crossbeam_channel::{Receiver, Sender, unbounded};
use eframe::egui;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::time::timeout;

use slirc_proto::{Command, Message, Transport};

// ============================================================================
// Channel Protocol: UI <-> Backend Communication
// ============================================================================

/// Actions sent from the UI to the Backend
#[derive(Debug, Clone)]
enum BackendAction {
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
enum GuiEvent {
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

// ============================================================================
// Backend: Async Network Loop (runs in a separate thread)
// ============================================================================

fn run_backend(
    action_rx: Receiver<BackendAction>,
    event_tx: Sender<GuiEvent>,
) {
    // Create a Tokio runtime for this thread
    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    rt.block_on(async move {
        let mut transport: Option<Transport> = None;
        let mut current_nick = String::new();

        loop {
            // Check for actions from the UI (non-blocking)
            while let Ok(action) = action_rx.try_recv() {
                match action {
                    BackendAction::Connect { server, port, nickname, username, realname } => {
                        current_nick = nickname.clone();
                        
                        // Try to connect
                        let addr = format!("{}:{}", server, port);
                        let _ = event_tx.send(GuiEvent::RawMessage(format!("Connecting to {}...", addr)));
                        
                        match TcpStream::connect(&addr).await {
                            Ok(stream) => {
                                let mut t = Transport::tcp(stream);
                                
                                // Send NICK
                                let nick_msg = Message::nick(&nickname);
                                if let Err(e) = t.write_message(&nick_msg).await {
                                    let _ = event_tx.send(GuiEvent::Error(format!("Failed to send NICK: {}", e)));
                                    continue;
                                }
                                let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", nick_msg)));
                                
                                // Send USER
                                let user_msg = Message::user(&username, &realname);
                                if let Err(e) = t.write_message(&user_msg).await {
                                    let _ = event_tx.send(GuiEvent::Error(format!("Failed to send USER: {}", e)));
                                    continue;
                                }
                                let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", user_msg)));
                                
                                transport = Some(t);
                            }
                            Err(e) => {
                                let _ = event_tx.send(GuiEvent::Error(format!("Connection failed: {}", e)));
                            }
                        }
                    }
                    
                    BackendAction::Disconnect => {
                        if let Some(ref mut t) = transport {
                            let quit_msg = Message::quit_with_message("Leaving");
                            let _ = t.write_message(&quit_msg).await;
                        }
                        transport = None;
                        let _ = event_tx.send(GuiEvent::Disconnected("User disconnected".into()));
                    }
                    
                    BackendAction::Join(channel) => {
                        if let Some(ref mut t) = transport {
                            let join_msg = Message::join(&channel);
                            let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", join_msg)));
                            if let Err(e) = t.write_message(&join_msg).await {
                                let _ = event_tx.send(GuiEvent::Error(format!("Failed to join: {}", e)));
                            }
                        }
                    }
                    
                    BackendAction::Part { channel, message } => {
                        if let Some(ref mut t) = transport {
                            let part_msg = if let Some(msg) = message {
                                Message::part_with_message(&channel, &msg)
                            } else {
                                Message::part(&channel)
                            };
                            let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", part_msg)));
                            if let Err(e) = t.write_message(&part_msg).await {
                                let _ = event_tx.send(GuiEvent::Error(format!("Failed to part: {}", e)));
                            }
                        }
                    }
                    BackendAction::Nick(newnick) => {
                        if let Some(ref mut t) = transport {
                            let nick_msg = Message::nick(&newnick);
                            let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", nick_msg)));
                            if let Err(e) = t.write_message(&nick_msg).await {
                                let _ = event_tx.send(GuiEvent::Error(format!("Failed to send NICK: {}", e)));
                            } else {
                                let old_nick = current_nick.clone();
                                current_nick = newnick.clone();
                                let _ = event_tx.send(GuiEvent::NickChanged { old: old_nick, new: newnick.clone() });
                            }
                        } else {
                            let _ = event_tx.send(GuiEvent::Error("Not connected".into()));
                        }
                    }
                    BackendAction::Quit(reason) => {
                        if let Some(ref mut t) = transport {
                            let quit_msg = if let Some(r) = reason {
                                Message::quit_with_message(&r)
                            } else {
                                Message::quit()
                            };
                            let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", quit_msg)));
                            let _ = t.write_message(&quit_msg).await;
                        }
                        transport = None;
                        let _ = event_tx.send(GuiEvent::Disconnected("User quit".into()));
                    }
                    
                    BackendAction::SendMessage { target, text } => {
                        if let Some(ref mut t) = transport {
                            let privmsg = Message::privmsg(&target, &text);
                            let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", privmsg)));
                            if let Err(e) = t.write_message(&privmsg).await {
                                let _ = event_tx.send(GuiEvent::Error(format!("Failed to send: {}", e)));
                            } else {
                                // Echo our own message to the UI
                                let _ = event_tx.send(GuiEvent::MessageReceived {
                                    target: target.clone(),
                                    sender: current_nick.clone(),
                                    text,
                                });
                            }
                        }
                    }
                }
            }

            // Read from the network (with short timeout so we can check for actions)
            if let Some(ref mut t) = transport {
                match timeout(Duration::from_millis(50), t.read_message()).await {
                    Ok(Ok(Some(message))) => {
                        let _ = event_tx.send(GuiEvent::RawMessage(format!("← {}", message)));
                        
                        match &message.command {
                            // PING -> PONG
                            Command::PING(server, _) => {
                                let pong = Message::pong(server);
                                let _ = t.write_message(&pong).await;
                                let _ = event_tx.send(GuiEvent::RawMessage(format!("→ {}", pong)));
                            }
                            
                            // RPL_WELCOME (001) - Registration complete
                            Command::Response(code, _) if code.code() == 1 => {
                                let _ = event_tx.send(GuiEvent::Connected);
                            }
                            
                            // RPL_TOPIC (332)
                            Command::Response(code, args) if code.code() == 332 => {
                                if args.len() >= 3 {
                                    let _ = event_tx.send(GuiEvent::Topic {
                                        channel: args[1].clone(),
                                        topic: args[2].clone(),
                                    });
                                }
                            }
                            
                            // RPL_NAMREPLY (353)
                            Command::Response(code, args) if code.code() == 353 => {
                                if args.len() >= 4 {
                                    let channel = args[2].clone();
                                    let names: Vec<String> = args[3]
                                        .split_whitespace()
                                        .map(|s| s.trim_start_matches(&['@', '+', '%', '&', '~'][..]).to_string())
                                        .collect();
                                    let _ = event_tx.send(GuiEvent::Names { channel, names });
                                }
                            }
                            
                            // RPL_MOTD (372) and RPL_MOTDSTART (375)
                            Command::Response(code, args) if code.code() == 372 || code.code() == 375 => {
                                if let Some(text) = args.last() {
                                    let _ = event_tx.send(GuiEvent::Motd(text.clone()));
                                }
                            }
                            
                            // PRIVMSG
                            Command::PRIVMSG(target, text) => {
                                let sender = message.source_nickname().unwrap_or("unknown").to_string();
                                let _ = event_tx.send(GuiEvent::MessageReceived {
                                    target: target.clone(),
                                    sender,
                                    text: text.clone(),
                                });
                            }
                            
                            // NOTICE
                            Command::NOTICE(target, text) => {
                                let sender = message.source_nickname().unwrap_or("server").to_string();
                                let _ = event_tx.send(GuiEvent::MessageReceived {
                                    target: target.clone(),
                                    sender: format!("-{}-", sender),
                                    text: text.clone(),
                                });
                            }
                            
                            // JOIN
                            Command::JOIN(channel, _, _) => {
                                let nick = message.source_nickname().unwrap_or("").to_string();
                                if nick == current_nick {
                                    let _ = event_tx.send(GuiEvent::JoinedChannel(channel.clone()));
                                } else {
                                    let _ = event_tx.send(GuiEvent::UserJoined {
                                        channel: channel.clone(),
                                        nick,
                                    });
                                }
                            }
                            
                            // PART
                            Command::PART(channel, msg) => {
                                let nick = message.source_nickname().unwrap_or("").to_string();
                                if nick == current_nick {
                                    let _ = event_tx.send(GuiEvent::PartedChannel(channel.clone()));
                                } else {
                                    let _ = event_tx.send(GuiEvent::UserParted {
                                        channel: channel.clone(),
                                        nick,
                                        message: msg.clone(),
                                    });
                                }
                            }
                            
                            // QUIT - could update user lists
                            Command::QUIT(_) => {
                                // We could track this, but for simplicity we'll skip it
                            }
                            
                            // ERROR from server
                            Command::ERROR(msg) => {
                                let _ = event_tx.send(GuiEvent::Error(msg.clone()));
                            }
                            
                            _ => {}
                        }
                    }
                    Ok(Ok(None)) => {
                        // Connection closed
                        transport = None;
                        let _ = event_tx.send(GuiEvent::Disconnected("Connection closed by server".into()));
                    }
                    Ok(Err(e)) => {
                        let _ = event_tx.send(GuiEvent::Error(format!("Read error: {:?}", e)));
                        transport = None;
                        let _ = event_tx.send(GuiEvent::Disconnected("Read error".into()));
                    }
                    Err(_) => {
                        // Timeout - this is normal, just loop
                    }
                }
            } else {
                // No connection, sleep a bit to avoid busy-looping
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    });
}

// ============================================================================
// GUI State and Application
// ============================================================================

/// Represents a single buffer (channel, query, or system)
#[derive(Default)]
struct Buffer {
    messages: Vec<(String, String, String)>, // (timestamp, sender, text)
    users: Vec<String>,
    topic: String,
    unread: usize,
    has_mention: bool,
}

struct SlircApp {
    // Connection settings
    server_input: String,
    nickname_input: String,
    is_connected: bool,
    
    // Channels for backend communication
    action_tx: Sender<BackendAction>,
    event_rx: Receiver<GuiEvent>,
    
    // UI State
    buffers: HashMap<String, Buffer>,
    buffers_order: Vec<String>,
    active_buffer: String,
    channel_input: String,
    message_input: String,
    
    // System log
    system_log: Vec<String>,
    // Input history
    history: Vec<String>,
    history_pos: Option<usize>,
    history_saved_input: Option<String>,
}

// Default configuration
const DEFAULT_SERVER: &str = "irc.slirc.net:6667";
const DEFAULT_CHANNEL: &str = "#straylight";

impl SlircApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create channels for UI <-> Backend
        let (action_tx, action_rx) = unbounded::<BackendAction>();
        let (event_tx, event_rx) = unbounded::<GuiEvent>();
        
        // Spawn the backend thread
        thread::spawn(move || {
            run_backend(action_rx, event_tx);
        });
        
        let mut app = Self {
            server_input: DEFAULT_SERVER.into(),
            nickname_input: "slirc_user".into(),
            is_connected: false,
            
            action_tx,
            event_rx,
            
            buffers: HashMap::new(),
            active_buffer: "System".into(),
            channel_input: DEFAULT_CHANNEL.into(),
            message_input: String::new(),
            
            system_log: vec!["Welcome to SLIRC!".into()],
            history: Vec::new(),
            history_pos: None,
            history_saved_input: None,
            buffers_order: vec!["System".into()],
        };
        
        // Create the System buffer
        app.buffers.insert("System".into(), Buffer::default());
        
        app
    }

    fn ensure_buffer(&mut self, name: &str) -> &mut Buffer {
        if !self.buffers.contains_key(name) {
            self.buffers.insert(name.to_string(), Buffer::default());
            // keep insertion order
            if !self.buffers_order.contains(&name.to_string()) {
                self.buffers_order.push(name.to_string());
            }
        }
        self.buffers.get_mut(name).unwrap()
    }
    
    fn handle_user_command(&mut self) -> bool {
        let s = self.message_input.trim();
        if !s.starts_with('/') {
            return false;
        }

        // Remove leading '/'
        let cmdline = s[1..].trim();
        let mut parts = cmdline.split_whitespace();
        let cmd = parts.next().unwrap_or("").to_lowercase();
        match cmd.as_str() {
            "join" | "j" => {
                if let Some(chan) = parts.next() {
                    let channel = if chan.starts_with('#') || chan.starts_with('&') {
                        chan.to_string()
                    } else {
                        format!("#{}", chan)
                    };
                    let _ = self.action_tx.send(BackendAction::Join(channel));
                } else {
                    self.system_log.push("Usage: /join <channel>".into());
                }
            }
            "part" | "p" => {
                if let Some(chan) = parts.next() {
                    let channel = if chan.starts_with('#') || chan.starts_with('&') {
                        chan.to_string()
                    } else {
                        format!("#{}", chan)
                    };
                    let reason = parts.collect::<Vec<_>>().join(" ");
                    let _ = self.action_tx.send(BackendAction::Part { channel, message: if reason.is_empty() { None } else { Some(reason) } });
                } else {
                    // If no channel was provided, part the active buffer if it's a channel
                    if self.active_buffer.starts_with('#') || self.active_buffer.starts_with('&') {
                        let channel = self.active_buffer.clone();
                        let reason = parts.collect::<Vec<_>>().join(" ");
                        let _ = self.action_tx.send(BackendAction::Part { channel, message: if reason.is_empty() { None } else { Some(reason) } });
                    } else {
                        self.system_log.push("Usage: /part <channel>".into());
                    }
                }
            }
            "msg" | "privmsg" => {
                if let Some(target) = parts.next() {
                    let text = parts.collect::<Vec<_>>().join(" ");
                    if text.is_empty() {
                        self.system_log.push("Usage: /msg <target> <message>".into());
                    } else {
                        let target = target.to_string();
                        let _ = self.action_tx.send(BackendAction::SendMessage { target, text });
                    }
                } else {
                    self.system_log.push("Usage: /msg <target> <message>".into());
                }
            }
            "me" => {
                let text = parts.collect::<Vec<_>>().join(" ");
                if text.is_empty() {
                    self.system_log.push("Usage: /me <action>".into());
                } else {
                    // Use ACTION CTCP encoding
                    let action_text = format!("\x01ACTION {}\x01", text);
                    // Send to active buffer
                    if self.active_buffer != "System" {
                        let target = self.active_buffer.clone();
                        let _ = self.action_tx.send(BackendAction::SendMessage { target, text: action_text });
                    } else {
                        self.system_log.push("/me can only be used in a channel or PM".into());
                    }
                }
            }
            "nick" => {
                if let Some(newnick) = parts.next() {
                    // Update locally and send to server
                    self.nickname_input = newnick.to_string();
                    let _ = self.action_tx.send(BackendAction::Nick(newnick.to_string()));
                } else {
                    self.system_log.push("Usage: /nick <newnick>".into());
                }
            }
            "quit" | "exit" => {
                let reason = parts.collect::<Vec<_>>().join(" ");
                let _ = self.action_tx.send(BackendAction::Quit(if reason.is_empty() { None } else { Some(reason) }));
            }
            "help" => {
                self.system_log.push("Supported commands: /join, /part, /msg, /me, /nick, /quit".into());
            }
            unknown => {
                self.system_log.push(format!("Unknown command: /{}", unknown));
            }
        }
        true
    }
    
    fn process_events(&mut self) {
        // Drain all pending events from the backend
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                GuiEvent::Connected => {
                    self.is_connected = true;
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    self.system_log.push(format!("[{}] ✓ Connected and registered!", ts));
                }
                
                GuiEvent::Disconnected(reason) => {
                    self.is_connected = false;
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    self.system_log.push(format!("[{}] ✗ Disconnected: {}", ts, reason));
                }
                
                GuiEvent::Error(msg) => {
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    self.system_log.push(format!("[{}] ⚠ Error: {}", ts, msg));
                }
                GuiEvent::NickChanged { old, new } => {
                    // Update user lists in all buffers where the old nick existed
                    for (buffer_name, buffer) in self.buffers.iter_mut() {
                        if buffer.users.iter().any(|u| u == &old) {
                            for user in buffer.users.iter_mut() {
                                if user == &old {
                                    *user = new.clone();
                                }
                            }
                            let ts = Local::now().format("%H:%M:%S").to_string();
                            buffer.messages.push((ts.clone(), "*".into(), format!("{} is now known as {}", old, new)));
                            // If buffer not active, mark unread
                            if *buffer_name != self.active_buffer {
                                buffer.unread += 1;
                            }
                        }
                    }
                    // Update the UI nickname field when the server acknowledges it
                    self.nickname_input = new.clone();
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    self.system_log.push(format!("[{}] Nick changed to {} (was: {})", ts, new, old));
                }
                
                GuiEvent::RawMessage(msg) => {
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    self.system_log.push(format!("[{}] {}", ts, msg));
                    // Keep log from growing too large
                    if self.system_log.len() > 500 {
                        self.system_log.remove(0);
                    }
                }
                
                GuiEvent::MessageReceived { target, sender, text } => {
                    // If it's a PM, the target is the sender (for display)
                    let buffer_name = if target.starts_with('#') || target.starts_with('&') {
                        target.clone()
                    } else {
                        // Private message - use sender as buffer name
                        sender.clone()
                    };

                    let ts = Local::now().format("%H:%M:%S").to_string();
                    let mention = text.contains(&self.nickname_input);
                    let is_own_msg = sender == self.nickname_input;
                    let active = self.active_buffer.clone();
                    let buffer = self.ensure_buffer(&buffer_name);
                    buffer.messages.push((ts.clone(), sender.clone(), text.clone()));
                    // Keep user list updated if a new nick speaks
                    if buffer_name.starts_with('#') || buffer_name.starts_with('&') {
                        if !buffer.users.contains(&sender) {
                            buffer.users.push(sender.clone());
                            buffer.users.sort();
                        }
                    }
                    if !is_own_msg && buffer_name != active {
                        buffer.unread += 1;
                        if mention {
                            buffer.has_mention = true;
                        }
                    }
                }
                
                GuiEvent::JoinedChannel(channel) => {
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    self.system_log.push(format!("[{}] ✓ Joined {}", ts, channel));
                    let buffer = self.ensure_buffer(&channel);
                    buffer.unread = 0;
                    buffer.has_mention = false;
                    self.active_buffer = channel;
                }
                
                GuiEvent::PartedChannel(channel) => {
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    self.system_log.push(format!("[{}] ← Left {}", ts, channel));
                    self.buffers.remove(&channel);
                    self.buffers_order.retain(|b| b != &channel);
                    if self.active_buffer == channel {
                        self.active_buffer = "System".into();
                    }
                }
                
                GuiEvent::UserJoined { channel, nick } => {
                    let active = self.active_buffer.clone();
                    let buffer = self.ensure_buffer(&channel);
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    buffer.messages.push((ts.clone(), "→".into(), format!("{} joined", nick)));
                    if !buffer.users.contains(&nick) {
                        buffer.users.push(nick.clone());
                        buffer.users.sort();
                    }
                    if active != channel {
                        buffer.unread += 1;
                    }
                }
                
                GuiEvent::UserParted { channel, nick, message } => {
                    let active = self.active_buffer.clone();
                    let buffer = self.ensure_buffer(&channel);
                    let msg = message.map(|m| format!(" ({})", m)).unwrap_or_default();
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    buffer.messages.push((ts.clone(), "←".into(), format!("{} left{}", nick, msg)));
                    buffer.users.retain(|u| u != &nick);
                    if active != channel {
                        buffer.unread += 1;
                    }
                }
                
                GuiEvent::Motd(line) => {
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    self.system_log.push(format!("[{}] MOTD: {}", ts, line));
                }
                
                GuiEvent::Topic { channel, topic } => {
                    let active = self.active_buffer.clone();
                    let buffer = self.ensure_buffer(&channel);
                    buffer.topic = topic.clone();
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    buffer.messages.push((ts.clone(), "*".into(), format!("Topic: {}", topic)));
                    if active != channel {
                        buffer.unread += 1;
                    }
                }
                
                GuiEvent::Names { channel, names } => {
                    let buffer = self.ensure_buffer(&channel);
                    buffer.users = names;
                    buffer.users.sort();
                }
            }
        }
    }
}

impl eframe::App for SlircApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process network events
        self.process_events();
        
        // Request repaint to keep checking for events
        ctx.request_repaint_after(Duration::from_millis(100));
        
        // Top panel: Connection controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Server:");
                ui.add_enabled(
                    !self.is_connected,
                    egui::TextEdit::singleline(&mut self.server_input).desired_width(200.0),
                );
                
                ui.label("Nick:");
                ui.add_enabled(
                    !self.is_connected,
                    egui::TextEdit::singleline(&mut self.nickname_input).desired_width(100.0),
                );
                
                if !self.is_connected {
                    if ui.button("Connect").clicked() {
                        // Parse server:port
                        let parts: Vec<&str> = self.server_input.split(':').collect();
                        let server = parts[0].to_string();
                        let port: u16 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(6667);
                        
                        let _ = self.action_tx.send(BackendAction::Connect {
                            server,
                            port,
                            nickname: self.nickname_input.clone(),
                            username: self.nickname_input.clone(),
                            realname: format!("SLIRC User ({})", self.nickname_input),
                        });
                    }
                } else {
                    if ui.button("Disconnect").clicked() {
                        let _ = self.action_tx.send(BackendAction::Disconnect);
                    }
                    // Optionally change nick while connected
                    if ui.button("Change Nick").clicked() {
                        let _ = self.action_tx.send(BackendAction::Nick(self.nickname_input.clone()));
                    }
                    
                    ui.separator();
                    
                    // Join channel controls
                    ui.label("Channel:");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.channel_input).desired_width(100.0),
                    );
                    
                    if ui.button("+").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        let channel = if self.channel_input.starts_with('#') {
                            self.channel_input.clone()
                        } else {
                            format!("#{}", self.channel_input)
                        };
                        let _ = self.action_tx.send(BackendAction::Join(channel));
                        self.channel_input.clear();
                    }
                }
            });
        });
        
        // Left panel: Buffer list (vertical tabs similar to HexChat)
        egui::SidePanel::left("buffers_panel")
            .resizable(true)
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.heading("Buffers");
                ui.separator();
                // Show buffers in order
                ui.vertical(|ui| {
                    for name in self.buffers_order.clone() {
                        // Snapshot of the buffer's state to avoid borrow conflicts
                        let (unread, has_mention, _users_len, selected) = if let Some(b) = self.buffers.get(&name) {
                            (b.unread, b.has_mention, b.users.len(), self.active_buffer == name)
                        } else {
                            (0, false, 0, false)
                        };

                        ui.horizontal(|ui| {
                            let rich = if has_mention {
                                egui::RichText::new(name.clone()).color(egui::Color32::LIGHT_RED).strong()
                            } else if selected {
                                egui::RichText::new(name.clone()).color(egui::Color32::WHITE).strong()
                            } else {
                                egui::RichText::new(name.clone()).color(egui::Color32::LIGHT_GRAY)
                            };

                            if ui.selectable_label(selected, rich).clicked() {
                                self.active_buffer = name.clone();
                                if let Some(mut_b) = self.buffers.get_mut(&name) {
                                    mut_b.unread = 0;
                                    mut_b.has_mention = false;
                                }
                            }

                            if unread > 0 {
                                ui.label(egui::RichText::new(format!("({})", unread)).color(egui::Color32::LIGHT_BLUE));
                            }
                            if name != "System" {
                                if ui.small_button("x").clicked() {
                                    // send part
                                    let _ = self.action_tx.send(BackendAction::Part { channel: name.clone(), message: None });
                                    // Also remove from our local mapping immediately
                                    self.buffers.remove(&name);
                                    self.buffers_order.retain(|b| b != &name);
                                    if self.active_buffer == name { self.active_buffer = "System".into(); }
                                }
                            }
                        });
                    }
                });
            });
        
        // Right panel: User list (for channels)
        if self.active_buffer.starts_with('#') || self.active_buffer.starts_with('&') {
            egui::SidePanel::right("users_panel")
                .resizable(true)
                .default_width(120.0)
                .show(ctx, |ui| {
                    ui.heading("Users");
                    ui.separator();
                    
                    if let Some(buffer) = self.buffers.get(&self.active_buffer) {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for user in &buffer.users {
                                ui.label(user);
                            }
                        });
                    }
                });
        }
        
        // Bottom panel: Message input
        egui::TopBottomPanel::bottom("input_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.message_input)
                        .desired_width(ui.available_width() - 60.0)
                        .hint_text("Type a message..."),
                );
                
                let send_clicked = ui.button("Send").clicked();
                let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                // Input history navigation
                if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    if !self.history.is_empty() {
                        if self.history_pos.is_none() {
                            // store current text to restore if user navigates back
                            self.history_saved_input = Some(self.message_input.clone());
                            self.history_pos = Some(self.history.len() - 1);
                        } else if let Some(pos) = self.history_pos {
                            if pos > 0 {
                                self.history_pos = Some(pos - 1);
                            }
                        }
                        if let Some(pos) = self.history_pos {
                            if let Some(h) = self.history.get(pos) {
                                self.message_input = h.clone();
                            }
                        }
                    }
                }
                if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    if let Some(pos) = self.history_pos {
                        if pos + 1 < self.history.len() {
                            self.history_pos = Some(pos + 1);
                            if let Some(h) = self.history.get(pos + 1) {
                                self.message_input = h.clone();
                            }
                        } else {
                            // Exit history navigation
                            self.history_pos = None;
                            self.message_input = self.history_saved_input.take().unwrap_or_default();
                        }
                    }
                }

                if (send_clicked || enter_pressed) && !self.message_input.is_empty() {
                    // If it begins with a slash, treat as a command
                    if self.message_input.starts_with('/') {
                        if self.handle_user_command() {
                            self.history.push(self.message_input.clone());
                        }
                    } else {
                        // Normal message
                        if self.is_connected {
                            if self.active_buffer != "System" {
                                let _ = self.action_tx.send(BackendAction::SendMessage {
                                    target: self.active_buffer.clone(),
                                    text: self.message_input.clone(),
                                });
                                self.history.push(self.message_input.clone());
                            }
                        } else {
                            let ts = Local::now().format("%H:%M:%S").to_string();
                            self.system_log.push(format!("[{}] ⚠ Not connected: message not sent", ts));
                        }
                    }

                    // Reset history navigation and input
                    self.history_pos = None;
                    self.history_saved_input = None;
                    self.message_input.clear();
                    response.request_focus();
                }
            });
        });
        
        // Central panel: Messages and header
        egui::CentralPanel::default().show(ctx, |ui| {
            // Header: active buffer and topic
            ui.horizontal(|ui| {
                ui.heading(&self.active_buffer);
                if let Some(buffer) = self.buffers.get(&self.active_buffer) {
                    if !buffer.topic.is_empty() {
                        ui.separator();
                        ui.colored_label(egui::Color32::LIGHT_YELLOW, &buffer.topic);
                    }
                    // show user count
                    ui.separator();
                    ui.label(format!("Users: {}", buffer.users.len()));
                    if buffer.unread > 0 {
                        ui.colored_label(egui::Color32::LIGHT_BLUE, format!("Unread: {}", buffer.unread));
                    }
                    if buffer.has_mention { ui.colored_label(egui::Color32::LIGHT_RED, "Mention"); }
                }
            });
            ui.separator();
            // Show topic if there is one (keep backward compatibility)
            if let Some(buffer) = self.buffers.get(&self.active_buffer) {
                if !buffer.topic.is_empty() {
                    // (topic already displayed in header)
                }
            }
            
            // Messages area
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if self.active_buffer == "System" {
                        // Show system log
                        for line in &self.system_log {
                            ui.label(line);
                        }
                    } else if let Some(buffer) = self.buffers.get(&self.active_buffer) {
                        for (ts, sender, text) in &buffer.messages {
                            let mention = text.contains(&self.nickname_input);
                            if sender == &self.nickname_input {
                                // Align own messages to the right
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                    ui.label(egui::RichText::new(text).color(egui::Color32::from_rgb(80, 200, 120)));
                                    ui.label(
                                        egui::RichText::new(format!("<{}>", sender))
                                            .color(egui::Color32::LIGHT_BLUE)
                                            .strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("[{}]", ts))
                                        .color(egui::Color32::LIGHT_GRAY),
                                    );
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("[{}]", ts))
                                            .color(egui::Color32::LIGHT_GRAY),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("<{}>", sender))
                                            .color(egui::Color32::LIGHT_BLUE)
                                            .strong(),
                                    );
                                    if mention {
                                        ui.label(egui::RichText::new(text).color(egui::Color32::LIGHT_RED));
                                    } else {
                                        ui.label(text);
                                    }
                                });
                            }
                        }
                    }
                });
        });
    }
}

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
