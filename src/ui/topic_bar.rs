//! Topic bar component - displays channel info, topic, modes, and actions
//! Rendered above the message area for channels

use eframe::egui::{self, Stroke};
use std::collections::HashMap;

use crate::buffer::ChannelBuffer;
use crate::ui::theme::SlircTheme;

/// Actions that can be triggered from the topic bar
#[derive(Debug, Clone, PartialEq)]
pub enum TopicBarAction {
    /// User double-clicked the topic to edit it
    EditTopic(String),
    /// User clicked the search icon
    OpenSearch,
    /// User toggled notification mute
    ToggleMute,
    /// User clicked pinned messages
    ShowPinned,
}

/// Render the topic bar for a channel
///
/// Returns Some(action) if user interaction occurred
#[allow(clippy::too_many_arguments)]
pub fn render_topic_bar(
    ui: &mut egui::Ui,
    active_buffer: &str,
    buffers: &HashMap<String, ChannelBuffer>,
    nickname: &str,
    theme: &SlircTheme,
    system_log: &mut Vec<String>,
) -> Option<TopicBarAction> {
    // Only render for channels
    if !active_buffer.starts_with('#') && !active_buffer.starts_with('&') {
        return None;
    }

    let buffer = buffers.get(active_buffer)?;
    let mut action: Option<TopicBarAction> = None;

    // Topic bar frame with elevated background
    egui::Frame::new()
        .fill(theme.surface[2])
        .stroke(Stroke::new(1.0, theme.border_medium))
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // 1. Channel icon
                let icon = if active_buffer.starts_with('#') || active_buffer.starts_with('&') {
                    "#"
                } else {
                    "👤"
                };
                ui.label(
                    egui::RichText::new(icon)
                        .size(16.0)
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);

                // 2. Channel name (without prefix)
                let display_name = active_buffer
                    .strip_prefix('#')
                    .or_else(|| active_buffer.strip_prefix('&'))
                    .unwrap_or(active_buffer);
                ui.label(
                    egui::RichText::new(display_name)
                        .size(16.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.add_space(16.0);

                // 3. User count
                let user_count = buffer.users.len();
                if user_count > 1 {
                    ui.label(
                        egui::RichText::new(format!("👥 {}", user_count))
                            .size(13.0)
                            .color(theme.text_muted),
                    );
                    ui.add_space(8.0);
                }

                // 4. Mode badges
                if !buffer.channel_modes.is_empty() {
                    let mode_text = format!("+{}", buffer.channel_modes);

                    let badge_response = ui.add(
                        egui::Label::new(
                            egui::RichText::new(&mode_text)
                                .size(11.0)
                                .family(egui::FontFamily::Monospace)
                                .color(theme.text_secondary)
                                .background_color(theme.surface[4]),
                        )
                        .sense(egui::Sense::hover()),
                    );

                    if badge_response.hovered() {
                        egui::show_tooltip_at_pointer(
                            ui.ctx(),
                            ui.layer_id(),
                            egui::Id::new("mode_tooltip"),
                            |ui| {
                                ui.label(format_mode_description(&buffer.channel_modes));
                            },
                        );
                    }

                    ui.add_space(16.0);
                }

                // 5. Topic text (flex grow) - truncated with double-click to edit
                let topic = &buffer.topic;
                let topic_display = if topic.is_empty() {
                    "No topic set (double-click to edit)"
                } else {
                    topic.as_str()
                };

                let available_width = (ui.available_width() - 100.0).max(100.0);

                let topic_response = ui.add_sized(
                    egui::vec2(available_width, 20.0),
                    egui::Label::new(egui::RichText::new(topic_display).size(14.0).color(
                        if topic.is_empty() {
                            theme.text_muted
                        } else {
                            theme.text_secondary
                        },
                    ))
                    .truncate()
                    .sense(egui::Sense::click()),
                );

                // Double-click to edit
                if topic_response.double_clicked() {
                    let can_edit = check_topic_permission(active_buffer, buffers, nickname);
                    if can_edit {
                        action = Some(TopicBarAction::EditTopic(active_buffer.to_string()));
                    } else {
                        system_log.push(
                            "You don't have permission to change the topic (channel is +t)"
                                .to_string(),
                        );
                    }
                }

                // Tooltip with full topic
                if !topic.is_empty() && topic_response.hovered() && !topic_response.double_clicked()
                {
                    topic_response.on_hover_text(topic);
                }

                // 6. Action icons (right-aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Search icon
                    let search_response = ui.add(
                        egui::Label::new(
                            egui::RichText::new("🔍").size(16.0).color(theme.text_muted),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if search_response.clicked() {
                        action = Some(TopicBarAction::OpenSearch);
                    }
                    if search_response.hovered() {
                        search_response.on_hover_text("Search messages (Ctrl+F)");
                    }

                    ui.add_space(8.0);

                    // Pinned messages icon
                    let has_pinned = !buffer.pinned_messages.is_empty();
                    let pin_color = if has_pinned {
                        theme.accent
                    } else {
                        theme.text_muted
                    };

                    let pin_response = ui.add(
                        egui::Label::new(egui::RichText::new("📌").size(16.0).color(pin_color))
                            .sense(egui::Sense::click()),
                    );
                    if pin_response.clicked() {
                        action = Some(TopicBarAction::ShowPinned);
                    }
                    if pin_response.hovered() {
                        pin_response.on_hover_text("Pinned messages");
                    }

                    ui.add_space(8.0);

                    // Notification icon
                    let notifications_muted = buffer.notifications_muted;
                    let notif_icon = if notifications_muted { "🔕" } else { "🔔" };
                    let notif_color = if notifications_muted {
                        theme.text_muted
                    } else {
                        theme.text_secondary
                    };

                    let notif_response = ui.add(
                        egui::Label::new(
                            egui::RichText::new(notif_icon)
                                .size(16.0)
                                .color(notif_color),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if notif_response.clicked() {
                        action = Some(TopicBarAction::ToggleMute);
                    }
                    if notif_response.hovered() {
                        let status = if notifications_muted {
                            "unmuted"
                        } else {
                            "muted"
                        };
                        notif_response.on_hover_text(format!(
                            "Notifications are {} (click to toggle)",
                            status
                        ));
                    }
                });
            });
        });

    action
}

/// Check if user has permission to edit topic
///
/// If channel has +t mode, only half-ops (%) or higher can edit.
/// Otherwise, any channel member can edit.
pub fn check_topic_permission(
    channel: &str,
    buffers: &HashMap<String, ChannelBuffer>,
    nickname: &str,
) -> bool {
    if let Some(buffer) = buffers.get(channel) {
        // If +t mode, only ops/half-ops can edit
        if buffer.channel_modes.contains('t') {
            buffer.users.iter().any(|u| {
                u.nick == nickname && {
                    let rank = crate::ui::theme::prefix_rank(u.prefix);
                    rank >= 2 // Half-op or higher
                }
            })
        } else {
            // Anyone in channel can edit
            buffer.users.iter().any(|u| u.nick == nickname)
        }
    } else {
        false
    }
}

/// Format mode description for tooltip
pub fn format_mode_description(modes: &str) -> String {
    let mut descriptions = Vec::new();

    for ch in modes.chars() {
        let desc = match ch {
            'm' => "+m: Moderated (only voiced+ can speak)",
            't' => "+t: Topic restricted to operators",
            'n' => "+n: No external messages",
            's' => "+s: Secret channel (hidden from /list)",
            'i' => "+i: Invite only",
            'p' => "+p: Private channel",
            'k' => "+k: Key required to join",
            'l' => "+l: User limit set",
            _ => continue,
        };
        descriptions.push(desc);
    }

    if descriptions.is_empty() {
        "No modes set".to_string()
    } else {
        descriptions.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::UserInfo;

    #[test]
    fn test_format_mode_description() {
        assert_eq!(
            format_mode_description("mtn"),
            "+m: Moderated (only voiced+ can speak)\n+t: Topic restricted to operators\n+n: No external messages"
        );

        assert_eq!(format_mode_description(""), "No modes set");

        assert_eq!(
            format_mode_description("si"),
            "+s: Secret channel (hidden from /list)\n+i: Invite only"
        );
    }

    #[test]
    fn test_check_topic_permission_no_t_mode() {
        // Without +t, anyone in channel can edit
        let mut buffers = HashMap::new();
        let mut buffer = ChannelBuffer::new();
        buffer.users = vec![
            UserInfo {
                nick: "alice".to_string(),
                prefix: None,
            },
            UserInfo {
                nick: "bob".to_string(),
                prefix: Some('@'),
            },
        ];
        buffers.insert("#test".to_string(), buffer);

        assert!(check_topic_permission("#test", &buffers, "alice"));
        assert!(check_topic_permission("#test", &buffers, "bob"));
        assert!(!check_topic_permission("#test", &buffers, "charlie")); // Not in channel
    }

    #[test]
    fn test_check_topic_permission_with_t_mode() {
        // With +t, only ops+ can edit
        let mut buffers = HashMap::new();
        let mut buffer = ChannelBuffer::new();
        buffer.channel_modes = "t".to_string();
        buffer.users = vec![
            UserInfo {
                nick: "alice".to_string(),
                prefix: None,
            },
            UserInfo {
                nick: "bob".to_string(),
                prefix: Some('@'),
            },
            UserInfo {
                nick: "carol".to_string(),
                prefix: Some('%'),
            },
            UserInfo {
                nick: "dave".to_string(),
                prefix: Some('+'),
            },
        ];
        buffers.insert("#test".to_string(), buffer);

        assert!(!check_topic_permission("#test", &buffers, "alice")); // Regular user
        assert!(check_topic_permission("#test", &buffers, "bob")); // Op
        assert!(check_topic_permission("#test", &buffers, "carol")); // Half-op
        assert!(!check_topic_permission("#test", &buffers, "dave")); // Voice only - not enough
    }
}
