//! Modern font loading system for slirc-client
//! Bundles Inter (proportional) and JetBrains Mono (monospace) for consistent rendering.

use eframe::egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

/// Setup modern fonts (Inter + JetBrains Mono)
pub fn setup_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    // Load bundled fonts (Inter for UI, JetBrains Mono for IRC messages)
    fonts.font_data.insert(
        "Inter-Regular".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../fonts/Inter-Regular.ttf"
        ))),
    );

    fonts.font_data.insert(
        "Inter-Medium".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../fonts/Inter-Medium.ttf"
        ))),
    );

    fonts.font_data.insert(
        "Inter-Bold".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../fonts/Inter-Bold.ttf"
        ))),
    );

    fonts.font_data.insert(
        "JetBrainsMono-Regular".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../fonts/JetBrainsMono-Regular.ttf"
        ))),
    );

    fonts.font_data.insert(
        "JetBrainsMono-Medium".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../fonts/JetBrainsMono-Medium.ttf"
        ))),
    );

    fonts.font_data.insert(
        "JetBrainsMono-Bold".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../fonts/JetBrainsMono-Bold.ttf"
        ))),
    );

    // Set font families with proper fallbacks
    fonts.families.insert(
        FontFamily::Proportional,
        vec![
            "Inter-Regular".to_owned(),
            "Ubuntu-Light".to_owned(),      // egui default fallback
            "NotoEmoji-Regular".to_owned(), // emoji support
        ],
    );

    fonts.families.insert(
        FontFamily::Monospace,
        vec![
            "JetBrainsMono-Regular".to_owned(),
            "Hack".to_owned(), // egui default fallback
        ],
    );

    fonts
}
