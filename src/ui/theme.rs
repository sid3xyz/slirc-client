//! Modern color themes and styling utilities for the IRC client.
//! Implements the design system from /docs/MODERN_UI_DESIGN_PLAN.md
//! Inspired by Discord, Slack, and modern chat applications (2025 standards).

use eframe::egui::{self, Color32, FontFamily, FontId, Style, TextStyle};
use std::collections::BTreeMap;

/// Modern theme with semantic color system (7-level surface hierarchy)
#[derive(Clone, Debug)]
pub struct SlircTheme {
    pub name: String,
    pub surface: [Color32; 7],
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_active: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_disabled: Color32,
    pub border_subtle: Color32,
    pub border_medium: Color32,
    pub border_strong: Color32,
}

impl SlircTheme {
    /// Modern dark theme (Discord-inspired, 2025 standards)
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            surface: [
                Color32::from_rgb(10, 10, 15),    // surface_0: App background
                Color32::from_rgb(19, 19, 26),    // surface_1: Sidebar background
                Color32::from_rgb(28, 28, 38),    // surface_2: Message background
                Color32::from_rgb(37, 37, 50),    // surface_3: Hover state
                Color32::from_rgb(46, 46, 62),    // surface_4: Active selection
                Color32::from_rgb(56, 56, 74),    // surface_5: Elevated panels
                Color32::from_rgb(66, 66, 86),    // surface_6: Modals/dialogs
            ],
            accent: Color32::from_rgb(88, 101, 242),
            accent_hover: Color32::from_rgb(71, 82, 196),
            accent_active: Color32::from_rgb(60, 69, 165),
            success: Color32::from_rgb(67, 181, 129),
            warning: Color32::from_rgb(250, 166, 26),
            error: Color32::from_rgb(240, 71, 71),
            info: Color32::from_rgb(0, 175, 244),
            text_primary: Color32::WHITE,
            text_secondary: Color32::from_rgb(185, 187, 190),
            text_muted: Color32::from_rgb(114, 118, 125),
            text_disabled: Color32::from_rgb(79, 84, 92),
            border_subtle: Color32::from_rgb(32, 34, 37),
            border_medium: Color32::from_rgb(47, 49, 54),
            border_strong: Color32::from_rgb(64, 68, 75),
        }
    }

    /// Modern light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            surface: [
                Color32::from_rgb(255, 255, 255), // surface_0: App background
                Color32::from_rgb(246, 246, 247), // surface_1: Sidebar background
                Color32::from_rgb(242, 243, 245), // surface_2: Message background
                Color32::from_rgb(227, 229, 232), // surface_3: Hover state
                Color32::from_rgb(212, 215, 220), // surface_4: Active selection
                Color32::from_rgb(196, 201, 208), // surface_5: Elevated panels
                Color32::from_rgb(181, 187, 196), // surface_6: Modals/dialogs
            ],
            accent: Color32::from_rgb(88, 101, 242),
            accent_hover: Color32::from_rgb(71, 82, 196),
            accent_active: Color32::from_rgb(60, 69, 165),
            success: Color32::from_rgb(67, 181, 129),
            warning: Color32::from_rgb(250, 166, 26),
            error: Color32::from_rgb(240, 71, 71),
            info: Color32::from_rgb(0, 175, 244),
            text_primary: Color32::from_rgb(6, 6, 7),
            text_secondary: Color32::from_rgb(79, 86, 96),
            text_muted: Color32::from_rgb(116, 127, 141),
            text_disabled: Color32::from_rgb(180, 187, 196),
            border_subtle: Color32::from_rgb(230, 232, 236),
            border_medium: Color32::from_rgb(210, 213, 219),
            border_strong: Color32::from_rgb(180, 185, 192),
        }
    }

    /// Apply theme to egui Style
    pub fn apply_to_style(&self, style: &mut Style) {
        let dark_mode = self.name == "Dark";
        style.visuals.dark_mode = dark_mode;
        style.visuals.override_text_color = Some(self.text_primary);
        style.visuals.panel_fill = self.surface[1];
        style.visuals.window_fill = self.surface[0];
        style.visuals.extreme_bg_color = self.surface[0];
        style.visuals.faint_bg_color = self.surface[2];

        // Widget colors with modern states
        style.visuals.widgets.noninteractive.bg_fill = self.surface[1];
        style.visuals.widgets.noninteractive.weak_bg_fill = self.surface[0];
        style.visuals.widgets.noninteractive.fg_stroke.color = self.text_secondary;

        style.visuals.widgets.inactive.bg_fill = self.surface[2];
        style.visuals.widgets.inactive.weak_bg_fill = self.surface[1];
        style.visuals.widgets.inactive.fg_stroke.color = self.text_secondary;

        style.visuals.widgets.hovered.bg_fill = self.surface[3];
        style.visuals.widgets.hovered.weak_bg_fill = self.surface[2];
        style.visuals.widgets.hovered.fg_stroke.color = self.text_primary;

        style.visuals.widgets.active.bg_fill = self.surface[4];
        style.visuals.widgets.active.weak_bg_fill = self.surface[3];
        style.visuals.widgets.active.fg_stroke.color = self.accent;

        // Selection
        style.visuals.selection.bg_fill = self.accent.linear_multiply(0.3);
        style.visuals.selection.stroke.color = self.accent;

        // Hyperlinks
        style.visuals.hyperlink_color = self.info;

        // Spacing (8pt grid system)
        style.spacing.item_spacing = [8.0, 8.0].into();
        style.spacing.button_padding = [12.0, 6.0].into();
        style.spacing.indent = 20.0;
        style.spacing.scroll.bar_width = 8.0;
        style.spacing.scroll.bar_inner_margin = 2.0;
        style.spacing.scroll.bar_outer_margin = 0.0;
    }
}

/// Configure modern text styles (16px base font, proper hierarchy)
pub fn configure_text_styles() -> BTreeMap<TextStyle, FontId> {
    use FontFamily::{Monospace, Proportional};
    
    [
        (TextStyle::Small, FontId::new(10.0, Proportional)),
        (TextStyle::Body, FontId::new(14.0, Proportional)),
        (TextStyle::Button, FontId::new(13.0, Proportional)),
        (TextStyle::Heading, FontId::new(16.0, Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, Monospace)),
        // IRC-specific custom styles
        (TextStyle::Name("irc_message".into()), FontId::new(14.0, Monospace)),
        (TextStyle::Name("irc_timestamp".into()), FontId::new(11.0, Monospace)),
        (TextStyle::Name("irc_nick".into()), FontId::new(13.0, Proportional)),
        (TextStyle::Name("topic".into()), FontId::new(12.0, Proportional)),
        (TextStyle::Name("section_header".into()), FontId::new(11.0, Proportional)),
        (TextStyle::Name("channel_name".into()), FontId::new(14.0, Proportional)),
    ]
    .into()
}

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
pub fn prefix_color(theme: &SlircTheme, prefix: Option<char>) -> Color32 {
    match prefix {
        Some('@') | Some('~') | Some('&') => theme.success,  // Green for ops
        Some('+') | Some('%') => theme.warning,               // Orange for voice
        _ => theme.text_muted,                                // Gray for regular
    }
}

/// Standard mIRC color palette (legacy support)
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

/// Render a circular avatar with user initials
pub fn render_avatar(ui: &mut eframe::egui::Ui, nick: &str, size: f32) -> eframe::egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        eframe::egui::vec2(size, size),
        eframe::egui::Sense::hover(),
    );

    let bg_color = nick_color(nick);
    let painter = ui.painter();

    // Draw subtle shadow for depth
    let shadow_offset = eframe::egui::vec2(0.0, 1.5);
    painter.circle_filled(
        rect.center() + shadow_offset,
        size / 2.0,
        Color32::from_black_alpha(30),
    );

    // Draw circle with border
    painter.circle_filled(rect.center(), size / 2.0, bg_color);
    painter.circle_stroke(
        rect.center(),
        size / 2.0,
        eframe::egui::Stroke::new(1.5, Color32::from_white_alpha(15)),
    );

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
pub fn render_status_dot(
    ui: &mut eframe::egui::Ui,
    theme: &SlircTheme,
    prefix: Option<char>,
) -> eframe::egui::Response {
    let size = 8.0;
    let (rect, response) = ui.allocate_exact_size(
        eframe::egui::vec2(size, size),
        eframe::egui::Sense::hover(),
    );

    let color = prefix_color(theme, prefix);
    ui.painter().circle_filled(rect.center(), size / 2.0, color);

    response
}

/// Render unread badge
pub fn render_unread_badge(ui: &mut eframe::egui::Ui, theme: &SlircTheme, count: usize, has_mention: bool) {
    if count == 0 {
        return;
    }

    let (bg, fg) = if has_mention {
        (theme.error, Color32::WHITE)
    } else {
        (theme.accent, Color32::WHITE)
    };

    let text = if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    };

    let font_id = eframe::egui::FontId::new(10.0, eframe::egui::FontFamily::Proportional);
    let galley = ui.fonts(|f| f.layout_no_wrap(text.clone(), font_id.clone(), fg));

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

    #[test]
    fn test_theme_creation() {
        let dark = SlircTheme::dark();
        assert_eq!(dark.name, "Dark");
        assert_eq!(dark.surface.len(), 7);
        
        let light = SlircTheme::light();
        assert_eq!(light.name, "Light");
        assert_eq!(light.surface.len(), 7);
    }
}
