//! Color themes and styling utilities for the IRC client.

use eframe::egui::Color32;

/// Nick color palette - 12 distinct colors for visual differentiation.
const NICK_COLORS: [Color32; 12] = [
    Color32::from_rgb(0xFF, 0x66, 0x66), // Red
    Color32::from_rgb(0x66, 0xCC, 0xFF), // Cyan
    Color32::from_rgb(0xFF, 0xCC, 0x66), // Orange
    Color32::from_rgb(0x99, 0xCC, 0x99), // Green
    Color32::from_rgb(0xCC, 0x99, 0xFF), // Purple
    Color32::from_rgb(0xFF, 0x99, 0xCC), // Pink
    Color32::from_rgb(0x66, 0x99, 0xFF), // Blue
    Color32::from_rgb(0xFF, 0x99, 0x66), // Peach
    Color32::from_rgb(0x99, 0xFF, 0x99), // Light green
    Color32::from_rgb(0xFF, 0xCC, 0x99), // Tan
    Color32::from_rgb(0xCC, 0xFF, 0xFF), // Light cyan
    Color32::from_rgb(0xCC, 0xCC, 0xFF), // Lavender
];

/// Generate a consistent color for a nickname using FNV-1a hash.
pub fn nick_color(nick: &str) -> Color32 {
    let mut hash: u64 = 1469598103934665603u64; // FNV offset basis
    for b in nick.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211u64); // FNV prime
    }
    let idx = (hash as usize) % NICK_COLORS.len();
    NICK_COLORS[idx]
}

/// IRC user prefix ranks (higher = more privileged).
#[allow(dead_code)]
pub fn prefix_rank(prefix: Option<char>) -> u8 {
    match prefix {
        Some('~') => 5, // Owner
        Some('&') => 4, // Admin
        Some('@') => 3, // Operator
        Some('%') => 2, // Half-op
        Some('+') => 1, // Voice
        _ => 0,         // Regular user
    }
}

/// Color for user prefix display in user list.
pub fn prefix_color(prefix: Option<char>) -> Color32 {
    match prefix {
        Some('@') | Some('~') | Some('&') => Color32::from_rgb(255, 100, 100), // Ops in red
        Some('+') | Some('%') => Color32::from_rgb(100, 200, 255),             // Voice in cyan
        _ => Color32::LIGHT_GRAY,                                              // Regular users
    }
}

/// Standard mIRC color palette (codes 0-15) for IRC text formatting.
/// See: https://modern.ircdocs.horse/formatting.html
pub const MIRC_COLORS: [Color32; 16] = [
    Color32::from_rgb(255, 255, 255), // 0  - White
    Color32::from_rgb(0, 0, 0),       // 1  - Black
    Color32::from_rgb(0, 0, 127),     // 2  - Blue
    Color32::from_rgb(0, 147, 0),     // 3  - Green
    Color32::from_rgb(255, 0, 0),     // 4  - Red
    Color32::from_rgb(127, 0, 0),     // 5  - Brown
    Color32::from_rgb(156, 0, 156),   // 6  - Purple
    Color32::from_rgb(252, 127, 0),   // 7  - Orange
    Color32::from_rgb(255, 255, 0),   // 8  - Yellow
    Color32::from_rgb(0, 252, 0),     // 9  - Light Green
    Color32::from_rgb(0, 147, 147),   // 10 - Cyan
    Color32::from_rgb(0, 255, 255),   // 11 - Light Cyan
    Color32::from_rgb(0, 0, 252),     // 12 - Light Blue
    Color32::from_rgb(255, 0, 255),   // 13 - Pink
    Color32::from_rgb(127, 127, 127), // 14 - Grey
    Color32::from_rgb(210, 210, 210), // 15 - Light Grey
];

/// Get mIRC color by code number (0-15). Returns white for out-of-range codes.
pub fn mirc_color(code: u8) -> Color32 {
    MIRC_COLORS.get(code as usize).copied().unwrap_or(Color32::WHITE)
}

/// Message type colors.
pub mod msg_colors {
    use super::Color32;

    pub const TIMESTAMP: Color32 = Color32::from_rgb(128, 128, 128);
    pub const JOIN: Color32 = Color32::from_rgb(100, 180, 100);
    pub const PART: Color32 = Color32::from_rgb(150, 150, 150);
    pub const ACTION: Color32 = Color32::from_rgb(200, 100, 200);
    pub const TOPIC: Color32 = Color32::from_rgb(100, 150, 200);
    pub const NOTICE: Color32 = Color32::from_rgb(200, 150, 100);
    pub const NOTICE_TEXT: Color32 = Color32::from_rgb(220, 220, 180);
    pub const HIGHLIGHT: Color32 = Color32::from_rgb(255, 120, 120);
    pub const UNREAD: Color32 = Color32::from_rgb(200, 200, 255);
    pub const UNREAD_COUNT: Color32 = Color32::from_rgb(100, 150, 255);
}

/// Panel background colors for visual hierarchy.
pub mod panel_colors {
    use super::Color32;

    /// Sidebar background (darker than main area)
    pub fn sidebar_bg(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(28, 28, 32)
        } else {
            Color32::from_rgb(235, 235, 240)
        }
    }

    /// Main chat area background
    pub fn chat_bg(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(38, 38, 44)
        } else {
            Color32::from_rgb(250, 250, 252)
        }
    }

    /// Input bar background
    pub fn input_bg(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(32, 32, 38)
        } else {
            Color32::from_rgb(245, 245, 248)
        }
    }

    /// Focus indicator border color
    pub fn focus_border(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(100, 140, 200)
        } else {
            Color32::from_rgb(60, 100, 180)
        }
    }

    /// Subtle separator line
    pub fn separator(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(55, 55, 65)
        } else {
            Color32::from_rgb(210, 210, 220)
        }
    }
}

/// Global spacing constants for consistent UI rhythm.
pub mod spacing {
    /// Standard item spacing (y-axis between elements)
    pub const ITEM_SPACING_Y: f32 = 6.0;
    /// Compact item spacing for message lists
    pub const MESSAGE_SPACING_Y: f32 = 3.0;
    /// Panel internal margin (i8 for egui Margin compatibility)
    pub const PANEL_MARGIN: i8 = 8;
    /// Input field corner rounding
    pub const INPUT_ROUNDING: u8 = 6;
    /// Panel corner rounding
    pub const PANEL_ROUNDING: u8 = 4;
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
