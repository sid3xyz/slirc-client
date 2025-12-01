//! Main message rendering logic with grouping and avatars.

use eframe::egui::{self, Color32};
use slirc_proto::ctcp::Ctcp;

use crate::buffer::{ChannelBuffer, MessageType, RenderedMessage};
use crate::ui::theme::{self, SlircTheme};

use super::format::render_message_text;
use super::helpers::{contains_mention, timestamps_within_window};

/// Render the central message panel with message list.
/// Topic bar is rendered separately by ui::topic_bar module.
pub fn render_messages(
    _ctx: &egui::Context,
    ui: &mut egui::Ui,
    active_buffer: &str,
    buffers: &std::collections::HashMap<String, ChannelBuffer>,
    system_log: &[String],
    nickname: &str,
) {
    let dark_mode = ui.style().visuals.dark_mode;
    let theme = if dark_mode { SlircTheme::dark() } else { SlircTheme::light() };

    // Messages area with improved styling
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add_space(8.0);

            if active_buffer == "System" {
                render_system_log(ui, system_log, &theme);
            } else if let Some(buffer) = buffers.get(active_buffer) {
                render_grouped_messages(ui, buffer, nickname, &theme);
            }

            ui.add_space(8.0);
        });
}

/// Render system log with modern styling
fn render_system_log(ui: &mut egui::Ui, system_log: &[String], theme: &SlircTheme) {
    for line in system_log {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new(line)
                    .size(13.0)
                    .color(theme.text_muted),
            );
        });
        ui.add_space(2.0);
    }
}

/// Group consecutive messages from the same sender
struct MessageGroup<'a> {
    sender: &'a str,
    messages: Vec<&'a RenderedMessage>,
    first_timestamp: &'a str,
    is_system: bool,
}

/// Group messages by sender for modern display
fn group_messages(messages: &[RenderedMessage]) -> Vec<MessageGroup<'_>> {
    let mut groups: Vec<MessageGroup<'_>> = Vec::new();

    for msg in messages {
        let is_system = matches!(
            msg.msg_type,
            MessageType::Join | MessageType::Part | MessageType::Quit | MessageType::NickChange | MessageType::Topic
        );

        // Always start new group for system messages
        if is_system {
            groups.push(MessageGroup {
                sender: &msg.sender,
                messages: vec![msg],
                first_timestamp: &msg.timestamp,
                is_system: true,
            });
            continue;
        }

        // Check if we should continue the previous group:
        // - Same sender
        // - Compatible message type
        // - Within 5-minute time window from the *last* message in the group
        let should_group = groups.last().is_some_and(|last| {
            if last.is_system || last.sender != msg.sender {
                return false;
            }
            if !matches!(msg.msg_type, MessageType::Normal | MessageType::Action | MessageType::Notice) {
                return false;
            }
            // Check time gap from last message in the group
            let last_msg_ts = last.messages.last().map(|m| m.timestamp.as_str()).unwrap_or(last.first_timestamp);
            timestamps_within_window(last_msg_ts, &msg.timestamp)
        });

        if should_group {
            groups.last_mut().unwrap().messages.push(msg);
        } else {
            groups.push(MessageGroup {
                sender: &msg.sender,
                messages: vec![msg],
                first_timestamp: &msg.timestamp,
                is_system: false,
            });
        }
    }

    groups
}

/// Render messages with grouping and avatars
fn render_grouped_messages(
    ui: &mut egui::Ui,
    buffer: &ChannelBuffer,
    nickname: &str,
    theme: &SlircTheme,
) {
    let groups = group_messages(&buffer.messages);

    for group in groups {
        if group.is_system {
            // Render system message (join/part/etc) compactly
            render_system_message(ui, group.messages[0], theme);
        } else {
            // Render message group with avatar
            render_message_group(ui, &group, buffer, nickname, theme);
        }
    }
}

/// Render a system message (join, part, quit, etc.)
fn render_system_message(ui: &mut egui::Ui, msg: &RenderedMessage, theme: &SlircTheme) {
    let (icon, color, text) = match &msg.msg_type {
        MessageType::Join => ("→", theme.success, format!("{} joined the channel", msg.sender)),
        MessageType::Part => ("←", theme.text_muted, format!("{} left the channel", msg.sender)),
        MessageType::Quit => ("✕", theme.text_muted, format!("{} quit: {}", msg.sender, msg.text)),
        MessageType::NickChange => ("~", theme.info, format!("{} {}", msg.sender, msg.text)),
        MessageType::Topic => ("★", theme.info, msg.text.clone()),
        _ => ("•", theme.text_muted, msg.text.clone()),
    };

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(52.0); // Align with message content (avatar + margin)
        ui.label(
            egui::RichText::new(icon)
                .size(12.0)
                .color(color),
        );
        ui.label(
            egui::RichText::new(&text)
                .size(12.0)
                .color(theme.text_muted)
                .italics(),
        );
        ui.label(
            egui::RichText::new(&msg.timestamp)
                .size(10.0)
                .color(theme.text_muted),
        );
    });
    ui.add_space(4.0);
}

/// Render a group of messages from the same sender
fn render_message_group(
    ui: &mut egui::Ui,
    group: &MessageGroup<'_>,
    buffer: &ChannelBuffer,
    nickname: &str,
    theme: &SlircTheme,
) {
    // Add spacing between groups (cozy layout)
    ui.add_space(20.0);

    // Container for the message group with hover highlight
    let group_rect = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(
        egui::Rect::from_min_size(group_rect.min, egui::vec2(group_rect.width(), 0.0)),
        egui::Sense::hover(),
    );

    ui.horizontal(|ui| {
        ui.add_space(12.0);

        // Avatar
        theme::render_avatar(ui, group.sender, 36.0);

        ui.add_space(12.0);

        // Message content column
        ui.vertical(|ui| {
            // Header: nickname + timestamp
            ui.horizontal(|ui| {
                // Nickname with color
                let nick_color = theme::nick_color(group.sender);
                ui.label(
                    egui::RichText::new(group.sender)
                        .size(14.0)
                        .strong()
                        .color(nick_color),
                );

                ui.add_space(8.0);

                // Timestamp
                ui.label(
                    egui::RichText::new(group.first_timestamp)
                        .size(11.0)
                        .color(theme.text_muted),
                );
            });

            ui.add_space(2.0);

            // Messages in this group
            for (i, msg) in group.messages.iter().enumerate() {
                if i > 0 {
                    ui.add_space(2.0); // Tighter spacing within group
                }

                ui.horizontal(|ui| {
                    // Message content
                    ui.vertical(|ui| {
                        let mention = contains_mention(&msg.text, nickname);
                        render_message_content(ui, msg, buffer, mention, theme);
                    });

                    // Timestamp (faint, shown on hover)
                    if i > 0 {
                        let timestamp_response = ui.label(
                            egui::RichText::new(&msg.timestamp)
                                .size(10.0)
                                .color(Color32::from_white_alpha(40)),
                        );
                        timestamp_response.on_hover_text(&msg.timestamp);
                    }
                });
            }
        });
    });

    // Hover highlight effect with rounded corners
    if response.hovered() {
        let highlight_rect = egui::Rect::from_min_size(
            group_rect.min,
            egui::vec2(group_rect.width(), ui.min_rect().height()),
        );
        ui.painter().rect_filled(
            highlight_rect,
            4.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 8),
        );
    }
}

/// Render the content of a single message
fn render_message_content(
    ui: &mut egui::Ui,
    msg: &RenderedMessage,
    buffer: &ChannelBuffer,
    mention: bool,
    theme: &SlircTheme,
) {
    match &msg.msg_type {
        MessageType::Action => {
            // Use slirc_proto's CTCP parser to extract action text
            let action = Ctcp::parse(&msg.text)
                .and_then(|c| c.params.map(String::from))
                .unwrap_or_else(|| msg.text.clone());
            ui.label(
                egui::RichText::new(action)
                    .size(14.0)
                    .color(theme.accent)
                    .italics(),
            );
        }
        MessageType::Notice => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("[Notice]")
                        .size(12.0)
                        .color(theme.warning),
                );
                ui.label(
                    egui::RichText::new(&msg.text)
                        .size(14.0)
                        .color(theme.text_secondary),
                );
            });
        }
        MessageType::Normal => {
            // Highlight background for mentions with indicator strip
            if mention {
                let rect = ui.available_rect_before_wrap();

                // Main highlight background with rounded corners
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 26.0)),
                    6.0,
                    Color32::from_rgba_unmultiplied(250, 166, 26, 25), // warning color, subtle
                );

                // Left accent strip for visual indicator
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(3.0, 26.0),
                    ),
                    egui::CornerRadius {
                        nw: 3,
                        ne: 0,
                        sw: 3,
                        se: 0,
                    },
                    Color32::from_rgb(250, 166, 26), // warning color, solid
                );
            }

            render_message_text(ui, buffer, &msg.text, mention, theme);
        }
        _ => {
            ui.label(
                egui::RichText::new(&msg.text)
                    .size(14.0)
                    .color(theme.text_primary),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_messages_time_gap() {
        use crate::buffer::MessageType;

        let messages = vec![
            RenderedMessage {
                timestamp: "12:00:00".to_string(),
                sender: "alice".to_string(),
                text: "Hello".to_string(),
                msg_type: MessageType::Normal,
            },
            RenderedMessage {
                timestamp: "12:02:00".to_string(),
                sender: "alice".to_string(),
                text: "Still here".to_string(),
                msg_type: MessageType::Normal,
            },
            // 10 minute gap - should start new group
            RenderedMessage {
                timestamp: "12:12:00".to_string(),
                sender: "alice".to_string(),
                text: "Back again".to_string(),
                msg_type: MessageType::Normal,
            },
        ];

        let groups = group_messages(&messages);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].messages.len(), 2); // First two messages grouped
        assert_eq!(groups[1].messages.len(), 1); // Third message in new group
    }

    #[test]
    fn test_group_messages_different_senders() {
        use crate::buffer::MessageType;

        let messages = vec![
            RenderedMessage {
                timestamp: "12:00:00".to_string(),
                sender: "alice".to_string(),
                text: "Hello".to_string(),
                msg_type: MessageType::Normal,
            },
            RenderedMessage {
                timestamp: "12:00:30".to_string(),
                sender: "bob".to_string(),
                text: "Hi!".to_string(),
                msg_type: MessageType::Normal,
            },
            RenderedMessage {
                timestamp: "12:01:00".to_string(),
                sender: "alice".to_string(),
                text: "How are you?".to_string(),
                msg_type: MessageType::Normal,
            },
        ];

        let groups = group_messages(&messages);

        // Each sender should get their own group
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].sender, "alice");
        assert_eq!(groups[1].sender, "bob");
        assert_eq!(groups[2].sender, "alice");
    }
}
