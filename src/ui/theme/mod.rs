//! Modern color themes and styling utilities for the IRC client.

pub mod colors;
pub mod fonts;
pub mod widgets;

pub use colors::{SlircTheme, nick_color, prefix_rank, prefix_color, mirc_color, MIRC_COLORS};
pub use fonts::{configure_text_styles, apply_app_style};
pub use widgets::{render_avatar, generate_identicon_pattern};
