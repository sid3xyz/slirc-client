//! IRC formatting code parser and text styling.

use eframe::egui::{self, Color32};

use crate::buffer::ChannelBuffer;
use crate::ui::theme::{self, SlircTheme};

/// Represents a styled span of text with IRC formatting
#[derive(Debug, Clone)]
pub(crate) struct TextSpan {
    pub text: String,
    pub fg_color: Option<Color32>,
    #[allow(dead_code)]
    pub bg_color: Option<Color32>,
    pub bold: bool,
    pub italic: bool,
}

/// Parse IRC formatting codes into styled text spans
pub(crate) fn parse_irc_formatting(text: &str) -> Vec<TextSpan> {
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

/// Render message text with IRC formatting and URL detection
pub(crate) fn render_message_text(
    ui: &mut egui::Ui,
    buffer: &ChannelBuffer,
    text: &str,
    mention: bool,
    theme: &SlircTheme,
) {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static URL_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(https?://[^\s]+)").expect("URL regex pattern is valid"));

    let base_color = if mention {
        Color32::from_rgb(255, 210, 100)
    } else {
        theme.text_primary
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
                    ui.hyperlink_to(egui::RichText::new(url).size(14.0).color(theme.info), url);
                    if word.ends_with(char::is_whitespace) {
                        ui.label(" ");
                    }
                } else if buffer
                    .users
                    .iter()
                    .any(|u| u.nick == word.trim().trim_start_matches('@'))
                {
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
