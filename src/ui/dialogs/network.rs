//! Network manager dialog - manage saved IRC networks.

use eframe::egui;
use std::collections::HashSet;

use crate::config::Network;
use super::DialogAction;

/// Form state for creating/editing a network
#[derive(Default, Clone)]
pub struct NetworkForm {
    pub name: String,
    pub servers: String, // Comma-separated
    pub nick: String,
    pub auto_connect: bool,
    pub favorite_channels: String, // Comma-separated
    pub nickserv_password: String,
    pub use_tls: bool,
}

impl NetworkForm {
    /// Create a form from an existing network
    pub fn from_network(network: &Network) -> Self {
        Self {
            name: network.name.clone(),
            servers: network.servers.join(", "),
            nick: network.nick.clone(),
            auto_connect: network.auto_connect,
            favorite_channels: network.favorite_channels.join(", "),
            nickserv_password: network.nickserv_password.clone().unwrap_or_default(),
            use_tls: network.use_tls,
        }
    }

    /// Convert the form to a Network
    pub fn to_network(&self) -> Network {
        let servers: Vec<String> = self
            .servers
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let favorite_channels: Vec<String> = self
            .favorite_channels
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Network {
            name: self.name.clone(),
            servers,
            nick: self.nick.clone(),
            auto_connect: self.auto_connect,
            favorite_channels,
            nickserv_password: if self.nickserv_password.is_empty() {
                None
            } else {
                Some(self.nickserv_password.clone())
            },
            use_tls: self.use_tls,
            auto_reconnect: true,
        }
    }

    /// Check if the form has valid data
    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && !self.servers.trim().is_empty()
            && !self.nick.trim().is_empty()
    }
}

/// Self-contained network manager dialog state.
pub struct NetworkManagerDialog {
    /// Working copy of networks (modifications are local until saved)
    pub networks: Vec<Network>,
    /// Index of network currently being edited (None = adding new)
    editing_index: Option<usize>,
    /// Form for editing/creating network
    form: NetworkForm,
    /// Which networks are expanded in the list
    expanded: HashSet<String>,
    /// Track if networks were modified
    modified: bool,
}

impl NetworkManagerDialog {
    /// Create a new network manager dialog with a copy of the networks
    pub fn new(networks: Vec<Network>) -> Self {
        Self {
            networks,
            editing_index: None,
            form: NetworkForm::default(),
            expanded: HashSet::new(),
            modified: false,
        }
    }

    /// Start editing an existing network
    fn start_edit(&mut self, index: usize) {
        if let Some(network) = self.networks.get(index) {
            self.editing_index = Some(index);
            self.form = NetworkForm::from_network(network);
        }
    }

    /// Start adding a new network
    fn start_add(&mut self) {
        self.editing_index = None;
        self.form = NetworkForm::default();
    }

    /// Cancel editing
    fn cancel_edit(&mut self) {
        self.editing_index = None;
        self.form = NetworkForm::default();
    }

    /// Check if we're in edit mode
    fn is_editing(&self) -> bool {
        self.editing_index.is_some() || !self.form.name.is_empty()
    }

    /// Get the current networks (for saving)
    pub fn get_networks(&self) -> &[Network] {
        &self.networks
    }

    /// Check if networks were modified
    pub fn was_modified(&self) -> bool {
        self.modified
    }

    /// Render the network manager dialog.
    /// Returns `Some(DialogAction)` for connect/save/delete actions.
    /// 
    /// The second return value indicates if the dialog is still open.
    pub fn render(&mut self, ctx: &egui::Context) -> (Option<DialogAction>, bool) {
        let mut action: Option<DialogAction> = None;
        let mut should_close = false;
        let mut window_open = true;

        egui::Window::new("Network Manager")
            .open(&mut window_open)
            .resizable(true)
            .default_width(550.0)
            .show(ctx, |ui| {
                ui.heading("Saved Networks");
                ui.separator();

                // Network list - collect data we need first to avoid borrow issues
                let network_count = self.networks.len();
                let mut delete_index: Option<usize> = None;
                let mut edit_index: Option<usize> = None;
                let mut connect_network: Option<Network> = None;
                let mut toggle_expanded: Option<String> = None;
                
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for idx in 0..network_count {
                            let network = &self.networks[idx];
                            let network_name = network.name.clone();
                            let expanded = self.expanded.contains(&network_name);
                            
                            ui.horizontal(|ui| {
                                // Expand/collapse button
                                if ui.small_button(if expanded { "▾" } else { "▸" }).clicked() {
                                    toggle_expanded = Some(network_name.clone());
                                }

                                // Network name with auto-connect indicator
                                let label = if network.auto_connect {
                                    format!("✓ {}", network.name)
                                } else {
                                    network.name.clone()
                                };
                                ui.label(egui::RichText::new(label).strong());
                                
                                // Server list
                                ui.label(format!("({})", network.servers.join(", ")));

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("Delete").clicked() {
                                            delete_index = Some(idx);
                                        }

                                        if ui.button("Connect").clicked() {
                                            connect_network = Some(network.clone());
                                            should_close = true;
                                        }

                                        if ui.button("Edit").clicked() {
                                            edit_index = Some(idx);
                                        }
                                    },
                                );
                            });

                            // Expanded details
                            if expanded {
                                ui.indent("network_details", |ui| {
                                    ui.label(format!("Nick: {}", network.nick));
                                    ui.label(format!(
                                        "Favorites: {}",
                                        if network.favorite_channels.is_empty() {
                                            "(none)".to_string()
                                        } else {
                                            network.favorite_channels.join(", ")
                                        }
                                    ));
                                    ui.label(format!(
                                        "TLS: {}",
                                        if network.use_tls { "Yes" } else { "No" }
                                    ));
                                });
                                ui.separator();
                            }
                        }
                    });

                // Handle deletion
                if let Some(idx) = delete_index {
                    action = Some(DialogAction::NetworkDelete(idx));
                    self.networks.remove(idx);
                    self.modified = true;
                }

                ui.separator();

                // Quick add popular networks (only when not editing)
                if !self.is_editing() {
                    ui.horizontal(|ui| {
                        ui.label("Quick add:");
                        
                        if ui.small_button("Libera.Chat").clicked() {
                            self.networks.push(Network {
                                name: "Libera.Chat".to_string(),
                                servers: vec!["irc.libera.chat:6697".to_string()],
                                nick: "slirc_user".to_string(),
                                auto_connect: false,
                                favorite_channels: vec!["#slirc".to_string()],
                                nickserv_password: None,
                                use_tls: true,
                                auto_reconnect: true,
                            });
                            self.modified = true;
                        }
                        
                        if ui.small_button("OFTC").clicked() {
                            self.networks.push(Network {
                                name: "OFTC".to_string(),
                                servers: vec!["irc.oftc.net:6697".to_string()],
                                nick: "slirc_user".to_string(),
                                auto_connect: false,
                                favorite_channels: vec![],
                                nickserv_password: None,
                                use_tls: true,
                                auto_reconnect: true,
                            });
                            self.modified = true;
                        }
                        
                        if ui.small_button("EFnet").clicked() {
                            self.networks.push(Network {
                                name: "EFnet".to_string(),
                                servers: vec!["irc.choopa.net:9999".to_string()],
                                nick: "slirc_user".to_string(),
                                auto_connect: false,
                                favorite_channels: vec![],
                                nickserv_password: None,
                                use_tls: true,
                                auto_reconnect: true,
                            });
                            self.modified = true;
                        }
                        
                        if ui.small_button("Rizon").clicked() {
                            self.networks.push(Network {
                                name: "Rizon".to_string(),
                                servers: vec!["irc.rizon.net:6697".to_string()],
                                nick: "slirc_user".to_string(),
                                auto_connect: false,
                                favorite_channels: vec![],
                                nickserv_password: None,
                                use_tls: true,
                                auto_reconnect: true,
                            });
                            self.modified = true;
                        }
                    });
                    
                    ui.separator();
                }

                // Add/Edit form
                if self.is_editing() || ui.button("➕ Add Network").clicked() {
                    if !self.is_editing() {
                        self.start_add();
                    }

                    ui.heading(if self.editing_index.is_some() {
                        "Edit Network"
                    } else {
                        "New Network"
                    });
                    ui.separator();

                    egui::Grid::new("network_form")
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.form.name);
                            ui.end_row();

                            ui.label("Servers:");
                            ui.text_edit_singleline(&mut self.form.servers);
                            ui.end_row();
                            
                            ui.label("");
                            ui.label(
                                egui::RichText::new("(Comma-separated, e.g., irc.example.com:6697)")
                                    .small()
                                    .weak()
                            );
                            ui.end_row();

                            ui.label("Nickname:");
                            ui.text_edit_singleline(&mut self.form.nick);
                            ui.end_row();

                            ui.label("Favorite Channels:");
                            ui.text_edit_singleline(&mut self.form.favorite_channels);
                            ui.end_row();
                            
                            ui.label("");
                            ui.label(
                                egui::RichText::new("(Comma-separated, e.g., #rust, #linux)")
                                    .small()
                                    .weak()
                            );
                            ui.end_row();

                            ui.label("NickServ Password:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.form.nickserv_password)
                                    .password(true)
                            );
                            ui.end_row();
                            
                            ui.label("");
                            ui.label(
                                egui::RichText::new("(Optional, stored in system keyring)")
                                    .small()
                                    .weak()
                            );
                            ui.end_row();
                        });

                    ui.add_space(4.0);
                    ui.checkbox(&mut self.form.auto_connect, "Auto-connect on startup");
                    ui.checkbox(&mut self.form.use_tls, "🔒 Use TLS/SSL encryption");

                    ui.separator();
                    ui.horizontal(|ui| {
                        let can_save = self.form.is_valid();
                        
                        if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                            let network = self.form.to_network();
                            
                            if let Some(idx) = self.editing_index {
                                // Editing existing
                                action = Some(DialogAction::NetworkSave {
                                    index: Some(idx),
                                    network: network.clone(),
                                });
                                self.networks[idx] = network;
                            } else {
                                // Adding new
                                action = Some(DialogAction::NetworkSave {
                                    index: None,
                                    network: network.clone(),
                                });
                                self.networks.push(network);
                            }
                            
                            self.modified = true;
                            self.cancel_edit();
                        }

                        if ui.button("Cancel").clicked() {
                            self.cancel_edit();
                        }
                    });
                }

                // Close on Escape (only if not editing)
                if !self.is_editing() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    should_close = true;
                }
                
                // Process deferred actions after UI is done
                if let Some(name) = toggle_expanded {
                    if self.expanded.contains(&name) {
                        self.expanded.remove(&name);
                    } else {
                        self.expanded.insert(name);
                    }
                }
                
                if let Some(idx) = edit_index {
                    self.start_edit(idx);
                }
                
                if let Some(network) = connect_network {
                    action = Some(DialogAction::NetworkConnect(network));
                }
            });

        let still_open = window_open && !should_close;
        (action, still_open)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_form_default() {
        let form = NetworkForm::default();
        assert!(form.name.is_empty());
        assert!(form.servers.is_empty());
        assert!(!form.is_valid());
    }

    #[test]
    fn test_network_form_from_network() {
        let network = Network {
            name: "TestNet".to_string(),
            servers: vec!["irc.test.com:6667".to_string(), "irc2.test.com:6697".to_string()],
            nick: "testuser".to_string(),
            auto_connect: true,
            favorite_channels: vec!["#test".to_string(), "#rust".to_string()],
            nickserv_password: Some("secret".to_string()),
            use_tls: true,
            auto_reconnect: true,
        };
        
        let form = NetworkForm::from_network(&network);
        
        assert_eq!(form.name, "TestNet");
        assert_eq!(form.servers, "irc.test.com:6667, irc2.test.com:6697");
        assert_eq!(form.nick, "testuser");
        assert!(form.auto_connect);
        assert_eq!(form.favorite_channels, "#test, #rust");
        assert_eq!(form.nickserv_password, "secret");
        assert!(form.use_tls);
    }

    #[test]
    fn test_network_form_to_network() {
        let form = NetworkForm {
            name: "NewNet".to_string(),
            servers: "irc.new.com:6667, irc2.new.com:6697".to_string(),
            nick: "newuser".to_string(),
            auto_connect: false,
            favorite_channels: "#new, #test".to_string(),
            nickserv_password: String::new(),
            use_tls: false,
        };
        
        let network = form.to_network();
        
        assert_eq!(network.name, "NewNet");
        assert_eq!(network.servers, vec!["irc.new.com:6667", "irc2.new.com:6697"]);
        assert_eq!(network.nick, "newuser");
        assert!(!network.auto_connect);
        assert_eq!(network.favorite_channels, vec!["#new", "#test"]);
        assert!(network.nickserv_password.is_none());
        assert!(!network.use_tls);
    }

    #[test]
    fn test_network_form_is_valid() {
        let mut form = NetworkForm::default();
        assert!(!form.is_valid());
        
        form.name = "Test".to_string();
        assert!(!form.is_valid());
        
        form.servers = "irc.test.com:6667".to_string();
        assert!(!form.is_valid());
        
        form.nick = "testuser".to_string();
        assert!(form.is_valid());
    }

    #[test]
    fn test_network_manager_creation() {
        let networks = vec![
            Network {
                name: "Net1".to_string(),
                servers: vec!["irc.net1.com:6667".to_string()],
                nick: "user1".to_string(),
                auto_connect: false,
                favorite_channels: vec![],
                nickserv_password: None,
                use_tls: false,
                auto_reconnect: true,
            },
        ];
        
        let dialog = NetworkManagerDialog::new(networks.clone());
        
        assert_eq!(dialog.networks.len(), 1);
        assert_eq!(dialog.networks[0].name, "Net1");
        assert!(!dialog.was_modified());
    }

    #[test]
    fn test_network_manager_get_networks() {
        let networks = vec![
            Network {
                name: "Net1".to_string(),
                servers: vec!["irc.net1.com:6667".to_string()],
                nick: "user1".to_string(),
                auto_connect: false,
                favorite_channels: vec![],
                nickserv_password: None,
                use_tls: false,
                auto_reconnect: true,
            },
        ];
        
        let dialog = NetworkManagerDialog::new(networks);
        let result = dialog.get_networks();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Net1");
    }
}
