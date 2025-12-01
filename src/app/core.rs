//! Core SlircApp struct definition and initialization

use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;
use std::collections::HashSet;
use std::thread;

use crate::backend::run_backend;
use crate::config::{
    ConnectionConfig, load_nickserv_password, load_settings, save_settings, Settings,
};
use crate::dialog_manager::DialogManager;
use crate::input_state::InputState;
use crate::protocol::{BackendAction, GuiEvent};
use crate::state::ClientState;
use crate::ui;
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
    pub(super) fn get_theme(&self) -> ui::theme::SlircTheme {
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

    pub(super) fn save_networks(&self) {
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

    /// Initiate a connection to the server using current UI inputs.
    /// Sets state.server_name and state.our_nick before sending connect action.
    pub(super) fn do_connect(&mut self) {
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
