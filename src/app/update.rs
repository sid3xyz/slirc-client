//! Main update loop and global shortcuts

use eframe::egui;
use std::time::Duration;

use super::SlircApp;

impl eframe::App for SlircApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process network events
        self.process_events();

        // Global keyboard shortcuts (work even when input doesn't have focus)
        ctx.input(|i| {
            // Ctrl+N: Next channel
            if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
                self.state.next_buffer();
            }
            // Ctrl+K: Quick switcher (search overlay)
            if i.modifiers.ctrl && i.key_pressed(egui::Key::K) {
                self.quick_switcher.toggle();
            }
            // Ctrl+P: Previous channel
            if i.modifiers.ctrl && i.key_pressed(egui::Key::P) {
                self.state.prev_buffer();
            }
            // F1: Toggle help dialog
            if i.key_pressed(egui::Key::F1) {
                self.dialogs.toggle_help();
            }
            // Ctrl+/: Toggle shortcuts help overlay
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Slash) {
                self.show_shortcuts_help = !self.show_shortcuts_help;
            }
            // Ctrl+M: Minimize window
            if i.modifiers.ctrl && i.key_pressed(egui::Key::M) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
            // Ctrl+Shift+F: Toggle fullscreen
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::F) {
                let current_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!current_fullscreen));
            }
            // Ctrl+B: Toggle channel list
            if i.modifiers.ctrl && i.key_pressed(egui::Key::B) {
                self.show_channel_list = !self.show_channel_list;
            }
        });

        // Request repaint to keep checking for events
        ctx.request_repaint_after(Duration::from_millis(100));
        // Purge old status messages (toasts) older than 4 seconds
        self.state.purge_old_status_messages(4);

        // Render UI sections
        self.render_menu_bar(ctx);
        self.render_toolbar(ctx);

        // Left panel: Buffer list (vertical tabs similar to HexChat)
        if self.show_channel_list {
            use crate::ui;
            ui::panels::render_channel_list(
                ctx,
                &self.state.buffers,
                &self.state.buffers_order,
                &mut self.state.active_buffer,
                &mut self.context_menu_visible,
                &mut self.context_menu_target,
                &mut self.state.collapsed_sections,
                &mut self.state.channel_filter,
            );
            // Clear unread after switching buffer
            if let Some(buf) = self.state.buffers.get_mut(&self.state.active_buffer) {
                buf.clear_unread();
            }
        }
        // (Removed top horizontal buffer tabs — left navigation is the single source of truth.)

        // Right panel: User list (for channels)
        if self.show_user_list
            && (self.state.active_buffer.starts_with('#') || self.state.active_buffer.starts_with('&'))
        {
            if let Some(buffer) = self.state.buffers.get(&self.state.active_buffer) {
                use crate::ui;
                ui::panels::render_user_list(
                    ctx,
                    buffer,
                    &self.state.active_buffer,
                    &self.connection.nickname,
                    &mut self.context_menu_visible,
                    &mut self.context_menu_target,
                );
            }
        }

        // Bottom panel: Message input with polished styling
        let _enter_pressed = self.render_input_panel(ctx);

        // Central panel: Messages with dedicated topic bar and styled background
        self.render_central_panel(ctx);

        // Context menu popup (as a floating window)
        self.render_context_menu(ctx);

        // Floating buffer windows
        self.render_floating_windows(ctx);

        // Render dialogs using the new self-contained dialog pattern
        self.render_dialogs(ctx);

        // Quick switcher overlay (Ctrl+K)
        if let Some(selected_buffer) = self.quick_switcher.render(ctx, &self.state.buffers) {
            self.state.active_buffer = selected_buffer.clone();
            if let Some(buffer) = self.state.buffers.get_mut(&selected_buffer) {
                buffer.clear_unread();
                buffer.has_highlight = false;
            }
        }

        // Shortcuts help overlay (Ctrl+/ or F1)
        self.shortcuts.render_help_overlay(ctx, &mut self.show_shortcuts_help);
    }
}
