//! Modal dialogs and windows (help, network manager, topic editor, etc.).

use eframe::egui;
use std::collections::HashSet;

use crate::config::Network;
use crate::protocol::BackendAction;

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

/// Render the help dialog (F1).
pub fn render_help_dialog(ctx: &egui::Context, show_help_dialog: &mut bool) {
    if !*show_help_dialog {
        return;
    }

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
        *show_help_dialog = false;
    }
}

/// Render the nick change dialog.
#[allow(dead_code)]
pub fn render_nick_change_dialog(
    ctx: &egui::Context,
    nick_change_dialog_open: &mut bool,
    nick_change_input: &mut String,
    current_nick: &str,
    action_tx: &crossbeam_channel::Sender<BackendAction>,
) {
    if !*nick_change_dialog_open {
        return;
    }

    let mut open = true;
    let mut newnick = nick_change_input.clone();
    egui::Window::new("Change Nick")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("New nickname:");
            let _resp = ui.add(egui::TextEdit::singleline(&mut newnick));
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    if !newnick.trim().is_empty() && newnick != current_nick {
                        let _ = action_tx.send(BackendAction::Nick(newnick.clone()));
                    }
                    *nick_change_dialog_open = false;
                }
                if ui.button("Cancel").clicked() {
                    *nick_change_dialog_open = false;
                }
            });
        });
    *nick_change_input = newnick;
    if !open {
        *nick_change_dialog_open = false;
    }
}

/// Render the topic editor dialog.
pub fn render_topic_editor(
    ctx: &egui::Context,
    topic_editor_open: &mut Option<String>,
    buffers: &std::collections::HashMap<String, crate::buffer::ChannelBuffer>,
    action_tx: &crossbeam_channel::Sender<BackendAction>,
) {
    if let Some(channel) = topic_editor_open.clone() {
        let mut open = true;
        let initial_topic = buffers
            .get(&channel)
            .map(|b| b.topic.clone())
            .unwrap_or_default();
        let mut new_topic = initial_topic.clone();
        egui::Window::new(format!("Edit Topic: {}", channel))
            .open(&mut open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Edit the channel topic:");
                let _response = ui.add(egui::TextEdit::multiline(&mut new_topic).desired_rows(3));
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        if !new_topic.is_empty() {
                            let _ = action_tx.send(BackendAction::SetTopic {
                                channel: channel.clone(),
                                topic: new_topic.clone(),
                            });
                        }
                        *topic_editor_open = None;
                    }
                    if ui.button("Cancel").clicked() {
                        *topic_editor_open = None;
                    }
                });
            });
        if !open {
            *topic_editor_open = None;
        }
    }
}

/// Render the network manager dialog.
#[allow(clippy::too_many_arguments)]
pub fn render_network_manager(
    ctx: &egui::Context,
    network_manager_open: &mut bool,
    networks: &mut Vec<Network>,
    expanded_networks: &mut HashSet<String>,
    editing_network: &mut Option<usize>,
    network_form: &mut NetworkForm,
    action_tx: &crossbeam_channel::Sender<BackendAction>,
    save_networks_fn: &dyn Fn(&[Network]),
) {
    if !*network_manager_open {
        return;
    }

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
                    for (idx, network) in networks.iter().enumerate() {
                        let expanded = expanded_networks.contains(&network.name);
                        ui.horizontal(|ui| {
                            if ui.small_button(if expanded { "▾" } else { "▸" }).clicked() {
                                if expanded {
                                    expanded_networks.remove(&network.name);
                                } else {
                                    expanded_networks.insert(network.name.clone());
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
                                *editing_network = Some(idx);
                                let net = &networks[idx];
                                *network_form = NetworkForm {
                                    name: net.name.clone(),
                                    servers: net.servers.join(", "),
                                    nick: net.nick.clone(),
                                    auto_connect: net.auto_connect,
                                    favorite_channels: net.favorite_channels.join(", "),
                                    nickserv_password: net.nickserv_password.clone().unwrap_or_default(),
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

                                    let _ = action_tx.send(BackendAction::Connect {
                                        server,
                                        port,
                                        nickname: network.nick.clone(),
                                        username: network.nick.clone(),
                                        realname: format!("SLIRC User ({})", network.nick),
                                        use_tls: network.use_tls,
                                    });

                                    // Auto-join favorite channels
                                    for channel in &network.favorite_channels {
                                        let _ = action_tx.send(BackendAction::Join(channel.clone()));
                                    }

                                    *network_manager_open = false;
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
                        networks.remove(idx);
                        save_networks_fn(networks);
                    }
                });

            ui.separator();

            // Add/Edit network form
            if editing_network.is_some() || ui.button("Add Network").clicked() && editing_network.is_none() {
                if editing_network.is_none() {
                    // Start adding a new network
                    *network_form = NetworkForm::default();
                }

                ui.heading(if editing_network.is_some() {
                    "Edit Network"
                } else {
                    "New Network"
                });
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut network_form.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Servers:");
                    ui.text_edit_singleline(&mut network_form.servers);
                });
                ui.label("(Comma-separated, e.g., irc.libera.chat:6667, irc.libera.chat:6697)");

                ui.horizontal(|ui| {
                    ui.label("Nickname:");
                    ui.text_edit_singleline(&mut network_form.nick);
                });

                ui.checkbox(&mut network_form.auto_connect, "Auto-connect on startup");

                ui.horizontal(|ui| {
                    ui.label("Favorite Channels:");
                    ui.text_edit_singleline(&mut network_form.favorite_channels);
                });
                ui.label("(Comma-separated, e.g., #channel1, #channel2)");

                ui.horizontal(|ui| {
                    ui.label("NickServ Password:");
                    ui.add(egui::TextEdit::singleline(&mut network_form.nickserv_password).password(true));
                });
                ui.label("(Optional, stored securely in system keyring)");

                ui.checkbox(&mut network_form.use_tls, "🔒 Use TLS/SSL encryption");

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        let servers: Vec<String> = network_form
                            .servers
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();

                        let favorite_channels: Vec<String> = network_form
                            .favorite_channels
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();

                        let network = Network {
                            name: network_form.name.clone(),
                            servers,
                            nick: network_form.nick.clone(),
                            auto_connect: network_form.auto_connect,
                            favorite_channels,
                            nickserv_password: if network_form.nickserv_password.is_empty() {
                                None
                            } else {
                                Some(network_form.nickserv_password.clone())
                            },
                            use_tls: network_form.use_tls,
                        };

                        if let Some(idx) = *editing_network {
                            networks[idx] = network;
                        } else {
                            networks.push(network);
                        }

                        save_networks_fn(networks);
                        *editing_network = None;
                        *network_form = NetworkForm::default();
                    }

                    if ui.button("Cancel").clicked() {
                        *editing_network = None;
                        *network_form = NetworkForm::default();
                    }
                });
            }
        });

    if !open {
        *network_manager_open = false;
        *editing_network = None;
    }
}

/// Render floating status toasts (top-right corner).
pub fn render_status_toasts(
    ctx: &egui::Context,
    status_messages: &[(String, std::time::Instant)],
) {
    if status_messages.is_empty() {
        return;
    }

    let msgs: Vec<String> = status_messages.iter().map(|(m, _t)| m.clone()).collect();
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
