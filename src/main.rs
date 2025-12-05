//! SLIRC Client - An IRC client built with egui and slirc-proto
//!
//! Architecture:
//! - Main thread: runs the egui UI
//! - Backend thread: runs a Tokio runtime for async network I/O
//! - Communication via crossbeam channels (lock-free, sync-safe)

use eframe::egui;
use slirc_client::app::SlircApp;
use slirc_client::fonts;
use slirc_client::ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0]) // Modern default size
            .with_min_inner_size([800.0, 600.0]), // Minimum for 2.5-column layout
        ..Default::default()
    };

    eframe::run_native(
        "SLIRC - IRC Client",
        options,
        Box::new(|cc| {
            // Setup modern fonts (Inter + JetBrains Mono)
            cc.egui_ctx.set_fonts(fonts::setup_fonts());

            // Setup modern text styles
            cc.egui_ctx.all_styles_mut(|style| {
                style.text_styles = ui::theme::configure_text_styles();
            });

            Ok(Box::new(SlircApp::new(cc)))
        }),
    )
}
