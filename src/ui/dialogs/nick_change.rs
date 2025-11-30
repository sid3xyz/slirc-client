//! Nick change dialog - allows changing the user's nickname.

use eframe::egui;

use super::DialogAction;

/// Self-contained nick change dialog state.
pub struct NickChangeDialog {
    /// Whether the dialog is visible
    pub open: bool,
    /// The new nickname being entered
    pub nick_input: String,
    /// The current nickname (for display/comparison)
    current_nick: String,
}

impl NickChangeDialog {
    /// Create a new nick change dialog initialized with the current nick
    pub fn new(current_nick: &str) -> Self {
        Self {
            open: true,
            nick_input: current_nick.to_string(),
            current_nick: current_nick.to_string(),
        }
    }

    /// Render the nick change dialog.
    /// Returns `Some(DialogAction::ChangeNick)` if the user confirmed a nick change.
    pub fn render(&mut self, ctx: &egui::Context) -> Option<DialogAction> {
        if !self.open {
            return None;
        }

        let mut action: Option<DialogAction> = None;
        let mut still_open = true;

        egui::Window::new("Change Nick")
            .open(&mut still_open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("New nickname:");
                
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.nick_input)
                        .desired_width(200.0)
                );
                
                // Request focus on the input when dialog opens
                response.request_focus();

                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    let can_save = !self.nick_input.trim().is_empty() 
                        && self.nick_input.trim() != self.current_nick;
                    
                    if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                        action = Some(DialogAction::ChangeNick(self.nick_input.trim().to_string()));
                        self.open = false;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.open = false;
                    }
                });

                // Also submit on Enter key
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let can_save = !self.nick_input.trim().is_empty() 
                        && self.nick_input.trim() != self.current_nick;
                    if can_save {
                        action = Some(DialogAction::ChangeNick(self.nick_input.trim().to_string()));
                        self.open = false;
                    }
                }

                // Close on Escape
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.open = false;
                }
            });

        if !still_open {
            self.open = false;
        }

        action
    }

    /// Check if the dialog is open
    pub fn is_open(&self) -> bool {
        self.open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nick_change_dialog_creation() {
        let dialog = NickChangeDialog::new("testuser");
        assert!(dialog.open);
        assert_eq!(dialog.nick_input, "testuser");
        assert_eq!(dialog.current_nick, "testuser");
    }

    #[test]
    fn test_nick_change_dialog_is_open() {
        let mut dialog = NickChangeDialog::new("testuser");
        assert!(dialog.is_open());
        
        dialog.open = false;
        assert!(!dialog.is_open());
    }
}
