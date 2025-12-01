//! Keyboard shortcut registry and help overlay.
//!
//! Centralizes all keyboard shortcuts used across the application,
//! providing a single source of truth for shortcut definitions and
//! a help overlay dialog (Ctrl+/) to display them to users.

use eframe::egui;

/// Category of shortcuts for organization in help overlay
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutCategory {
    File,
    Edit,
    View,
    Server,
    Window,
    Navigation,
}

impl ShortcutCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Server => "Server",
            Self::Window => "Window",
            Self::Navigation => "Navigation",
        }
    }
}

/// A keyboard shortcut definition
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub category: ShortcutCategory,
    pub key_text: &'static str,
    pub description: &'static str,
    #[allow(dead_code)]
    pub action_id: &'static str,
}

/// Global keyboard shortcut registry
#[allow(dead_code)] // Will be used when integrated with app.rs
pub struct ShortcutRegistry {
    shortcuts: Vec<Shortcut>,
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcutRegistry {
    /// Create a new registry with all application shortcuts
    pub fn new() -> Self {
        let shortcuts = vec![
            // File menu
            Shortcut {
                category: ShortcutCategory::File,
                key_text: "Ctrl+N",
                description: "Connect to server",
                action_id: "file.connect",
            },
            Shortcut {
                category: ShortcutCategory::File,
                key_text: "Ctrl+D",
                description: "Disconnect from server",
                action_id: "file.disconnect",
            },
            Shortcut {
                category: ShortcutCategory::File,
                key_text: "Ctrl+,",
                description: "Network Manager",
                action_id: "file.network_manager",
            },
            Shortcut {
                category: ShortcutCategory::File,
                key_text: "Ctrl+Q",
                description: "Quit application",
                action_id: "file.quit",
            },
            // Edit menu
            Shortcut {
                category: ShortcutCategory::Edit,
                key_text: "Ctrl+X",
                description: "Cut",
                action_id: "edit.cut",
            },
            Shortcut {
                category: ShortcutCategory::Edit,
                key_text: "Ctrl+C",
                description: "Copy",
                action_id: "edit.copy",
            },
            Shortcut {
                category: ShortcutCategory::Edit,
                key_text: "Ctrl+V",
                description: "Paste",
                action_id: "edit.paste",
            },
            Shortcut {
                category: ShortcutCategory::Edit,
                key_text: "Ctrl+A",
                description: "Select All",
                action_id: "edit.select_all",
            },
            // View menu
            Shortcut {
                category: ShortcutCategory::View,
                key_text: "Ctrl+K",
                description: "Quick Switcher",
                action_id: "view.quick_switcher",
            },
            Shortcut {
                category: ShortcutCategory::View,
                key_text: "Ctrl+U",
                description: "Toggle User List",
                action_id: "view.toggle_user_list",
            },
            Shortcut {
                category: ShortcutCategory::View,
                key_text: "Ctrl+B",
                description: "Toggle Channel List",
                action_id: "view.toggle_channel_list",
            },
            // Server menu
            Shortcut {
                category: ShortcutCategory::Server,
                key_text: "Ctrl+J",
                description: "Join Channel",
                action_id: "server.join_channel",
            },
            Shortcut {
                category: ShortcutCategory::Server,
                key_text: "Ctrl+W",
                description: "Part Channel",
                action_id: "server.part_channel",
            },
            Shortcut {
                category: ShortcutCategory::Server,
                key_text: "Ctrl+L",
                description: "List Channels",
                action_id: "server.list_channels",
            },
            // Window menu
            Shortcut {
                category: ShortcutCategory::Window,
                key_text: "Ctrl+M",
                description: "Minimize",
                action_id: "window.minimize",
            },
            Shortcut {
                category: ShortcutCategory::Window,
                key_text: "Ctrl+Shift+F",
                description: "Toggle Fullscreen",
                action_id: "window.fullscreen",
            },
            // Navigation
            Shortcut {
                category: ShortcutCategory::Navigation,
                key_text: "Ctrl+P",
                description: "Previous Channel",
                action_id: "nav.prev_channel",
            },
            Shortcut {
                category: ShortcutCategory::Navigation,
                key_text: "Ctrl+N",
                description: "Next Channel",
                action_id: "nav.next_channel",
            },
            Shortcut {
                category: ShortcutCategory::Navigation,
                key_text: "F1",
                description: "Help / Keyboard Shortcuts",
                action_id: "nav.help",
            },
            Shortcut {
                category: ShortcutCategory::Navigation,
                key_text: "Ctrl+/",
                description: "Show Keyboard Shortcuts",
                action_id: "nav.shortcuts",
            },
        ];

        Self { shortcuts }
    }

    /// Get all shortcuts for a specific category
    pub fn by_category(&self, category: ShortcutCategory) -> Vec<&Shortcut> {
        self.shortcuts
            .iter()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Get all shortcuts
    #[allow(dead_code)]
    pub fn all(&self) -> &[Shortcut] {
        &self.shortcuts
    }

    /// Find a shortcut by action ID
    #[allow(dead_code)]
    pub fn find(&self, action_id: &str) -> Option<&Shortcut> {
        self.shortcuts.iter().find(|s| s.action_id == action_id)
    }

    /// Render the keyboard shortcuts help overlay (Ctrl+/ or F1)
    pub fn render_help_overlay(&self, ctx: &egui::Context, open: &mut bool) {
        let mut should_close = false;

        egui::Window::new("⌨ Keyboard Shortcuts")
            .open(open)
            .collapsible(false)
            .resizable(false)
            .default_width(500.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Keyboard Shortcuts");
                ui.add_space(8.0);

                // Render shortcuts grouped by category
                for category in [
                    ShortcutCategory::File,
                    ShortcutCategory::Edit,
                    ShortcutCategory::View,
                    ShortcutCategory::Server,
                    ShortcutCategory::Window,
                    ShortcutCategory::Navigation,
                ] {
                    let category_shortcuts = self.by_category(category);
                    if category_shortcuts.is_empty() {
                        continue;
                    }

                    ui.label(
                        egui::RichText::new(category.name())
                            .strong()
                            .color(egui::Color32::from_rgb(88, 101, 242)),
                    );
                    ui.add_space(4.0);

                    // Table of shortcuts
                    egui::Grid::new(format!("shortcuts_{:?}", category))
                        .num_columns(2)
                        .spacing([20.0, 6.0])
                        .show(ui, |ui| {
                            for shortcut in category_shortcuts {
                                // Shortcut key (monospace, highlighted)
                                ui.label(
                                    egui::RichText::new(shortcut.key_text)
                                        .monospace()
                                        .color(egui::Color32::from_rgb(200, 200, 200))
                                        .background_color(egui::Color32::from_rgb(55, 60, 70)),
                                );

                                // Description
                                ui.label(shortcut.description);
                                ui.end_row();
                            }
                        });

                    ui.add_space(12.0);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Press Esc or F1 to close").weak());
                    });
                });
            });

        if should_close {
            *open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_initialization() {
        let registry = ShortcutRegistry::new();
        assert!(!registry.all().is_empty(), "Registry should have shortcuts");
    }

    #[test]
    fn test_find_shortcut() {
        let registry = ShortcutRegistry::new();
        let shortcut = registry.find("file.connect");
        assert!(shortcut.is_some(), "Should find file.connect shortcut");
        assert_eq!(shortcut.unwrap().key_text, "Ctrl+N");
    }

    #[test]
    fn test_by_category() {
        let registry = ShortcutRegistry::new();
        let file_shortcuts = registry.by_category(ShortcutCategory::File);
        assert!(!file_shortcuts.is_empty(), "Should have File shortcuts");

        let nav_shortcuts = registry.by_category(ShortcutCategory::Navigation);
        assert!(!nav_shortcuts.is_empty(), "Should have Navigation shortcuts");
    }

    #[test]
    fn test_all_categories_represented() {
        let registry = ShortcutRegistry::new();
        for category in [
            ShortcutCategory::File,
            ShortcutCategory::Edit,
            ShortcutCategory::View,
            ShortcutCategory::Server,
            ShortcutCategory::Window,
            ShortcutCategory::Navigation,
        ] {
            let shortcuts = registry.by_category(category);
            assert!(
                !shortcuts.is_empty(),
                "Category {:?} should have shortcuts",
                category
            );
        }
    }
}
