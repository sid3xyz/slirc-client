//! Help dialog (F1) - shows keyboard shortcuts and commands.

use eframe::egui;

/// Self-contained help dialog state.
#[derive(Default)]
pub struct HelpDialog {
    /// Whether the dialog is visible
    pub open: bool,
}

impl HelpDialog {
    /// Create a new help dialog (closed by default)
    pub fn new() -> Self {
        Self { open: false }
    }

    /// Toggle the dialog visibility
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Show the dialog
    pub fn show(&mut self) {
        self.open = true;
    }

    /// Hide the dialog
    #[allow(dead_code)]
    pub fn hide(&mut self) {
        self.open = false;
    }

    /// Render the help dialog.
    /// Returns true if the dialog is still open, false if it was closed.
    pub fn render(&mut self, ctx: &egui::Context) -> bool {
        if !self.open {
            return false;
        }

        let mut still_open = true;
        egui::Window::new("Help")
            .open(&mut still_open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Shortcuts & Commands");
                ui.separator();
                
                ui.label("Keyboard Shortcuts:");
                ui.label("  • F1: Toggle this help dialog");
                ui.label("  • Ctrl+N: Next channel");
                ui.label("  • Ctrl+P: Previous channel");
                ui.label("  • Ctrl+K: Quick switcher");
                ui.label("  • Enter: Send message (Shift+Enter for newline)");
                ui.label("  • Tab: Auto-complete nicks/channels/commands");
                ui.label("  • ↑/↓: Navigate input history");
                ui.label("  • Esc: Clear input");
                
                ui.separator();
                
                ui.label("Slash commands:");
                ui.label("  /join <#channel> - join a channel");
                ui.label("  /part [#channel] - leave a channel");
                ui.label("  /nick <nick> - change nick");
                ui.label("  /msg <nick> <text> - send private message");
                ui.label("  /me <text> - send CTCP ACTION");
                ui.label("  /whois <nick> - request WHOIS");
                ui.label("  /topic [text] - view/set channel topic");
                ui.label("  /kick <nick> [reason] - kick user (ops only)");
                ui.label("  /list - request channel list");
                ui.label("  /quit [reason] - disconnect from server");
            });

        if !still_open {
            self.open = false;
        }

        self.open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_dialog_toggle() {
        let mut dialog = HelpDialog::new();
        assert!(!dialog.open);
        
        dialog.toggle();
        assert!(dialog.open);
        
        dialog.toggle();
        assert!(!dialog.open);
    }

    #[test]
    fn test_help_dialog_show_hide() {
        let mut dialog = HelpDialog::new();
        
        dialog.show();
        assert!(dialog.open);
        
        dialog.hide();
        assert!(!dialog.open);
    }
}
