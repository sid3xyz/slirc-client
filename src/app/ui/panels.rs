//! Menu bar, toolbar, and central panel rendering

use eframe::egui;

use crate::app::SlircApp;
use crate::ui;

impl SlircApp {
    /// Render the menu bar at the top of the window
    pub(in crate::app) fn render_menu_bar(&mut self, ctx: &egui::Context) {
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
                            self.dialogs
                                .open_network_manager(self.state.networks.clone());
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
    pub(in crate::app) fn render_toolbar(&mut self, ctx: &egui::Context) {
        let theme = self.get_theme();

        // Compact toolbar below menu bar (for quick actions)
        let toolbar_bg = theme.surface[1];
        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(toolbar_bg)
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .stroke(egui::Stroke::new(1.0, theme.border_medium)),
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

    /// Render the central panel with messages
    pub(in crate::app) fn render_central_panel(&mut self, ctx: &egui::Context) {
        let theme = self.get_theme();
        let chat_bg = theme.surface[0]; // Base surface for messages
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(chat_bg).inner_margin(0.0))
            .show(ctx, |ui| {
                // Use state.our_nick if connected, otherwise fall back to UI input
                let current_nick = if self.state.our_nick.is_empty() {
                    &self.connection.nickname
                } else {
                    &self.state.our_nick
                };

                // Render topic bar for channels (above messages)
                if let Some(topic_action) = ui::topic_bar::render_topic_bar(
                    ui,
                    &self.state.active_buffer,
                    &self.state.buffers,
                    current_nick,
                    &theme,
                    &mut self.state.system_log,
                ) {
                    match topic_action {
                        ui::topic_bar::TopicBarAction::EditTopic(channel) => {
                            let current_topic = self
                                .state
                                .buffers
                                .get(&channel)
                                .map(|b| b.topic.clone())
                                .unwrap_or_default();
                            self.dialogs.open_topic_editor(&channel, &current_topic);
                        }
                        ui::topic_bar::TopicBarAction::ToggleMute => {
                            if let Some(buffer) =
                                self.state.buffers.get_mut(&self.state.active_buffer)
                            {
                                buffer.notifications_muted = !buffer.notifications_muted;
                            }
                        }
                        ui::topic_bar::TopicBarAction::OpenSearch => {
                            // TODO: Implement channel search (Phase 6)
                            self.state
                                .system_log
                                .push("Search not implemented yet".to_string());
                        }
                        ui::topic_bar::TopicBarAction::ShowPinned => {
                            // TODO: Implement pinned messages view
                            self.state
                                .system_log
                                .push("Pinned messages not implemented yet".to_string());
                        }
                    }
                }

                // Messages panel with inner margin
                egui::Frame::new()
                    .fill(chat_bg)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui::messages::render_messages(
                            ctx,
                            ui,
                            &self.state.active_buffer,
                            &self.state.buffers,
                            &self.state.system_log,
                            current_nick,
                        );
                    });
            });
    }
}
