//! Modern color themes and styling utilities for the IRC client.
//! Inspired by Discord, Slack, and Element design patterns.

use eframe::egui::Color32;

/// Modern nick color palette - 16 vibrant, accessible colors
const NICK_COLORS: [Color32; 16] = [
    Color32::from_rgb(231, 76, 60),   // Vibrant red
    Color32::from_rgb(46, 204, 113),  // Emerald green
    Color32::from_rgb(52, 152, 219),  // Bright blue
    Color32::from_rgb(155, 89, 182),  // Amethyst purple
    Color32::from_rgb(241, 196, 15),  // Sunflower yellow
    Color32::from_rgb(230, 126, 34),  // Carrot orange
    Color32::from_rgb(26, 188, 156),  // Turquoise
    Color32::from_rgb(236, 100, 166), // Pink
    Color32::from_rgb(142, 68, 173),  // Wisteria
    Color32::from_rgb(41, 128, 185),  // Belize blue
    Color32::from_rgb(39, 174, 96),   // Nephritis
    Color32::from_rgb(243, 156, 18),  // Orange
    Color32::from_rgb(192, 57, 43),   // Pomegranate
    Color32::from_rgb(22, 160, 133),  // Green sea
    Color32::from_rgb(211, 84, 0),    // Pumpkin
    Color32::from_rgb(102, 178, 255), // Light blue
];

/// Generate a consistent color for a nickname using FNV-1a hash.
pub fn nick_color(nick: &str) -> Color32 {
    let mut hash: u64 = 1469598103934665603u64;
    for b in nick.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211u64);
    }
    let idx = (hash as usize) % NICK_COLORS.len();
    NICK_COLORS[idx]
}

/// IRC user prefix ranks (higher = more privileged).
#[allow(dead_code)]
pub fn prefix_rank(prefix: Option<char>) -> u8 {
    match prefix {
        Some('~') => 5,
        Some('&') => 4,
        Some('@') => 3,
        Some('%') => 2,
        Some('+') => 1,
        _ => 0,
    }
}

/// Color for user prefix/status indicator
pub fn prefix_color(prefix: Option<char>) -> Color32 {
    match prefix {
        Some('@') | Some('~') | Some('&') => Color32::from_rgb(67, 181, 129),  // Green for ops
        Some('+') | Some('%') => Color32::from_rgb(250, 166, 26),               // Orange for voice
        _ => Color32::from_rgb(116, 127, 141),                                  // Gray for regular
    }
}

/// Standard mIRC color palette
pub const MIRC_COLORS: [Color32; 16] = [
    Color32::from_rgb(255, 255, 255),
    Color32::from_rgb(0, 0, 0),
    Color32::from_rgb(0, 0, 127),
    Color32::from_rgb(0, 147, 0),
    Color32::from_rgb(255, 0, 0),
    Color32::from_rgb(127, 0, 0),
    Color32::from_rgb(156, 0, 156),
    Color32::from_rgb(252, 127, 0),
    Color32::from_rgb(255, 255, 0),
    Color32::from_rgb(0, 252, 0),
    Color32::from_rgb(0, 147, 147),
    Color32::from_rgb(0, 255, 255),
    Color32::from_rgb(0, 0, 252),
    Color32::from_rgb(255, 0, 255),
    Color32::from_rgb(127, 127, 127),
    Color32::from_rgb(210, 210, 210),
];

pub fn mirc_color(code: u8) -> Color32 {
    MIRC_COLORS.get(code as usize).copied().unwrap_or(Color32::WHITE)
}

/// Modern dark theme palette (Discord-inspired)
pub mod dark {
    use super::Color32;

    // Layered backgrounds for depth
    pub const BG_DARKEST: Color32 = Color32::from_rgb(18, 18, 23);
    pub const BG_DARKER: Color32 = Color32::from_rgb(24, 25, 31);
    pub const BG_DARK: Color32 = Color32::from_rgb(32, 34, 41);
    pub const BG_BASE: Color32 = Color32::from_rgb(40, 43, 52);
    pub const BG_ELEVATED: Color32 = Color32::from_rgb(50, 54, 65);
    pub const BG_HOVER: Color32 = Color32::from_rgb(58, 62, 75);
    pub const BG_ACTIVE: Color32 = Color32::from_rgb(66, 71, 86);

    // Text hierarchy
    pub const TEXT_NORMAL: Color32 = Color32::from_rgb(220, 221, 225);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(148, 155, 164);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(96, 102, 112);

    // Accent colors
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(88, 101, 242);
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(67, 181, 129);
    pub const ACCENT_YELLOW: Color32 = Color32::from_rgb(250, 166, 26);
    pub const ACCENT_RED: Color32 = Color32::from_rgb(237, 66, 69);
    pub const ACCENT_PINK: Color32 = Color32::from_rgb(235, 69, 158);

    // UI elements
    pub const BORDER: Color32 = Color32::from_rgb(55, 58, 70);
    pub const SCROLLBAR: Color32 = Color32::from_rgb(60, 64, 76);
    pub const SCROLLBAR_HOVER: Color32 = Color32::from_rgb(75, 80, 95);
}

/// Modern light theme palette
pub mod light {
    use super::Color32;

    pub const BG_DARKEST: Color32 = Color32::from_rgb(220, 222, 228);
    pub const BG_DARKER: Color32 = Color32::from_rgb(235, 237, 242);
    pub const BG_DARK: Color32 = Color32::from_rgb(242, 243, 247);
    pub const BG_BASE: Color32 = Color32::from_rgb(255, 255, 255);
    pub const BG_ELEVATED: Color32 = Color32::from_rgb(248, 249, 252);
    pub const BG_HOVER: Color32 = Color32::from_rgb(240, 241, 245);
    pub const BG_ACTIVE: Color32 = Color32::from_rgb(230, 232, 238);

    pub const TEXT_NORMAL: Color32 = Color32::from_rgb(30, 31, 34);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(80, 85, 95);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(130, 135, 145);

    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(66, 82, 210);
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(45, 155, 100);
    pub const ACCENT_RED: Color32 = Color32::from_rgb(210, 55, 60);

    pub const BORDER: Color32 = Color32::from_rgb(210, 212, 220);
    pub const SCROLLBAR: Color32 = Color32::from_rgb(190, 195, 205);
}

/// Message type colors - context-aware
pub mod msg_colors {
    use super::Color32;

    pub const TIMESTAMP: Color32 = Color32::from_rgb(116, 127, 141);
    pub const JOIN: Color32 = Color32::from_rgb(67, 181, 129);
    pub const PART: Color32 = Color32::from_rgb(148, 155, 164);
    pub const QUIT: Color32 = Color32::from_rgb(148, 155, 164);
    pub const ACTION: Color32 = Color32::from_rgb(155, 89, 182);
    pub const TOPIC: Color32 = Color32::from_rgb(52, 152, 219);
    pub const NOTICE: Color32 = Color32::from_rgb(250, 166, 26);
    pub const NOTICE_TEXT: Color32 = Color32::from_rgb(225, 200, 150);
    pub const HIGHLIGHT: Color32 = Color32::from_rgb(255, 210, 100);
    pub const SYSTEM: Color32 = Color32::from_rgb(116, 127, 141);
}

/// Panel background colors with proper depth hierarchy
pub mod panel_colors {
    use super::{dark, light, Color32};

    pub fn sidebar_bg(dark_mode: bool) -> Color32 {
        if dark_mode { dark::BG_DARKER } else { light::BG_DARKER }
    }

    pub fn chat_bg(dark_mode: bool) -> Color32 {
        if dark_mode { dark::BG_DARK } else { light::BG_BASE }
    }

    pub fn input_bg(dark_mode: bool) -> Color32 {
        if dark_mode { dark::BG_BASE } else { light::BG_ELEVATED }
    }

    pub fn input_field_bg(dark_mode: bool) -> Color32 {
        if dark_mode { dark::BG_ELEVATED } else { light::BG_BASE }
    }

    pub fn hover_bg(dark_mode: bool) -> Color32 {
        if dark_mode { dark::BG_HOVER } else { light::BG_HOVER }
    }

    pub fn active_bg(dark_mode: bool) -> Color32 {
        if dark_mode { dark::BG_ACTIVE } else { light::BG_ACTIVE }
    }

    pub fn separator(dark_mode: bool) -> Color32 {
        if dark_mode { dark::BORDER } else { light::BORDER }
    }

    pub fn focus_border(_dark_mode: bool) -> Color32 {
        dark::ACCENT_BLUE
    }
}

/// Text colors with proper hierarchy
pub mod text_colors {
    use super::{dark, light, Color32};

    pub fn primary(dark_mode: bool) -> Color32 {
        if dark_mode { dark::TEXT_NORMAL } else { light::TEXT_NORMAL }
    }

    pub fn secondary(dark_mode: bool) -> Color32 {
        if dark_mode { dark::TEXT_MUTED } else { light::TEXT_MUTED }
    }

    pub fn muted(dark_mode: bool) -> Color32 {
        if dark_mode { dark::TEXT_FAINT } else { light::TEXT_FAINT }
    }
}

/// Global spacing constants for consistent UI rhythm
pub mod spacing {
    /// Space between messages from different users
    pub const MESSAGE_GROUP_SPACING: f32 = 16.0;
    /// Space between consecutive messages from same user
    pub const MESSAGE_CONTINUATION_SPACING: f32 = 2.0;
    /// General item spacing
    pub const MESSAGE_SPACING_Y: f32 = 4.0;
    /// Channel list item height
    pub const CHANNEL_ITEM_HEIGHT: f32 = 32.0;
    /// User list item height
    pub const USER_ITEM_HEIGHT: f32 = 28.0;
    /// Panel margin
    pub const PANEL_MARGIN: i8 = 12;
    /// Input field corner rounding
    pub const INPUT_ROUNDING: u8 = 8;
    /// General corner rounding
    pub const CORNER_RADIUS: f32 = 6.0;
    /// Avatar size
    pub const AVATAR_SIZE: f32 = 36.0;
    /// Small avatar size (user list)
    pub const AVATAR_SIZE_SMALL: f32 = 8.0;
}

/// Render a circular avatar with user initials
pub fn render_avatar(ui: &mut eframe::egui::Ui, nick: &str, size: f32) -> eframe::egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        eframe::egui::vec2(size, size),
        eframe::egui::Sense::hover(),
    );

    let bg_color = nick_color(nick);
    let painter = ui.painter();

    // Draw circle
    painter.circle_filled(rect.center(), size / 2.0, bg_color);

    // Draw initials
    let initials: String = nick.chars().next().unwrap_or('?').to_uppercase().collect();
    let font_id = eframe::egui::FontId::new(size * 0.45, eframe::egui::FontFamily::Proportional);

    painter.text(
        rect.center(),
        eframe::egui::Align2::CENTER_CENTER,
        initials,
        font_id,
        Color32::WHITE,
    );

    response
}

/// Render a small status dot for user list
pub fn render_status_dot(ui: &mut eframe::egui::Ui, prefix: Option<char>) -> eframe::egui::Response {
    let size = spacing::AVATAR_SIZE_SMALL;
    let (rect, response) = ui.allocate_exact_size(
        eframe::egui::vec2(size, size),
        eframe::egui::Sense::hover(),
    );

    let color = prefix_color(prefix);
    ui.painter().circle_filled(rect.center(), size / 2.0, color);

    response
}

/// Render unread badge
pub fn render_unread_badge(ui: &mut eframe::egui::Ui, count: usize, has_mention: bool) {
    if count == 0 {
        return;
    }

    let (bg, fg) = if has_mention {
        (dark::ACCENT_RED, Color32::WHITE)
    } else {
        (dark::ACCENT_BLUE, Color32::WHITE)
    };

    let text = if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    };

    let font_id = eframe::egui::FontId::new(10.0, eframe::egui::FontFamily::Proportional);
    let galley = ui.fonts(|f| f.layout_no_wrap(text, font_id, fg));

    let padding = 4.0;
    let min_width = 18.0;
    let badge_width = galley.size().x.max(min_width - padding * 2.0) + padding * 2.0;
    let badge_height = 16.0;

    let (rect, _) = ui.allocate_exact_size(
        eframe::egui::vec2(badge_width, badge_height),
        eframe::egui::Sense::hover(),
    );

    ui.painter().rect_filled(rect, badge_height / 2.0, bg);
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        fg,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nick_color_deterministic() {
        let c1 = nick_color("alice");
        let c2 = nick_color("alice");
        assert_eq!(c1, c2);
        let c3 = nick_color("bob");
        assert_ne!(c1, c3);
    }

    #[test]
    fn test_prefix_rank_ordering() {
        assert!(prefix_rank(Some('~')) > prefix_rank(Some('@')));
        assert!(prefix_rank(Some('@')) > prefix_rank(Some('+')));
        assert!(prefix_rank(Some('+')) > prefix_rank(None));
    }
}
