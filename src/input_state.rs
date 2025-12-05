//! Input state management for message composition, history, and tab completion.
//!
//! This module separates input handling concerns from the main application state,
//! following the pattern used by modern IRC clients like Halloy.

use std::collections::HashMap;

use crate::buffer::ChannelBuffer;

/// Manages all input-related state for the IRC client.
///
/// This includes message composition, command history navigation,
/// tab completion, and channel/nickname input fields.
#[derive(Default)]
pub struct InputState {
    /// Current message being composed
    pub message_input: String,

    /// Channel name input (for join operations)
    pub channel_input: String,

    /// Command/message history (for up/down arrow navigation)
    pub history: Vec<String>,

    /// Current position in history (None = not navigating)
    pub history_pos: Option<usize>,

    /// Saved input when entering history mode
    pub history_saved_input: Option<String>,

    /// Tab completion candidates
    pub completions: Vec<String>,

    /// Current completion index (for cycling through completions)
    pub completion_index: Option<usize>,

    /// Original prefix that was completed
    pub completion_prefix: Option<String>,

    /// Whether we're completing a channel name
    pub completion_target_channel: bool,

    /// Last input text (for detecting changes that should reset completion)
    pub last_input_text: String,
}

impl InputState {
    /// Create a new InputState with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Collect tab completion candidates based on the given prefix.
    ///
    /// Supports:
    /// - IRC commands (starting with /)
    /// - Channel names (starting with # or &)
    /// - User nicknames from active buffer
    /// - @mentions
    pub fn collect_completions(
        &self,
        prefix: &str,
        buffers_order: &[String],
        active_buffer: &str,
        buffers: &HashMap<String, ChannelBuffer>,
    ) -> Vec<String> {
        let mut matches: Vec<String> = Vec::new();
        let mut search_prefix = prefix;
        let mut keep_lead = "";

        // Command completion when prefix starts with /
        if prefix.starts_with('/') {
            let commands = vec![
                "/join", "/j", "/part", "/p", "/msg", "/privmsg", "/me", "/whois", "/w", "/topic",
                "/t", "/kick", "/k", "/nick", "/quit", "/exit", "/help",
            ];
            for cmd in commands {
                if cmd.starts_with(prefix) {
                    matches.push(cmd.to_string());
                }
            }
        } else if let Some(stripped) = prefix.strip_prefix('@') {
            // Keep the '@' in the suggestion, but search without it
            search_prefix = stripped;
            keep_lead = "@";
        }

        if prefix.starts_with('#') || prefix.starts_with('&') {
            // channel completions
            for b in buffers_order {
                if b.starts_with(prefix) {
                    matches.push(b.clone());
                }
            }
        } else if !prefix.starts_with('/') {
            // user completions from active buffer (skip if completing commands)
            if let Some(buffer) = buffers.get(active_buffer) {
                for u in &buffer.users {
                    if u.nick.starts_with(search_prefix) {
                        matches.push(format!("{}{}", keep_lead, u.nick.clone()));
                    }
                }
            }
            // also add channel names for messages starting with '#'
            for b in buffers_order {
                if b.starts_with(prefix) {
                    matches.push(b.clone());
                }
            }
        }
        matches.sort();
        matches.dedup();
        matches
    }

    /// Apply a completion to the current message input.
    ///
    /// Replaces the last word with the completion and adds appropriate suffix:
    /// - Commands get a space
    /// - Nicks at start of line get ": "
    /// - Other completions get a space
    pub fn apply_completion(
        &mut self,
        completion: &str,
        last_word_start: usize,
        _last_word_end: usize,
    ) {
        let is_first_token = self.message_input[..last_word_start].trim().is_empty();
        let is_command = completion.starts_with('/');
        let suffix = if is_command {
            " "
        } else if is_first_token {
            ": "
        } else {
            " "
        };
        let before = &self.message_input[..last_word_start];
        self.message_input = format!("{}{}{}", before, completion, suffix);
        // reset history navigation when using completions
        self.history_pos = None;
        self.history_saved_input = None;
    }

    /// Get the bounds of the last word in the message input.
    ///
    /// Returns (start_idx, end_idx) of the word being completed.
    pub fn current_last_word_bounds(&self) -> (usize, usize) {
        let idx = self
            .message_input
            .rfind(|c: char| c.is_whitespace())
            .map_or(0, |i| i + 1);
        (idx, self.message_input.len())
    }

    /// Cycle through tab completion candidates.
    ///
    /// Returns true if a completion was applied, false otherwise.
    pub fn cycle_completion(&mut self, direction: isize) -> bool {
        if self.completions.is_empty() {
            return false;
        }
        if let Some(idx) = self.completion_index {
            let len = self.completions.len();
            let next_idx = ((idx as isize + direction).rem_euclid(len as isize)) as usize;
            self.completion_index = Some(next_idx);
            let comp = self.completions[next_idx].clone();
            let (start, end) = self.current_last_word_bounds();
            self.apply_completion(&comp, start, end);
            true
        } else {
            // start cycling
            self.completion_index = Some(0);
            if let Some(comp) = self.completions.first() {
                let comp = comp.clone();
                let (start, end) = self.current_last_word_bounds();
                self.apply_completion(&comp, start, end);
                true
            } else {
                false
            }
        }
    }

    /// Navigate up in command history.
    #[allow(dead_code)]
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        if self.history_pos.is_none() {
            // Store current text to restore if user navigates back
            self.history_saved_input = Some(self.message_input.clone());
            self.history_pos = Some(self.history.len() - 1);
        } else if let Some(pos) = self.history_pos {
            if pos > 0 {
                self.history_pos = Some(pos - 1);
            }
        }

        if let Some(pos) = self.history_pos {
            if let Some(h) = self.history.get(pos) {
                self.message_input = h.clone();
            }
        }
    }

    /// Navigate down in command history.
    #[allow(dead_code)]
    pub fn history_down(&mut self) {
        if let Some(pos) = self.history_pos {
            if pos + 1 < self.history.len() {
                self.history_pos = Some(pos + 1);
                if let Some(h) = self.history.get(pos + 1) {
                    self.message_input = h.clone();
                }
            } else {
                // Exit history navigation
                self.history_pos = None;
                self.message_input = self.history_saved_input.take().unwrap_or_default();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_new() {
        let input = InputState::new();
        assert!(input.message_input.is_empty());
        assert!(input.history.is_empty());
        assert!(input.history_pos.is_none());
    }

    #[test]
    fn test_history_navigation() {
        let mut input = InputState::new();
        input.history = vec!["first".into(), "second".into(), "third".into()];
        input.message_input = "current".into();

        // Navigate up
        input.history_up();
        assert_eq!(input.message_input, "third");
        assert_eq!(input.history_saved_input, Some("current".into()));

        input.history_up();
        assert_eq!(input.message_input, "second");

        input.history_up();
        assert_eq!(input.message_input, "first");

        // Navigate down
        input.history_down();
        assert_eq!(input.message_input, "second");

        input.history_down();
        assert_eq!(input.message_input, "third");

        // Exit history mode
        input.history_down();
        assert_eq!(input.message_input, "current");
        assert!(input.history_pos.is_none());
    }

    #[test]
    fn test_word_bounds() {
        let mut input = InputState::new();
        input.message_input = "hello world test".into();

        let (start, end) = input.current_last_word_bounds();
        assert_eq!(start, 12);
        assert_eq!(end, 16);
        assert_eq!(&input.message_input[start..end], "test");
    }

    #[test]
    fn test_apply_completion_nick() {
        let mut input = InputState::new();
        input.message_input = "tes".into();

        input.apply_completion("testuser", 0, 3);
        assert_eq!(input.message_input, "testuser: ");
    }

    #[test]
    fn test_apply_completion_command() {
        let mut input = InputState::new();
        input.message_input = "/jo".into();

        input.apply_completion("/join", 0, 3);
        assert_eq!(input.message_input, "/join ");
    }
}
