//! Quick switcher overlay for fast channel/DM navigation (Ctrl+K)
//! Discord/Slack-style fuzzy search interface

use eframe::egui::{self, Color32, Key};
use crate::buffer::ChannelBuffer;
use crate::ui::theme::SlircTheme;
use std::collections::HashMap;

/// Quick switcher state
#[derive(Default)]
pub struct QuickSwitcher {
    pub visible: bool,
    pub query: String,
    pub selected_index: usize,
    matches: Vec<String>,
}

impl QuickSwitcher {
    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.query.clear();
            self.selected_index = 0;
        }
    }

    /// Show the quick switcher
    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.selected_index = 0;
    }

    /// Hide the quick switcher
    pub fn hide(&mut self) {
        self.visible = false;
        self.query.clear();
        self.selected_index = 0;
    }

    /// Render the quick switcher overlay
    /// Returns Some(buffer_name) if user selected a buffer
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        buffers: &HashMap<String, ChannelBuffer>,
    ) -> Option<String> {
        if !self.visible {
            return None;
        }

        let theme = if ctx.style().visuals.dark_mode {
            SlircTheme::dark()
        } else {
            SlircTheme::light()
        };

        let mut selected_buffer: Option<String> = None;

        // Capture Escape key to close
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.hide();
            return None;
        }

        // Capture Enter key to select
        if ctx.input(|i| i.key_pressed(Key::Enter)) && !self.matches.is_empty() {
            if self.selected_index < self.matches.len() {
                selected_buffer = Some(self.matches[self.selected_index].clone());
                self.hide();
                return selected_buffer;
            }
        }

        // Arrow key navigation
        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
            if self.selected_index < self.matches.len().saturating_sub(1) {
                self.selected_index += 1;
            }
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
            self.selected_index = self.selected_index.saturating_sub(1);
        }

        // Update matches based on query
        self.update_matches(buffers);

        // Modal overlay
        egui::Window::new("Quick Switcher")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
            .fixed_size(egui::vec2(500.0, 400.0))
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(theme.surface[6])
                    .stroke(egui::Stroke::new(1.0, theme.border_strong))
                    .rounding(8.0),
            )
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Search input
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new("🔍")
                                .size(18.0)
                                .color(theme.text_muted),
                        );
                        ui.add_space(8.0);
                        
                        let search_response = ui.add(
                            egui::TextEdit::singleline(&mut self.query)
                                .hint_text("Search channels, DMs...")
                                .desired_width(ui.available_width() - 16.0)
                                .font(egui::TextStyle::Heading),
                        );
                        
                        // Auto-focus on first frame
                        if ui.memory(|m| m.has_focus(search_response.id)) {
                            search_response.request_focus();
                        } else {
                            search_response.request_focus();
                        }
                    });
                    ui.add_space(12.0);

                    // Separator
                    ui.separator();
                    ui.add_space(8.0);

                    // Results list
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            if self.matches.is_empty() {
                                ui.add_space(40.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new("No matches found")
                                            .size(14.0)
                                            .color(theme.text_muted),
                                    );
                                });
                            } else {
                                for (i, buffer_name) in self.matches.iter().enumerate() {
                                    let is_selected = i == self.selected_index;
                                    
                                    let response = self.render_result_item(
                                        ui,
                                        buffer_name,
                                        buffers.get(buffer_name),
                                        is_selected,
                                        &theme,
                                    );

                                    if response.clicked() {
                                        selected_buffer = Some(buffer_name.clone());
                                        self.hide();
                                        break;
                                    }

                                    if response.hovered() {
                                        self.selected_index = i;
                                    }
                                }
                            }
                        });

                    ui.add_space(8.0);
                    
                    // Footer with hints
                    ui.separator();
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new("↑↓ Navigate")
                                .size(11.0)
                                .color(theme.text_muted),
                        );
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new("↵ Select")
                                .size(11.0)
                                .color(theme.text_muted),
                        );
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new("Esc Close")
                                .size(11.0)
                                .color(theme.text_muted),
                        );
                    });
                    ui.add_space(8.0);
                });
            });

        selected_buffer
    }

    /// Render a single result item
    fn render_result_item(
        &self,
        ui: &mut egui::Ui,
        buffer_name: &str,
        buffer: Option<&ChannelBuffer>,
        is_selected: bool,
        theme: &SlircTheme,
    ) -> egui::Response {
        let height = 48.0;
        let available_width = ui.available_width();

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(available_width, height),
            egui::Sense::click(),
        );

        // Background
        let bg_color = if is_selected {
            theme.surface[4]
        } else if response.hovered() {
            theme.surface[3]
        } else {
            Color32::TRANSPARENT
        };

        if is_selected || response.hovered() {
            ui.painter().rect_filled(rect, 6.0, bg_color);
        }

        // Icon
        let icon = if buffer_name == "System" {
            "⚙"
        } else if buffer_name.starts_with('#') || buffer_name.starts_with('&') {
            "#"
        } else {
            "👤"
        };

        ui.painter().text(
            egui::pos2(rect.min.x + 24.0, rect.center().y - 4.0),
            egui::Align2::LEFT_CENTER,
            icon,
            egui::FontId::new(20.0, egui::FontFamily::Proportional),
            theme.text_primary,
        );

        // Buffer name
        let display_name = if buffer_name.starts_with('#') {
            &buffer_name[1..]
        } else {
            buffer_name
        };

        ui.painter().text(
            egui::pos2(rect.min.x + 56.0, rect.center().y - 6.0),
            egui::Align2::LEFT_CENTER,
            display_name,
            egui::FontId::new(15.0, egui::FontFamily::Proportional),
            theme.text_primary,
        );

        // Topic or user count
        if let Some(buf) = buffer {
            let subtitle = if !buf.topic.is_empty() {
                &buf.topic
            } else if !buf.users.is_empty() {
                &format!("{} users", buf.users.len())
            } else {
                ""
            };

            if !subtitle.is_empty() {
                ui.painter().text(
                    egui::pos2(rect.min.x + 56.0, rect.center().y + 8.0),
                    egui::Align2::LEFT_CENTER,
                    subtitle,
                    egui::FontId::new(12.0, egui::FontFamily::Proportional),
                    theme.text_muted,
                );
            }
        }

        response
    }

    /// Update matches based on current query
    fn update_matches(&mut self, buffers: &HashMap<String, ChannelBuffer>) {
        let query_lower = self.query.to_lowercase();
        
        // Collect all buffer names
        let mut all_buffers: Vec<String> = buffers.keys().cloned().collect();
        all_buffers.push("System".to_string());
        
        // Sort and filter
        if query_lower.is_empty() {
            // No query: show all, sorted
            all_buffers.sort();
            self.matches = all_buffers;
        } else {
            // Fuzzy match: contains query substring
            let mut matches: Vec<String> = all_buffers
                .into_iter()
                .filter(|name| name.to_lowercase().contains(&query_lower))
                .collect();
            
            // Sort by relevance: exact prefix match first, then contains
            matches.sort_by(|a, b| {
                let a_lower = a.to_lowercase();
                let b_lower = b.to_lowercase();
                let a_prefix = a_lower.starts_with(&query_lower);
                let b_prefix = b_lower.starts_with(&query_lower);
                
                match (a_prefix, b_prefix) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.cmp(b),
                }
            });
            
            self.matches = matches;
        }
        
        // Reset selection if out of bounds
        if self.selected_index >= self.matches.len() && !self.matches.is_empty() {
            self.selected_index = 0;
        }
    }
}
