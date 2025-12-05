//! Channel/buffer list panel rendering with search, collapsible sections, and unread badges.

use crate::buffer::ChannelBuffer;
use crate::ui::theme::SlircTheme;
use eframe::egui::{self, Color32, Stroke};
use std::collections::HashMap;

/// Render the left channel list panel.
#[allow(clippy::too_many_arguments)]
pub fn render_channel_list(
    ctx: &egui::Context,
    buffers: &HashMap<String, ChannelBuffer>,
    buffers_order: &[String],
    active_buffer: &mut String,
    context_menu_visible: &mut bool,
    context_menu_target: &mut Option<String>,
    collapsed_sections: &mut std::collections::HashSet<String>,
    channel_filter: &mut String,
) {
    let dark_mode = ctx.style().visuals.dark_mode;
    let theme = if dark_mode {
        SlircTheme::dark()
    } else {
        SlircTheme::light()
    };
    let sidebar_bg = theme.surface[1];

    egui::SidePanel::left("buffers_panel")
        .resizable(true)
        .default_width(220.0)
        .min_width(180.0)
        .frame(
            egui::Frame::new()
                .fill(sidebar_bg)
                .inner_margin(egui::Margin::same(0))
                .stroke(Stroke::new(1.0, theme.border_medium)),
        )
        .show(ctx, |ui| {
            // Header with separator
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("CHANNELS")
                        .size(11.0)
                        .strong()
                        .color(theme.text_muted),
                );
            });
            ui.add_space(6.0);

            // Subtle separator line
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let sep_rect = egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::vec2(ui.available_width() - 32.0, 1.0),
                );
                ui.painter().rect_filled(sep_rect, 0.0, theme.surface[3]);
            });
            ui.add_space(8.0);

            // Search/filter input (Phase 3)
            if buffers_order.len() > 3 {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let search_response = ui.add_sized(
                        egui::vec2(ui.available_width() - 32.0, 32.0),
                        egui::TextEdit::singleline(channel_filter)
                            .hint_text("🔍 Search channels...")
                            .desired_width(f32::INFINITY),
                    );

                    // Clear button when text present
                    if !channel_filter.is_empty() {
                        ui.add_space(-28.0);
                        if ui.small_button("✕").clicked() {
                            channel_filter.clear();
                        }
                    }

                    // Focus on Ctrl+K handled in app.rs
                    if search_response.changed() {
                        // Filter will be applied below
                    }
                });
                ui.add_space(8.0);
            }

            // Channel list with collapsible sections (Phase 3)
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Filter and categorize buffers
                    let filtered_buffers: Vec<&String> = buffers_order
                        .iter()
                        .filter(|name| {
                            if channel_filter.is_empty() {
                                true
                            } else {
                                name.to_lowercase().contains(&channel_filter.to_lowercase())
                            }
                        })
                        .collect();

                    let mut channels = Vec::new();
                    let mut dms = Vec::new();
                    let mut system = Vec::new();

                    for name in filtered_buffers {
                        if name.as_str() == "System" {
                            system.push(name);
                        } else if name.starts_with('#') || name.starts_with('&') {
                            channels.push(name);
                        } else {
                            dms.push(name);
                        }
                    }

                    // CHANNELS section (collapsible - Phase 3)
                    if !channels.is_empty() {
                        let channels_collapsed = collapsed_sections.contains("channels");
                        ui.add_space(4.0);

                        let header_response = ui
                            .horizontal(|ui| {
                                ui.add_space(16.0);
                                let caret = if channels_collapsed { "▶" } else { "▼" };
                                ui.label(
                                    egui::RichText::new(caret).size(9.0).color(theme.text_muted),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("CHANNELS")
                                        .size(11.0)
                                        .strong()
                                        .color(theme.text_muted),
                                );
                            })
                            .response;

                        if header_response.clicked() {
                            if channels_collapsed {
                                collapsed_sections.remove("channels");
                            } else {
                                collapsed_sections.insert("channels".to_string());
                            }
                        }

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            let sep_rect = egui::Rect::from_min_size(
                                ui.cursor().min,
                                egui::vec2(ui.available_width() - 32.0, 1.0),
                            );
                            ui.painter().rect_filled(sep_rect, 0.0, theme.surface[3]);
                        });
                        ui.add_space(8.0);

                        if !channels_collapsed {
                            for name in &channels {
                                let (unread, has_highlight, selected) =
                                    if let Some(b) = buffers.get(name.as_str()) {
                                        (
                                            b.unread_count,
                                            b.has_highlight,
                                            active_buffer == name.as_str(),
                                        )
                                    } else {
                                        (0, false, false)
                                    };

                                ui.add_space(2.0);

                                let clicked = render_channel_item(
                                    ui,
                                    name.as_str(),
                                    unread,
                                    has_highlight,
                                    selected,
                                    &theme,
                                );

                                if clicked.0 {
                                    *active_buffer = name.to_string();
                                }
                                if clicked.1 {
                                    *context_menu_visible = true;
                                    *context_menu_target = Some(name.to_string());
                                }

                                ui.add_space(2.0);
                            }
                        }
                    }

                    // PRIVATE MESSAGES section (collapsible - Phase 3)
                    if !dms.is_empty() {
                        let dms_collapsed = collapsed_sections.contains("dms");
                        ui.add_space(12.0);

                        let header_response = ui
                            .horizontal(|ui| {
                                ui.add_space(16.0);
                                let caret = if dms_collapsed { "▶" } else { "▼" };
                                ui.label(
                                    egui::RichText::new(caret).size(9.0).color(theme.text_muted),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("PRIVATE MESSAGES")
                                        .size(11.0)
                                        .strong()
                                        .color(theme.text_muted),
                                );
                            })
                            .response;

                        if header_response.clicked() {
                            if dms_collapsed {
                                collapsed_sections.remove("dms");
                            } else {
                                collapsed_sections.insert("dms".to_string());
                            }
                        }

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            let sep_rect = egui::Rect::from_min_size(
                                ui.cursor().min,
                                egui::vec2(ui.available_width() - 32.0, 1.0),
                            );
                            ui.painter().rect_filled(sep_rect, 0.0, theme.surface[3]);
                        });
                        ui.add_space(8.0);

                        if !dms_collapsed {
                            for name in &dms {
                                let (unread, has_highlight, selected) =
                                    if let Some(b) = buffers.get(name.as_str()) {
                                        (
                                            b.unread_count,
                                            b.has_highlight,
                                            active_buffer == name.as_str(),
                                        )
                                    } else {
                                        (0, false, false)
                                    };

                                ui.add_space(2.0);

                                let clicked = render_channel_item(
                                    ui,
                                    name.as_str(),
                                    unread,
                                    has_highlight,
                                    selected,
                                    &theme,
                                );

                                if clicked.0 {
                                    *active_buffer = name.to_string();
                                }
                                if clicked.1 {
                                    *context_menu_visible = true;
                                    *context_menu_target = Some(name.to_string());
                                }

                                ui.add_space(2.0);
                            }
                        }
                    }

                    // System buffer (always visible, no collapse)
                    for name in &system {
                        let (unread, has_highlight, selected) =
                            if let Some(b) = buffers.get(name.as_str()) {
                                (
                                    b.unread_count,
                                    b.has_highlight,
                                    active_buffer == name.as_str(),
                                )
                            } else {
                                (0, false, false)
                            };

                        ui.add_space(2.0);

                        let clicked = render_channel_item(
                            ui,
                            name.as_str(),
                            unread,
                            has_highlight,
                            selected,
                            &theme,
                        );

                        if clicked.0 {
                            *active_buffer = name.to_string();
                        }
                        if clicked.1 {
                            *context_menu_visible = true;
                            *context_menu_target = Some(name.to_string());
                        }

                        ui.add_space(2.0);
                    }

                    // Hint if no results after filtering
                    if !channel_filter.is_empty()
                        && channels.is_empty()
                        && dms.is_empty()
                        && system.is_empty()
                    {
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new("No matching channels")
                                    .size(12.0)
                                    .color(theme.text_muted)
                                    .italics(),
                            );
                        });
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
    theme: &SlircTheme,
) -> (bool, bool) {
    let height = 32.0;
    let available_width = ui.available_width();

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(available_width, height), egui::Sense::click());

    let hovered = response.hovered();

    // Background color
    let bg_color = if selected {
        theme.surface[4]
    } else if hovered {
        theme.surface[3]
    } else {
        Color32::TRANSPARENT
    };

    // Draw background with left accent for selected
    if selected || hovered {
        ui.painter().rect_filled(rect, 6.0, bg_color);
    }

    // Selected indicator bar on left
    if selected {
        let indicator_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 8.0, rect.center().y - 10.0),
            egui::vec2(3.0, 20.0),
        );
        ui.painter().rect_filled(indicator_rect, 1.5, theme.accent);
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
        theme.text_primary
    } else {
        theme.text_muted
    };

    ui.painter().text(
        egui::pos2(rect.min.x + 20.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        icon,
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
        icon_color,
    );

    // Channel name
    let text_color = if has_highlight {
        theme.error
    } else if selected || unread > 0 {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let display_name = if let Some(stripped) = name.strip_prefix('#') {
        stripped
    } else {
        name
    };

    let font = egui::FontId::new(13.0, egui::FontFamily::Proportional);

    ui.painter().text(
        egui::pos2(rect.min.x + 44.0, rect.center().y),
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
            theme.error
        } else {
            theme.accent
        };

        let badge_font = egui::FontId::new(10.0, egui::FontFamily::Proportional);
        let galley = ui.fonts(|f| f.layout_no_wrap(badge_text, badge_font, Color32::WHITE));

        let badge_width = galley.size().x.max(16.0) + 10.0;
        let badge_height = 18.0;
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(
                rect.max.x - badge_width - 16.0,
                rect.center().y - badge_height / 2.0,
            ),
            egui::vec2(badge_width, badge_height),
        );

        // Draw subtle shadow for depth
        let shadow_rect = badge_rect.translate(egui::vec2(0.0, 1.0));
        ui.painter().rect_filled(
            shadow_rect,
            badge_height / 2.0,
            Color32::from_black_alpha(20),
        );

        ui.painter()
            .rect_filled(badge_rect, badge_height / 2.0, badge_color);
        ui.painter().galley(
            badge_rect.center() - galley.size() / 2.0,
            galley,
            Color32::WHITE,
        );
    }

    (response.clicked(), response.secondary_clicked())
}
