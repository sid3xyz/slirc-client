//! Modern message rendering for the central chat panel.
//! Features: message grouping, avatars, hover states, improved typography.

use eframe::egui::{self, Color32};
use slirc_proto::ctcp::Ctcp;

use crate::buffer::{ChannelBuffer, MessageType, RenderedMessage};
use crate::ui::theme::{self, SlircTheme};

/// Check if a message contains a mention of the given nickname.
///
/// # Mention Detection Rules
///
/// A mention is detected when:
/// 1. The nickname appears as a complete word (not part of another word)
/// 2. Case-insensitive matching (alice matches ALICE, Alice, etc.)
/// 3. Common IRC mention formats: "nick:", "nick,", "@nick"
///
/// # Examples
///
/// ```ignore
/// contains_mention("Hey alice, how are you?", "alice") == true
/// contains_mention("Hey ALICE: check this out", "alice") == true
/// contains_mention("Hey @alice", "alice") == true
/// contains_mention("alice's message", "alice") == true  // apostrophe is word boundary
/// contains_mention("malice aforethought", "alice") == false  // alice is inside word
/// ```
fn contains_mention(text: &str, nickname: &str) -> bool {
    if nickname.is_empty() {
        return false;
    }

    let text_lower = text.to_lowercase();
    let nick_lower = nickname.to_lowercase();

    // Find all occurrences of the nickname
    let mut search_start = 0;
    while let Some(pos) = text_lower[search_start..].find(&nick_lower) {
        let abs_pos = search_start + pos;
        let end_pos = abs_pos + nick_lower.len();

        // Check if this is a word boundary match
        let at_start = abs_pos == 0
            || !text.as_bytes()[abs_pos - 1].is_ascii_alphanumeric()
            || (abs_pos > 0 && text.as_bytes()[abs_pos - 1] == b'@'); // @mention

        let at_end = end_pos >= text.len()
            || !text.as_bytes()[end_pos].is_ascii_alphanumeric();

        if at_start && at_end {
            return true;
        }

        // Move past this occurrence
        search_start = abs_pos + 1;
    }

    false
}

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

/// Maximum time gap (in seconds) before starting a new message group.
/// Messages from the same sender within 5 minutes are grouped together.
const GROUP_TIME_GAP_SECONDS: u32 = 300;

/// Parse a timestamp string "HH:MM:SS" into total seconds since midnight.
/// Returns None if parsing fails.
fn parse_timestamp_seconds(ts: &str) -> Option<u32> {
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let hours: u32 = parts[0].parse().ok()?;
    let minutes: u32 = parts[1].parse().ok()?;
    let seconds: u32 = parts[2].parse().ok()?;
    Some(hours * 3600 + minutes * 60 + seconds)
}

/// Check if two timestamps are within the grouping window (5 minutes).
/// Handles midnight wraparound.
fn timestamps_within_window(ts1: &str, ts2: &str) -> bool {
    let Some(secs1) = parse_timestamp_seconds(ts1) else {
        return false;
    };
    let Some(secs2) = parse_timestamp_seconds(ts2) else {
        return false;
    };

    // Calculate difference, handling midnight wraparound
    let diff = if secs2 >= secs1 {
        secs2 - secs1
    } else {
        // Midnight crossed: add 24 hours to second timestamp
        (secs2 + 86400) - secs1
    };

    diff <= GROUP_TIME_GAP_SECONDS
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

/// Render message text with IRC formatting and URL detection
fn render_message_text(
    ui: &mut egui::Ui,
    buffer: &ChannelBuffer,
    text: &str,
    mention: bool,
    theme: &SlircTheme,
) {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static URL_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(https?://[^\s]+)").expect("URL regex pattern is valid")
    });

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
                    ui.hyperlink_to(
                        egui::RichText::new(url).size(14.0).color(theme.info),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp_seconds() {
        assert_eq!(parse_timestamp_seconds("00:00:00"), Some(0));
        assert_eq!(parse_timestamp_seconds("00:00:01"), Some(1));
        assert_eq!(parse_timestamp_seconds("00:01:00"), Some(60));
        assert_eq!(parse_timestamp_seconds("01:00:00"), Some(3600));
        assert_eq!(parse_timestamp_seconds("12:30:45"), Some(45045));
        assert_eq!(parse_timestamp_seconds("23:59:59"), Some(86399));

        // Invalid formats
        assert_eq!(parse_timestamp_seconds("invalid"), None);
        assert_eq!(parse_timestamp_seconds("12:30"), None);
        assert_eq!(parse_timestamp_seconds(""), None);
        assert_eq!(parse_timestamp_seconds("aa:bb:cc"), None);
    }

    #[test]
    fn test_timestamps_within_window() {
        // Same timestamp
        assert!(timestamps_within_window("12:00:00", "12:00:00"));

        // Within 5 minutes
        assert!(timestamps_within_window("12:00:00", "12:04:59"));
        assert!(timestamps_within_window("12:00:00", "12:05:00"));

        // Just outside 5 minutes
        assert!(!timestamps_within_window("12:00:00", "12:05:01"));
        assert!(!timestamps_within_window("12:00:00", "12:10:00"));

        // Test midnight wraparound
        assert!(timestamps_within_window("23:58:00", "00:01:00")); // 3 minutes across midnight

        // Invalid timestamps should return false
        assert!(!timestamps_within_window("invalid", "12:00:00"));
        assert!(!timestamps_within_window("12:00:00", "invalid"));
    }

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

    #[test]
    fn test_contains_mention_basic() {
        // Basic word boundary matches
        assert!(contains_mention("hey alice how are you", "alice"));
        assert!(contains_mention("alice: check this out", "alice"));
        assert!(contains_mention("hello alice!", "alice"));

        // Case insensitivity
        assert!(contains_mention("Hey ALICE, how are you?", "alice"));
        assert!(contains_mention("Hey Alice, how are you?", "alice"));
        assert!(contains_mention("hey alice, how are you?", "ALICE"));

        // @ mentions
        assert!(contains_mention("@alice check this out", "alice"));
        assert!(contains_mention("hey @alice", "alice"));
    }

    #[test]
    fn test_contains_mention_word_boundaries() {
        // Should NOT match when nick is inside another word
        assert!(!contains_mention("malice aforethought", "alice"));
        assert!(!contains_mention("bobcat is cute", "bob"));
        assert!(!contains_mention("jacoby is here", "jacob"));

        // Should match at start/end of text
        assert!(contains_mention("alice", "alice"));
        assert!(contains_mention("hi alice", "alice"));
        assert!(contains_mention("alice says hi", "alice"));

        // Should match with punctuation
        assert!(contains_mention("alice's message", "alice"));
        assert!(contains_mention("(alice)", "alice"));
        assert!(contains_mention("[alice]", "alice"));
    }

    #[test]
    fn test_contains_mention_edge_cases() {
        // Empty nickname should not match
        assert!(!contains_mention("hello world", ""));

        // Empty text should not match
        assert!(!contains_mention("", "alice"));

        // Single character nick
        assert!(contains_mention("hey x what's up", "x"));
        assert!(!contains_mention("hex is cool", "x")); // x inside word
    }
}

