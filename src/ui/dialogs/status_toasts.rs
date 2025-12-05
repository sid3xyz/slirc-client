//! Status toast notifications - floating messages in top-right corner.

use eframe::egui;

/// Render floating status toasts (top-right corner).
///
/// # Arguments
/// * `ctx` - The egui context
/// * `status_messages` - List of (message, timestamp) pairs
pub fn render_status_toasts(ctx: &egui::Context, status_messages: &[(String, std::time::Instant)]) {
    if status_messages.is_empty() {
        return;
    }

    let msgs: Vec<String> = status_messages.iter().map(|(m, _t)| m.clone()).collect();

    egui::Area::new(egui::Id::new("status_toast_area"))
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 50.0]) // Below menu bar
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 30, 230))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    for msg in msgs {
                        ui.label(egui::RichText::new(msg).color(egui::Color32::LIGHT_GREEN));
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    // Status toasts are purely UI, tested via integration tests
}
