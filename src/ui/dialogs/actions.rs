//! Dialog action types - dialogs return actions instead of mutating state directly.
//!
//! This follows the immediate-mode GUI pattern where dialogs return results
//! that the main app processes, avoiding callback hell and borrow checker issues.

use crate::config::Network;

/// Actions that dialogs can return to the main application.
/// The app processes these in its update loop.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DialogAction {
    // Network Manager actions
    NetworkConnect(Network),
    NetworkSave {
        index: Option<usize>,
        network: Network,
    },
    NetworkDelete(usize),

    // Nick change
    ChangeNick(String),

    // Channel browser
    JoinChannel(String),

    // Topic editor
    SetTopic {
        channel: String,
        topic: String,
    },
}
