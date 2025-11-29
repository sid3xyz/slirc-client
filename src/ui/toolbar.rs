//! Top toolbar rendering with connection controls.

use crossbeam_channel::Sender;
use eframe::egui;

use crate::protocol::BackendAction;

/// Render the top toolbar with connection controls and menus.
pub fn render_toolbar(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    server_input: &mut String,
    nickname_input: &mut String,
    channel_input: &mut String,
    is_connected: bool,
    use_tls: &mut bool,
    action_tx: &Sender<BackendAction>,
    network_manager_open: &mut bool,
    nick_change_dialog_open: &mut bool,
    nick_change_input: &mut String,
    show_channel_list: &mut bool,
    show_user_list: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.spacing_mut().button_padding = egui::vec2(4.0, 2.0);

        // Main menu button
        ui.menu_button("≡", |ui| {
            ui.set_min_width(150.0);
            if ui.button("Network List...").clicked() {
                *network_manager_open = true;
                ui.close_menu();
            }
            ui.separator();
            if ui
                .add_enabled(!is_connected, egui::Button::new("Connect"))
                .clicked()
            {
                *network_manager_open = true;
                ui.close_menu();
            }
            if ui
                .add_enabled(is_connected, egui::Button::new("Disconnect"))
                .clicked()
            {
                let _ = action_tx.send(BackendAction::Disconnect);
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Quit").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        // View menu
        ui.menu_button("View", |ui| {
            ui.checkbox(show_channel_list, "Channel List");
            ui.checkbox(show_user_list, "User List");
        });

        ui.separator();

        if !is_connected {
            // Server/nick inputs when disconnected
            ui.add(
                egui::TextEdit::singleline(server_input)
                    .hint_text("irc.server.net:6667")
                    .desired_width(160.0),
            );

            ui.add(
                egui::TextEdit::singleline(nickname_input)
                    .hint_text("nickname")
                    .desired_width(80.0),
            );

            ui.checkbox(use_tls, "🔒 TLS");

            if ui.button("Connect").clicked() {
                let parts: Vec<&str> = server_input.split(':').collect();
                let server = parts[0].to_string();
                let port: u16 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(if *use_tls { 6697 } else { 6667 });

                let _ = action_tx.send(BackendAction::Connect {
                    server,
                    port,
                    nickname: nickname_input.clone(),
                    username: nickname_input.clone(),
                    realname: format!("SLIRC User ({})", nickname_input),
                    use_tls: *use_tls,
                });
            }
        } else {
            // Nick button and channel join when connected
            if ui.button(nickname_input.as_str()).clicked() {
                *nick_change_input = nickname_input.clone();
                *nick_change_dialog_open = true;
            }

            ui.separator();
            let response = ui.add(
                egui::TextEdit::singleline(channel_input)
                    .hint_text("#channel")
                    .desired_width(120.0),
            );

            if ui.button("Join").clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                let channel = if channel_input.starts_with('#') || channel_input.starts_with('&') {
                    channel_input.clone()
                } else {
                    format!("#{}", channel_input)
                };
                let _ = action_tx.send(BackendAction::Join(channel));
                channel_input.clear();
            }
        }

        // Right side - connection status indicator
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if is_connected {
                ui.label(
                    egui::RichText::new(server_input.as_str())
                        .color(egui::Color32::DARK_GRAY)
                        .small(),
                );
                ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(0, 200, 0)));
            } else {
                ui.label(egui::RichText::new("○").color(egui::Color32::from_rgb(150, 150, 150)));
            }
        });
    });
}
