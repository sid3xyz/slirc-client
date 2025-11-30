//! Topic editor dialog - edit channel topics.

use eframe::egui;

use super::DialogAction;

/// Self-contained topic editor dialog state.
pub struct TopicEditorDialog {
    /// The channel whose topic we're editing
    pub channel: String,
    /// The current/edited topic text
    pub topic_input: String,
    /// The original topic (for comparison)
    original_topic: String,
}

impl TopicEditorDialog {
    /// Create a new topic editor dialog for a channel
    pub fn new(channel: &str, current_topic: &str) -> Self {
        Self {
            channel: channel.to_string(),
            topic_input: current_topic.to_string(),
            original_topic: current_topic.to_string(),
        }
    }

    /// Render the topic editor dialog.
    /// Returns `Some(DialogAction::SetTopic)` if the user saved a new topic.
    /// Returns `None` if still editing or cancelled.
    /// 
    /// The second return value indicates if the dialog is still open.
    pub fn render(&mut self, ctx: &egui::Context) -> (Option<DialogAction>, bool) {
        let mut action: Option<DialogAction> = None;
        // Use separate bools to avoid borrow conflict with .open()
        let mut window_open = true;
        let mut should_close = false;

        egui::Window::new(format!("Edit Topic: {}", self.channel))
            .open(&mut window_open)
            .resizable(true)
            .default_width(450.0)
            .show(ctx, |ui| {
                ui.label("Edit the channel topic:");
                ui.add_space(4.0);
                
                ui.add(
                    egui::TextEdit::multiline(&mut self.topic_input)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY)
                );
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    // Only enable save if topic changed
                    let changed = self.topic_input != self.original_topic;
                    
                    if ui.add_enabled(changed, egui::Button::new("Save")).clicked() {
                        action = Some(DialogAction::SetTopic {
                            channel: self.channel.clone(),
                            topic: self.topic_input.clone(),
                        });
                        should_close = true;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                    
                    if changed {
                        ui.label(
                            egui::RichText::new("(modified)")
                                .small()
                                .color(egui::Color32::YELLOW)
                        );
                    }
                });

                // Close on Escape
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    should_close = true;
                }
            });

        let still_open = window_open && !should_close;
        (action, still_open)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_editor_creation() {
        let dialog = TopicEditorDialog::new("#rust", "Welcome to #rust!");
        assert_eq!(dialog.channel, "#rust");
        assert_eq!(dialog.topic_input, "Welcome to #rust!");
        assert_eq!(dialog.original_topic, "Welcome to #rust!");
    }

    #[test]
    fn test_topic_editor_channel() {
        let dialog = TopicEditorDialog::new("#test", "Test topic");
        assert_eq!(dialog.channel, "#test");
    }
}
