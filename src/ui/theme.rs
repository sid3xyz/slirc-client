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

/// Message type colors.
pub mod msg_colors {
    use super::Color32;

    pub const TIMESTAMP: Color32 = Color32::DARK_GRAY;
    pub const JOIN: Color32 = Color32::from_rgb(100, 150, 100);
    pub const PART: Color32 = Color32::GRAY;
    pub const ACTION: Color32 = Color32::from_rgb(200, 100, 200);
    pub const TOPIC: Color32 = Color32::from_rgb(100, 150, 200);
    pub const NOTICE: Color32 = Color32::from_rgb(200, 150, 100);
    pub const NOTICE_TEXT: Color32 = Color32::from_rgb(200, 200, 150);
    pub const HIGHLIGHT: Color32 = Color32::from_rgb(255, 100, 100);
    pub const UNREAD: Color32 = Color32::from_rgb(200, 200, 255);
    pub const UNREAD_COUNT: Color32 = Color32::from_rgb(100, 150, 255);
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
