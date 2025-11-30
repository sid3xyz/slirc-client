//! Modal dialogs and windows - self-contained dialog components.
//!
//! Each dialog owns its editing state and returns `DialogAction`s
//! instead of mutating external state directly. This follows egui
//! best practices and avoids borrow checker issues.
//!
//! # Architecture
//!
//! Dialogs are stored as `Option<Dialog>` in `SlircApp`:
//! - `None` = dialog is closed
//! - `Some(dialog)` = dialog is open with its state
//!
//! Dialogs return `Option<DialogAction>` from their `show()` method,
//! which the app processes in its update loop.

mod actions;
mod channel_browser;
mod help;
mod network;
mod nick_change;
mod status_toasts;
mod topic_editor;

// Re-export dialog types and actions
pub use actions::DialogAction;
pub use channel_browser::{ChannelBrowserDialog, ChannelListItem};
pub use help::HelpDialog;
pub use network::NetworkManagerDialog;
pub use nick_change::NickChangeDialog;
pub use status_toasts::render_status_toasts;
pub use topic_editor::TopicEditorDialog;
