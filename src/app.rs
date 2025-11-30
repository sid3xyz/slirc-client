use chrono::Local;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui::{self};
use slirc_proto::ctcp::Ctcp;
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
use crate::logging::Logger;
use crate::protocol::{BackendAction, GuiEvent};
use crate::ui;
use crate::ui::dialogs::{
    ChannelBrowserDialog, ChannelListItem, DialogAction, HelpDialog,
    NetworkManagerDialog, NickChangeDialog, TopicEditorDialog,
};

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
    // UI visibility toggles
    pub show_channel_list: bool,
    pub show_user_list: bool,
    pub expanded_networks: HashSet<String>,
    // Status messages (toasts)
    pub status_messages: Vec<(String, std::time::Instant)>,
    // Chat logger
    pub logger: Option<Logger>,
    // Quick switcher (Ctrl+K)
    pub quick_switcher: ui::quick_switcher::QuickSwitcher,
    
    // Dialogs - Option<Dialog> pattern: None = closed, Some = open with state
    pub help_dialog: HelpDialog,
    pub nick_change_dialog: Option<NickChangeDialog>,
    pub topic_editor_dialog: Option<TopicEditorDialog>,
    pub network_manager_dialog: Option<NetworkManagerDialog>,
    pub channel_browser_dialog: Option<ChannelBrowserDialog>,
}

impl SlircApp {
    /// Get the current theme based on the theme string ("dark" or "light")
    fn get_theme(&self) -> ui::theme::SlircTheme {
        match self.theme.as_str() {
            "light" => ui::theme::SlircTheme::light(),
            _ => ui::theme::SlircTheme::dark(),
        }
    }

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

        // Set professional font sizes and improved spacing
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Small,
                egui::FontId::new(11.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(13.0, egui::FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(13.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Heading,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();
        // Increase global spacing for breathing room
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(12);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        
        // Modern button styling
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 60, 70);
        style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(55, 60, 70);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
        
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 76, 88);
        style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(70, 76, 88);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
        
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(88, 101, 242);
        style.visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(88, 101, 242);
        style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
        
        // Text input styling
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(30, 32, 38);
        style.visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(88, 101, 242, 100);
        
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
            networks: Vec::new(),
            show_channel_list: true,
            show_user_list: true,
            expanded_networks: HashSet::new(),
            status_messages: Vec::new(),
            logger: Logger::new().ok(), // Initialize logger, silently fail if can't create
            quick_switcher: ui::quick_switcher::QuickSwitcher::default(),
            
            // Dialogs - Option pattern for open/closed state
            help_dialog: HelpDialog::new(),
            nick_change_dialog: None,
            topic_editor_dialog: None,
            network_manager_dialog: None,
            channel_browser_dialog: None,
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
                            auto_reconnect: network.auto_reconnect,
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
        
        // Command completion when prefix starts with /
        if prefix.starts_with('/') {
            // List of available IRC commands
            let commands = vec![
                "/join", "/j", "/part", "/p", "/msg", "/privmsg", "/me", 
                "/whois", "/w", "/topic", "/t", "/kick", "/k", "/nick", 
                "/quit", "/exit", "/help"
            ];
            for cmd in commands {
                if cmd.starts_with(prefix) {
                    matches.push(cmd.to_string());
                }
            }
        } else if let Some(stripped) = prefix.strip_prefix('@') {
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
        } else if !prefix.starts_with('/') {
            // user completions from active buffer (skip if completing commands)
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
        // Exception: commands starting with '/' just get a space
        let is_first_token = self.message_input[..last_word_start].trim().is_empty();
        let is_command = completion.starts_with('/');
        let suffix = if is_command {
            " "
        } else if is_first_token {
            ": "
        } else {
            " "
        };
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
        // Collect channel list events separately
        let mut regular_events = Vec::new();
        
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                GuiEvent::ChannelListItem {
                    channel,
                    user_count,
                    topic,
                } => {
                    // Add to channel browser dialog if open
                    if let Some(ref mut dialog) = self.channel_browser_dialog {
                        dialog.add_channel(ChannelListItem {
                            channel,
                            user_count,
                            topic,
                        });
                    }
                }
                GuiEvent::ChannelListEnd => {
                    // Mark loading complete and show dialog
                    if let Some(ref mut dialog) = self.channel_browser_dialog {
                        dialog.set_loading_complete();
                    }
                }
                other => {
                    regular_events.push(other);
                }
            }
        }
        
        // Process regular events
        for event in regular_events {
            self.process_single_event(event);
        }
    }
    
    fn process_single_event(&mut self, event: GuiEvent) {
        events::process_single_event(
            event,
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
            &self.logger,
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
            // Ctrl+K: Quick switcher (search overlay)
            if i.modifiers.ctrl && i.key_pressed(egui::Key::K) {
                self.quick_switcher.toggle();
            }
            // Ctrl+P: Previous channel
            if i.modifiers.ctrl && i.key_pressed(egui::Key::P) {
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
                self.help_dialog.toggle();
            }
        });

        // Request repaint to keep checking for events
        ctx.request_repaint_after(Duration::from_millis(100));
        // Purge old status messages (toasts) older than 4 seconds
        self.status_messages
            .retain(|(_, t)| t.elapsed() < std::time::Duration::from_secs(4));

        let theme = self.get_theme();

        // Modern horizontal menu bar (Discord/Slack-inspired with IRC-specific menus)
        egui::TopBottomPanel::top("menu_bar")
            .frame(
                egui::Frame::new()
                    .fill(theme.surface[1])
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .stroke(egui::Stroke::new(1.0, theme.border_medium)),
            )
            .show(ctx, |ui| {
                if let Some(menu_action) = ui::menu::render_menu_bar(
                    ctx,
                    ui,
                    self.is_connected,
                    &self.active_buffer,
                    &mut self.show_channel_list,
                    &mut self.show_user_list,
                    &mut self.quick_switcher,
                    &self.action_tx,
                ) {
                    match menu_action {
                        ui::menu::MenuAction::NetworkManager => {
                            self.network_manager_dialog = Some(NetworkManagerDialog::new(self.networks.clone()));
                        }
                        ui::menu::MenuAction::Help => {
                            self.help_dialog.show();
                        }
                        ui::menu::MenuAction::ChannelBrowser => {
                            self.channel_browser_dialog = Some(ChannelBrowserDialog::new());
                        }
                    }
                }
            });

        // Compact toolbar below menu bar (for quick actions)
        let toolbar_bg = theme.surface[1];
        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(toolbar_bg)
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .stroke(egui::Stroke::new(
                        1.0,
                        theme.border_medium,
                    )),
            )
            .show(ctx, |ui| {
                if let Some(toolbar_action) = ui::toolbar::render_toolbar(
                    ui,
                    ctx,
                    &mut self.server_input,
                    &mut self.nickname_input,
                    &mut self.channel_input,
                    self.is_connected,
                    &mut self.use_tls,
                    &self.action_tx,
                ) {
                    match toolbar_action {
                        ui::toolbar::ToolbarAction::OpenNickChangeDialog => {
                            self.nick_change_dialog = Some(NickChangeDialog::new(&self.nickname_input));
                        }
                    }
                }
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

        // Bottom panel: Message input with polished styling
        let dark_mode = ctx.style().visuals.dark_mode;
        let theme = self.get_theme();
        let input_bg = theme.surface[1];

        egui::TopBottomPanel::bottom("input_panel")
            .frame(
                egui::Frame::new()
                    .fill(input_bg)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .stroke(egui::Stroke::new(
                        1.0,
                        theme.border_medium,
                    )),
            )
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Styled input frame with rounding and focus indication
                let input_frame = egui::Frame::new()
                    .fill(if dark_mode {
                        egui::Color32::from_rgb(45, 45, 52)
                    } else {
                        egui::Color32::WHITE
                    })
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(10, 8));

                input_frame.show(ui, |ui| {
                let response = ui.add(
                    egui::TextEdit::multiline(&mut self.message_input)
                        .desired_rows(1)
                        .desired_width(ui.available_width() - 4.0)
                        .frame(false)
                        .hint_text("Type a message... (Enter to send)"),
                );
                
                // Draw focus ring (two rects: outer border, inner transparent)
                if response.has_focus() {
                    let outer = response.rect.expand(2.0);
                    ui.painter().rect_filled(outer, 8.0, theme.accent.linear_multiply(0.3));
                }

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
                }); // close input_frame
            });
        });

        // Central panel: Messages with dedicated topic bar and styled background
        let theme = self.get_theme();
        let chat_bg = theme.surface[2];
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(chat_bg)
                    .inner_margin(12.0),
            )
            .show(ctx, |ui| {
                if let Some(msg_action) = ui::messages::render_messages(
                    ctx,
                    ui,
                    &self.active_buffer,
                    &self.buffers,
                    &self.system_log,
                    &self.nickname_input,
                ) {
                    match msg_action {
                        ui::messages::MessagePanelAction::OpenTopicEditor(channel) => {
                            let current_topic = self.buffers
                                .get(&channel)
                                .map(|b| b.topic.clone())
                                .unwrap_or_default();
                            self.topic_editor_dialog = Some(TopicEditorDialog::new(&channel, &current_topic));
                        }
                    }
                }
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
                                // Check if this is a CTCP ACTION message using slirc_proto
                                if let Some(ctcp) = Ctcp::parse(&msg.text) {
                                    if let Some(action) = ctcp.params {
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
                                        continue;
                                    }
                                }
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
                        });
                    }
                });
            if !open {
                self.open_windows.remove(&open_name);
            }
        }

        // Render dialogs using the new self-contained dialog pattern
        self.render_dialogs(ctx);
        
        // Quick switcher overlay (Ctrl+K)
        if let Some(selected_buffer) = self.quick_switcher.render(ctx, &self.buffers) {
            self.active_buffer = selected_buffer.clone();
            if let Some(buffer) = self.buffers.get_mut(&selected_buffer) {
                buffer.clear_unread();
                buffer.has_highlight = false;
            }
        }
    }
}

impl SlircApp {
    /// Render all dialogs and handle their actions
    fn render_dialogs(&mut self, ctx: &egui::Context) {
        // Floating status toasts (top-right corner)
        ui::dialogs::render_status_toasts(ctx, &self.status_messages);
        
        // Help dialog (F1) - simple toggle, no actions
        self.help_dialog.render(ctx);
        
        // Collect actions and state changes first to avoid borrow issues
        let mut actions: Vec<DialogAction> = Vec::new();
        let mut close_nick_dialog = false;
        let mut close_topic_dialog = false;
        let mut close_network_dialog = false;
        let mut save_networks = false;
        let mut networks_to_save: Option<Vec<crate::config::Network>> = None;
        let mut close_channel_browser = false;
        
        // Nick change dialog
        if let Some(ref mut dialog) = self.nick_change_dialog {
            if let Some(action) = dialog.render(ctx) {
                actions.push(action);
            }
            if !dialog.is_open() {
                close_nick_dialog = true;
            }
        }
        
        // Topic editor dialog
        if let Some(ref mut dialog) = self.topic_editor_dialog {
            let (action, still_open) = dialog.render(ctx);
            if let Some(action) = action {
                actions.push(action);
            }
            if !still_open {
                close_topic_dialog = true;
            }
        }
        
        // Network manager dialog
        if let Some(ref mut dialog) = self.network_manager_dialog {
            let (action, still_open) = dialog.render(ctx);
            if let Some(action) = action {
                actions.push(action);
            }
            if !still_open {
                if dialog.was_modified() {
                    networks_to_save = Some(dialog.get_networks().to_vec());
                    save_networks = true;
                }
                close_network_dialog = true;
            }
        }
        
        // Channel browser dialog
        if let Some(ref mut dialog) = self.channel_browser_dialog {
            let (action, still_open) = dialog.render(ctx);
            if let Some(action) = action {
                actions.push(action);
            }
            if !still_open {
                close_channel_browser = true;
            }
        }
        
        // Now process collected actions (no longer borrowing dialog fields)
        for action in actions {
            self.handle_dialog_action(action);
        }
        
        // Close dialogs as needed
        if close_nick_dialog {
            self.nick_change_dialog = None;
        }
        if close_topic_dialog {
            self.topic_editor_dialog = None;
        }
        if close_network_dialog {
            if save_networks {
                if let Some(networks) = networks_to_save {
                    self.networks = networks;
                    self.save_networks();
                }
            }
            self.network_manager_dialog = None;
        }
        if close_channel_browser {
            self.channel_browser_dialog = None;
        }
    }
    
    /// Handle dialog actions by sending appropriate backend commands
    fn handle_dialog_action(&mut self, action: DialogAction) {
        match action {
            DialogAction::ChangeNick(new_nick) => {
                let _ = self.action_tx.send(BackendAction::Nick(new_nick));
            }
            DialogAction::SetTopic { channel, topic } => {
                let _ = self.action_tx.send(BackendAction::SetTopic { channel, topic });
            }
            DialogAction::JoinChannel(channel) => {
                let _ = self.action_tx.send(BackendAction::Join(channel));
            }
            DialogAction::NetworkConnect(network) => {
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
                        auto_reconnect: network.auto_reconnect,
                    });

                    // Auto-join favorite channels
                    for channel in &network.favorite_channels {
                        let _ = self.action_tx.send(BackendAction::Join(channel.clone()));
                    }
                }
            }
            DialogAction::NetworkSave { index: _, network: _ } => {
                // Network already saved in dialog, just need to persist
                // This is handled when dialog closes
            }
            DialogAction::NetworkDelete(_) => {
                // Network already deleted in dialog, just need to persist
                // This is handled when dialog closes
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
