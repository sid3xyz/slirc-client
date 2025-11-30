//! Top toolbar rendering with connection controls.

use crossbeam_channel::Sender;
use eframe::egui::{self, Color32, RichText, Stroke};

use crate::protocol::BackendAction;

/// Actions that the toolbar can request
#[derive(Debug, Clone, PartialEq)]
pub enum ToolbarAction {
    /// User clicked Connect button
    Connect,
    /// User wants to change nickname
    OpenNickChangeDialog,
}

/// Render the top toolbar with connection controls.
/// Returns Some(ToolbarAction) if an action was requested.
#[allow(clippy::too_many_arguments)]
pub fn render_toolbar(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    server_input: &mut String,
    nickname_input: &mut String,
    channel_input: &mut String,
    is_connected: bool,
    use_tls: &mut bool,
    action_tx: &Sender<BackendAction>,
) -> Option<ToolbarAction> {
    let mut toolbar_action: Option<ToolbarAction> = None;
    let dark_mode = ctx.style().visuals.dark_mode;
    let text_secondary = if dark_mode {
        Color32::from_rgb(148, 155, 164)
    } else {
        Color32::from_rgb(99, 100, 102)
    };

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);

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
                toolbar_action = Some(ToolbarAction::Connect);
            }
        } else {
            // Nick button and channel join when connected
            if ui.button(nickname_input.as_str()).clicked() {
                toolbar_action = Some(ToolbarAction::OpenNickChangeDialog);
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
                    RichText::new(server_input.as_str())
                        .color(text_secondary)
                        .small(),
                );
                ui.add_space(4.0);
                // Green glowing dot for connected
                let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                let center = rect.center();
                // Glow effect
                ui.painter().circle_filled(center, 6.0, Color32::from_rgba_unmultiplied(34, 197, 94, 40));
                ui.painter().circle_filled(center, 4.0, Color32::from_rgb(34, 197, 94));
            } else {
                // Gray hollow circle for disconnected
                let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                let center = rect.center();
                ui.painter().circle_stroke(center, 4.0, Stroke::new(1.5, Color32::from_rgb(100, 100, 100)));
            }
        });
    });

    toolbar_action
}
