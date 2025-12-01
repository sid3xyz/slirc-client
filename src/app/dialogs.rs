//! Dialog rendering orchestration

use eframe::egui;

use super::SlircApp;
use crate::config::load_nickserv_password;
use crate::protocol::BackendAction;
use crate::ui;
use crate::ui::dialogs::DialogAction;

impl SlircApp {
    /// Render all dialogs and handle their actions
    pub(super) fn render_dialogs(&mut self, ctx: &egui::Context) {
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
}
