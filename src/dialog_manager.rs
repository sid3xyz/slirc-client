//! Dialog management for centralized dialog state and rendering.
//!
//! This module consolidates all dialog state into a single DialogManager,
//! reducing clutter in the main SlircApp struct and providing a clean API
//! for opening, rendering, and handling dialog actions.

use eframe::egui::Context;

use crate::config::Network;
use crate::ui::dialogs::{
    ChannelBrowserDialog, ChannelListItem, DialogAction, HelpDialog,
    NetworkManagerDialog, NickChangeDialog, TopicEditorDialog,
};

/// Manages all application dialogs in one place.
///
/// Uses the Option<Dialog> pattern where None = closed, Some = open.
pub struct DialogManager {
    pub help_dialog: HelpDialog,
    pub nick_change_dialog: Option<NickChangeDialog>,
    pub topic_editor_dialog: Option<TopicEditorDialog>,
    pub network_manager_dialog: Option<NetworkManagerDialog>,
    pub channel_browser_dialog: Option<ChannelBrowserDialog>,
}

impl DialogManager {
    /// Create a new DialogManager with all dialogs closed except help.
    pub fn new() -> Self {
        Self {
            help_dialog: HelpDialog::new(),
            nick_change_dialog: None,
            topic_editor_dialog: None,
            network_manager_dialog: None,
            channel_browser_dialog: None,
        }
    }
    
    /// Open the nick change dialog with the given current nickname.
    pub fn open_nick_change(&mut self, current_nick: &str) {
        self.nick_change_dialog = Some(NickChangeDialog::new(current_nick));
    }
    
    /// Open the topic editor dialog for the given channel.
    pub fn open_topic_editor(&mut self, channel: &str, current_topic: &str) {
        self.topic_editor_dialog = Some(TopicEditorDialog::new(channel, current_topic));
    }
    
    /// Open the network manager dialog with the given networks.
    pub fn open_network_manager(&mut self, networks: Vec<Network>) {
        self.network_manager_dialog = Some(NetworkManagerDialog::new(networks));
    }
    
    /// Open the channel browser dialog.
    pub fn open_channel_browser(&mut self) {
        self.channel_browser_dialog = Some(ChannelBrowserDialog::new());
    }
    
    /// Toggle the help dialog.
    pub fn toggle_help(&mut self) {
        self.help_dialog.toggle();
    }
    
    /// Show the help dialog.
    pub fn show_help(&mut self) {
        self.help_dialog.show();
    }
    
    /// Add a channel to the channel browser dialog if it's open.
    pub fn add_channel_to_browser(&mut self, item: ChannelListItem) {
        if let Some(ref mut dialog) = self.channel_browser_dialog {
            dialog.add_channel(item);
        }
    }
    
    /// Mark channel browser loading as complete.
    pub fn channel_browser_complete(&mut self) {
        if let Some(ref mut dialog) = self.channel_browser_dialog {
            dialog.set_loading_complete();
        }
    }
    
    /// Render all dialogs and collect their actions.
    ///
    /// Returns a tuple of (actions, networks_to_save) where networks_to_save
    /// is Some if the network manager dialog was closed with modifications.
    pub fn render(&mut self, ctx: &Context) -> (Vec<DialogAction>, Option<Vec<Network>>) {
        let mut actions: Vec<DialogAction> = Vec::new();
        let mut networks_to_save: Option<Vec<Network>> = None;
        
        // Help dialog (F1) - simple toggle, no actions
        self.help_dialog.render(ctx);
        
        // Nick change dialog
        let mut close_nick_dialog = false;
        if let Some(ref mut dialog) = self.nick_change_dialog {
            if let Some(action) = dialog.render(ctx) {
                actions.push(action);
            }
            if !dialog.is_open() {
                close_nick_dialog = true;
            }
        }
        if close_nick_dialog {
            self.nick_change_dialog = None;
        }
        
        // Topic editor dialog
        let mut close_topic_dialog = false;
        if let Some(ref mut dialog) = self.topic_editor_dialog {
            let (action, still_open) = dialog.render(ctx);
            if let Some(action) = action {
                actions.push(action);
            }
            if !still_open {
                close_topic_dialog = true;
            }
        }
        if close_topic_dialog {
            self.topic_editor_dialog = None;
        }
        
        // Network manager dialog
        let mut close_network_dialog = false;
        if let Some(ref mut dialog) = self.network_manager_dialog {
            let (action, still_open) = dialog.render(ctx);
            if let Some(action) = action {
                actions.push(action);
            }
            if !still_open {
                if dialog.was_modified() {
                    networks_to_save = Some(dialog.get_networks().to_vec());
                }
                close_network_dialog = true;
            }
        }
        if close_network_dialog {
            self.network_manager_dialog = None;
        }
        
        // Channel browser dialog
        let mut close_channel_browser = false;
        if let Some(ref mut dialog) = self.channel_browser_dialog {
            let (action, still_open) = dialog.render(ctx);
            if let Some(action) = action {
                actions.push(action);
            }
            if !still_open {
                close_channel_browser = true;
            }
        }
        if close_channel_browser {
            self.channel_browser_dialog = None;
        }
        
        (actions, networks_to_save)
    }
}

impl Default for DialogManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dialog_manager_new() {
        let dm = DialogManager::new();
        assert!(dm.nick_change_dialog.is_none());
        assert!(dm.topic_editor_dialog.is_none());
        assert!(dm.network_manager_dialog.is_none());
        assert!(dm.channel_browser_dialog.is_none());
    }
    
    #[test]
    fn test_open_nick_change() {
        let mut dm = DialogManager::new();
        dm.open_nick_change("testuser");
        assert!(dm.nick_change_dialog.is_some());
    }
    
    #[test]
    fn test_open_topic_editor() {
        let mut dm = DialogManager::new();
        dm.open_topic_editor("#test", "Test topic");
        assert!(dm.topic_editor_dialog.is_some());
    }
    
    #[test]
    fn test_open_network_manager() {
        let mut dm = DialogManager::new();
        dm.open_network_manager(vec![]);
        assert!(dm.network_manager_dialog.is_some());
    }
    
    #[test]
    fn test_open_channel_browser() {
        let mut dm = DialogManager::new();
        dm.open_channel_browser();
        assert!(dm.channel_browser_dialog.is_some());
    }
}
