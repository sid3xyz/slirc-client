//! Message rendering for the central chat panel.

use eframe::egui::{self, Color32, FontFamily, FontId};

use crate::buffer::{ChannelBuffer, MessageType, RenderedMessage};
use crate::ui::theme::spacing;

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

    // Topic bar (HexChat style - clean dedicated bar above messages)
    if active_buffer != "System" {
        egui::TopBottomPanel::top("topic_bar").show_inside(ui, |ui| {
            // Add subtle background color for visual separation
            let bg_color = if dark_mode {
                egui::Color32::from_rgb(32, 32, 38)
            } else {
                egui::Color32::from_rgb(242, 242, 246)
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
                    ui.add_space(6.0);
                    let topic_response = ui.add(
                        egui::Label::new(
                            egui::RichText::new(topic_text)
                                .italics()
                                .color(if dark_mode {
                                    egui::Color32::from_gray(160)
                                } else {
                                    egui::Color32::from_gray(100)
                                }),
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
            // Improved spacing between messages for readability
            ui.spacing_mut().item_spacing.y = spacing::MESSAGE_SPACING_Y;

            if active_buffer == "System" {
                // Show system log
                for line in system_log {
                    ui.label(line);
                }
            } else if let Some(buffer) = buffers.get(active_buffer) {
                for msg in &buffer.messages {
                    let mention = msg.text.contains(nickname);
                    render_message(ui, msg, buffer, nickname, mention, dark_mode);
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
    dark_mode: bool,
) {
    // Helper for monospaced, aligned timestamps
    let timestamp_text = |ts: &str| -> egui::RichText {
        egui::RichText::new(format!("[{}]", ts))
            .font(FontId::new(12.0, FontFamily::Monospace))
            .color(crate::ui::theme::msg_colors::TIMESTAMP)
    };

    match &msg.msg_type {
        MessageType::Join | MessageType::Part | MessageType::Quit | MessageType::NickChange => {
            ui.horizontal(|ui| {
                ui.label(timestamp_text(&msg.timestamp));
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
                ui.label(timestamp_text(&msg.timestamp));
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
                ui.label(timestamp_text(&msg.timestamp));
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
                ui.label(timestamp_text(&msg.timestamp));
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
                ui.label(timestamp_text(&msg.timestamp));
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
                        dark_mode,
                    );
                } else {
                    render_message_text(ui, buffer, &msg.text, None, dark_mode);
                }
            });
        }
    }
}

/// Render message text with IRC formatting, URL detection, emote highlighting, and nick coloring.
fn render_message_text(
    ui: &mut egui::Ui,
    _buffer: &ChannelBuffer,
    text: &str,
    accent: Option<Color32>,
    _dark_mode: bool,
) {
    use once_cell::sync::Lazy;
    use regex::Regex;
    
    // Compile regexes once at startup - these patterns are constant
    static URL_RE: Lazy<Regex> = Lazy::new(|| 
        Regex::new(r"^(https?://|www\.)[^\s]+$")
            .expect("URL regex pattern is valid")
    );
    static EMOTE_RE: Lazy<Regex> = Lazy::new(|| 
        Regex::new(r"^:([a-zA-Z0-9_]+):$")
            .expect("Emote regex pattern is valid")
    );
    
    ui.spacing_mut().item_spacing.x = 0.0; // Remove spacing between items
    
    // Parse IRC formatting codes and render as styled spans
    let spans = parse_irc_formatting(text);
    
    for span in spans {
        if let Some(color) = accent {
            // If we have an accent color (highlight), override span colors
            render_span_with_override(ui, buffer, &span, Some(color), &URL_RE, &EMOTE_RE);
        } else {
            render_span(ui, buffer, &span, &URL_RE, &EMOTE_RE);
        }
    }
}

/// Represents a styled span of text with IRC formatting
#[derive(Debug, Clone)]
struct TextSpan {
    text: String,
    fg_color: Option<Color32>,
    bg_color: Option<Color32>,
    bold: bool,
    italic: bool,
}

impl TextSpan {
    fn new(text: String) -> Self {
        Self {
            text,
            fg_color: None,
            bg_color: None,
            bold: false,
            italic: false,
        }
    }
}

/// Parse IRC formatting codes into styled text spans
/// Supports: \x02 (bold), \x1D (italic), \x0F (reset), \x03 (color)
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
                // Bold toggle
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
                // Italic toggle
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
                // Reset all formatting
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
                // Color code
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
                
                i += 1; // Move past \x03
                
                // Check if this is just a reset (no digits follow)
                if i >= chars.len() || !chars[i].is_ascii_digit() {
                    fg_color = None;
                    bg_color = None;
                    continue;
                }
                
                // Parse foreground color (1 or 2 digits)
                let mut fg_code = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() && fg_code.len() < 2 {
                    fg_code.push(chars[i]);
                    i += 1;
                }
                
                if let Ok(code) = fg_code.parse::<u8>() {
                    fg_color = Some(crate::ui::theme::mirc_color(code));
                }
                
                // Check for optional background color (comma separator)
                if i < chars.len() && chars[i] == ',' {
                    i += 1; // Skip comma
                    let mut bg_code = String::new();
                    while i < chars.len() && chars[i].is_ascii_digit() && bg_code.len() < 2 {
                        bg_code.push(chars[i]);
                        i += 1;
                    }
                    if let Ok(code) = bg_code.parse::<u8>() {
                        bg_color = Some(crate::ui::theme::mirc_color(code));
                    }
                }
            }
            ch => {
                current_text.push(ch);
                i += 1;
            }
        }
    }
    
    // Push final span if any text remains
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

/// Render a text span with its formatting
fn render_span(
    ui: &mut egui::Ui,
    buffer: &ChannelBuffer,
    span: &TextSpan,
    url_re: &regex::Regex,
    emote_re: &regex::Regex,
) {
    // Split span text into words for URL/emote/nick detection
    let words: Vec<&str> = span.text.split_whitespace().collect();
    
    for (i, &word) in words.iter().enumerate() {
        let prefix = if i > 0 { " " } else { "" };
        let stripped = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '#' && c != '@');
        let stripped_nick = stripped.trim_start_matches('@');
        
        // Build RichText with span formatting
        let mut rich_text = egui::RichText::new(format!("{}{}", prefix, word));
        
        if span.bold {
            rich_text = rich_text.strong();
        }
        if span.italic {
            rich_text = rich_text.italics();
        }
        
        // Handle URLs specially (always hyperlinked)
        if url_re.is_match(word) {
            if i > 0 {
                ui.label(" ");
            }
            ui.hyperlink_to(word, word);
        } else if emote_re.is_match(word) {
            // Emotes get special color
            rich_text = rich_text.color(egui::Color32::from_rgb(255, 205, 0));
            ui.label(rich_text);
        } else if buffer.users.iter().any(|u| u.nick == stripped_nick) {
            // Nick mentions get nick color
            let col = crate::ui::theme::nick_color(stripped_nick);
            rich_text = rich_text.color(col);
            ui.label(rich_text);
        } else {
            // Apply span colors
            if let Some(fg) = span.fg_color {
                rich_text = rich_text.color(fg);
            }
            // Background color rendering (egui has limited support, use background_color if available)
            if let Some(_bg) = span.bg_color {
                // Note: egui doesn't support text background colors directly in RichText
                // This would require custom rendering. For now, we skip bg colors.
            }
            ui.label(rich_text);
        }
    }
}

/// Render a text span with accent color override (for highlights)
fn render_span_with_override(
    ui: &mut egui::Ui,
    buffer: &ChannelBuffer,
    span: &TextSpan,
    override_color: Option<Color32>,
    url_re: &regex::Regex,
    emote_re: &regex::Regex,
) {
    let words: Vec<&str> = span.text.split_whitespace().collect();
    
    for (i, &word) in words.iter().enumerate() {
        let prefix = if i > 0 { " " } else { "" };
        
        let mut rich_text = egui::RichText::new(format!("{}{}", prefix, word));
        
        if span.bold {
            rich_text = rich_text.strong();
        }
        if span.italic {
            rich_text = rich_text.italics();
        }
        
        if url_re.is_match(word) {
            if i > 0 {
                ui.label(" ");
            }
            ui.hyperlink_to(word, word);
        } else if emote_re.is_match(word) {
            if let Some(color) = override_color {
                rich_text = rich_text.color(color).italics();
            }
            ui.label(rich_text);
        } else {
            if let Some(color) = override_color {
                rich_text = rich_text.color(color);
            }
            ui.label(rich_text);
        }
    }
}

