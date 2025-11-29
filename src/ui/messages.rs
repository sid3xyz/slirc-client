//! Message rendering for the central chat panel.

use eframe::egui::{self, Color32};
use regex::Regex;

use crate::buffer::{ChannelBuffer, MessageType, RenderedMessage};

/// Render the central message panel with topic bar and message list.
pub fn render_messages(
    _ctx: &egui::Context,
    ui: &mut egui::Ui,
    active_buffer: &str,
    buffers: &std::collections::HashMap<String, ChannelBuffer>,
    system_log: &[String],
    nickname: &str,
    topic_editor_open: &mut Option<String>,
) {
    // Topic bar (HexChat style - clean dedicated bar above messages)
    if active_buffer != "System" {
        egui::TopBottomPanel::top("topic_bar").show_inside(ui, |ui| {
            // Add subtle background color for visual separation
            let bg_color = if ui.style().visuals.dark_mode {
                egui::Color32::from_gray(35)
            } else {
                egui::Color32::from_gray(245)
            };
            ui.painter()
                .rect_filled(ui.available_rect_before_wrap(), 0.0, bg_color);

            if let Some(buffer) = buffers.get(active_buffer) {
                let topic_text = if buffer.topic.is_empty() {
                    "No topic is set"
                } else {
                    &buffer.topic
                };

                let is_op = buffer.users.iter().any(|u| {
                    u.nick == nickname && crate::ui::theme::prefix_rank(u.prefix) >= 3
                });

                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    let topic_response = ui.add(
                        egui::Label::new(
                            egui::RichText::new(topic_text)
                                .italics()
                                .color(egui::Color32::LIGHT_GRAY),
                        )
                        .wrap()
                        .sense(if is_op {
                            egui::Sense::click()
                        } else {
                            egui::Sense::hover()
                        }),
                    );

                    if is_op && topic_response.double_clicked() {
                        *topic_editor_open = Some(active_buffer.to_string());
                    }
                    if is_op {
                        topic_response.on_hover_text("Double-click to edit topic");
                    }
                });
            }
        });
    }

    // Messages area
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 1.0; // Tighter line spacing like HexChat

            if active_buffer == "System" {
                // Show system log
                for line in system_log {
                    ui.label(line);
                }
            } else if let Some(buffer) = buffers.get(active_buffer) {
                for msg in &buffer.messages {
                    let mention = msg.text.contains(nickname);
                    render_message(ui, msg, buffer, nickname, mention);
                }
            }
        });
}

/// Render a single message based on its type.
fn render_message(
    ui: &mut egui::Ui,
    msg: &RenderedMessage,
    buffer: &ChannelBuffer,
    _nickname: &str,
    mention: bool,
) {
    match &msg.msg_type {
        MessageType::Join | MessageType::Part | MessageType::Quit | MessageType::NickChange => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("[{}]", msg.timestamp))
                        .color(crate::ui::theme::msg_colors::TIMESTAMP),
                );
                ui.label(
                    egui::RichText::new(&msg.sender)
                        .color(crate::ui::theme::msg_colors::JOIN),
                );
                ui.label(
                    egui::RichText::new(&msg.text)
                        .color(crate::ui::theme::msg_colors::PART)
                        .italics(),
                );
            });
        }
        MessageType::Action => {
            let action = if msg.text.starts_with("\x01ACTION ") && msg.text.ends_with('\x01') {
                &msg.text[8..msg.text.len() - 1]
            } else {
                &msg.text
            };
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("[{}]", msg.timestamp))
                        .color(crate::ui::theme::msg_colors::TIMESTAMP),
                );
                ui.label(
                    egui::RichText::new("*").color(crate::ui::theme::msg_colors::ACTION),
                );
                ui.label(
                    egui::RichText::new(&msg.sender)
                        .color(crate::ui::theme::nick_color(&msg.sender)),
                );
                ui.label(
                    egui::RichText::new(action)
                        .color(crate::ui::theme::msg_colors::ACTION)
                        .italics(),
                );
            });
        }
        MessageType::Topic => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("[{}]", msg.timestamp))
                        .color(crate::ui::theme::msg_colors::TIMESTAMP),
                );
                ui.label(
                    egui::RichText::new("*").color(crate::ui::theme::msg_colors::TOPIC),
                );
                ui.label(
                    egui::RichText::new(&msg.text).color(crate::ui::theme::msg_colors::TOPIC),
                );
            });
        }
        MessageType::Notice => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("[{}]", msg.timestamp))
                        .color(crate::ui::theme::msg_colors::TIMESTAMP),
                );
                ui.label(
                    egui::RichText::new(&msg.sender)
                        .color(crate::ui::theme::msg_colors::NOTICE),
                );
                ui.label(
                    egui::RichText::new(&msg.text)
                        .color(crate::ui::theme::msg_colors::NOTICE_TEXT),
                );
            });
        }
        MessageType::Normal => {
            let prefix = buffer
                .users
                .iter()
                .find(|u| u.nick == msg.sender)
                .and_then(|u| u.prefix)
                .map(|c| c.to_string())
                .unwrap_or_default();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("[{}]", msg.timestamp))
                        .color(crate::ui::theme::msg_colors::TIMESTAMP),
                );
                let nick_display = format!("{}{}:", prefix, msg.sender);
                ui.label(
                    egui::RichText::new(nick_display)
                        .color(crate::ui::theme::nick_color(&msg.sender)),
                );
                if mention {
                    render_message_text(
                        ui,
                        buffer,
                        &msg.text,
                        Some(crate::ui::theme::msg_colors::HIGHLIGHT),
                    );
                } else {
                    render_message_text(ui, buffer, &msg.text, None);
                }
            });
        }
    }
}

/// Render message text with URL detection, emote highlighting, and nick coloring.
fn render_message_text(
    ui: &mut egui::Ui,
    buffer: &ChannelBuffer,
    text: &str,
    accent: Option<Color32>,
) {
    // tokenize by whitespace and color tokens: nicks, emotes (:emote:), urls
    let url_re = Regex::new(r"^(https?://|www\.)[\w\-\.\/~%&=:+?#]+$").unwrap();
    let emote_re = Regex::new(r"^:([a-zA-Z0-9_]+):$").unwrap();
    let tokens: Vec<&str> = text.split_whitespace().collect();

    ui.spacing_mut().item_spacing.x = 0.0; // Remove spacing between items

    for (i, &tok) in tokens.iter().enumerate() {
        let prefix = if i > 0 { " " } else { "" };
        let stripped = tok.trim_matches(|c: char| !c.is_alphanumeric() && c != '#' && c != '@');
        // If the token is prefixed with '@' to indicate a mention (e.g. `@nick`),
        // normalize for lookup by stripping the '@' for nickname matching.
        let stripped_nick = stripped.trim_start_matches('@');
        if let Some(color) = accent {
            if url_re.is_match(tok) {
                if i > 0 {
                    ui.label(" ");
                }
                ui.hyperlink_to(tok, tok);
            } else if emote_re.is_match(tok) {
                ui.label(
                    egui::RichText::new(format!("{}{}", prefix, tok))
                        .color(color)
                        .italics(),
                );
            } else {
                ui.label(egui::RichText::new(format!("{}{}", prefix, tok)).color(color));
            }
        } else if url_re.is_match(tok) {
            if i > 0 {
                ui.label(" ");
            }
            ui.hyperlink_to(tok, tok);
        } else if emote_re.is_match(tok) {
            ui.label(
                egui::RichText::new(format!("{}{}", prefix, tok))
                    .color(egui::Color32::from_rgb(255, 205, 0))
                    .italics(),
            );
        } else if buffer.users.iter().any(|u| u.nick == stripped_nick) {
            let col = crate::ui::theme::nick_color(stripped_nick);
            ui.label(egui::RichText::new(format!("{}{}", prefix, tok)).color(col));
        } else {
            // default
            ui.label(format!("{}{}", prefix, tok));
        }
    }
}
