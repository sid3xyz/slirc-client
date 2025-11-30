//! Side panel rendering (channel list, user list).

use std::collections::HashMap;

use eframe::egui::{self, Stroke};

use crate::buffer::ChannelBuffer;
use crate::protocol::UserInfo;
use crate::ui::theme::{panel_colors, spacing};

/// Render the left channel list panel.
pub fn render_channel_list(
    ctx: &egui::Context,
    buffers: &HashMap<String, ChannelBuffer>,
    buffers_order: &[String],
    active_buffer: &mut String,
    context_menu_visible: &mut bool,
    context_menu_target: &mut Option<String>,
) {
    let dark_mode = ctx.style().visuals.dark_mode;
    let sidebar_bg = panel_colors::sidebar_bg(dark_mode);

    egui::SidePanel::left("buffers_panel")
        .resizable(true)
        .default_width(180.0)
        .frame(
            egui::Frame::new()
                .fill(sidebar_bg)
                .inner_margin(egui::Margin::same(spacing::PANEL_MARGIN))
                .stroke(Stroke::new(1.0, panel_colors::separator(dark_mode))),
        )
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Channels")
                    .size(13.0)
                    .strong()
                    .color(if dark_mode {
                        egui::Color32::from_gray(180)
                    } else {
                        egui::Color32::from_gray(80)
                    }),
            );
            ui.add_space(6.0);
            ui.vertical(|ui| {
                // Hint if no channels joined
                if buffers_order.len() <= 1 {
                    ui.label(
                        egui::RichText::new(
                            "No channels joined. Use the 'Channel' field in the top bar to join a channel.",
                        )
                        .color(egui::Color32::LIGHT_GRAY),
                    );
                    ui.separator();
                }

                for name in buffers_order {
                    let (unread, has_highlight, selected) = if let Some(b) = buffers.get(name) {
                        (
                            b.unread_count,
                            b.has_highlight,
                            active_buffer == name,
                        )
                    } else {
                        (0, false, false)
                    };

                    ui.horizontal(|ui| {
                        let rich = if has_highlight {
                            egui::RichText::new(name)
                                .color(egui::Color32::from_rgb(255, 100, 100))
                                .strong()
                        } else if selected {
                            egui::RichText::new(name)
                                .color(egui::Color32::WHITE)
                                .strong()
                        } else if unread > 0 {
                            egui::RichText::new(name).color(egui::Color32::from_rgb(200, 200, 255))
                        } else {
                            egui::RichText::new(name).color(egui::Color32::GRAY)
                        };

                        let resp = ui.selectable_label(selected, rich);
                        if resp.clicked() {
                            *active_buffer = name.clone();
                            if let Some(_buf) = buffers.get(name) {
                                // Note: We can't mutate here, caller handles this
                            }
                        }
                        if resp.secondary_clicked() {
                            *context_menu_visible = true;
                            *context_menu_target = Some(name.clone());
                        }

                        if unread > 0 {
                            ui.label(
                                egui::RichText::new(format!("({})", unread))
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                                    .small(),
                            );
                        }
                    });
                }
            });
        });
}

/// Render the right user list panel.
pub fn render_user_list(
    ctx: &egui::Context,
    buffer: &ChannelBuffer,
    _active_buffer: &str,
    _nickname_input: &str,
    context_menu_visible: &mut bool,
    context_menu_target: &mut Option<String>,
) {
    let dark_mode = ctx.style().visuals.dark_mode;
    let sidebar_bg = panel_colors::sidebar_bg(dark_mode);

    egui::SidePanel::right("users_panel")
        .resizable(true)
        .default_width(140.0)
        .frame(
            egui::Frame::new()
                .fill(sidebar_bg)
                .inner_margin(egui::Margin::same(spacing::PANEL_MARGIN))
                .stroke(Stroke::new(1.0, panel_colors::separator(dark_mode))),
        )
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Users")
                        .size(13.0)
                        .strong()
                        .color(if dark_mode {
                            egui::Color32::from_gray(180)
                        } else {
                            egui::Color32::from_gray(80)
                        }),
                );
                ui.label(
                    egui::RichText::new(format!("({})", buffer.users.len()))
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            });
            ui.add_space(6.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for user in &buffer.users {
                    let prefix = user.prefix.map(|c| c.to_string()).unwrap_or_default();
                    let nick_color = crate::ui::theme::prefix_color(user.prefix);

                    let label_text = format!("{}{}", prefix, user.nick);
                    let resp = ui.selectable_label(false, egui::RichText::new(&label_text).color(nick_color));
                    
                    if resp.secondary_clicked() {
                        *context_menu_visible = true;
                        *context_menu_target = Some(format!("user:{}", user.nick));
                    }
                    
                    if resp.double_clicked() {
                        // Open PM - handled by caller
                    }
                }
            });
        });
}

/// Sort users by prefix rank (ops first) then alphabetically.
pub fn sort_users(users: &mut [UserInfo]) {
    users.sort_by(|a, b| {
        let ar = crate::ui::theme::prefix_rank(a.prefix);
        let br = crate::ui::theme::prefix_rank(b.prefix);
        br.cmp(&ar).then(a.nick.cmp(&b.nick))
    });
}
