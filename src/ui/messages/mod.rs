//! Modern message rendering for the central chat panel.
//! Features: message grouping, avatars, hover states, improved typography.

mod format;
mod helpers;
mod render;

// Re-export public API
pub use render::render_messages;

// Re-export for testing
#[cfg(test)]
pub(crate) use helpers::{contains_mention, parse_timestamp_seconds, timestamps_within_window};
