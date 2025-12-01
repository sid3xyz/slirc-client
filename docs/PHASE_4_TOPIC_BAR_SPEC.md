# Phase 4: Topic Bar Enhancement - Implementation Specification

**Date:** November 30, 2025
**Status:** Planning
**Estimated Effort:** 18-22 hours
**Prerequisites:** Phase 1-3 Complete ✅
**Deliverables:** New `src/ui/topic_bar.rs` module, enhanced `app.rs` integration

---

## Overview

The topic bar is a prominent horizontal component displayed above the message area that shows:
- Channel name and icon
- Current topic (with inline editing)
- User count indicator
- Channel mode badges
- Action icons (notifications, pinned messages, search)

**Design Philosophy:** Discord/Slack-inspired with IRC-specific features (modes, operator actions)

---

## Visual Specifications

### Layout & Dimensions

```
┌─────────────────────────────────────────────────────────────────────────┐
│ # channel-name    👥 42  +mtn    Topic text here (double-click to edit) │ 🔔 📌 🔍
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘
```

**Dimensions:**
- **Height:** 52px (48px content + 4px bottom border)
- **Padding:** 16px horizontal, 12px vertical
- **Border bottom:** 1px solid border_medium
- **Background:** surface[2] (elevated above message area)

**Layout Grid:**
```
[16px] [Icon 20px] [8px] [Channel Name] [16px] [👥 Count] [8px] [+Badges] [16px] [Topic - Flex Grow] [16px] [Icons] [16px]
```

---

## Component Breakdown

### 1. Channel Icon (Left)

**Implementation:**
```rust
// Size: 20x20px
// For channels: # or & prefix
// For DMs: 👤 emoji
// For System: ⚙ emoji

let icon = if active_buffer.starts_with('#') || active_buffer.starts_with('&') {
    "#"
} else if active_buffer == "System" {
    "⚙"
} else {
    "👤"
};

ui.label(
    egui::RichText::new(icon)
        .size(16.0)
        .color(theme.text_secondary)
);
```

**Spacing:** 8px right margin

---

### 2. Channel Name (Left-Center)

**Implementation:**
```rust
// Font: 16px, semi-bold
// Color: text_primary
// Behavior: Click to show channel info (future)

let display_name = if let Some(stripped) = active_buffer.strip_prefix('#') {
    stripped
} else {
    active_buffer.as_str()
};

ui.label(
    egui::RichText::new(display_name)
        .size(16.0)
        .strong()
        .color(theme.text_primary)
);
```

**Spacing:** 16px right margin

---

### 3. User Count Indicator (Left-Center)

**Implementation:**
```rust
// Font: 14px regular
// Color: text_muted
// Format: "👥 42" or hide if < 2 users
// Only show for channels, not DMs or System

if active_buffer.starts_with('#') || active_buffer.starts_with('&') {
    if let Some(buffer) = buffers.get(active_buffer) {
        let user_count = buffer.users.len();
        if user_count > 1 {
            ui.label(
                egui::RichText::new(format!("👥 {}", user_count))
                    .size(13.0)
                    .color(theme.text_muted)
            );
        }
    }
}
```

**Spacing:** 8px right margin

---

### 4. Channel Mode Badges (Left-Center)

**Implementation:**
```rust
// Font: 11px, monospace
// Display format: "+mtn" as single badge
// Color: Surface[5] background, text_secondary foreground
// Only for channels with modes set

if active_buffer.starts_with('#') || active_buffer.starts_with('&') {
    if let Some(buffer) = buffers.get(active_buffer) {
        if !buffer.channel_modes.is_empty() {
            // Render rounded rectangle badge
            let mode_text = format!("+{}", buffer.channel_modes);
            let badge_font = egui::FontId::new(10.0, egui::FontFamily::Monospace);
            let galley = ui.fonts(|f| f.layout_no_wrap(mode_text, badge_font, theme.text_secondary));

            let badge_width = galley.size().x + 12.0;
            let badge_height = 20.0;
            let badge_pos = ui.cursor().min;
            let badge_rect = egui::Rect::from_min_size(
                badge_pos,
                egui::vec2(badge_width, badge_height),
            );

            ui.painter().rect_filled(badge_rect, 4.0, theme.surface[5]);
            ui.painter().galley(
                badge_rect.center() - galley.size() / 2.0,
                galley,
                theme.text_secondary,
            );

            ui.add_space(badge_width + 16.0);
        }
    }
}
```

**Mode Legend:**
- `+m` - Moderated
- `+t` - Topic restricted to ops
- `+n` - No external messages
- `+s` - Secret channel
- `+i` - Invite only
- `+p` - Private
- `+k` - Key required

**Tooltip:** Show full mode descriptions on hover

**Spacing:** 16px right margin

---

### 5. Topic Text (Center - Flex Grow)

**Implementation:**
```rust
// Font: 14px regular
// Color: text_secondary
// Behavior:
//   - Truncate with ellipsis if too long
//   - Double-click to edit (if user has permissions)
//   - Show full text in tooltip on hover

let topic = buffer.topic.as_str();
let topic_display = if topic.is_empty() {
    "No topic set"
} else {
    topic
};

let topic_response = ui.add_sized(
    egui::vec2(ui.available_width() - 120.0, 24.0), // Reserve space for icons
    egui::Label::new(
        egui::RichText::new(topic_display)
            .size(14.0)
            .color(if topic.is_empty() { theme.text_muted } else { theme.text_secondary })
    )
    .truncate(true)
    .sense(egui::Sense::click())
);

// Double-click to edit
if topic_response.double_clicked() {
    // Check if user has permission (ops/half-ops if +t, anyone otherwise)
    let can_edit = check_topic_permission(active_buffer, buffers, nickname);
    if can_edit {
        dialogs.open_topic_editor(active_buffer, topic);
    } else {
        system_log.push("You don't have permission to change the topic".to_string());
    }
}

// Tooltip with full topic
if !topic.is_empty() && topic_response.hovered() {
    topic_response.on_hover_text(topic);
}
```

**Permission Check:**
```rust
fn check_topic_permission(
    channel: &str,
    buffers: &HashMap<String, ChannelBuffer>,
    nickname: &str,
) -> bool {
    if let Some(buffer) = buffers.get(channel) {
        // If +t mode, only ops/half-ops can edit
        if buffer.channel_modes.contains('t') {
            buffer.users.iter().any(|u| {
                u.nick == nickname && {
                    let rank = crate::ui::theme::prefix_rank(u.prefix);
                    rank >= 2 // Half-op or higher
                }
            })
        } else {
            // Anyone in channel can edit
            buffer.users.iter().any(|u| u.nick == nickname)
        }
    } else {
        false
    }
}
```

**Spacing:** Grows to fill available space, 16px right margin before icons

---

### 6. Action Icons (Right)

**Implementation:**
```rust
// Size: 24x24px click targets, 18px icons
// Layout: Right-aligned, 8px spacing between icons
// Color: text_muted (idle), text_primary (hover), accent (active)

ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
    // Search icon
    let search_response = ui.add(
        egui::Label::new(
            egui::RichText::new("🔍")
                .size(16.0)
                .color(if hovered_search { theme.text_primary } else { theme.text_muted })
        )
        .sense(egui::Sense::click())
    );
    if search_response.clicked() {
        // Open search dialog (Phase 6)
        dialogs.open_channel_search(active_buffer);
    }
    if search_response.hovered() {
        search_response.on_hover_text("Search messages (Ctrl+F)");
    }

    ui.add_space(8.0);

    // Pinned messages icon
    let pin_response = ui.add(
        egui::Label::new(
            egui::RichText::new("📌")
                .size(16.0)
                .color(if has_pinned { theme.accent } else if hovered_pin { theme.text_primary } else { theme.text_muted })
        )
        .sense(egui::Sense::click())
    );
    if pin_response.clicked() {
        // Toggle pinned messages view (future)
        state.show_pinned_messages = !state.show_pinned_messages;
    }
    if pin_response.hovered() {
        pin_response.on_hover_text("Pinned messages");
    }

    ui.add_space(8.0);

    // Notification settings icon
    let notif_response = ui.add(
        egui::Label::new(
            egui::RichText::new("🔔")
                .size(16.0)
                .color(if notifications_muted { theme.text_muted } else if hovered_notif { theme.text_primary } else { theme.text_secondary })
        )
        .sense(egui::Sense::click())
    );
    if notif_response.clicked() {
        // Toggle notification mute (future)
        if let Some(buffer) = buffers.get_mut(active_buffer) {
            buffer.notifications_muted = !buffer.notifications_muted;
        }
    }
    if notif_response.hovered() {
        let status = if notifications_muted { "unmute" } else { "mute" };
        notif_response.on_hover_text(format!("Click to {} notifications", status));
    }
});
```

**Icon Meanings:**
- 🔔 - Notification settings (muted/unmuted)
- 📌 - Pinned messages (highlighted when pinned messages exist)
- 🔍 - Search in channel

**Spacing:** 8px between icons, 16px right margin

---

## State Management

### New Fields in ChannelBuffer

```rust
// In src/buffer.rs

pub struct ChannelBuffer {
    // ... existing fields

    /// Channel modes (e.g., "mtn" for +m+t+n)
    pub channel_modes: String,

    /// Whether notifications are muted for this channel
    pub notifications_muted: bool,

    /// List of pinned message IDs (future)
    pub pinned_messages: Vec<usize>,
}

impl ChannelBuffer {
    pub fn new(name: String) -> Self {
        Self {
            // ... existing initialization
            channel_modes: String::new(),
            notifications_muted: false,
            pinned_messages: Vec::new(),
        }
    }
}
```

### New Backend Events

```rust
// In src/protocol.rs

#[derive(Debug, Clone)]
pub enum GuiEvent {
    // ... existing variants

    /// Channel mode changed (e.g., +m, -t)
    ChannelMode {
        channel: String,
        modes: String,
        set_by: String,
    },
}
```

### Event Processing

```rust
// In src/events.rs

GuiEvent::ChannelMode { channel, modes, set_by } => {
    let buffer = state.ensure_buffer(&channel);

    // Update channel_modes string
    // Parse modes: +m-t means add m, remove t
    for ch in modes.chars() {
        match ch {
            '+' => continue, // Flag to add
            '-' => continue, // Flag to remove
            mode => {
                // Add or remove from channel_modes
                // (Simplified - real implementation needs +/- tracking)
                if !buffer.channel_modes.contains(mode) {
                    buffer.channel_modes.push(mode);
                }
            }
        }
    }

    // Log to buffer
    let ts = Local::now().format("%H:%M:%S").to_string();
    let mode_msg = RenderedMessage::new(
        ts,
        "*".into(),
        format!("{} set mode {}", set_by, modes)
    ).with_type(MessageType::Mode);
    buffer.add_message(mode_msg, active == channel, false);

    None
}
```

---

## Module Structure

### New File: `src/ui/topic_bar.rs`

```rust
//! Topic bar component - displays channel info, topic, modes, and actions
//! Rendered above the message area for channels

use eframe::egui::{self, Color32};
use std::collections::HashMap;

use crate::buffer::ChannelBuffer;
use crate::ui::theme::SlircTheme;
use crate::ui::dialogs::DialogManager;
use crate::protocol::UserInfo;

/// Render the topic bar for a channel
///
/// Returns true if topic was double-clicked (caller should open editor)
#[allow(clippy::too_many_arguments)]
pub fn render_topic_bar(
    ui: &mut egui::Ui,
    active_buffer: &str,
    buffers: &HashMap<String, ChannelBuffer>,
    nickname: &str,
    theme: &SlircTheme,
    dialogs: &mut DialogManager,
    system_log: &mut Vec<String>,
) -> bool {
    // Hide for System buffer or DMs (unless we want to show something there)
    if active_buffer == "System" || (!active_buffer.starts_with('#') && !active_buffer.starts_with('&')) {
        return false;
    }

    let mut topic_double_clicked = false;

    let buffer = buffers.get(active_buffer);

    ui.horizontal(|ui| {
        ui.add_space(16.0);

        // 1. Channel icon
        render_channel_icon(ui, active_buffer, theme);
        ui.add_space(8.0);

        // 2. Channel name
        render_channel_name(ui, active_buffer, theme);
        ui.add_space(16.0);

        // 3. User count
        if let Some(buf) = buffer {
            render_user_count(ui, buf, theme);
            ui.add_space(8.0);

            // 4. Mode badges
            render_mode_badges(ui, buf, theme);
            ui.add_space(16.0);

            // 5. Topic (flex grow)
            topic_double_clicked = render_topic_text(
                ui,
                active_buffer,
                buf,
                buffers,
                nickname,
                theme,
                dialogs,
                system_log,
            );

            ui.add_space(16.0);
        }

        // 6. Action icons
        render_action_icons(ui, active_buffer, buffer, theme);

        ui.add_space(16.0);
    });

    topic_double_clicked
}

fn render_channel_icon(ui: &mut egui::Ui, channel: &str, theme: &SlircTheme) {
    let icon = if channel.starts_with('#') || channel.starts_with('&') {
        "#"
    } else {
        "👤"
    };

    ui.label(
        egui::RichText::new(icon)
            .size(16.0)
            .color(theme.text_secondary)
    );
}

fn render_channel_name(ui: &mut egui::Ui, channel: &str, theme: &SlircTheme) {
    let display_name = if let Some(stripped) = channel.strip_prefix('#') {
        stripped
    } else {
        channel
    };

    ui.label(
        egui::RichText::new(display_name)
            .size(16.0)
            .strong()
            .color(theme.text_primary)
    );
}

fn render_user_count(ui: &mut egui::Ui, buffer: &ChannelBuffer, theme: &SlircTheme) {
    let user_count = buffer.users.len();
    if user_count > 1 {
        ui.label(
            egui::RichText::new(format!("👥 {}", user_count))
                .size(13.0)
                .color(theme.text_muted)
        );
    }
}

fn render_mode_badges(ui: &mut egui::Ui, buffer: &ChannelBuffer, theme: &SlircTheme) {
    if !buffer.channel_modes.is_empty() {
        let mode_text = format!("+{}", buffer.channel_modes);

        // Create badge background
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(mode_text.len() as f32 * 8.0 + 12.0, 20.0),
            egui::Sense::hover(),
        );

        ui.painter().rect_filled(rect, 4.0, theme.surface[5]);

        // Draw text
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &mode_text,
            egui::FontId::new(10.0, egui::FontFamily::Monospace),
            theme.text_secondary,
        );

        // Tooltip
        if ui.rect_contains_pointer(rect) {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("mode_tooltip"), |ui| {
                ui.label(format_mode_description(&buffer.channel_modes));
            });
        }
    }
}

fn render_topic_text(
    ui: &mut egui::Ui,
    channel: &str,
    buffer: &ChannelBuffer,
    buffers: &HashMap<String, ChannelBuffer>,
    nickname: &str,
    theme: &SlircTheme,
    dialogs: &mut DialogManager,
    system_log: &mut Vec<String>,
) -> bool {
    let topic = &buffer.topic;
    let topic_display = if topic.is_empty() {
        "No topic set (double-click to set)"
    } else {
        topic.as_str()
    };

    let available_width = ui.available_width().max(100.0) - 120.0; // Reserve for icons

    let topic_response = ui.add_sized(
        egui::vec2(available_width, 24.0),
        egui::Label::new(
            egui::RichText::new(topic_display)
                .size(14.0)
                .color(if topic.is_empty() { theme.text_muted } else { theme.text_secondary })
        )
        .truncate(true)
        .sense(egui::Sense::click())
    );

    let mut double_clicked = false;

    if topic_response.double_clicked() {
        let can_edit = check_topic_permission(channel, buffers, nickname);
        if can_edit {
            dialogs.open_topic_editor(channel, topic);
            double_clicked = true;
        } else {
            system_log.push("You don't have permission to change the topic (channel is +t)".to_string());
        }
    }

    // Tooltip with full topic
    if !topic.is_empty() && topic_response.hovered() {
        topic_response.on_hover_text(topic);
    }

    double_clicked
}

fn render_action_icons(
    ui: &mut egui::Ui,
    _channel: &str,
    buffer: Option<&ChannelBuffer>,
    theme: &SlircTheme,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        // Search icon
        let search_response = ui.add(
            egui::Label::new(
                egui::RichText::new("🔍")
                    .size(16.0)
                    .color(theme.text_muted)
            )
            .sense(egui::Sense::click())
        );
        if search_response.hovered() {
            search_response.on_hover_text("Search messages (Ctrl+F)");
        }

        ui.add_space(8.0);

        // Pinned messages icon
        let has_pinned = buffer.map(|b| !b.pinned_messages.is_empty()).unwrap_or(false);
        let pin_color = if has_pinned { theme.accent } else { theme.text_muted };

        let pin_response = ui.add(
            egui::Label::new(
                egui::RichText::new("📌")
                    .size(16.0)
                    .color(pin_color)
            )
            .sense(egui::Sense::click())
        );
        if pin_response.hovered() {
            pin_response.on_hover_text("Pinned messages");
        }

        ui.add_space(8.0);

        // Notification icon
        let notifications_muted = buffer.map(|b| b.notifications_muted).unwrap_or(false);
        let notif_icon = if notifications_muted { "🔕" } else { "🔔" };
        let notif_color = if notifications_muted { theme.text_muted } else { theme.text_secondary };

        let notif_response = ui.add(
            egui::Label::new(
                egui::RichText::new(notif_icon)
                    .size(16.0)
                    .color(notif_color)
            )
            .sense(egui::Sense::click())
        );
        if notif_response.hovered() {
            let status = if notifications_muted { "unmuted" } else { "muted" };
            notif_response.on_hover_text(format!("Notifications are {} (click to toggle)", status));
        }
    });
}

/// Check if user has permission to edit topic
fn check_topic_permission(
    channel: &str,
    buffers: &HashMap<String, ChannelBuffer>,
    nickname: &str,
) -> bool {
    if let Some(buffer) = buffers.get(channel) {
        // If +t mode, only ops/half-ops can edit
        if buffer.channel_modes.contains('t') {
            buffer.users.iter().any(|u| {
                u.nick == nickname && {
                    let rank = crate::ui::theme::prefix_rank(u.prefix);
                    rank >= 2 // Half-op or higher
                }
            })
        } else {
            // Anyone in channel can edit
            buffer.users.iter().any(|u| u.nick == nickname)
        }
    } else {
        false
    }
}

/// Format mode description for tooltip
fn format_mode_description(modes: &str) -> String {
    let mut descriptions = Vec::new();

    for ch in modes.chars() {
        let desc = match ch {
            'm' => "Moderated (only voiced+ can speak)",
            't' => "Topic restricted to operators",
            'n' => "No external messages",
            's' => "Secret channel (hidden from /list)",
            'i' => "Invite only",
            'p' => "Private channel",
            'k' => "Key required to join",
            'l' => "User limit set",
            _ => continue,
        };
        descriptions.push(desc);
    }

    if descriptions.is_empty() {
        "No modes set".to_string()
    } else {
        descriptions.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_mode_description() {
        assert_eq!(
            format_mode_description("mtn"),
            "Moderated (only voiced+ can speak)\nTopic restricted to operators\nNo external messages"
        );

        assert_eq!(
            format_mode_description(""),
            "No modes set"
        );
    }

    #[test]
    fn test_check_topic_permission_no_t_mode() {
        // Without +t, anyone in channel can edit
        let mut buffers = HashMap::new();
        let mut buffer = ChannelBuffer::new("#test".to_string());
        buffer.users = vec![
            UserInfo { nick: "alice".to_string(), prefix: None },
            UserInfo { nick: "bob".to_string(), prefix: Some('@') },
        ];
        buffers.insert("#test".to_string(), buffer);

        assert!(check_topic_permission("#test", &buffers, "alice"));
        assert!(check_topic_permission("#test", &buffers, "bob"));
    }

    #[test]
    fn test_check_topic_permission_with_t_mode() {
        // With +t, only ops+ can edit
        let mut buffers = HashMap::new();
        let mut buffer = ChannelBuffer::new("#test".to_string());
        buffer.channel_modes = "t".to_string();
        buffer.users = vec![
            UserInfo { nick: "alice".to_string(), prefix: None },
            UserInfo { nick: "bob".to_string(), prefix: Some('@') },
        ];
        buffers.insert("#test".to_string(), buffer);

        assert!(!check_topic_permission("#test", &buffers, "alice"));
        assert!(check_topic_permission("#test", &buffers, "bob"));
    }
}
```

---

## Integration with app.rs

### Add Module Import

```rust
// In src/app.rs
mod ui {
    pub mod dialogs;
    pub mod menu;
    pub mod messages;
    pub mod panels;
    pub mod quick_switcher;
    pub mod shortcuts;
    pub mod theme;
    pub mod topic_bar;  // NEW
}
```

### Render in Central Panel

```rust
// In src/app.rs, render_central_panel() method

fn render_central_panel(&mut self, ctx: &egui::Context) {
    let theme = self.get_theme();

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(theme.surface[0])
                .inner_margin(egui::Margin::same(0)),
        )
        .show(ctx, |ui| {
            // TOPIC BAR (NEW - Phase 4)
            if self.state.active_buffer.starts_with('#') || self.state.active_buffer.starts_with('&') {
                egui::Frame::new()
                    .fill(theme.surface[2])
                    .stroke(egui::Stroke::new(1.0, theme.border_medium))
                    .inner_margin(egui::Margin::symmetric(0, 12))
                    .show(ui, |ui| {
                        ui::topic_bar::render_topic_bar(
                            ui,
                            &self.state.active_buffer,
                            &self.state.buffers,
                            &self.connection.nickname,
                            &theme,
                            &mut self.dialogs,
                            &mut self.state.system_log,
                        );
                    });
            }

            // MESSAGE AREA (existing)
            let buffer_name = self.state.active_buffer.clone();
            if let Some(buffer) = self.state.buffers.get(&buffer_name) {
                // ... existing message rendering
            }

            // INPUT AREA (existing)
            // ... existing input rendering
        });
}
```

---

## Testing Checklist

### Visual Testing

- [ ] Topic bar displays correctly for channels
- [ ] Topic bar hidden for DMs and System buffer
- [ ] User count updates when users join/part
- [ ] Mode badges appear when modes are set
- [ ] Mode tooltip shows correct descriptions
- [ ] Topic truncates with ellipsis when too long
- [ ] Topic tooltip shows full text on hover
- [ ] Double-click opens topic editor (with permission)
- [ ] Double-click shows error if no permission
- [ ] Action icons render right-aligned
- [ ] Icon tooltips appear on hover
- [ ] Notification icon toggles muted/unmuted state
- [ ] 52px height is maintained
- [ ] Border separates topic bar from messages
- [ ] Dark and light themes both look correct

### Functional Testing

- [ ] Topic editor dialog opens on double-click
- [ ] Topic updates when user submits new topic
- [ ] MODE command updates channel_modes field
- [ ] ChannelMode event processes correctly
- [ ] Permission check works for +t channels
- [ ] Permission check works for channels without +t
- [ ] User count updates in real-time
- [ ] Notification mute state persists
- [ ] Mode badge colors match theme

### Edge Cases

- [ ] Empty topic displays placeholder
- [ ] Very long topic truncates properly
- [ ] Topic with special characters (emojis, Unicode)
- [ ] Channel with no users (user count hidden)
- [ ] Channel with 1 user (user count hidden)
- [ ] Channel with 100+ users (count displays correctly)
- [ ] Multiple modes set (e.g., +mntis)
- [ ] Modes being added and removed
- [ ] User not in channel (can't edit topic)

---

## Performance Considerations

### Optimizations

1. **Topic Truncation:** Use egui's built-in truncate instead of manual substring
2. **User Count:** Cache value, only update on join/part events
3. **Mode Badges:** Only render if modes exist
4. **Tooltip Generation:** Lazy - only create on hover

### Memory

- Channel modes: Small string (typically < 10 chars)
- Notification mute: Single bool per buffer
- Pinned messages: Vec<usize> - minimal overhead

### Render Time

- Expected: < 1ms per frame (simple layout + text rendering)
- No complex animations or heavy computation

---

## Future Enhancements

### Phase 5+ Features

1. **Pinned Messages View:**
   - Click 📌 to show pinned messages panel
   - Pin/unpin messages from context menu
   - Persist pinned messages to config

2. **Channel Search:**
   - Click 🔍 to open search dialog (Phase 6)
   - Fuzzy search through message history
   - Highlight search results in messages

3. **Notification Settings:**
   - Click 🔔 to open detailed settings
   - Per-channel notification rules
   - @mention-only mode
   - Sound/desktop notification toggles

4. **Channel Info Panel:**
   - Click channel name to show info
   - Creation date, modes, topic history
   - Quick actions (leave, invite, etc.)

5. **Topic History:**
   - Track who changed topic and when
   - View previous topics
   - Revert to previous topic (if op)

---

## Dependencies

### Existing Code to Modify

1. **src/buffer.rs:**
   - Add `channel_modes: String`
   - Add `notifications_muted: bool`
   - Add `pinned_messages: Vec<usize>`

2. **src/protocol.rs:**
   - Add `GuiEvent::ChannelMode` variant

3. **src/events.rs:**
   - Add handler for `ChannelMode` event
   - Update mode parsing logic

4. **src/backend/handler.rs:**
   - Parse MODE commands from server
   - Send ChannelMode event to GUI

5. **src/app.rs:**
   - Import topic_bar module
   - Render topic bar in central panel
   - Update theme references

### No Breaking Changes

All changes are additive. Existing code continues to work without modification.

---

## Estimated Timeline

**Total Effort:** 18-22 hours

### Breakdown

1. **ChannelBuffer Enhancement (2 hours):**
   - Add new fields
   - Update initialization
   - Update tests

2. **Backend Event Handling (4 hours):**
   - Add ChannelMode event
   - Implement MODE parsing in backend
   - Add event processing in events.rs
   - Test mode changes

3. **Topic Bar Module (8 hours):**
   - Create topic_bar.rs
   - Implement all sub-components
   - Add permission checking
   - Write unit tests

4. **Integration (2 hours):**
   - Import module in app.rs
   - Add rendering call in central panel
   - Test with existing dialogs

5. **Visual Testing (2 hours):**
   - Test in dark/light themes
   - Test all interactions
   - Fix layout issues

6. **Documentation (2 hours):**
   - Update this spec with findings
   - Add inline code documentation
   - Update AUDIT_AND_FORWARD_PATH.md

---

## Success Criteria

**Phase 4 is complete when:**

1. ✅ Topic bar renders for all channels
2. ✅ User count displays and updates correctly
3. ✅ Mode badges appear and have tooltips
4. ✅ Topic text truncates and shows full text on hover
5. ✅ Double-click topic opens editor (with permission check)
6. ✅ Action icons render and have tooltips
7. ✅ Notification mute toggles work
8. ✅ All tests pass (unit + visual)
9. ✅ No clippy warnings
10. ✅ Git commit with purge checklist
11. ✅ Documentation updated

---

**End of Specification**
