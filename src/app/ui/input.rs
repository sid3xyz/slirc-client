//! Message input panel with history and tab completion

use chrono::Local;
use eframe::egui;

use crate::app::SlircApp;
use crate::commands;
use crate::protocol::BackendAction;

impl SlircApp {
    /// Render the input panel at the bottom of the window
    /// Returns Some(true) if Enter was pressed and message sent (for focus control)
    pub(in crate::app) fn render_input_panel(&mut self, ctx: &egui::Context) -> Option<bool> {
        let dark_mode = ctx.style().visuals.dark_mode;
        let theme = self.get_theme();
        let input_bg = theme.surface[1];

        let mut enter_pressed = false;

        egui::TopBottomPanel::bottom("input_panel")
            .frame(
                egui::Frame::new()
                    .fill(input_bg)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .stroke(egui::Stroke::new(
                        1.0,
                        theme.border_medium,
                    )),
            )
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Styled input frame with rounding and focus indication
                let input_frame = egui::Frame::new()
                    .fill(if dark_mode {
                        egui::Color32::from_rgb(45, 45, 52)
                    } else {
                        egui::Color32::WHITE
                    })
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(10, 8));

                input_frame.show(ui, |ui| {
                let response = ui.add(
                    egui::TextEdit::multiline(&mut self.input.message_input)
                        .desired_rows(1)
                        .desired_width(ui.available_width() - 4.0)
                        .frame(false)
                        .hint_text("Type a message... (Enter to send)"),
                );

                // Draw focus ring (two rects: outer border, inner transparent)
                if response.has_focus() {
                    let outer = response.rect.expand(2.0);
                    ui.painter().rect_filled(outer, 8.0, theme.accent.linear_multiply(0.3));
                }

                // Detect Enter (without Shift) to send a message. Shift+Enter inserts newline in the
                // multiline text edit by default.
                let enter_detected = response.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);

                // Input history navigation
                if response.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::ArrowUp))
                    && !self.input.history.is_empty()
                {
                    if self.input.history_pos.is_none() {
                        // store current text to restore if user navigates back
                        self.input.history_saved_input = Some(self.input.message_input.clone());
                        self.input.history_pos = Some(self.input.history.len() - 1);
                    } else if let Some(pos) = self.input.history_pos {
                        if pos > 0 {
                            self.input.history_pos = Some(pos - 1);
                        }
                    }
                    if let Some(pos) = self.input.history_pos {
                        if let Some(h) = self.input.history.get(pos) {
                            self.input.message_input = h.clone();
                        }
                    }
                }
                if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    if let Some(pos) = self.input.history_pos {
                        if pos + 1 < self.input.history.len() {
                            self.input.history_pos = Some(pos + 1);
                            if let Some(h) = self.input.history.get(pos + 1) {
                                self.input.message_input = h.clone();
                            }
                        } else {
                            // Exit history navigation
                            self.input.history_pos = None;
                            self.input.message_input =
                                self.input.history_saved_input.take().unwrap_or_default();
                        }
                    }
                }

                // Tab completion: Tab cycles forward; Shift+Tab cycles backward
                let tab_pressed =
                    response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Tab));
                let shift = ui.input(|i| i.modifiers.shift);
                if tab_pressed {
                    // compute current prefix (last token)
                    let (start, end) = self.input.current_last_word_bounds();
                    let prefix = self.input.message_input[start..end].trim();
                    if self.input.completions.is_empty() {
                        // first time: gather completions
                        self.input.completions = self.input.collect_completions(
                            prefix,
                            &self.state.buffers_order,
                            &self.state.active_buffer,
                            &self.state.buffers
                        );
                        self.input.completion_prefix = Some(prefix.to_string());
                        self.input.completion_target_channel =
                            prefix.starts_with('#') || prefix.starts_with('&');
                    }
                    if !self.input.completions.is_empty() {
                        if shift {
                            self.input.cycle_completion(-1);
                        } else {
                            self.input.cycle_completion(1);
                        }
                    }
                }

                // Reset completions if the user changed the input text
                if self.input.last_input_text != self.input.message_input && !tab_pressed {
                    self.input.completions.clear();
                    self.input.completion_index = None;
                    self.input.completion_prefix = None;
                }
                self.input.last_input_text = self.input.message_input.clone();

                // Esc to cancel input (clear the text field)
                if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.input.message_input.clear();
                    self.input.history_pos = None;
                    self.input.history_saved_input = None;
                    self.input.completions.clear();
                    self.input.completion_index = None;
                    self.input.completion_prefix = None;
                }

                if enter_detected && !self.input.message_input.is_empty() {
                    // If it begins with a slash, treat as a command
                    if self.input.message_input.starts_with('/') {
                        if commands::handle_user_command(
                            &self.input.message_input,
                            &self.state.active_buffer,
                            &self.state.buffers,
                            &self.action_tx,
                            &mut self.state.system_log,
                            &mut self.connection.nickname,
                        ) {
                            self.input.history.push(self.input.message_input.clone());
                        }
                    } else {
                        // Normal message
                        if self.state.is_connected {
                            if self.state.active_buffer != "System" {
                                let _ = self.action_tx.send(BackendAction::SendMessage {
                                    target: self.state.active_buffer.clone(),
                                    text: self.input.message_input.clone(),
                                });
                                self.input.history.push(self.input.message_input.clone());
                            }
                        } else {
                            let ts = Local::now().format("%H:%M:%S").to_string();
                            self.state.system_log
                                .push(format!("[{}] ⚠ Not connected: message not sent", ts));
                        }
                    }

                    // Reset history navigation and input
                    self.input.history_pos = None;
                    self.input.history_saved_input = None;
                    self.input.message_input.clear();
                    response.request_focus();
                    enter_pressed = true;
                }
                }); // close input_frame
            });
        });

        if enter_pressed {
            Some(true)
        } else {
            None
        }
    }
}
