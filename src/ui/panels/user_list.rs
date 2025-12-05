//! User list panel rendering with role grouping and status indicators.

use crate::buffer::ChannelBuffer;
use crate::protocol::UserInfo;
use crate::ui::theme::{self, SlircTheme};
use eframe::egui::{self, Color32, Stroke};

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
    let theme = if dark_mode {
        SlircTheme::dark()
    } else {
        SlircTheme::light()
    };
    let sidebar_bg = theme.surface[1];

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
                .stroke(Stroke::new(1.0, theme.border_medium)),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Operators section
                    if !ops.is_empty() {
                        render_user_section(
                            ui,
                            "OPERATORS",
                            &ops,
                            &theme,
                            context_menu_visible,
                            context_menu_target,
                        );
                    }

                    // Voiced section
                    if !voiced.is_empty() {
                        render_user_section(
                            ui,
                            "VOICED",
                            &voiced,
                            &theme,
                            context_menu_visible,
                            context_menu_target,
                        );
                    }

                    // Regular users section
                    if !regular.is_empty() {
                        let label = format!("ONLINE — {}", regular.len());
                        render_user_section(
                            ui,
                            &label,
                            &regular,
                            &theme,
                            context_menu_visible,
                            context_menu_target,
                        );
                    }
                });
        });
}

/// Render a section of users (Operators, Voiced, Online)
fn render_user_section(
    ui: &mut egui::Ui,
    title: &str,
    users: &[&UserInfo],
    theme: &SlircTheme,
    context_menu_visible: &mut bool,
    context_menu_target: &mut Option<String>,
) {
    // Section header with icon
    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.add_space(16.0);

        // Section icon
        let icon = if title.starts_with("OPERATORS") {
            "★"
        } else if title.starts_with("VOICED") {
            "♦"
        } else {
            "●"
        };

        ui.label(egui::RichText::new(icon).size(9.0).color(theme.text_muted));
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(title)
                .size(10.0)
                .strong()
                .color(theme.text_muted),
        );
    });
    ui.add_space(6.0);

    // Subtle separator
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        let sep_rect = egui::Rect::from_min_size(
            ui.cursor().min,
            egui::vec2(ui.available_width() - 32.0, 1.0),
        );
        ui.painter().rect_filled(sep_rect, 0.0, theme.surface[3]);
    });
    ui.add_space(8.0);

    // Users in section
    for user in users {
        let clicked = render_user_item(ui, user, theme);
        if clicked.1 {
            *context_menu_visible = true;
            *context_menu_target = Some(format!("user:{}", user.nick));
        }
    }
}

/// Render a single user item
/// Returns (left_clicked, right_clicked)
fn render_user_item(ui: &mut egui::Ui, user: &UserInfo, theme: &SlircTheme) -> (bool, bool) {
    let height = 32.0; // Increased from 28 for better touch targets
    let available_width = ui.available_width();

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(available_width, height), egui::Sense::click());

    let hovered = response.hovered();

    // Hover background with rounded corners
    if hovered {
        ui.painter().rect_filled(rect, 4.0, theme.surface[3]);
    }

    // Avatar (smaller for user list)
    ui.painter().circle_filled(
        egui::pos2(rect.min.x + 20.0, rect.center().y),
        10.0,
        theme::nick_color(&user.nick),
    );

    // Role indicator overlay on avatar
    let status_color = theme::prefix_color(theme, user.prefix);
    let ring_center = egui::pos2(rect.min.x + 20.0, rect.center().y);
    ui.painter()
        .circle_stroke(ring_center, 10.0, egui::Stroke::new(2.0, status_color));

    // Username with role color hint
    let nick_color = if user.prefix.is_some() {
        status_color
    } else {
        theme.text_secondary
    };

    ui.painter().text(
        egui::pos2(rect.min.x + 38.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &user.nick,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
        nick_color,
    );

    // Role badge (for ops/voiced)
    if let Some(prefix) = user.prefix {
        let badge_char = match prefix {
            '@' => "OP",
            '~' => "OW",
            '&' => "AD",
            '%' => "HO",
            '+' => "V",
            _ => "",
        };

        if !badge_char.is_empty() {
            let badge_font = egui::FontId::new(8.0, egui::FontFamily::Proportional);
            let galley =
                ui.fonts(|f| f.layout_no_wrap(badge_char.to_string(), badge_font, Color32::WHITE));

            let badge_width = galley.size().x + 6.0;
            let badge_height = 14.0;
            let badge_rect = egui::Rect::from_min_size(
                egui::pos2(
                    rect.max.x - badge_width - 12.0,
                    rect.center().y - badge_height / 2.0,
                ),
                egui::vec2(badge_width, badge_height),
            );

            ui.painter().rect_filled(badge_rect, 3.0, status_color);
            ui.painter().galley(
                badge_rect.center() - galley.size() / 2.0,
                galley,
                Color32::WHITE,
            );
        }
    }

    // Tooltip on hover with full role name
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
