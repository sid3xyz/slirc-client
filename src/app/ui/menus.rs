//! Context menus and floating windows

use eframe::egui;
use slirc_proto::ctcp::Ctcp;

use crate::app::SlircApp;
use crate::buffer::ChannelBuffer;
use crate::protocol::BackendAction;
use crate::ui;

impl SlircApp {
    /// Render context menu popup (as a floating window)
    pub(in crate::app) fn render_context_menu(&mut self, ctx: &egui::Context) {
        if self.context_menu_visible {
            if let Some(target) = self.context_menu_target.clone() {
                // If the target starts with "user:", this is a user context menu
                if let Some(user) = target.strip_prefix("user:") {
                    egui::Window::new(format!("User: {}", user))
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            if ui.button("Query (PM)").clicked() {
                                // Create or switch to private message buffer
                                if !self.state.buffers.contains_key(user) {
                                    self.state.buffers.insert(user.to_string(), ChannelBuffer::new());
                                    self.state.buffers_order.push(user.to_string());
                                }
                                self.state.active_buffer = user.to_string();
                                self.context_menu_visible = false;
                            }
                            if ui.button("Whois").clicked() {
                                let _ = self.action_tx.send(BackendAction::Whois(user.to_string()));
                                self.context_menu_visible = false;
                            }
                            if ui.button("Cancel").clicked() {
                                self.context_menu_visible = false;
                            }
                            // Show op actions if we're an op in this channel
                            if self.state.active_buffer.starts_with('#')
                                || self.state.active_buffer.starts_with('&')
                            {
                                let is_op = self
                                    .state
                                    .buffers
                                    .get(&self.state.active_buffer)
                                    .map(|b| {
                                        b.users.iter().any(|u| {
                                            u.nick == self.connection.nickname
                                                && ui::theme::prefix_rank(u.prefix) >= 3
                                        })
                                    })
                                    .unwrap_or(false);
                                if is_op {
                                    ui.separator();
                                    ui.label("Op Actions:");
                                    if ui.button("Op (+o)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.state.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "+o".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Deop (-o)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.state.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "-o".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Voice (+v)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.state.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "+v".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Devoice (-v)").clicked() {
                                        let _ = self.action_tx.send(BackendAction::SetUserMode {
                                            channel: self.state.active_buffer.clone(),
                                            nick: user.to_string(),
                                            mode: "-v".to_string(),
                                        });
                                        self.context_menu_visible = false;
                                    }
                                    if ui.button("Kick").clicked() {
                                        let _ = self.action_tx.send(BackendAction::Kick {
                                            channel: self.state.active_buffer.clone(),
                                            nick: user.to_string(),
                                            reason: None,
                                        });
                                        self.context_menu_visible = false;
                                    }
                                }
                            }
                        });
                } else {
                    egui::Window::new(format!("Actions: {}", target))
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            if ui.button("Part").clicked() {
                                let _ = self.action_tx.send(BackendAction::Part {
                                    channel: target.clone(),
                                    message: None,
                                });
                                self.context_menu_visible = false;
                            }
                            if ui.button("Close").clicked() {
                                self.state.buffers.remove(&target);
                                self.state.buffers_order.retain(|b| b != &target);
                                if self.state.active_buffer == target {
                                    self.state.active_buffer = "System".into();
                                }
                                self.context_menu_visible = false;
                            }
                            if ui.button("Open in new window").clicked() {
                                self.open_windows.insert(target.clone());
                                self.context_menu_visible = false;
                            }
                            if ui.button("Cancel").clicked() {
                                self.context_menu_visible = false;
                            }
                        });
                }
            }
        }
    }

    /// Render floating buffer windows
    pub(in crate::app) fn render_floating_windows(&mut self, ctx: &egui::Context) {
        for open_name in self.open_windows.clone() {
            let mut open = true;
            egui::Window::new(format!("Window: {}", open_name))
                .open(&mut open)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&open_name);
                    });
                    ui.separator();
                    if let Some(buffer) = self.state.buffers.get(&open_name) {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for msg in &buffer.messages {
                                // Check if this is a CTCP ACTION message using slirc_proto
                                if let Some(ctcp) = Ctcp::parse(&msg.text) {
                                    if let Some(action) = ctcp.params {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(format!("[{}]", msg.timestamp))
                                                    .color(egui::Color32::LIGHT_GRAY),
                                            );
                                            ui.label(
                                                egui::RichText::new("*")
                                                    .color(egui::Color32::from_rgb(255, 150, 0)),
                                            );
                                            ui.label(
                                                egui::RichText::new(&msg.sender)
                                                    .color(ui::theme::nick_color(&msg.sender)),
                                            );
                                            ui.label(
                                                egui::RichText::new(action)
                                                    .color(egui::Color32::from_rgb(255, 150, 0))
                                                    .italics(),
                                            );
                                        });
                                        continue;
                                    }
                                }
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("[{}]", msg.timestamp))
                                            .color(egui::Color32::LIGHT_GRAY),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("<{}>", msg.sender))
                                            .color(egui::Color32::LIGHT_BLUE)
                                            .strong(),
                                    );
                                    ui.label(&msg.text);
                                });
                            }
                        });
                    }
                });
            if !open {
                self.open_windows.remove(&open_name);
            }
        }
    }
}
