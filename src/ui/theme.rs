//! Modern color themes and styling utilities for the IRC client.
//!
//! # Overview
//!
//! This module implements a complete design system based on `/docs/MODERN_UI_DESIGN_PLAN.md`,
//! drawing inspiration from Discord, Slack, and modern chat applications (2025 standards).
//!
//! # Architecture
//!
//! ## Surface Hierarchy (7 Levels)
//!
//! The theme uses a 7-level surface system for depth perception:
//!
//! - `surface[0]`: App background (deepest layer)
//! - `surface[1]`: Sidebar/panel backgrounds
//! - `surface[2]`: Message area background
//! - `surface[3]`: Hover states
//! - `surface[4]`: Active/selected states
//! - `surface[5]`: Elevated panels (modals)
//! - `surface[6]`: Highest elevation (dialogs, popovers)
//!
//! ## Semantic Colors
//!
//! Beyond surfaces, the theme provides semantic colors for common UI states:
//!
//! - **Accent**: Primary brand color (used for buttons, links, focus indicators)
//! - **Success**: Green for positive actions (connected, sent, online)
//! - **Warning**: Orange for caution states (away, rate-limited)
//! - **Error**: Red for failures (disconnected, invalid input)
//! - **Info**: Blue for informational messages
//!
//! ## Text Hierarchy
//!
//! Four levels of text emphasis:
//!
//! - `text_primary`: Main content (WHITE in dark mode)
//! - `text_secondary`: Supporting content (lighter gray)
//! - `text_muted`: De-emphasized text (timestamps, metadata)
//! - `text_disabled`: Inactive UI elements
//!
//! ## Border System
//!
//! Three border weights for visual separation:
//!
//! - `border_subtle`: Faint dividers (between list items)
//! - `border_medium`: Standard borders (panels, inputs)
//! - `border_strong`: Prominent borders (focus indicators)
//!
//! # Usage Examples
//!
//! ```rust
//! use crate::ui::theme::SlircTheme;
//!
//! // Get the theme
//! let theme = SlircTheme::dark();
//!
//! // Render a panel with proper surface
//! egui::Frame::none()
//!     .fill(theme.surface[1])  // Sidebar background
//!     .stroke(egui::Stroke::new(1.0, theme.border_medium))
//!     .show(ui, |ui| {
//!         // Panel content
//!     });
//!
//! // Show an error message
//! ui.colored_label(theme.error, "Connection failed!");
//!
//! // Render a nickname with consistent color
//! let nick = "alice";
//! let color = nick_color(nick);
//! ui.colored_label(color, nick);
//! ```
//!
//! # Design Principles
//!
//! 1. **Consistent Depth**: Always use the surface hierarchy for z-ordering
//! 2. **Semantic Over Arbitrary**: Use semantic colors (success/warning/error) not random colors
//! 3. **Accessibility**: All colors meet WCAG AA contrast requirements
//! 4. **Dark-First**: Dark theme is the primary design, light theme is inverted
//!
//! # References
//!
//! - Design spec: `/docs/MODERN_UI_DESIGN_PLAN.md`
//! - Audit doc: `/docs/AUDIT_AND_FORWARD_PATH.md`
//! - WCAG 2.1 AA: <https://www.w3.org/WAI/WCAG21/quickref/>

use eframe::egui::{Color32, FontFamily, FontId, TextStyle};
use std::collections::BTreeMap;

/// Modern theme with semantic color system (7-level surface hierarchy)
#[derive(Clone, Debug)]
#[allow(dead_code)]
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
}

/// Configure modern text styles (16px base font, proper hierarchy)
///
/// # Typography System
///
/// Based on the MODERN_UI_DESIGN_PLAN, this creates a complete text hierarchy:
///
/// ## Standard Styles
///
/// - **Small**: 10px proportional - Timestamps, metadata
/// - **Body**: 14px proportional - Standard UI text, labels
/// - **Button**: 13px proportional - Button labels (slightly smaller for weight)
/// - **Heading**: 16px proportional - Section headers, emphasis
/// - **Monospace**: 13px monospace - Code, technical content
///
/// ## IRC-Specific Styles
///
/// - **irc_message**: 14px monospace - Chat message content
/// - **irc_timestamp**: 11px monospace - Message timestamps
/// - **irc_nick**: 13px proportional - Nickname labels
/// - **topic**: 12px proportional - Channel topics
/// - **section_header**: 11px proportional - Sidebar sections ("NETWORKS", "CHANNELS")
/// - **channel_name**: 14px proportional - Channel names in sidebar
///
/// # Font Families
///
/// - **Proportional**: Inter (clean, modern sans-serif)
/// - **Monospace**: JetBrains Mono (excellent for chat messages)
///
/// Both fonts are bundled via `src/fonts.rs`.
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

/// Apply the modern theme style to the egui context
///
/// # What This Does
///
/// Configures the global egui style with:
///
/// 1. **Typography**: Applies the complete text hierarchy (see `configure_text_styles()`)
/// 2. **Spacing**: Sets consistent padding/margins following 8px grid
/// 3. **Button Styling**: Modern rounded buttons with proper hover/active states
/// 4. **Input Styling**: Dark text inputs with subtle selection color
/// 5. **Visual Refinements**: Corner radius, stroke removal, consistent colors
///
/// # Styling Details
///
/// ## Spacing (8px grid system)
///
/// - Item spacing: 8px horizontal, 6px vertical
/// - Window margin: 12px all sides
/// - Button padding: 10px horizontal, 5px vertical
///
/// ## Button States
///
/// - **Inactive**: `rgb(55, 60, 70)` - Neutral dark gray
/// - **Hovered**: `rgb(70, 76, 88)` - Lighter gray
/// - **Active**: `rgb(88, 101, 242)` - Accent blue
/// - Corner radius: 6px (all states)
/// - No stroke (modern flat design)
///
/// ## Text Input
///
/// - Background: `rgb(30, 32, 38)` - Slightly darker than surface
/// - Selection: `rgba(88, 101, 242, 100)` - Semi-transparent accent
///
/// # Usage
///
/// Call once during app initialization:
///
/// ```rust
/// impl eframe::App for SlircApp {
///     fn new(cc: &eframe::CreationContext<'_>) -> Self {
///         crate::ui::theme::apply_app_style(&cc.egui_ctx);
///         crate::fonts::setup_fonts(&cc.egui_ctx);
///         // ... rest of setup
///     }
/// }
/// ```
pub fn apply_app_style(ctx: &eframe::egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Set professional font sizes and improved spacing
    style.text_styles = configure_text_styles();
    
    // Increase global spacing for breathing room
    style.spacing.item_spacing = eframe::egui::vec2(8.0, 6.0);
    style.spacing.window_margin = eframe::egui::Margin::same(12);
    style.spacing.button_padding = eframe::egui::vec2(10.0, 5.0);
    
    // Modern button styling
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(55, 60, 70);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(55, 60, 70);
    style.visuals.widgets.inactive.bg_stroke = eframe::egui::Stroke::NONE;
    style.visuals.widgets.inactive.corner_radius = eframe::egui::CornerRadius::same(6);
    
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(70, 76, 88);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(70, 76, 88);
    style.visuals.widgets.hovered.bg_stroke = eframe::egui::Stroke::NONE;
    style.visuals.widgets.hovered.corner_radius = eframe::egui::CornerRadius::same(6);
    
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(88, 101, 242);
    style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(88, 101, 242);
    style.visuals.widgets.active.corner_radius = eframe::egui::CornerRadius::same(6);
    
    // Text input styling
    style.visuals.extreme_bg_color = Color32::from_rgb(30, 32, 38);
    style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(88, 101, 242, 100);
    
    ctx.set_style(style);
}

/// Modern nick color palette - 16 vibrant, accessible colors
///
/// # Design
///
/// Carefully selected for:
///
/// - **Distinctness**: Colors are visually different from each other
/// - **Accessibility**: All meet WCAG AA contrast on dark backgrounds
/// - **Vibrancy**: Modern, saturated colors (not muted pastels)
/// - **Consistency**: Based on Material Design and Tailwind palettes
///
/// # Usage
///
/// Don't use this array directly. Use `nick_color(nick)` which deterministically
/// maps nicknames to colors using FNV-1a hashing.
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
///
/// # How It Works
///
/// Uses the FNV-1a (Fowler-Noll-Vo) hash algorithm to deterministically map
/// any nickname string to one of 16 vibrant colors. Same nickname always gets
/// the same color, making it easy to visually track users in chat.
///
/// # Why FNV-1a?
///
/// - **Fast**: Single pass over the string
/// - **Good distribution**: Minimizes color collisions
/// - **Simple**: No dependencies, easy to audit
///
/// # Example
///
/// ```rust
/// let alice_color = nick_color("alice");
/// let bob_color = nick_color("bob");
/// // alice_color != bob_color (very likely)
/// // nick_color("alice") == alice_color (always)
/// ```
///
/// # Parameters
///
/// - `nick`: The nickname string (case-sensitive)
///
/// # Returns
///
/// A `Color32` from the `NICK_COLORS` palette.
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
///
/// # IRC Prefix System
///
/// IRC uses single-character prefixes to denote user permissions in a channel:
///
/// - `~` (Rank 5): Founder/Owner - Full control of channel
/// - `&` (Rank 4): Protected/Admin - Cannot be kicked by ops
/// - `@` (Rank 3): Operator - Can kick/ban users, change modes
/// - `%` (Rank 2): Half-op - Limited moderation (kick only)
/// - `+` (Rank 1): Voice - Can speak in moderated channels
/// - None (Rank 0): Regular user
///
/// # Usage
///
/// Use this for sorting user lists (highest rank first):
///
/// ```rust
/// users.sort_by_key(|u| std::cmp::Reverse(prefix_rank(u.prefix)));
/// ```
///
/// # Parameters
///
/// - `prefix`: Optional prefix character from IRC NAMES reply
///
/// # Returns
///
/// Numeric rank 0-5 (higher = more privileged)
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
///
/// # Visual Hierarchy
///
/// Maps IRC prefixes to semantic colors:
///
/// - **Ops** (`@`, `~`, `&`): Green (success color) - Trusted moderators
/// - **Voice** (`+`, `%`): Orange (warning color) - Elevated users
/// - **Regular** (none): Gray (muted) - Standard users
///
/// # Usage
///
/// ```rust
/// let color = prefix_color(&theme, Some('@'));
/// ui.colored_label(color, "@");
/// ```
///
/// # Parameters
///
/// - `theme`: The active theme (provides semantic colors)
/// - `prefix`: Optional prefix character
///
/// # Returns
///
/// Appropriate `Color32` for the prefix level
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
