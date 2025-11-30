//! Channel browser dialog - browse and join channels from /list.

use eframe::egui;

use super::DialogAction;

/// Channel information from the server's LIST response
#[derive(Clone, Debug)]
pub struct ChannelListItem {
    pub channel: String,
    pub user_count: usize,
    pub topic: String,
}

/// Self-contained channel browser dialog state.
pub struct ChannelBrowserDialog {
    /// The list of channels from the server
    pub channels: Vec<ChannelListItem>,
    /// Filter text for searching channels
    pub filter: String,
    /// Whether we're still loading the channel list
    pub is_loading: bool,
}

impl ChannelBrowserDialog {
    /// Create a new channel browser dialog
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            filter: String::new(),
            is_loading: true,
        }
    }

    /// Add a channel to the list (called as LIST responses arrive)
    pub fn add_channel(&mut self, item: ChannelListItem) {
        self.channels.push(item);
    }

    /// Mark loading as complete
    pub fn set_loading_complete(&mut self) {
        self.is_loading = false;
    }

    /// Clear the channel list (for refresh)
    pub fn clear(&mut self) {
        self.channels.clear();
        self.filter.clear();
        self.is_loading = true;
    }

    /// Render the channel browser dialog.
    /// Returns `Some(DialogAction::JoinChannel)` if the user wants to join a channel.
    /// 
    /// The second return value indicates if the dialog is still open.
    pub fn render(&mut self, ctx: &egui::Context) -> (Option<DialogAction>, bool) {
        let mut action: Option<DialogAction> = None;
        let mut should_close = false;
        let mut window_open = true;

        egui::Window::new("Channel Browser")
            .open(&mut window_open)
            .resizable(true)
            .default_width(700.0)
            .default_height(500.0)
            .show(ctx, |ui| {
                ui.heading("Available Channels");
                ui.separator();

                // Filter input
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.filter)
                            .desired_width(200.0)
                    );
                    if ui.button("Clear").clicked() {
                        self.filter.clear();
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🔄 Refresh").clicked() {
                            // The app will need to send a new LIST command
                            // For now, just clear the list
                            self.clear();
                        }
                    });
                });
                ui.add_space(8.0);

                if self.is_loading {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading channel list...");
                    });
                } else if self.channels.is_empty() {
                    ui.label("No channels available. The server may not support /list.");
                } else {
                    // Filter channels
                    let filter_lower = self.filter.to_lowercase();
                    let filtered: Vec<&ChannelListItem> = if self.filter.is_empty() {
                        self.channels.iter().collect()
                    } else {
                        self.channels
                            .iter()
                            .filter(|item| {
                                item.channel.to_lowercase().contains(&filter_lower)
                                    || item.topic.to_lowercase().contains(&filter_lower)
                            })
                            .collect()
                    };

                    ui.label(format!(
                        "Showing {} of {} channels",
                        filtered.len(),
                        self.channels.len()
                    ));
                    ui.separator();

                    // Channel list table
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            egui::Grid::new("channel_list_grid")
                                .striped(true)
                                .spacing([10.0, 4.0])
                                .show(ui, |ui| {
                                    // Header
                                    ui.label(egui::RichText::new("Channel").strong());
                                    ui.label(egui::RichText::new("Users").strong());
                                    ui.label(egui::RichText::new("Topic").strong());
                                    ui.label("");
                                    ui.end_row();

                                    // Rows (limit to 200 for performance)
                                    for item in filtered.iter().take(200) {
                                        ui.label(&item.channel);
                                        ui.label(format!("{}", item.user_count));
                                        
                                        // Truncate long topics
                                        let topic_display = if item.topic.len() > 60 {
                                            format!("{}...", &item.topic[..60])
                                        } else {
                                            item.topic.clone()
                                        };
                                        ui.label(topic_display);
                                        
                                        if ui.button("Join").clicked() {
                                            action = Some(DialogAction::JoinChannel(
                                                item.channel.clone(),
                                            ));
                                            should_close = true;
                                        }
                                        ui.end_row();
                                    }
                                    
                                    if filtered.len() > 200 {
                                        ui.label("...");
                                        ui.label("");
                                        ui.label(format!(
                                            "({} more channels, use filter to narrow)",
                                            filtered.len() - 200
                                        ));
                                        ui.label("");
                                        ui.end_row();
                                    }
                                });
                        });
                }

                // Close on Escape
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    should_close = true;
                }
            });

        let still_open = window_open && !should_close;
        (action, still_open)
    }
}

impl Default for ChannelBrowserDialog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_browser_creation() {
        let dialog = ChannelBrowserDialog::new();
        assert!(dialog.channels.is_empty());
        assert!(dialog.filter.is_empty());
        assert!(dialog.is_loading);
    }

    #[test]
    fn test_channel_browser_with_channels() {
        let mut dialog = ChannelBrowserDialog::new();
        dialog.add_channel(ChannelListItem {
            channel: "#rust".to_string(),
            user_count: 100,
            topic: "Welcome to Rust!".to_string(),
        });
        dialog.set_loading_complete();
        assert_eq!(dialog.channels.len(), 1);
        assert!(!dialog.is_loading);
    }

    #[test]
    fn test_channel_browser_add_channel() {
        let mut dialog = ChannelBrowserDialog::new();
        dialog.add_channel(ChannelListItem {
            channel: "#test".to_string(),
            user_count: 50,
            topic: "Test channel".to_string(),
        });
        assert_eq!(dialog.channels.len(), 1);
        assert_eq!(dialog.channels[0].channel, "#test");
    }

    #[test]
    fn test_channel_browser_clear() {
        let mut dialog = ChannelBrowserDialog::new();
        dialog.add_channel(ChannelListItem {
            channel: "#test".to_string(),
            user_count: 10,
            topic: "Test".to_string(),
        });
        dialog.set_loading_complete();
        dialog.filter = "test".to_string();
        
        dialog.clear();
        
        assert!(dialog.channels.is_empty());
        assert!(dialog.filter.is_empty());
        assert!(dialog.is_loading);
    }
}
