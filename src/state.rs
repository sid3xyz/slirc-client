//! Core application state, separated from UI logic.
//!
//! `ClientState` holds all data that represents the IRC session:
//! buffers, networks, connection status, etc. This separation allows
//! UI components to receive state as a parameter rather than owning it.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::buffer::ChannelBuffer;
use crate::config::Network;
use crate::logging::Logger;

/// Core application state for the IRC client.
///
/// This struct contains all session data, separated from UI concerns.
/// It is owned by `SlircApp` and passed to UI components as needed.
#[derive(Default)]
pub struct ClientState {
    /// Whether we are currently connected to a server.
    pub is_connected: bool,

    /// Message buffers keyed by channel/query name.
    pub buffers: HashMap<String, ChannelBuffer>,

    /// Ordered list of buffer names (for sidebar display).
    pub buffers_order: Vec<String>,

    /// Currently active/visible buffer.
    pub active_buffer: String,

    /// Configured networks.
    pub networks: Vec<Network>,

    /// System log messages (shown in "System" buffer).
    pub system_log: Vec<String>,

    /// Networks expanded in the sidebar tree view.
    pub expanded_networks: HashSet<String>,

    /// Status toast messages with creation time (auto-expire).
    pub status_messages: Vec<(String, Instant)>,

    /// Chat logger for persisting messages to disk.
    pub logger: Option<Logger>,
}

impl ClientState {
    /// Create a new ClientState with default values.
    pub fn new() -> Self {
        let mut state = Self {
            is_connected: false,
            buffers: HashMap::new(),
            buffers_order: vec!["System".into()],
            active_buffer: "System".into(),
            networks: Vec::new(),
            system_log: vec!["Welcome to SLIRC!".into()],
            expanded_networks: HashSet::new(),
            status_messages: Vec::new(),
            logger: Logger::new().ok(),
        };

        // Create the System buffer
        state.buffers.insert("System".into(), ChannelBuffer::new());

        state
    }

    /// Get a mutable reference to a buffer, creating it if needed.
    #[allow(dead_code)]
    pub fn ensure_buffer(&mut self, name: &str) -> &mut ChannelBuffer {
        if !self.buffers.contains_key(name) {
            self.buffers.insert(name.to_string(), ChannelBuffer::new());
            self.buffers_order.push(name.to_string());
        }
        self.buffers.get_mut(name).expect("Buffer should exist")
    }

    /// Switch to the next buffer in order.
    pub fn next_buffer(&mut self) {
        if let Some(current_idx) = self.buffers_order.iter().position(|b| b == &self.active_buffer) {
            let next_idx = (current_idx + 1) % self.buffers_order.len();
            if let Some(next_buffer) = self.buffers_order.get(next_idx) {
                self.active_buffer = next_buffer.clone();
                if let Some(buffer) = self.buffers.get_mut(next_buffer) {
                    buffer.clear_unread();
                    buffer.has_highlight = false;
                }
            }
        }
    }

    /// Switch to the previous buffer in order.
    pub fn prev_buffer(&mut self) {
        if let Some(current_idx) = self.buffers_order.iter().position(|b| b == &self.active_buffer) {
            let prev_idx = if current_idx == 0 {
                self.buffers_order.len() - 1
            } else {
                current_idx - 1
            };
            if let Some(prev_buffer) = self.buffers_order.get(prev_idx) {
                self.active_buffer = prev_buffer.clone();
                if let Some(buffer) = self.buffers.get_mut(prev_buffer) {
                    buffer.clear_unread();
                    buffer.has_highlight = false;
                }
            }
        }
    }

    /// Switch to a specific buffer by name.
    #[allow(dead_code)]
    pub fn switch_to_buffer(&mut self, name: &str) {
        if self.buffers.contains_key(name) {
            self.active_buffer = name.to_string();
            if let Some(buffer) = self.buffers.get_mut(name) {
                buffer.clear_unread();
                buffer.has_highlight = false;
            }
        }
    }

    /// Purge status messages older than the given duration.
    pub fn purge_old_status_messages(&mut self, max_age_secs: u64) {
        self.status_messages
            .retain(|(_, created)| created.elapsed().as_secs() < max_age_secs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_state_new() {
        let state = ClientState::new();
        assert!(!state.is_connected);
        assert!(state.buffers.contains_key("System"));
        assert_eq!(state.active_buffer, "System");
        assert_eq!(state.buffers_order, vec!["System"]);
    }

    #[test]
    fn test_ensure_buffer() {
        let mut state = ClientState::new();
        state.ensure_buffer("#test");
        assert!(state.buffers.contains_key("#test"));
        assert!(state.buffers_order.contains(&"#test".to_string()));
    }

    #[test]
    fn test_next_prev_buffer() {
        let mut state = ClientState::new();
        state.ensure_buffer("#chan1");
        state.ensure_buffer("#chan2");

        assert_eq!(state.active_buffer, "System");
        state.next_buffer();
        assert_eq!(state.active_buffer, "#chan1");
        state.next_buffer();
        assert_eq!(state.active_buffer, "#chan2");
        state.next_buffer();
        assert_eq!(state.active_buffer, "System"); // wrap around

        state.prev_buffer();
        assert_eq!(state.active_buffer, "#chan2");
    }

    #[test]
    fn test_switch_to_buffer() {
        let mut state = ClientState::new();
        state.ensure_buffer("#test");
        state.switch_to_buffer("#test");
        assert_eq!(state.active_buffer, "#test");

        // Switching to non-existent buffer does nothing
        state.switch_to_buffer("#nonexistent");
        assert_eq!(state.active_buffer, "#test");
    }
}
