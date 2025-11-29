//! UI rendering modules for the SLIRC client.
//!
//! This module contains all egui-based UI rendering code, organized by component:
//! - `toolbar`: Top toolbar with connection controls
//! - `panels`: Side panels (channel list, user list)
//! - `messages`: Message area rendering
//! - `dialogs`: Modal dialogs (help, network manager, etc.)
//! - `theme`: Color schemes and styling utilities

mod dialogs;
mod messages;
mod panels;
mod theme;
mod toolbar;

pub use dialogs::*;
pub use messages::*;
pub use panels::*;
pub use theme::*;
pub use toolbar::*;
