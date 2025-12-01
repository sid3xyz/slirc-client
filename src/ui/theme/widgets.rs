//! Avatar rendering and identicon generation.

use eframe::egui::Color32;
use super::colors::nick_color;

/// Render a circular avatar with identicon pattern generated from nickname hash.
///
/// # Identicon Design
///
/// Creates a 5x5 symmetric pattern within a circular mask:
/// - Uses FNV-1a hash of nickname for determinism
/// - Vertically symmetric (like a Rorschach inkblot)
/// - Background color from `nick_color()` palette
/// - Foreground pattern in contrasting white/light color
///
/// # Parameters
///
/// - `ui`: The egui UI context
/// - `nick`: The nickname to generate an identicon for
/// - `size`: Diameter of the avatar in pixels
///
/// # Returns
///
/// Response for the avatar widget (for hover/click handling)
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

    // Draw background circle
    painter.circle_filled(rect.center(), size / 2.0, bg_color);

    // Generate identicon pattern using hash
    let pattern = generate_identicon_pattern(nick);
    let fg_color = Color32::from_white_alpha(200);
    let cell_size = size / 6.0; // 5 cells + 0.5 padding each side
    let offset = cell_size * 0.5;

    // Draw the 5x5 symmetric pattern
    for row in 0..5 {
        for col in 0..5 {
            // Use symmetric index (mirror left to right)
            let pattern_col = if col < 3 { col } else { 4 - col };
            let bit_index = row * 3 + pattern_col;

            if pattern & (1 << bit_index) != 0 {
                let cell_x = rect.left() + offset + (col as f32 * cell_size);
                let cell_y = rect.top() + offset + (row as f32 * cell_size);
                let center = eframe::egui::pos2(cell_x + cell_size / 2.0, cell_y + cell_size / 2.0);

                // Check if cell center is within the circle (clip to circle)
                let dist = (center - rect.center()).length();
                if dist < size / 2.0 - cell_size * 0.3 {
                    painter.rect_filled(
                        eframe::egui::Rect::from_min_size(
                            eframe::egui::pos2(cell_x, cell_y),
                            eframe::egui::vec2(cell_size * 0.85, cell_size * 0.85),
                        ),
                        cell_size * 0.2, // rounded corners
                        fg_color,
                    );
                }
            }
        }
    }

    // Draw subtle border
    painter.circle_stroke(
        rect.center(),
        size / 2.0,
        eframe::egui::Stroke::new(1.5, Color32::from_white_alpha(15)),
    );

    response
}

/// Generate a 15-bit pattern for a 5x5 symmetric identicon.
///
/// Uses FNV-1a hash to deterministically generate a pattern from nickname.
/// Only 15 bits needed because the pattern is symmetric (3 columns × 5 rows).
pub fn generate_identicon_pattern(nick: &str) -> u16 {
    // FNV-1a hash
    let mut hash: u64 = 1469598103934665603u64;
    for b in nick.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211u64);
    }

    // Use bottom 15 bits for the pattern, ensure center column has some fill
    (hash as u16 & 0x7FFF) | 0x0084
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identicon_pattern_deterministic() {
        let pattern1 = generate_identicon_pattern("alice");
        let pattern2 = generate_identicon_pattern("alice");
        assert_eq!(pattern1, pattern2, "Same nick should produce same pattern");

        let pattern3 = generate_identicon_pattern("bob");
        assert_ne!(pattern1, pattern3, "Different nicks should produce different patterns");
    }

    #[test]
    fn test_identicon_pattern_has_visible_bits() {
        // The pattern should always have at least some bits set for visual interest
        let pattern = generate_identicon_pattern("x");
        assert!(pattern != 0, "Pattern should not be empty");
        assert!(pattern & 0x0084 != 0, "Pattern should have center bits set");
    }
}
