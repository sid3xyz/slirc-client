//! Modern side panel rendering (channel list, user list).
//! Features: hover states, unread badges, status indicators, improved layout.

use std::collections::HashMap;

use eframe::egui::{self, Color32, Stroke};

use crate::buffer::ChannelBuffer;
use crate::protocol::UserInfo;
use crate::ui::theme::{self, dark, panel_colors, spacing, text_colors};

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
        .default_width(220.0)
        .min_width(180.0)
        .frame(
            egui::Frame::new()
                .fill(sidebar_bg)
                .inner_margin(egui::Margin::same(0))
                .stroke(Stroke::new(1.0, panel_colors::separator(dark_mode))),
        )
        .show(ctx, |ui| {
            // Header
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("CHANNELS")
                        .size(11.0)
                        .strong()
                        .color(text_colors::muted(dark_mode)),
                );
            });
            ui.add_space(8.0);

            // Channel list
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Hint if no channels joined
                    if buffers_order.len() <= 1 {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new("No channels yet")
                                    .size(12.0)
                                    .color(text_colors::muted(dark_mode))
                                    .italics(),
                            );
                        });
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new("Join one using the toolbar")
                                    .size(11.0)
                                    .color(text_colors::muted(dark_mode)),
                            );
                        });
                    }

                    for name in buffers_order {
                        let (unread, has_highlight, selected) = if let Some(b) = buffers.get(name) {
                            (b.unread_count, b.has_highlight, active_buffer == name)
                        } else {
                            (0, false, false)
                        };

                        let clicked = render_channel_item(
                            ui,
                            name,
                            unread,
                            has_highlight,
                            selected,
                            dark_mode,
                        );

                        if clicked.0 {
                            *active_buffer = name.clone();
                        }
                        if clicked.1 {
                            *context_menu_visible = true;
                            *context_menu_target = Some(name.clone());
                        }
                    }
                });
        });
}

/// Render a single channel item with modern styling
/// Returns (left_clicked, right_clicked)
fn render_channel_item(
    ui: &mut egui::Ui,
    name: &str,
    unread: usize,
    has_highlight: bool,
    selected: bool,
    dark_mode: bool,
) -> (bool, bool) {
    let height = spacing::CHANNEL_ITEM_HEIGHT;
    let available_width = ui.available_width();

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(available_width, height),
        egui::Sense::click(),
    );

    let hovered = response.hovered();

    // Background color
    let bg_color = if selected {
        panel_colors::active_bg(dark_mode)
    } else if hovered {
        panel_colors::hover_bg(dark_mode)
    } else {
        Color32::TRANSPARENT
    };

    // Draw background with left accent for selected
    if selected || hovered {
        ui.painter().rect_filled(rect, 0.0, bg_color);
    }

    // Selected indicator bar on left
    if selected {
        let indicator_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(3.0, height),
        );
        ui.painter().rect_filled(indicator_rect, 0.0, dark::ACCENT_BLUE);
    }

    // Icon
    let icon = if name == "System" {
        "⚙"
    } else if name.starts_with('#') || name.starts_with('&') {
        "#"
    } else {
        "👤"
    };

    let icon_color = if selected || has_highlight || unread > 0 {
        text_colors::primary(dark_mode)
    } else {
        text_colors::muted(dark_mode)
    };

    ui.painter().text(
        egui::pos2(rect.min.x + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        icon,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
        icon_color,
    );

    // Channel name
    let text_color = if has_highlight {
        dark::ACCENT_RED
    } else if selected {
        text_colors::primary(dark_mode)
    } else if unread > 0 {
        text_colors::primary(dark_mode)
    } else {
        text_colors::secondary(dark_mode)
    };

    let display_name = if name.starts_with('#') {
        &name[1..]
    } else {
        name
    };

    let font = if selected || unread > 0 {
        egui::FontId::new(13.0, egui::FontFamily::Proportional)
    } else {
        egui::FontId::new(13.0, egui::FontFamily::Proportional)
    };

    ui.painter().text(
        egui::pos2(rect.min.x + 36.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        display_name,
        font,
        text_color,
    );

    // Unread badge
    if unread > 0 {
        let badge_text = if unread > 99 {
            "99+".to_string()
        } else {
            unread.to_string()
        };

        let badge_color = if has_highlight {
            dark::ACCENT_RED
        } else {
            dark::ACCENT_BLUE
        };

        let badge_font = egui::FontId::new(10.0, egui::FontFamily::Proportional);
        let galley = ui.fonts(|f| f.layout_no_wrap(badge_text, badge_font, Color32::WHITE));

        let badge_width = galley.size().x.max(14.0) + 8.0;
        let badge_height = 16.0;
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(rect.max.x - badge_width - 12.0, rect.center().y - badge_height / 2.0),
            egui::vec2(badge_width, badge_height),
        );

        ui.painter().rect_filled(badge_rect, badge_height / 2.0, badge_color);
        ui.painter().galley(
            badge_rect.center() - galley.size() / 2.0,
            galley,
            Color32::WHITE,
        );
    }

    (response.clicked(), response.secondary_clicked())
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

    // Group users by role
    let (ops, voiced, regular): (Vec<_>, Vec<_>, Vec<_>) = {
        let mut ops = Vec::new();
        let mut voiced = Vec::new();
        let mut regular = Vec::new();

        for user in &buffer.users {
            match user.prefix {
                Some('@') | Some('~') | Some('&') => ops.push(user),
                Some('+') | Some('%') => voiced.push(user),
                _ => regular.push(user),
            }
        }

        (ops, voiced, regular)
    };

    egui::SidePanel::right("users_panel")
        .resizable(true)
        .default_width(180.0)
        .min_width(140.0)
        .frame(
            egui::Frame::new()
                .fill(sidebar_bg)
                .inner_margin(egui::Margin::same(0))
                .stroke(Stroke::new(1.0, panel_colors::separator(dark_mode))),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Operators section
                    if !ops.is_empty() {
                        render_user_section(ui, "OPERATORS", &ops, dark_mode, context_menu_visible, context_menu_target);
                    }

                    // Voiced section
                    if !voiced.is_empty() {
                        render_user_section(ui, "VOICED", &voiced, dark_mode, context_menu_visible, context_menu_target);
                    }

                    // Regular users section
                    if !regular.is_empty() {
                        let label = format!("ONLINE — {}", regular.len());
                        render_user_section(ui, &label, &regular, dark_mode, context_menu_visible, context_menu_target);
                    }
                });
        });
}

/// Render a section of users (Operators, Voiced, Online)
fn render_user_section(
    ui: &mut egui::Ui,
    title: &str,
    users: &[&UserInfo],
    dark_mode: bool,
    context_menu_visible: &mut bool,
    context_menu_target: &mut Option<String>,
) {
    // Section header
    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new(title)
                .size(10.0)
                .strong()
                .color(text_colors::muted(dark_mode)),
        );
    });
    ui.add_space(4.0);

    // Users in section
    for user in users {
        let clicked = render_user_item(ui, user, dark_mode);
        if clicked.1 {
            *context_menu_visible = true;
            *context_menu_target = Some(format!("user:{}", user.nick));
        }
    }
}

/// Render a single user item
/// Returns (left_clicked, right_clicked)
fn render_user_item(ui: &mut egui::Ui, user: &UserInfo, dark_mode: bool) -> (bool, bool) {
    let height = spacing::USER_ITEM_HEIGHT;
    let available_width = ui.available_width();

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(available_width, height),
        egui::Sense::click(),
    );

    let hovered = response.hovered();

    // Hover background
    if hovered {
        ui.painter().rect_filled(rect, 0.0, panel_colors::hover_bg(dark_mode));
    }

    // Status dot
    let status_color = theme::prefix_color(user.prefix);
    let dot_center = egui::pos2(rect.min.x + 20.0, rect.center().y);
    ui.painter().circle_filled(dot_center, 4.0, status_color);

    // Username
    let nick_color = theme::nick_color(&user.nick);
    ui.painter().text(
        egui::pos2(rect.min.x + 32.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &user.nick,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
        nick_color,
    );

    // Tooltip on hover
    if hovered {
        let prefix_text = match user.prefix {
            Some('@') => "Operator",
            Some('~') => "Owner",
            Some('&') => "Admin",
            Some('%') => "Half-Op",
            Some('+') => "Voice",
            _ => "User",
        };
        response.clone().on_hover_text(prefix_text);
    }

    (response.clicked(), response.secondary_clicked())
}

/// Sort users by prefix rank (ops first) then alphabetically.
pub fn sort_users(users: &mut [UserInfo]) {
    users.sort_by(|a, b| {
        let ar = theme::prefix_rank(a.prefix);
        let br = theme::prefix_rank(b.prefix);
        br.cmp(&ar).then(a.nick.cmp(&b.nick))
    });
}
