//! Font configuration and text style definitions.
//!
//! Typography system based on MODERN_UI_DESIGN_PLAN with proper hierarchy.

use eframe::egui::{Color32, FontFamily, FontId, TextStyle};
use std::collections::BTreeMap;

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
/// ```ignore
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
