//! Modern message rendering for the central chat panel.
//! Features: message grouping, avatars, hover states, improved typography.

mod format;
mod helpers;
mod render;

// Re-export public API
pub use render::render_messages;
