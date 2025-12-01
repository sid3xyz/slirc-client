use chrono::Local;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui::{self};
use slirc_proto::ctcp::Ctcp;
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use crate::backend::run_backend;
use crate::buffer::ChannelBuffer;
use crate::commands;
use crate::config::{
    ConnectionConfig, load_nickserv_password, load_settings, save_settings, Settings,
};
use crate::dialog_manager::DialogManager;
use crate::events;
use crate::input_state::InputState;
use crate::protocol::{BackendAction, GuiEvent};
use crate::state::ClientState;
use crate::ui;
use crate::ui::dialogs::{
    ChannelListItem, DialogAction,
};
use crate::ui::shortcuts::ShortcutRegistry;

pub struct SlircApp {
    // Core state (buffers, networks, connection status, etc.)
    pub state: ClientState,

    // Connection settings (form inputs)
    pub connection: ConnectionConfig,

    // Channels for backend communication
    pub action_tx: Sender<BackendAction>,
    pub event_rx: Receiver<GuiEvent>,

    // Input state (message composition, history, tab completion)
    pub input: InputState,

    // Context menu state
    pub context_menu_visible: bool,
    pub context_menu_target: Option<String>,
    pub open_windows: HashSet<String>,

    // Theme
    pub theme: String,

    // UI visibility toggles
    pub show_channel_list: bool,
    pub show_user_list: bool,

    // Quick switcher (Ctrl+K)
    pub quick_switcher: ui::quick_switcher::QuickSwitcher,

    // Dialogs - managed centrally by DialogManager
    pub dialogs: DialogManager,

    // Keyboard shortcuts registry
    pub shortcuts: ShortcutRegistry,
    pub show_shortcuts_help: bool,
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
        // Use the centralized font loader from fonts.rs
        cc.egui_ctx.set_fonts(crate::fonts::setup_fonts());

        // Apply modern theme styling
        ui::theme::apply_app_style(&cc.egui_ctx);

        let state = ClientState::new();

        let mut app = Self {
            state,
            connection: ConnectionConfig::default(),

            action_tx,
            event_rx,

            input: InputState::new(),

            context_menu_visible: false,
            context_menu_target: None,
            open_windows: HashSet::new(),

            theme: "dark".to_string(),

            show_channel_list: true,
            show_user_list: true,
            quick_switcher: ui::quick_switcher::QuickSwitcher::default(),

            // Dialogs - managed centrally by DialogManager
            dialogs: DialogManager::new(),

            // Keyboard shortcuts
            shortcuts: ShortcutRegistry::new(),
            show_shortcuts_help: false,
        };

        // Restore settings if present
        if let Some(s) = settings {
            if !s.server.is_empty() {
                app.connection.server = s.server;
            }
            if !s.nick.is_empty() {
                app.connection.nickname = s.nick;
            }
            if !s.history.is_empty() {
                app.input.history = s.history;
            }
            if !s.default_channel.is_empty() {
                app.input.channel_input = s.default_channel;
            }
            if !s.theme.is_empty() {
                app.theme = s.theme;
            }
            app.state.networks = s.networks.clone();

            // Auto-connect to networks with auto_connect flag
            for network in &s.networks {
                if network.auto_connect {
                    if let Some(server_addr) = network.servers.first() {
                        let parts: Vec<&str> = server_addr.split(':').collect();
                        let server = parts[0].to_string();
                        let port: u16 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(6667);

                        // Set state fields for event processing
                        app.state.server_name = server_addr.clone();
                        app.state.our_nick = network.nick.clone();

                        let _ = app.action_tx.send(BackendAction::Connect {
                            server,
                            port,
                            nickname: network.nick.clone(),
                            username: network.nick.clone(),
                            realname: format!("SLIRC User ({})", network.nick),
                            use_tls: network.use_tls,
                            auto_reconnect: network.auto_reconnect,
                            sasl_password: load_nickserv_password(&network.name),
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
            server: self.connection.server.clone(),
            nick: self.connection.nickname.clone(),
            default_channel: self.input.channel_input.clone(),
            history: self.input.history.clone(),
            theme: self.theme.clone(),
            networks: self.state.networks.clone(),
        };
        if let Err(e) = save_settings(&settings) {
            eprintln!("Failed to save networks: {}", e);
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
                    self.dialogs.add_channel_to_browser(ChannelListItem {
                        channel,
                        user_count,
                        topic,
                    });
                }
                GuiEvent::ChannelListEnd => {
                    // Mark loading complete and show dialog
                    self.dialogs.channel_browser_complete();
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
        // Process event and check if nick changed
        if let Some(new_nick) = events::process_single_event(&mut self.state, event) {
            // Update UI nickname field when server confirms nick change
            self.connection.nickname = new_nick;
        }
    }

    /// Initiate a connection to the server using current UI inputs.
    /// Sets state.server_name and state.our_nick before sending connect action.
    fn do_connect(&mut self) {
        // Parse server:port from connection config
        let (server, port) = self.connection.parse_server();

        // Set state fields for event processing (like Halloy's configured_nick pattern)
        self.state.server_name = self.connection.server.clone();
        self.state.our_nick = self.connection.nickname.clone();

        let _ = self.action_tx.send(BackendAction::Connect {
            server,
            port,
            nickname: self.connection.nickname.clone(),
            username: self.connection.nickname.clone(),
            realname: format!("SLIRC User ({})", self.connection.nickname),
            use_tls: self.connection.use_tls,
            auto_reconnect: true,
            sasl_password: None,
        });
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
                self.state.next_buffer();
            }
            // Ctrl+K: Quick switcher (search overlay)
            if i.modifiers.ctrl && i.key_pressed(egui::Key::K) {
                self.quick_switcher.toggle();
            }
            // Ctrl+P: Previous channel
            if i.modifiers.ctrl && i.key_pressed(egui::Key::P) {
                self.state.prev_buffer();
            }
            // F1: Toggle help dialog
            if i.key_pressed(egui::Key::F1) {
                self.dialogs.toggle_help();
            }
            // Ctrl+/: Toggle shortcuts help overlay
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Slash) {
                self.show_shortcuts_help = !self.show_shortcuts_help;
            }
            // Ctrl+M: Minimize window
            if i.modifiers.ctrl && i.key_pressed(egui::Key::M) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
            // Ctrl+Shift+F: Toggle fullscreen
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::F) {
                let current_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!current_fullscreen));
            }
            // Ctrl+B: Toggle channel list
            if i.modifiers.ctrl && i.key_pressed(egui::Key::B) {
                self.show_channel_list = !self.show_channel_list;
            }
        });

        // Request repaint to keep checking for events
        ctx.request_repaint_after(Duration::from_millis(100));
        // Purge old status messages (toasts) older than 4 seconds
        self.state.purge_old_status_messages(4);

        // Render UI sections
        self.render_menu_bar(ctx);
        self.render_toolbar(ctx);

        // Left panel: Buffer list (vertical tabs similar to HexChat)
        if self.show_channel_list {
            ui::panels::render_channel_list(
                ctx,
                &self.state.buffers,
                &self.state.buffers_order,
                &mut self.state.active_buffer,
                &mut self.context_menu_visible,
                &mut self.context_menu_target,
                &mut self.state.collapsed_sections,
                &mut self.state.channel_filter,
            );
            // Clear unread after switching buffer
            if let Some(buf) = self.state.buffers.get_mut(&self.state.active_buffer) {
                buf.clear_unread();
            }
        }
        // (Removed top horizontal buffer tabs — left navigation is the single source of truth.)

        // Right panel: User list (for channels)
        if self.show_user_list
            && (self.state.active_buffer.starts_with('#') || self.state.active_buffer.starts_with('&'))
        {
            if let Some(buffer) = self.state.buffers.get(&self.state.active_buffer) {
                ui::panels::render_user_list(
                    ctx,
                    buffer,
                    &self.state.active_buffer,
                    &self.connection.nickname,
                    &mut self.context_menu_visible,
                    &mut self.context_menu_target,
                );
            }
        }

        // Bottom panel: Message input with polished styling
        let _enter_pressed = self.render_input_panel(ctx);

        // Central panel: Messages with dedicated topic bar and styled background
        self.render_central_panel(ctx);

        // Context menu popup (as a floating window)
        self.render_context_menu(ctx);

        // Floating buffer windows
        self.render_floating_windows(ctx);

        // Render dialogs using the new self-contained dialog pattern
        self.render_dialogs(ctx);

        // Quick switcher overlay (Ctrl+K)
        if let Some(selected_buffer) = self.quick_switcher.render(ctx, &self.state.buffers) {
            self.state.active_buffer = selected_buffer.clone();
            if let Some(buffer) = self.state.buffers.get_mut(&selected_buffer) {
                buffer.clear_unread();
                buffer.has_highlight = false;
            }
        }

        // Shortcuts help overlay (Ctrl+/ or F1)
        self.shortcuts.render_help_overlay(ctx, &mut self.show_shortcuts_help);
    }
}

impl SlircApp {
    /// Render all dialogs and handle their actions
    fn render_dialogs(&mut self, ctx: &egui::Context) {
        // Floating status toasts (top-right corner)
        ui::dialogs::render_status_toasts(ctx, &self.state.status_messages);

        // Delegate to DialogManager for all dialog rendering
        let (actions, networks_to_save) = self.dialogs.render(ctx);

        // Process actions
        for action in actions {
            self.handle_dialog_action(action);
        }

        // Save networks if needed
        if let Some(networks) = networks_to_save {
            self.state.networks = networks;
            self.save_networks();
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

                    // Set state fields for event processing
                    self.state.server_name = server_addr.clone();
                    self.state.our_nick = network.nick.clone();

                    let _ = self.action_tx.send(BackendAction::Connect {
                        server,
                        port,
                        nickname: network.nick.clone(),
                        username: network.nick.clone(),
                        realname: format!("SLIRC User ({})", network.nick),
                        use_tls: network.use_tls,
                        auto_reconnect: network.auto_reconnect,
                        sasl_password: load_nickserv_password(&network.name),
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

    /// Render the menu bar at the top of the window
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
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
                    self.state.is_connected,
                    &self.state.active_buffer,
                    &mut self.show_channel_list,
                    &mut self.show_user_list,
                    &mut self.quick_switcher,
                    &self.action_tx,
                ) {
                    match menu_action {
                        ui::menu::MenuAction::NetworkManager => {
                            self.dialogs.open_network_manager(self.state.networks.clone());
                        }
                        ui::menu::MenuAction::Help => {
                            self.show_shortcuts_help = true;
                        }
                        ui::menu::MenuAction::ChannelBrowser => {
                            self.dialogs.open_channel_browser();
                        }
                    }
                }
            });
    }

    /// Render the toolbar below the menu bar
    fn render_toolbar(&mut self, ctx: &egui::Context) {
        let theme = self.get_theme();

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
                    &mut self.connection.server,
                    &mut self.connection.nickname,
                    &mut self.input.channel_input,
                    self.state.is_connected,
                    &mut self.connection.use_tls,
                    &self.action_tx,
                ) {
                    match toolbar_action {
                        ui::toolbar::ToolbarAction::Connect => {
                            self.do_connect();
                        }
                        ui::toolbar::ToolbarAction::OpenNickChangeDialog => {
                            self.dialogs.open_nick_change(&self.connection.nickname);
                        }
                    }
                }
            });
    }

    /// Render the input panel at the bottom of the window
    /// Returns Some(true) if Enter was pressed and message sent (for focus control)
    fn render_input_panel(&mut self, ctx: &egui::Context) -> Option<bool> {
        let dark_mode = ctx.style().visuals.dark_mode;
        let theme = self.get_theme();
        let input_bg = theme.surface[1];

        let mut enter_pressed = false;

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
                    egui::TextEdit::multiline(&mut self.input.message_input)
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
                let enter_detected = response.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);

                // Input history navigation
                if response.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::ArrowUp))
                    && !self.input.history.is_empty()
                {
                    if self.input.history_pos.is_none() {
                        // store current text to restore if user navigates back
                        self.input.history_saved_input = Some(self.input.message_input.clone());
                        self.input.history_pos = Some(self.input.history.len() - 1);
                    } else if let Some(pos) = self.input.history_pos {
                        if pos > 0 {
                            self.input.history_pos = Some(pos - 1);
                        }
                    }
                    if let Some(pos) = self.input.history_pos {
                        if let Some(h) = self.input.history.get(pos) {
                            self.input.message_input = h.clone();
                        }
                    }
                }
                if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    if let Some(pos) = self.input.history_pos {
                        if pos + 1 < self.input.history.len() {
                            self.input.history_pos = Some(pos + 1);
                            if let Some(h) = self.input.history.get(pos + 1) {
                                self.input.message_input = h.clone();
                            }
                        } else {
                            // Exit history navigation
                            self.input.history_pos = None;
                            self.input.message_input =
                                self.input.history_saved_input.take().unwrap_or_default();
                        }
                    }
                }

                // Tab completion: Tab cycles forward; Shift+Tab cycles backward
                let tab_pressed =
                    response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Tab));
                let shift = ui.input(|i| i.modifiers.shift);
                if tab_pressed {
                    // compute current prefix (last token)
                    let (start, end) = self.input.current_last_word_bounds();
                    let prefix = self.input.message_input[start..end].trim();
                    if self.input.completions.is_empty() {
                        // first time: gather completions
                        self.input.completions = self.input.collect_completions(
                            prefix,
                            &self.state.buffers_order,
                            &self.state.active_buffer,
                            &self.state.buffers
                        );
                        self.input.completion_prefix = Some(prefix.to_string());
                        self.input.completion_target_channel =
                            prefix.starts_with('#') || prefix.starts_with('&');
                    }
                    if !self.input.completions.is_empty() {
                        if shift {
                            self.input.cycle_completion(-1);
                        } else {
                            self.input.cycle_completion(1);
                        }
                    }
                }

                // Reset completions if the user changed the input text
                if self.input.last_input_text != self.input.message_input && !tab_pressed {
                    self.input.completions.clear();
                    self.input.completion_index = None;
                    self.input.completion_prefix = None;
                }
                self.input.last_input_text = self.input.message_input.clone();

                // Esc to cancel input (clear the text field)
                if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.input.message_input.clear();
                    self.input.history_pos = None;
                    self.input.history_saved_input = None;
                    self.input.completions.clear();
                    self.input.completion_index = None;
                    self.input.completion_prefix = None;
                }

                if enter_detected && !self.input.message_input.is_empty() {
                    // If it begins with a slash, treat as a command
                    if self.input.message_input.starts_with('/') {
                        if commands::handle_user_command(
                            &self.input.message_input,
                            &self.state.active_buffer,
                            &self.state.buffers,
                            &self.action_tx,
                            &mut self.state.system_log,
                            &mut self.connection.nickname,
                        ) {
                            self.input.history.push(self.input.message_input.clone());
                        }
                    } else {
                        // Normal message
                        if self.state.is_connected {
                            if self.state.active_buffer != "System" {
                                let _ = self.action_tx.send(BackendAction::SendMessage {
                                    target: self.state.active_buffer.clone(),
                                    text: self.input.message_input.clone(),
                                });
                                self.input.history.push(self.input.message_input.clone());
                            }
                        } else {
                            let ts = Local::now().format("%H:%M:%S").to_string();
                            self.state.system_log
                                .push(format!("[{}] ⚠ Not connected: message not sent", ts));
                        }
                    }

                    // Reset history navigation and input
                    self.input.history_pos = None;
                    self.input.history_saved_input = None;
                    self.input.message_input.clear();
                    response.request_focus();
                    enter_pressed = true;
                }
                }); // close input_frame
            });
        });

        if enter_pressed {
            Some(true)
        } else {
            None
        }
    }

    /// Render the central panel with messages
    fn render_central_panel(&mut self, ctx: &egui::Context) {
        let theme = self.get_theme();
        let chat_bg = theme.surface[2];
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(chat_bg)
                    .inner_margin(12.0),
            )
            .show(ctx, |ui| {
                // Use state.our_nick if connected, otherwise fall back to UI input
                let current_nick = if self.state.our_nick.is_empty() {
                    &self.connection.nickname
                } else {
                    &self.state.our_nick
                };
                if let Some(msg_action) = ui::messages::render_messages(
                    ctx,
                    ui,
                    &self.state.active_buffer,
                    &self.state.buffers,
                    &self.state.system_log,
                    current_nick,
                ) {
                    match msg_action {
                        ui::messages::MessagePanelAction::OpenTopicEditor(channel) => {
                            let current_topic = self.state.buffers
                                .get(&channel)
                                .map(|b| b.topic.clone())
                                .unwrap_or_default();
                            self.dialogs.open_topic_editor(&channel, &current_topic);
                        }
                    }
                }
            });
    }

    /// Render context menu popup (as a floating window)
    fn render_context_menu(&mut self, ctx: &egui::Context) {
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
                                if !self.state.buffers.contains_key(user) {
                                    self.state.buffers.insert(user.to_string(), ChannelBuffer::new());
                                    self.state.buffers_order.push(user.to_string());
                                }
                                self.state.active_buffer = user.to_string();
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
                            if self.state.active_buffer.starts_with('#')
                                || self.state.active_buffer.starts_with('&')
                            {
                                let is_op = self
                                    .state
                                    .buffers
                                    .get(&self.state.active_buffer)
                                    .map(|b| {
                                        b.users.iter().any(|u| {
                                            u.nick == self.connection.nickname
                                                && ui::theme::prefix_rank(u.prefix) >= 3
                                        })
                                    })
                                    .unwrap_or(false);
                                if is_op {
                                    ui.separator();
                                    ui.label("Op Actions:");
                                    if ui.button("Op (+o)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.state.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "+o".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Deop (-o)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.state.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "-o".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Voice (+v)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.state.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "+v".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Devoice (-v)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.state.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "-v".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Kick").clicked() {
                                        let _ = self.action_tx.send(BackendAction::Kick {
                                            channel: self.state.active_buffer.clone(),
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
                                self.state.buffers.remove(&target);
                                self.state.buffers_order.retain(|b| b != &target);
                                if self.state.active_buffer == target {
                                    self.state.active_buffer = "System".into();
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
    }

    /// Render floating buffer windows
    fn render_floating_windows(&mut self, ctx: &egui::Context) {
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
                    if let Some(buffer) = self.state.buffers.get(&open_name) {
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
    }
}

impl Drop for SlircApp {
    fn drop(&mut self) {
        // Persist settings on exit
        let settings = Settings {
            server: self.connection.server.clone(),
            nick: self.connection.nickname.clone(),
            default_channel: self.input.channel_input.clone(),
            history: self.input.history.clone(),
            theme: self.theme.clone(),
            networks: self.state.networks.clone(),
        };
        if let Err(e) = save_settings(&settings) {
            eprintln!("Failed to save settings: {}", e);
        }
    }
}
