//! UI rendering modules for the SLIRC client.
//!
//! This module contains all egui-based UI rendering code, organized by component:
//! - `toolbar`: Top toolbar with connection controls
//! - `panels`: Side panels (channel list, user list)
//! - `messages`: Message area rendering
//! - `dialogs`: Modal dialogs (help, network manager, etc.)
//! - `theme`: Color schemes and styling utilities

pub mod dialogs;
pub mod messages;
pub mod panels;
pub mod theme;
pub mod toolbar;

// Re-export commonly used items
pub use panels::sort_users;
