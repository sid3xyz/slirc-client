use chrono::Local;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;

use crate::backend::run_backend;
use crate::buffer::ChannelBuffer;
use crate::commands;
use crate::config::{
    load_settings, save_settings, Network, Settings, DEFAULT_CHANNEL, DEFAULT_SERVER,
};
use crate::events;
use crate::protocol::{BackendAction, GuiEvent};
use crate::ui;
use crate::ui::dialogs::NetworkForm;

pub struct SlircApp {
    // Connection settings
    pub server_input: String,
    pub nickname_input: String,
    pub is_connected: bool,
    pub use_tls: bool,

    // Channels for backend communication
    pub action_tx: Sender<BackendAction>,
    pub event_rx: Receiver<GuiEvent>,

    // UI State - HexChat style
    pub buffers: HashMap<String, ChannelBuffer>,
    pub buffers_order: Vec<String>,
    pub active_buffer: String,
    pub channel_input: String,
    pub message_input: String,

    // System log
    pub system_log: Vec<String>,
    // Input history
    pub history: Vec<String>,
    pub history_pos: Option<usize>,
    pub history_saved_input: Option<String>,
    // Context menus and floating windows
    pub context_menu_visible: bool,
    pub context_menu_target: Option<String>,
    pub open_windows: HashSet<String>,
    // topic editor dialog state: which channel (if any) we're currently editing
    pub topic_editor_open: Option<String>,
    // Tab completion state
    pub completions: Vec<String>,
    pub completion_index: Option<usize>,
    pub completion_prefix: Option<String>,
    pub completion_target_channel: bool,
    pub last_input_text: String,
    pub theme: String,
    // If we loaded a fallback font from the system, store it here
    pub font_fallback: Option<String>,
    // Network management
    pub networks: Vec<Network>,
    pub network_manager_open: bool,
    pub editing_network: Option<usize>, // Index of network being edited, None = new
    pub network_form: NetworkForm,
    // UI visibility toggles
    pub show_channel_list: bool,
    pub show_user_list: bool,
    pub expanded_networks: HashSet<String>,
    pub show_help_dialog: bool,
    pub nick_change_dialog_open: bool,
    pub nick_change_input: String,
    // Status messages (toasts)
    pub status_messages: Vec<(String, std::time::Instant)>,
}

impl SlircApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Create channels for UI <-> Backend
        let (action_tx, action_rx) = unbounded::<BackendAction>();
        let (event_tx, event_rx) = unbounded::<GuiEvent>();

        // Spawn the backend thread
        thread::spawn(move || {
            run_backend(action_rx, event_tx);
        });
        // Try to load persisted settings and apply theme in creation context
        let settings = load_settings();
        if let Some(s) = &settings {
            match s.theme.as_str() {
                "light" => cc.egui_ctx.set_visuals(egui::Visuals::light()),
                _ => cc.egui_ctx.set_visuals(egui::Visuals::dark()),
            }
        }

        // Load professional IRC-appropriate fonts
        // Prioritize monospace fonts that render well for IRC (like HexChat)
        let mut fonts = egui::FontDefinitions::default();

        // Candidate font paths (ordered by quality for IRC)
        let candidates = [
            // Linux - prefer monospace fonts for IRC
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/liberation-mono/LiberationMono-Regular.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansMono-Regular.ttf",
            "/usr/share/fonts/truetype/ubuntu/UbuntuMono-R.ttf",
            "/usr/share/fonts/truetype/hack/Hack-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            // macOS
            "/System/Library/Fonts/Monaco.ttf",
            "/Library/Fonts/Courier New.ttf",
            "/System/Library/Fonts/Menlo.ttc",
            // Windows
            "C:\\Windows\\Fonts\\consola.ttf", // Consolas
            "C:\\Windows\\Fonts\\cour.ttf",    // Courier New
        ];

        let mut chosen_font: Option<String> = None;
        for path in candidates.iter() {
            let p = std::path::Path::new(path);
            if p.exists() {
                if let Ok(bytes) = std::fs::read(p) {
                    fonts.font_data.insert(
                        "irc_font".to_owned(),
                        egui::FontData::from_owned(bytes).into(),
                    );
                    // Use as primary font for better rendering
                    if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                        proportional.insert(0, "irc_font".to_owned());
                    }
                    if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                        monospace.insert(0, "irc_font".to_owned());
                    }
                    cc.egui_ctx.set_fonts(fonts);
                    chosen_font = Some(path.to_string());
                    break;
                }
            }
        }

        // Set better font sizes for IRC (slightly larger for readability)
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Small,
                egui::FontId::new(10.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(13.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(13.0, egui::FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(12.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Heading,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();
        cc.egui_ctx.set_style(style);

        let mut app = Self {
            server_input: DEFAULT_SERVER.into(),
            nickname_input: "slirc_user".into(),
            is_connected: false,
            use_tls: false,

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
            completions: Vec::new(),
            completion_index: None,
            completion_prefix: None,
            completion_target_channel: false,
            last_input_text: String::new(),
            theme: "dark".into(), // Default theme
            context_menu_visible: false,
            context_menu_target: None,
            open_windows: HashSet::new(),
            buffers_order: vec!["System".into()],
            font_fallback: chosen_font,
            topic_editor_open: None,
            networks: Vec::new(),
            network_manager_open: false,
            editing_network: None,
            network_form: NetworkForm::default(),
            show_channel_list: true,
            show_user_list: true,
            expanded_networks: HashSet::new(),
            show_help_dialog: false,
            nick_change_dialog_open: false,
            nick_change_input: String::new(),
            status_messages: Vec::new(),
        };

        // Create the System buffer
        app.buffers.insert("System".into(), ChannelBuffer::new());
        // Restore settings if present
        if let Some(s) = settings {
            if !s.server.is_empty() {
                app.server_input = s.server;
            }
            if !s.nick.is_empty() {
                app.nickname_input = s.nick;
            }
            if !s.history.is_empty() {
                app.history = s.history;
            }
            if !s.default_channel.is_empty() {
                app.channel_input = s.default_channel;
            }
            if !s.theme.is_empty() {
                app.theme = s.theme;
            }
            app.networks = s.networks.clone();

            // Auto-connect to networks with auto_connect flag
            for network in &s.networks {
                if network.auto_connect {
                    if let Some(server_addr) = network.servers.first() {
                        let parts: Vec<&str> = server_addr.split(':').collect();
                        let server = parts[0].to_string();
                        let port: u16 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(6667);

                        let _ = app.action_tx.send(BackendAction::Connect {
                            server,
                            port,
                            nickname: network.nick.clone(),
                            username: network.nick.clone(),
                            realname: format!("SLIRC User ({})", network.nick),
                            use_tls: network.use_tls,
                        });

                        // Auto-join favorite channels
                        for channel in &network.favorite_channels {
                            let _ = app.action_tx.send(BackendAction::Join(channel.clone()));
                        }

                        // Only auto-connect to the first network with the flag
                        break;
                    }
                }
            }
        }
        app
    }

    fn save_networks(&self) {
        let settings = Settings {
            server: self.server_input.clone(),
            nick: self.nickname_input.clone(),
            default_channel: self.channel_input.clone(),
            history: self.history.clone(),
            theme: self.theme.clone(),
            networks: self.networks.clone(),
        };
        if let Err(e) = save_settings(&settings) {
            eprintln!("Failed to save networks: {}", e);
        }
    }

    fn collect_completions(&self, prefix: &str) -> Vec<String> {
        let mut matches: Vec<String> = Vec::new();
        let mut search_prefix = prefix;
        let mut keep_lead = "";
        if let Some(stripped) = prefix.strip_prefix('@') {
            // Keep the '@' in the suggestion, but search without it
            search_prefix = stripped;
            keep_lead = "@";
        }
        if prefix.starts_with('#') || prefix.starts_with('&') {
            // channel completions
            for b in &self.buffers_order {
                if b.starts_with(prefix) {
                    matches.push(b.clone());
                }
            }
        } else {
            // user completions from active buffer
            if let Some(buffer) = self.buffers.get(&self.active_buffer) {
                for u in &buffer.users {
                    if u.nick.starts_with(search_prefix) {
                        matches.push(format!("{}{}", keep_lead, u.nick.clone()));
                    }
                }
            }
            // also add channel names for messages starting with '#'
            for b in &self.buffers_order {
                if b.starts_with(prefix) {
                    matches.push(b.clone());
                }
            }
        }
        matches.sort();
        matches.dedup();
        matches
    }

    fn apply_completion(
        &mut self,
        completion: &str,
        last_word_start: usize,
        _last_word_end: usize,
    ) {
        // Replace last token in message_input with completion
        // If this was the first token in the message, add a trailing ': ' similar to HexChat
        let is_first_token = self.message_input[..last_word_start].trim().is_empty();
        let suffix = if is_first_token { ": " } else { " " };
        let before = &self.message_input[..last_word_start];
        self.message_input = format!("{}{}{}", before, completion, suffix);
        // reset history navigation when using completions
        self.history_pos = None;
        self.history_saved_input = None;
    }

    fn current_last_word_bounds(&self) -> (usize, usize) {
        // returns (start_idx, end_idx) of the last word in message_input
        let idx = self
            .message_input
            .rfind(|c: char| c.is_whitespace())
            .map_or(0, |i| i + 1);
        (idx, self.message_input.len())
    }

    fn cycle_completion(&mut self, direction: isize) -> bool {
        if self.completions.is_empty() {
            return false;
        }
        if let Some(idx) = self.completion_index {
            let len = self.completions.len();
            let next_idx = ((idx as isize + direction).rem_euclid(len as isize)) as usize;
            self.completion_index = Some(next_idx);
            let comp = self.completions[next_idx].clone();
            let (start, end) = self.current_last_word_bounds();
            self.apply_completion(&comp, start, end);
            true
        } else {
            // start cycling
            self.completion_index = Some(0);
            if let Some(comp) = self.completions.first() {
                let comp = comp.clone();
                let (start, end) = self.current_last_word_bounds();
                self.apply_completion(&comp, start, end);
                true
            } else {
                false
            }
        }
    }

    pub fn process_events(&mut self) {
        events::process_events(
            &self.event_rx,
            &mut self.is_connected,
            &mut self.buffers,
            &mut self.buffers_order,
            &mut self.active_buffer,
            &mut self.nickname_input,
            &mut self.system_log,
            &mut self.expanded_networks,
            &mut self.status_messages,
            &self.server_input,
            &self.font_fallback,
        );
    }
}

impl eframe::App for SlircApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process network events
        self.process_events();

        // Global keyboard shortcuts (work even when input doesn't have focus)
        ctx.input(|i| {
            // Ctrl+N: Next channel
            if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
                if let Some(current_idx) = self
                    .buffers_order
                    .iter()
                    .position(|b| b == &self.active_buffer)
                {
                    let next_idx = (current_idx + 1) % self.buffers_order.len();
                    if let Some(next_buffer) = self.buffers_order.get(next_idx) {
                        self.active_buffer = next_buffer.clone();
                        if let Some(buffer) = self.buffers.get_mut(next_buffer) {
                            buffer.clear_unread();
                            buffer.has_highlight = false;
                        }
                    }
                }
            }
            // Ctrl+K: Previous channel (or Ctrl+P for "previous")
            if i.modifiers.ctrl && (i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::P)) {
                if let Some(current_idx) = self
                    .buffers_order
                    .iter()
                    .position(|b| b == &self.active_buffer)
                {
                    let prev_idx = if current_idx == 0 {
                        self.buffers_order.len() - 1
                    } else {
                        current_idx - 1
                    };
                    if let Some(prev_buffer) = self.buffers_order.get(prev_idx) {
                        self.active_buffer = prev_buffer.clone();
                        if let Some(buffer) = self.buffers.get_mut(prev_buffer) {
                            buffer.clear_unread();
                            buffer.has_highlight = false;
                        }
                    }
                }
            }
            // F1: Toggle help dialog
            if i.key_pressed(egui::Key::F1) {
                self.show_help_dialog = !self.show_help_dialog;
            }
        });

        // Request repaint to keep checking for events
        ctx.request_repaint_after(Duration::from_millis(100));
        // Purge old status messages (toasts) older than 4 seconds
        self.status_messages
            .retain(|(_, t)| t.elapsed() < std::time::Duration::from_secs(4));

        // Single compact toolbar (HexChat style - no separate menu bar)
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui::toolbar::render_toolbar(
                ui,
                ctx,
                &mut self.server_input,
                &mut self.nickname_input,
                &mut self.channel_input,
                self.is_connected,
                &mut self.use_tls,
                &self.action_tx,
                &mut self.network_manager_open,
                &mut self.nick_change_dialog_open,
                &mut self.nick_change_input,
                &mut self.show_channel_list,
                &mut self.show_user_list,
            );
        });

        // Left panel: Buffer list (vertical tabs similar to HexChat)
        if self.show_channel_list {
            ui::panels::render_channel_list(
                ctx,
                &self.buffers,
                &self.buffers_order,
                &mut self.active_buffer,
                &mut self.context_menu_visible,
                &mut self.context_menu_target,
            );
            // Clear unread after switching buffer
            if let Some(buf) = self.buffers.get_mut(&self.active_buffer) {
                buf.clear_unread();
            }
        }
        // (Removed top horizontal buffer tabs — left navigation is the single source of truth.)

        // Right panel: User list (for channels)
        if self.show_user_list
            && (self.active_buffer.starts_with('#') || self.active_buffer.starts_with('&'))
        {
            if let Some(buffer) = self.buffers.get(&self.active_buffer) {
                ui::panels::render_user_list(
                    ctx,
                    buffer,
                    &self.active_buffer,
                    &self.nickname_input,
                    &mut self.context_menu_visible,
                    &mut self.context_menu_target,
                );
            }
        }

        // Bottom panel: Message input
        egui::TopBottomPanel::bottom("input_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::multiline(&mut self.message_input)
                        .desired_rows(1)
                        .desired_width(ui.available_width())
                        .hint_text("Type a message... (Enter to send, Shift+Enter for newline)"),
                );

                // Detect Enter (without Shift) to send a message. Shift+Enter inserts newline in the
                // multiline text edit by default.
                let enter_pressed = response.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);

                // Input history navigation
                if response.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::ArrowUp))
                    && !self.history.is_empty()
                {
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
                            self.message_input =
                                self.history_saved_input.take().unwrap_or_default();
                        }
                    }
                }

                // Tab completion: Tab cycles forward; Shift+Tab cycles backward
                let tab_pressed =
                    response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Tab));
                let shift = ui.input(|i| i.modifiers.shift);
                if tab_pressed {
                    // compute current prefix (last token)
                    let (start, end) = self.current_last_word_bounds();
                    let prefix = self.message_input[start..end].trim();
                    if self.completions.is_empty() {
                        // first time: gather completions
                        self.completions = self.collect_completions(prefix);
                        self.completion_prefix = Some(prefix.to_string());
                        self.completion_target_channel =
                            prefix.starts_with('#') || prefix.starts_with('&');
                    }
                    if !self.completions.is_empty() {
                        if shift {
                            self.cycle_completion(-1);
                        } else {
                            self.cycle_completion(1);
                        }
                    }
                }

                // Reset completions if the user changed the input text
                if self.last_input_text != self.message_input && !tab_pressed {
                    self.completions.clear();
                    self.completion_index = None;
                    self.completion_prefix = None;
                }
                self.last_input_text = self.message_input.clone();

                // Esc to cancel input (clear the text field)
                if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.message_input.clear();
                    self.history_pos = None;
                    self.history_saved_input = None;
                    self.completions.clear();
                    self.completion_index = None;
                    self.completion_prefix = None;
                }

                if enter_pressed && !self.message_input.is_empty() {
                    // If it begins with a slash, treat as a command
                    if self.message_input.starts_with('/') {
                        if commands::handle_user_command(
                            &self.message_input,
                            &self.active_buffer,
                            &self.buffers,
                            &self.action_tx,
                            &mut self.system_log,
                            &mut self.nickname_input,
                        ) {
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
                            self.system_log
                                .push(format!("[{}] ⚠ Not connected: message not sent", ts));
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

        // Central panel: Messages with dedicated topic bar
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::messages::render_messages(
                ctx,
                ui,
                &self.active_buffer,
                &self.buffers,
                &self.system_log,
                &self.nickname_input,
                &mut self.topic_editor_open,
            );
        });

        // Context menu popup (as a floating window)
        if self.context_menu_visible {
            if let Some(target) = self.context_menu_target.clone() {
                // If the target starts with "user:", this is a user context menu
                if let Some(user) = target.strip_prefix("user:") {
                    egui::Window::new(format!("User: {}", user))
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            if ui.button("Query (PM)").clicked() {
                                // Create or switch to private message buffer
                                if !self.buffers.contains_key(user) {
                                    self.buffers.insert(user.to_string(), ChannelBuffer::new());
                                    self.buffers_order.push(user.to_string());
                                }
                                self.active_buffer = user.to_string();
                                self.context_menu_visible = false;
                            }
                            if ui.button("Whois").clicked() {
                                let _ = self.action_tx.send(BackendAction::Whois(user.to_string()));
                                self.context_menu_visible = false;
                            }
                            if ui.button("Cancel").clicked() {
                                self.context_menu_visible = false;
                            }
                            // Show op actions if we're an op in this channel
                            if self.active_buffer.starts_with('#')
                                || self.active_buffer.starts_with('&')
                            {
                                let is_op = self
                                    .buffers
                                    .get(&self.active_buffer)
                                    .map(|b| {
                                        b.users.iter().any(|u| {
                                            u.nick == self.nickname_input
                                                && ui::theme::prefix_rank(u.prefix) >= 3
                                        })
                                    })
                                    .unwrap_or(false);
                                if is_op {
                                    ui.separator();
                                    ui.label("Op Actions:");
                                    if ui.button("Op (+o)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "+o".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Deop (-o)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "-o".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Voice (+v)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "+v".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Devoice (-v)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "-v".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Kick").clicked() {
                                        let _ = self.action_tx.send(BackendAction::Kick {
                                            channel: self.active_buffer.clone(),
                                            nick: user.to_string(),
                                            reason: None,
                                        });
                                        self.context_menu_visible = false;
                                    }
                                }
                            }
                        });
                } else {
                    egui::Window::new(format!("Actions: {}", target))
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            if ui.button("Part").clicked() {
                                let _ = self.action_tx.send(BackendAction::Part {
                                    channel: target.clone(),
                                    message: None,
                                });
                                self.context_menu_visible = false;
                            }
                            if ui.button("Close").clicked() {
                                self.buffers.remove(&target);
                                self.buffers_order.retain(|b| b != &target);
                                if self.active_buffer == target {
                                    self.active_buffer = "System".into();
                                }
                                self.context_menu_visible = false;
                            }
                            if ui.button("Open in new window").clicked() {
                                self.open_windows.insert(target.clone());
                                self.context_menu_visible = false;
                            }
                            if ui.button("Cancel").clicked() {
                                self.context_menu_visible = false;
                            }
                        });
                }
            }
        }

        // Floating buffer windows
        for open_name in self.open_windows.clone() {
            let mut open = true;
            egui::Window::new(format!("Window: {}", open_name))
                .open(&mut open)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&open_name);
                    });
                    ui.separator();
                    if let Some(buffer) = self.buffers.get(&open_name) {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for msg in &buffer.messages {
                                // Check if this is a CTCP ACTION message
                                if msg.text.starts_with("\x01ACTION ") && msg.text.ends_with('\x01')
                                {
                                    // Extract action text (remove \x01ACTION  and trailing \x01)
                                    let action = &msg.text[8..msg.text.len() - 1];
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("[{}]", msg.timestamp))
                                                .color(egui::Color32::LIGHT_GRAY),
                                        );
                                        ui.label(
                                            egui::RichText::new("*")
                                                .color(egui::Color32::from_rgb(255, 150, 0)),
                                        );
                                        ui.label(
                                            egui::RichText::new(&msg.sender)
                                                .color(ui::theme::nick_color(&msg.sender)),
                                        );
                                        ui.label(
                                            egui::RichText::new(action)
                                                .color(egui::Color32::from_rgb(255, 150, 0))
                                                .italics(),
                                        );
                                    });
                                } else {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("[{}]", msg.timestamp))
                                                .color(egui::Color32::LIGHT_GRAY),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!("<{}>", msg.sender))
                                                .color(egui::Color32::LIGHT_BLUE)
                                                .strong(),
                                        );
                                        ui.label(&msg.text);
                                    });
                                }
                            }
                        });
                    }
                });
            if !open {
                self.open_windows.remove(&open_name);
            }
        }
        // Topic editor window (if open)
        if let Some(channel) = self.topic_editor_open.clone() {
            let mut open = true;
            // Clone the topic string for editing to avoid borrowing self while rendering UI
            let initial_topic = self
                .buffers
                .get(&channel)
                .map(|b| b.topic.clone())
                .unwrap_or_default();
            let mut new_topic = initial_topic.clone();
            egui::Window::new(format!("Edit Topic: {}", channel))
                .open(&mut open)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label("Edit the channel topic:");
                    let _response =
                        ui.add(egui::TextEdit::multiline(&mut new_topic).desired_rows(3));
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if !new_topic.is_empty() {
                                let _ = self.action_tx.send(BackendAction::SetTopic {
                                    channel: channel.clone(),
                                    topic: new_topic.clone(),
                                });
                            }
                            self.topic_editor_open = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.topic_editor_open = None;
                        }
                    });
                });
            if !open {
                self.topic_editor_open = None;
            }
        }

        // Network Manager window
        if self.network_manager_open {
            let mut open = true;
            egui::Window::new("Network Manager")
                .open(&mut open)
                .resizable(true)
                .default_width(500.0)
                .show(ctx, |ui| {
                    ui.heading("Saved Networks");
                    ui.separator();

                    // List of networks
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            let mut to_delete: Option<usize> = None;
                            for (idx, network) in self.networks.iter().enumerate() {
                                let expanded = self.expanded_networks.contains(&network.name);
                                ui.horizontal(|ui| {
                                    if ui.small_button(if expanded { "▾" } else { "▸" }).clicked()
                                    {
                                        if expanded {
                                            self.expanded_networks.remove(&network.name);
                                        } else {
                                            self.expanded_networks.insert(network.name.clone());
                                        }
                                    }
                                    let label = if network.auto_connect {
                                        format!("✓ {}", network.name)
                                    } else {
                                        network.name.clone()
                                    };
                                    ui.label(egui::RichText::new(label).strong());
                                    ui.label(format!("({})", network.servers.join(", ")));

                                    if ui.button("Edit").clicked() {
                                        self.editing_network = Some(idx);
                                        let net = &self.networks[idx];
                                        self.network_form = NetworkForm {
                                            name: net.name.clone(),
                                            servers: net.servers.join(", "),
                                            nick: net.nick.clone(),
                                            auto_connect: net.auto_connect,
                                            favorite_channels: net.favorite_channels.join(", "),
                                            nickserv_password: net
                                                .nickserv_password
                                                .clone()
                                                .unwrap_or_default(),
                                            use_tls: net.use_tls,
                                        };
                                    }

                                    if ui.button("Connect").clicked() {
                                        if let Some(server_addr) = network.servers.first() {
                                            let parts: Vec<&str> = server_addr.split(':').collect();
                                            let server = parts[0].to_string();
                                            let port: u16 = parts
                                                .get(1)
                                                .and_then(|p| p.parse().ok())
                                                .unwrap_or(6667);

                                            let _ = self.action_tx.send(BackendAction::Connect {
                                                server,
                                                port,
                                                nickname: network.nick.clone(),
                                                username: network.nick.clone(),
                                                realname: format!("SLIRC User ({})", network.nick),
                                                use_tls: network.use_tls,
                                            });

                                            // Auto-join favorite channels after a brief delay
                                            // (We should track connection state better, but this is a start)
                                            for channel in &network.favorite_channels {
                                                let _ = self
                                                    .action_tx
                                                    .send(BackendAction::Join(channel.clone()));
                                            }

                                            self.network_manager_open = false;
                                        }
                                    }

                                    if ui.button("Delete").clicked() {
                                        to_delete = Some(idx);
                                    }
                                });
                                // Show details if expanded
                                if expanded {
                                    ui.add_space(6.0);
                                    ui.label(format!("Nick: {}", network.nick));
                                    ui.label(format!(
                                        "Favorites: {}",
                                        network.favorite_channels.join(", ")
                                    ));
                                    ui.separator();
                                }
                            }

                            if let Some(idx) = to_delete {
                                self.networks.remove(idx);
                                self.save_networks();
                            }
                        });

                    ui.separator();

                    // Add/Edit network form
                    if self.editing_network.is_some()
                        || ui.button("Add Network").clicked() && self.editing_network.is_none()
                    {
                        if self.editing_network.is_none() {
                            // Start adding a new network
                            self.network_form = NetworkForm::default();
                        }

                        ui.heading(if self.editing_network.is_some() {
                            "Edit Network"
                        } else {
                            "New Network"
                        });
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.network_form.name);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Servers:");
                            ui.text_edit_singleline(&mut self.network_form.servers);
                        });
                        ui.label(
                            "(Comma-separated, e.g., irc.libera.chat:6667, irc.libera.chat:6697)",
                        );

                        ui.horizontal(|ui| {
                            ui.label("Nickname:");
                            ui.text_edit_singleline(&mut self.network_form.nick);
                        });

                        ui.checkbox(
                            &mut self.network_form.auto_connect,
                            "Auto-connect on startup",
                        );

                        ui.horizontal(|ui| {
                            ui.label("Favorite Channels:");
                            ui.text_edit_singleline(&mut self.network_form.favorite_channels);
                        });
                        ui.label("(Comma-separated, e.g., #channel1, #channel2)");

                        ui.horizontal(|ui| {
                            ui.label("NickServ Password:");
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut self.network_form.nickserv_password,
                                )
                                .password(true),
                            );
                        });
                        ui.label("(Optional, stored in plain text - use with caution!)");

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                let servers: Vec<String> = self
                                    .network_form
                                    .servers
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();

                                let favorite_channels: Vec<String> = self
                                    .network_form
                                    .favorite_channels
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();

                                let network = Network {
                                    name: self.network_form.name.clone(),
                                    servers,
                                    nick: self.network_form.nick.clone(),
                                    auto_connect: self.network_form.auto_connect,
                                    favorite_channels,
                                    nickserv_password: if self
                                        .network_form
                                        .nickserv_password
                                        .is_empty()
                                    {
                                        None
                                    } else {
                                        Some(self.network_form.nickserv_password.clone())
                                    },
                                    use_tls: self.network_form.use_tls,
                                };

                                if let Some(idx) = self.editing_network {
                                    self.networks[idx] = network;
                                } else {
                                    self.networks.push(network);
                                }

                                self.save_networks();
                                self.editing_network = None;
                                self.network_form = NetworkForm::default();
                            }

                            if ui.button("Cancel").clicked() {
                                self.editing_network = None;
                                self.network_form = NetworkForm::default();
                            }
                        });
                    }
                });

            if !open {
                self.network_manager_open = false;
                self.editing_network = None;
            }
        }

        // Floating status toasts (top-right corner)
        if !self.status_messages.is_empty() {
            let msgs: Vec<String> = self
                .status_messages
                .iter()
                .map(|(m, _t)| m.clone())
                .collect();
            egui::Area::new(egui::Id::new("status_toast_area"))
                .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        for m in msgs {
                            ui.label(egui::RichText::new(m).color(egui::Color32::LIGHT_GREEN));
                        }
                    });
                });
        }

        // Help dialog (F1 toggles visibility)
        if self.show_help_dialog {
            let mut open = true;
            egui::Window::new("Help")
                .open(&mut open)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Shortcuts & Commands");
                    ui.separator();
                    ui.label("Keyboard Shortcuts:");
                    ui.label("  - F1: Toggle this help dialog");
                    ui.label("  - Ctrl+N: Next channel");
                    ui.label("  - Ctrl+K / Ctrl+P: Previous channel");
                    ui.label("  - Enter: Send message (Shift+Enter for newline)");
                    ui.separator();
                    ui.label("Slash commands:");
                    ui.label("  /join <#channel> - join a channel");
                    ui.label("  /part [#channel] - leave a channel");
                    ui.label("  /nick <nick> - change nick");
                    ui.label("  /me <text> - send CTCP ACTION");
                    ui.label("  /whois <nick> - request WHOIS");
                    ui.label("  /topic <text> - set channel topic");
                    ui.label("  /kick <nick> <reason> - kick with reason");
                });
            if !open {
                self.show_help_dialog = false;
            }
        }

        // Nick change dialog
        if self.nick_change_dialog_open {
            let mut open = true;
            let mut newnick = self.nick_change_input.clone();
            egui::Window::new("Change Nick")
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("New nickname:");
                    let _resp = ui.add(egui::TextEdit::singleline(&mut newnick));
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if !newnick.trim().is_empty() && newnick != self.nickname_input {
                                let _ = self.action_tx.send(BackendAction::Nick(newnick.clone()));
                            }
                            self.nick_change_dialog_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.nick_change_dialog_open = false;
                        }
                    });
                });
            if !open {
                self.nick_change_dialog_open = false;
            }
        }
    }
}

impl Drop for SlircApp {
    fn drop(&mut self) {
        // Persist settings on exit
        let settings = Settings {
            server: self.server_input.clone(),
            nick: self.nickname_input.clone(),
            default_channel: self.channel_input.clone(),
            history: self.history.clone(),
            theme: self.theme.clone(),
            networks: self.networks.clone(),
        };
        if let Err(e) = save_settings(&settings) {
            eprintln!("Failed to save settings: {}", e);
        }
    }
}
