//! Traditional horizontal menu bar (Discord/Slack-inspired with IRC-specific menus)
//! File, Edit, View, Server, Window, Help

use eframe::egui;
use crate::protocol::BackendAction;

/// Actions that the menu can request
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    NetworkManager,
    Help,
    ChannelBrowser,
}

/// Render the traditional horizontal menu bar
/// Returns Some(MenuAction) if an action was requested
#[allow(clippy::too_many_arguments)]
pub fn render_menu_bar(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    is_connected: bool,
    active_buffer: &str,
    show_channel_list: &mut bool,
    show_user_list: &mut bool,
    quick_switcher: &mut crate::ui::quick_switcher::QuickSwitcher,
    action_tx: &crossbeam_channel::Sender<BackendAction>,
) -> Option<MenuAction> {
    let mut menu_action: Option<MenuAction> = None;
    
    egui::menu::bar(ui, |ui| {
        // File Menu
        ui.menu_button("File", |ui| {
            ui.horizontal(|ui| {
                if ui.add_enabled(!is_connected, egui::Button::new("Connect..."))
                    .on_hover_text("Connect to IRC server")
                    .clicked()
                {
                    menu_action = Some(MenuAction::NetworkManager);
                    ui.close_menu();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Ctrl+N").weak().small());
                });
            });

            ui.horizontal(|ui| {
                if ui.add_enabled(is_connected, egui::Button::new("Disconnect"))
                    .on_hover_text("Disconnect from current server")
                    .clicked()
                {
                    let _ = action_tx.send(BackendAction::Quit(Some("User disconnected".to_string())));
                    ui.close_menu();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Ctrl+D").weak().small());
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Network Manager...")
                    .on_hover_text("Manage saved IRC networks")
                    .clicked()
                {
                    menu_action = Some(MenuAction::NetworkManager);
                    ui.close_menu();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Ctrl+,").weak().small());
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Quit")
                    .on_hover_text("Exit slirc")
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Ctrl+Q").weak().small());
                });
            });
        });

        // Edit Menu - Removed as all items were disabled/TODO

        // View Menu
        ui.menu_button("View", |ui| {
            ui.horizontal(|ui| {
                if ui.button("Quick Switcher").clicked() {
                    quick_switcher.toggle();
                    ui.close_menu();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Ctrl+K").weak().small());
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.checkbox(show_channel_list, "Show Channel List");
            });

            ui.horizontal(|ui| {
                ui.checkbox(show_user_list, "Show User List");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Ctrl+U").weak().small());
                });
            });
        });

        // Server Menu (IRC-specific)
        ui.menu_button("Server", |ui| {
            let in_channel = active_buffer != "System" && active_buffer.starts_with('#');

            ui.add_enabled_ui(is_connected, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Join Channel...")
                        .on_hover_text("Join an IRC channel")
                        .clicked()
                    {
                        // Trigger join dialog in toolbar
                        ui.close_menu();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Ctrl+J").weak().small());
                    });
                });
            });

            ui.horizontal(|ui| {
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
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Ctrl+W").weak().small());
                });
            });

            ui.separator();

            ui.add_enabled_ui(is_connected, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("List Channels...")
                        .on_hover_text("List all channels on server")
                        .clicked()
                    {
                        let _ = action_tx.send(BackendAction::List);
                        menu_action = Some(MenuAction::ChannelBrowser);
                        ui.close_menu();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Ctrl+L").weak().small());
                    });
                });
            });
        });

        // Help Menu
        ui.menu_button("Help", |ui| {
            ui.horizontal(|ui| {
                if ui.button("Keyboard Shortcuts")
                    .on_hover_text("Show keyboard shortcuts")
                    .clicked()
                {
                    menu_action = Some(MenuAction::Help);
                    ui.close_menu();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Ctrl+/").weak().small());
                });
            });

            ui.horizontal(|ui| {
                if ui.button("IRC Commands...")
                    .on_hover_text("List available IRC commands")
                    .clicked()
                {
                    menu_action = Some(MenuAction::Help);
                    ui.close_menu();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("F1").weak().small());
                });
            });
        });
    });
    
    menu_action
}
