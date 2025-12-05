//! Modern color themes and styling utilities for the IRC client.

pub mod colors;
pub mod fonts;
pub mod widgets;

pub use colors::{mirc_color, nick_color, prefix_color, prefix_rank, SlircTheme, MIRC_COLORS};
pub use fonts::{apply_app_style, configure_text_styles};
pub use widgets::{generate_identicon_pattern, render_avatar};
