//! Traditional horizontal menu bar (Discord/Slack-inspired with IRC-specific menus)
//! File, Edit, View, Server, Window, Help

use eframe::egui;
use crate::protocol::BackendAction;

/// Render the traditional horizontal menu bar
pub fn render_menu_bar(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    is_connected: bool,
    active_buffer: &str,
    show_user_list: &mut bool,
    show_help_dialog: &mut bool,
    network_manager_open: &mut bool,
    show_channel_browser: &mut bool,
    channel_list_loading: &mut bool,
    action_tx: &crossbeam_channel::Sender<BackendAction>,
) {
    egui::menu::bar(ui, |ui| {
        // File Menu
        ui.menu_button("File", |ui| {
            if ui.add_enabled(!is_connected, egui::Button::new("Connect..."))
                .on_hover_text("Connect to IRC server")
                .clicked()
            {
                *network_manager_open = true;
                ui.close_menu();
            }
            add_shortcut_text(ui, "Ctrl+N");

            if ui.add_enabled(is_connected, egui::Button::new("Disconnect"))
                .on_hover_text("Disconnect from current server")
                .clicked()
            {
                let _ = action_tx.send(BackendAction::Quit(Some("User disconnected".to_string())));
                ui.close_menu();
            }
            add_shortcut_text(ui, "Ctrl+D");

            ui.separator();

            if ui.button("Network Manager...")
                .on_hover_text("Manage saved IRC networks")
                .clicked()
            {
                *network_manager_open = true;
                ui.close_menu();
            }
            add_shortcut_text(ui, "Ctrl+,");

            ui.separator();

            if ui.button("Quit")
                .on_hover_text("Exit slirc")
                .clicked()
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            add_shortcut_text(ui, "Ctrl+Q");
        });

        // Edit Menu
        ui.menu_button("Edit", |ui| {
            // Copy is handled by egui internally
            ui.add_enabled_ui(false, |ui| {
                ui.button("Copy");
                add_shortcut_text(ui, "Ctrl+C");
            });

            ui.separator();

            ui.add_enabled_ui(false, |ui| {
                ui.button("Find in Chat...");
                add_shortcut_text(ui, "Ctrl+F");
            });

            ui.separator();

            if ui.button("Preferences...")
                .on_hover_text("Open settings dialog")
                .clicked()
            {
                // TODO: Implement preferences dialog
                ui.close_menu();
            }
        });

        // View Menu
        ui.menu_button("View", |ui| {
            // Quick switcher (future feature)
            ui.add_enabled_ui(false, |ui| {
                ui.button("Quick Switcher");
                add_shortcut_text(ui, "Ctrl+K");
            });

            ui.separator();

            if ui.checkbox(show_user_list, "Show User List").changed() {
                // Checkbox auto-updates the value
            }
            add_shortcut_text(ui, "Ctrl+U");

            ui.separator();

            ui.add_enabled_ui(false, |ui| {
                ui.button("Zoom In");
                add_shortcut_text(ui, "Ctrl++");
            });

            ui.add_enabled_ui(false, |ui| {
                ui.button("Zoom Out");
                add_shortcut_text(ui, "Ctrl+-");
            });

            ui.add_enabled_ui(false, |ui| {
                ui.button("Reset Zoom");
                add_shortcut_text(ui, "Ctrl+0");
            });
        });

        // Server Menu (IRC-specific)
        ui.menu_button("Server", |ui| {
            let in_channel = active_buffer != "System" && active_buffer.starts_with('#');

            ui.add_enabled_ui(is_connected, |ui| {
                if ui.button("Join Channel...")
                    .on_hover_text("Join an IRC channel")
                    .clicked()
                {
                    // Trigger join dialog in toolbar
                    ui.close_menu();
                }
                add_shortcut_text(ui, "Ctrl+J");
            });

            if ui.add_enabled(is_connected && in_channel, egui::Button::new("Part Channel"))
                .on_hover_text("Leave current channel")
                .clicked()
            {
                let _ = action_tx.send(BackendAction::Part {
                    channel: active_buffer.to_string(),
                    message: None,
                });
                ui.close_menu();
            }
            add_shortcut_text(ui, "Ctrl+W");

            ui.separator();

            ui.add_enabled_ui(is_connected, |ui| {
                if ui.button("List Channels...")
                    .on_hover_text("List all channels on server")
                    .clicked()
                {
                    let _ = action_tx.send(BackendAction::List);
                    *channel_list_loading = true;
                    ui.close_menu();
                }
                add_shortcut_text(ui, "Ctrl+L");
            });

            ui.add_enabled_ui(is_connected, |ui| {
                if ui.button("Search Users...").clicked() {
                    // TODO: Implement user search
                    ui.close_menu();
                }
            });

            ui.separator();

            ui.add_enabled_ui(is_connected, |ui| {
                if ui.button("Server Info").clicked() {
                    // TODO: Show server info
                    ui.close_menu();
                }
            });

            if ui.add_enabled(is_connected, egui::Button::new("Reconnect"))
                .on_hover_text("Reconnect to server")
                .clicked()
            {
                // TODO: Implement reconnect
                ui.close_menu();
            }
            add_shortcut_text(ui, "Ctrl+R");
        });

        // Help Menu
        ui.menu_button("Help", |ui| {
            if ui.button("Keyboard Shortcuts")
                .on_hover_text("Show keyboard shortcuts")
                .clicked()
            {
                *show_help_dialog = true;
                ui.close_menu();
            }
            add_shortcut_text(ui, "Ctrl+/");

            if ui.button("IRC Commands...")
                .on_hover_text("List available IRC commands")
                .clicked()
            {
                *show_help_dialog = true;
                ui.close_menu();
            }
            add_shortcut_text(ui, "F1");

            ui.separator();

            ui.add_enabled_ui(false, |ui| {
                ui.button("Check for Updates");
            });

            ui.add_enabled_ui(false, |ui| {
                ui.button("Report Issue...");
            });

            ui.separator();

            if ui.button("About slirc").clicked() {
                // TODO: Show about dialog
                ui.close_menu();
            }
        });
    });
}

/// Helper to add right-aligned shortcut text in menu items
fn add_shortcut_text(ui: &mut egui::Ui, shortcut: &str) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(
            egui::RichText::new(shortcut)
                .size(11.0)
                .color(ui.style().visuals.weak_text_color()),
        );
    });
}
