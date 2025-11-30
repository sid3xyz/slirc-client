//! Modern message rendering for the central chat panel.
//! Features: message grouping, avatars, hover states, improved typography.

use eframe::egui::{self, Color32, FontFamily, FontId};

use crate::buffer::{ChannelBuffer, MessageType, RenderedMessage};
use crate::ui::theme::{self, dark, panel_colors, spacing, text_colors};

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
    let dark_mode = ui.style().visuals.dark_mode;

    // Topic bar - modern style with subtle background
    if active_buffer != "System" {
        render_topic_bar(ui, active_buffer, buffers, nickname, topic_editor_open, dark_mode);
    }

    // Messages area with improved styling
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add_space(8.0);

            if active_buffer == "System" {
                render_system_log(ui, system_log, dark_mode);
            } else if let Some(buffer) = buffers.get(active_buffer) {
                render_grouped_messages(ui, buffer, nickname, dark_mode);
            }

            ui.add_space(8.0);
        });
}

/// Render topic bar with modern styling
fn render_topic_bar(
    ui: &mut egui::Ui,
    active_buffer: &str,
    buffers: &std::collections::HashMap<String, ChannelBuffer>,
    nickname: &str,
    topic_editor_open: &mut Option<String>,
    dark_mode: bool,
) {
    let bg_color = if dark_mode {
        dark::BG_BASE
    } else {
        Color32::from_rgb(248, 249, 252)
    };

    egui::TopBottomPanel::top("topic_bar")
        .frame(
            egui::Frame::new()
                .fill(bg_color)
                .inner_margin(egui::Margin::symmetric(16, 10))
                .stroke(egui::Stroke::new(1.0, panel_colors::separator(dark_mode))),
        )
        .show_inside(ui, |ui| {
            if let Some(buffer) = buffers.get(active_buffer) {
                let topic_text = if buffer.topic.is_empty() {
                    "No topic set — Double-click to set one"
                } else {
                    &buffer.topic
                };

                let is_op = buffer.users.iter().any(|u| {
                    u.nick == nickname && theme::prefix_rank(u.prefix) >= 3
                });

                let topic_response = ui.add(
                    egui::Label::new(
                        egui::RichText::new(topic_text)
                            .size(13.0)
                            .color(text_colors::secondary(dark_mode)),
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
            }
        });
}

/// Render system log with modern styling
fn render_system_log(ui: &mut egui::Ui, system_log: &[String], dark_mode: bool) {
    for line in system_log {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new(line)
                    .size(13.0)
                    .color(text_colors::muted(dark_mode)),
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

        // Check if we should continue the previous group
        let should_group = groups.last().map_or(false, |last| {
            !last.is_system
                && last.sender == msg.sender
                && matches!(msg.msg_type, MessageType::Normal | MessageType::Action | MessageType::Notice)
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
    dark_mode: bool,
) {
    let groups = group_messages(&buffer.messages);

    for group in groups {
        if group.is_system {
            // Render system message (join/part/etc) compactly
            render_system_message(ui, group.messages[0], dark_mode);
        } else {
            // Render message group with avatar
            render_message_group(ui, &group, buffer, nickname, dark_mode);
        }
    }
}

/// Render a system message (join, part, quit, etc.)
fn render_system_message(ui: &mut egui::Ui, msg: &RenderedMessage, dark_mode: bool) {
    let (icon, color, text) = match &msg.msg_type {
        MessageType::Join => ("→", theme::msg_colors::JOIN, format!("{} joined the channel", msg.sender)),
        MessageType::Part => ("←", theme::msg_colors::PART, format!("{} left the channel", msg.sender)),
        MessageType::Quit => ("✕", theme::msg_colors::PART, format!("{} quit: {}", msg.sender, msg.text)),
        MessageType::NickChange => ("~", dark::ACCENT_BLUE, format!("{} {}", msg.sender, msg.text)),
        MessageType::Topic => ("★", theme::msg_colors::TOPIC, msg.text.clone()),
        _ => ("•", text_colors::muted(dark_mode), msg.text.clone()),
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
                .color(text_colors::muted(dark_mode))
                .italics(),
        );
        ui.label(
            egui::RichText::new(&msg.timestamp)
                .size(10.0)
                .color(text_colors::muted(dark_mode)),
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
    dark_mode: bool,
) {
    // Add spacing between groups
    ui.add_space(spacing::MESSAGE_GROUP_SPACING);

    // Container for the message group with hover highlight
    let group_rect = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(
        egui::Rect::from_min_size(group_rect.min, egui::vec2(group_rect.width(), 0.0)),
        egui::Sense::hover(),
    );

    ui.horizontal(|ui| {
        ui.add_space(12.0);

        // Avatar
        theme::render_avatar(ui, group.sender, spacing::AVATAR_SIZE);

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
                        .color(text_colors::muted(dark_mode)),
                );
            });

            ui.add_space(2.0);

            // Messages in this group
            for (i, msg) in group.messages.iter().enumerate() {
                if i > 0 {
                    ui.add_space(spacing::MESSAGE_CONTINUATION_SPACING);
                }

                let mention = msg.text.contains(nickname);
                render_message_content(ui, msg, buffer, mention, dark_mode);
            }
        });
    });

    // Hover highlight effect
    if response.hovered() {
        let highlight_rect = egui::Rect::from_min_size(
            group_rect.min,
            egui::vec2(group_rect.width(), ui.min_rect().height()),
        );
        ui.painter().rect_filled(
            highlight_rect,
            0.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 3),
        );
    }
}

/// Render the content of a single message
fn render_message_content(
    ui: &mut egui::Ui,
    msg: &RenderedMessage,
    buffer: &ChannelBuffer,
    mention: bool,
    dark_mode: bool,
) {
    match &msg.msg_type {
        MessageType::Action => {
            let action = if msg.text.starts_with("\x01ACTION ") && msg.text.ends_with('\x01') {
                &msg.text[8..msg.text.len() - 1]
            } else {
                &msg.text
            };
            ui.label(
                egui::RichText::new(action)
                    .size(14.0)
                    .color(theme::msg_colors::ACTION)
                    .italics(),
            );
        }
        MessageType::Notice => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("[Notice]")
                        .size(12.0)
                        .color(theme::msg_colors::NOTICE),
                );
                ui.label(
                    egui::RichText::new(&msg.text)
                        .size(14.0)
                        .color(theme::msg_colors::NOTICE_TEXT),
                );
            });
        }
        MessageType::Normal => {
            // Highlight background for mentions
            if mention {
                let rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 24.0)),
                    4.0,
                    Color32::from_rgba_unmultiplied(255, 180, 50, 25),
                );
            }

            render_message_text(ui, buffer, &msg.text, mention, dark_mode);
        }
        _ => {
            ui.label(
                egui::RichText::new(&msg.text)
                    .size(14.0)
                    .color(text_colors::primary(dark_mode)),
            );
        }
    }
}

/// Render message text with IRC formatting and URL detection
fn render_message_text(
    ui: &mut egui::Ui,
    buffer: &ChannelBuffer,
    text: &str,
    mention: bool,
    dark_mode: bool,
) {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static URL_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(https?://[^\s]+)").expect("URL regex pattern is valid")
    });

    let base_color = if mention {
        theme::msg_colors::HIGHLIGHT
    } else {
        text_colors::primary(dark_mode)
    };

    // Parse IRC formatting codes
    let spans = parse_irc_formatting(text);

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        for span in spans {
            // Split into words to detect URLs
            for word in span.text.split_inclusive(char::is_whitespace) {
                if URL_RE.is_match(word.trim()) {
                    let url = word.trim();
                    ui.hyperlink_to(
                        egui::RichText::new(url).size(14.0).color(dark::ACCENT_BLUE),
                        url,
                    );
                    if word.ends_with(char::is_whitespace) {
                        ui.label(" ");
                    }
                } else if buffer.users.iter().any(|u| u.nick == word.trim().trim_start_matches('@')) {
                    // Nick mention
                    let nick = word.trim().trim_start_matches('@');
                    let nick_col = theme::nick_color(nick);
                    let mut rich = egui::RichText::new(word).size(14.0).color(nick_col);
                    if span.bold {
                        rich = rich.strong();
                    }
                    if span.italic {
                        rich = rich.italics();
                    }
                    ui.label(rich);
                } else {
                    // Regular text with formatting
                    let color = span.fg_color.unwrap_or(base_color);
                    let mut rich = egui::RichText::new(word).size(14.0).color(color);
                    if span.bold {
                        rich = rich.strong();
                    }
                    if span.italic {
                        rich = rich.italics();
                    }
                    ui.label(rich);
                }
            }
        }
    });
}

/// Represents a styled span of text with IRC formatting
#[derive(Debug, Clone)]
struct TextSpan {
    text: String,
    fg_color: Option<Color32>,
    #[allow(dead_code)]
    bg_color: Option<Color32>,
    bold: bool,
    italic: bool,
}

/// Parse IRC formatting codes into styled text spans
fn parse_irc_formatting(text: &str) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut fg_color: Option<Color32> = None;
    let mut bg_color: Option<Color32> = None;
    let mut bold = false;
    let mut italic = false;

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '\x02' => {
                if !current_text.is_empty() {
                    spans.push(TextSpan {
                        text: current_text.clone(),
                        fg_color,
                        bg_color,
                        bold,
                        italic,
                    });
                    current_text.clear();
                }
                bold = !bold;
                i += 1;
            }
            '\x1D' => {
                if !current_text.is_empty() {
                    spans.push(TextSpan {
                        text: current_text.clone(),
                        fg_color,
                        bg_color,
                        bold,
                        italic,
                    });
                    current_text.clear();
                }
                italic = !italic;
                i += 1;
            }
            '\x0F' => {
                if !current_text.is_empty() {
                    spans.push(TextSpan {
                        text: current_text.clone(),
                        fg_color,
                        bg_color,
                        bold,
                        italic,
                    });
                    current_text.clear();
                }
                fg_color = None;
                bg_color = None;
                bold = false;
                italic = false;
                i += 1;
            }
            '\x03' => {
                if !current_text.is_empty() {
                    spans.push(TextSpan {
                        text: current_text.clone(),
                        fg_color,
                        bg_color,
                        bold,
                        italic,
                    });
                    current_text.clear();
                }

                i += 1;

                if i >= chars.len() || !chars[i].is_ascii_digit() {
                    fg_color = None;
                    bg_color = None;
                    continue;
                }

                let mut fg_code = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() && fg_code.len() < 2 {
                    fg_code.push(chars[i]);
                    i += 1;
                }

                if let Ok(code) = fg_code.parse::<u8>() {
                    fg_color = Some(theme::mirc_color(code));
                }

                if i < chars.len() && chars[i] == ',' {
                    i += 1;
                    let mut bg_code = String::new();
                    while i < chars.len() && chars[i].is_ascii_digit() && bg_code.len() < 2 {
                        bg_code.push(chars[i]);
                        i += 1;
                    }
                    if let Ok(code) = bg_code.parse::<u8>() {
                        bg_color = Some(theme::mirc_color(code));
                    }
                }
            }
            ch => {
                current_text.push(ch);
                i += 1;
            }
        }
    }

    if !current_text.is_empty() {
        spans.push(TextSpan {
            text: current_text,
            fg_color,
            bg_color,
            bold,
            italic,
        });
    }

    spans
}

