//! UI rendering modules for the SLIRC client.
//!
//! This module contains all egui-based UI rendering code, organized by component:
//! - `menu`: Traditional horizontal menu bar (File/Edit/View/Server/Window/Help)
//! - `toolbar`: Top toolbar with connection controls
//! - `panels`: Side panels (channel list, user list)
//! - `messages`: Message area rendering
//! - `dialogs`: Modal dialogs (help, network manager, etc.) - self-contained components
//! - `theme`: Color schemes and styling utilities
//! - `quick_switcher`: Quick channel/DM switcher (Ctrl+K)
//! - `shortcuts`: Keyboard shortcut registry and help overlay

pub mod dialogs;
pub mod menu;
pub mod messages;
pub mod panels;
pub mod quick_switcher;
pub mod shortcuts;
pub mod theme;
pub mod toolbar;
pub mod topic_bar;

// Re-export commonly used items
pub use panels::sort_users;
