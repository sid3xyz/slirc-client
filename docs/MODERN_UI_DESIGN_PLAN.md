# SLIRC Modern UI/UX Design Plan
## User-First Design Strategy for Modern IRC Client

**Date:** November 30, 2025  
**Target:** slirc-client (egui-based)  
**Research Foundation:** Discord, Slack, Microsoft Teams, Telegram, Element analysis

---

## Executive Summary

This document outlines a comprehensive modernization plan for slirc-client to meet and exceed 2025 chat application standards while respecting IRC's unique characteristics. The plan emphasizes:

- **Traditional top horizontal menu bar** for desktop convention
- **User-first thinking** with progressive disclosure and keyboard efficiency
- **Modern visual hierarchy** matching Discord/Slack aesthetics
- **Accessibility** as a core principle (WCAG AA compliance)

---

## 1. Design Philosophy

### 1.1 Core Principles

**User-First Thinking:**
- **Discoverability:** Traditional menu bar for new users
- **Efficiency:** Keyboard shortcuts for power users
- **Consistency:** Follow platform conventions (macOS/Windows/Linux)
- **Progressive Disclosure:** Hide complexity until needed

**IRC-Specific Considerations:**
- No avatars → Generate identicons from username hashes
- Text-heavy → Optimize typography and spacing
- Multi-server → Clear visual server/channel hierarchy
- Real-time → Unread indicators and attention management

**Modern Standards:**
- 16px base font size for readability
- 8pt grid system for consistent spacing
- Surface-based color hierarchy (7+ levels)
- Smooth animations (120-250ms transitions)

---

## 2. Top Menu Bar Specification

### 2.1 Menu Structure

Based on research, implement a **minimal but complete** menu bar that balances traditional desktop conventions with modern minimalism:

```
File | Edit | View | Server | Window | Help
```

#### **File Menu**
```
New Connection...         Ctrl+N
─────────────────
Open Logs...              Ctrl+O
Save Chat Log...          Ctrl+S
─────────────────
Preferences...            Ctrl+,
─────────────────
Quit                      Ctrl+Q
```

#### **Edit Menu**
```
Undo                      Ctrl+Z
Redo                      Ctrl+Shift+Z
─────────────────
Cut                       Ctrl+X
Copy                      Ctrl+C
Paste                     Ctrl+V
Select All                Ctrl+A
─────────────────
Find in Chat...           Ctrl+F
```

#### **View Menu**
```
Quick Switcher            Ctrl+K
─────────────────
✓ Show User List          Ctrl+U
✓ Show Topic Bar          
  Show Timestamps
─────────────────
Theme                     ▸
  ● Dark
  ○ Light  
  ○ System
─────────────────
Zoom In                   Ctrl++
Zoom Out                  Ctrl+-
Reset Zoom                Ctrl+0
─────────────────
Toggle Full Screen        F11
```

#### **Server Menu** (IRC-specific)
```
Join Channel...           Ctrl+J
Part Channel              Ctrl+W
─────────────────
List Channels...          Ctrl+L
Search Users...
─────────────────
Server Info
Disconnect                Ctrl+D
Reconnect                 Ctrl+R
```

#### **Window Menu** (macOS standard, optional on Linux/Windows)
```
Minimize                  Ctrl+M
Close Window              Ctrl+W
─────────────────
Bring All to Front
```

#### **Help Menu**
```
Keyboard Shortcuts        Ctrl+/
IRC Commands...
─────────────────
Check for Updates
Report Issue...
─────────────────
About slirc
```

### 2.2 Implementation Notes

**Platform Adaptations:**
- **macOS:** Full menu bar (required by HIG), includes "slirc" application menu
- **Windows/Linux:** Optional auto-hide menu bar (press `Alt` to reveal)
- **All platforms:** Right-click context menus duplicate common actions

**Egui Implementation:**
```rust
// In app.rs
egui::menu::bar(ctx, |ui| {
    ui.menu_button("File", |ui| {
        if ui.add_enabled(can_connect, egui::Button::new("New Connection..."))
            .on_hover_text("Connect to IRC server")
            .clicked() 
        {
            self.show_connect_dialog = true;
            ui.close_menu();
        }
        ui.add_shortcut_text("Ctrl+N");
        
        ui.separator();
        
        if ui.button("Preferences...").clicked() {
            self.show_preferences = true;
            ui.close_menu();
        }
        ui.add_shortcut_text("Ctrl+,");
        
        ui.separator();
        
        if ui.button("Quit").clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ui.add_shortcut_text("Ctrl+Q");
    });
    
    ui.menu_button("Edit", |ui| {
        // Standard edit menu
        ui.add_enabled_ui(has_selection, |ui| {
            if ui.button("Copy").clicked() {
                copy_to_clipboard();
                ui.close_menu();
            }
            ui.add_shortcut_text("Ctrl+C");
        });
        
        ui.separator();
        
        if ui.button("Find in Chat...").clicked() {
            self.show_search = true;
            ui.close_menu();
        }
        ui.add_shortcut_text("Ctrl+F");
    });
    
    ui.menu_button("View", |ui| {
        if ui.button("Quick Switcher").clicked() {
            self.show_quick_switcher = true;
            ui.close_menu();
        }
        ui.add_shortcut_text("Ctrl+K");
        
        ui.separator();
        
        ui.checkbox(&mut self.config.show_user_list, "Show User List");
        ui.add_shortcut_text("Ctrl+U");
        
        ui.checkbox(&mut self.config.show_topic_bar, "Show Topic Bar");
        ui.checkbox(&mut self.config.show_timestamps, "Show Timestamps");
        
        ui.separator();
        
        ui.menu_button("Theme", |ui| {
            ui.radio_value(&mut self.config.theme, Theme::Dark, "Dark");
            ui.radio_value(&mut self.config.theme, Theme::Light, "Light");
            ui.radio_value(&mut self.config.theme, Theme::System, "System");
        });
        
        ui.separator();
        
        if ui.button("Zoom In").clicked() {
            ctx.set_zoom_factor(ctx.zoom_factor() * 1.1);
        }
        ui.add_shortcut_text("Ctrl++");
        
        if ui.button("Zoom Out").clicked() {
            ctx.set_zoom_factor(ctx.zoom_factor() / 1.1);
        }
        ui.add_shortcut_text("Ctrl+-");
        
        if ui.button("Reset Zoom").clicked() {
            ctx.set_zoom_factor(1.0);
        }
        ui.add_shortcut_text("Ctrl+0");
    });
    
    ui.menu_button("Server", |ui| {
        if ui.button("Join Channel...").clicked() {
            self.show_join_dialog = true;
            ui.close_menu();
        }
        ui.add_shortcut_text("Ctrl+J");
        
        if ui.add_enabled(in_channel, egui::Button::new("Part Channel")).clicked() {
            self.part_current_channel();
            ui.close_menu();
        }
        ui.add_shortcut_text("Ctrl+W");
        
        ui.separator();
        
        if ui.button("List Channels...").clicked() {
            self.request_channel_list();
            ui.close_menu();
        }
        ui.add_shortcut_text("Ctrl+L");
        
        ui.separator();
        
        if ui.add_enabled(connected, egui::Button::new("Disconnect")).clicked() {
            self.disconnect();
            ui.close_menu();
        }
        ui.add_shortcut_text("Ctrl+D");
        
        if ui.add_enabled(!connected, egui::Button::new("Reconnect")).clicked() {
            self.reconnect();
            ui.close_menu();
        }
        ui.add_shortcut_text("Ctrl+R");
    });
    
    ui.menu_button("Help", |ui| {
        if ui.button("Keyboard Shortcuts").clicked() {
            self.show_shortcuts = true;
            ui.close_menu();
        }
        ui.add_shortcut_text("Ctrl+/");
        
        ui.separator();
        
        if ui.button("About slirc").clicked() {
            self.show_about = true;
            ui.close_menu();
        }
    });
});
```

---

## 3. Layout Architecture

### 3.1 Overall Structure

**2.5-Column Layout** optimized for IRC:

```
┌────────────────────────────────────────────────────────────────┐
│  File | Edit | View | Server | Window | Help      [Menu, 24px] │
├─────────────────┬──────────────────────────────┬────────────────┤
│  Server List    │      Main Chat Area          │   User List    │
│  220px          │      (flexible)              │   240px        │
│  (resizable)    │                              │   (collapse)   │
│                 │  ┌─────────────────────────┐ │                │
│ Libera.Chat     │  │ #rust - Topic Bar  48px│ │  @ ops (2)     │
│  # announcements│  └─────────────────────────┘ │   alice        │
│  # general ①    │                              │   bob          │
│  # rust         │  [Message Area - Scrollable] │                │
│  # webdev       │                              │  + voiced (5)  │
│                 │                              │   charlie      │
│ Freenode        │                              │   ...          │
│  # python       │  ┌─────────────────────────┐ │                │
│                 │  │ Message Input       56px│ │  users (142)   │
│ 📱 DMs          │  └─────────────────────────┘ │   david        │
│  alice          │                              │   ...          │
│  bob            │                              │                │
│                 │                              │  [Scroll]      │
└─────────────────┴──────────────────────────────┴────────────────┘
```

### 3.2 Responsive Behavior

**Breakpoints:**
- **≥ 1280px:** Full 3-column (Server List + Chat + User List)
- **1024-1279px:** Auto-collapse User List, show toggle button
- **800-1023px:** Narrow Server List to 180px
- **< 800px:** Overlay Server List (slide-out drawer)

**Resize Constraints:**
- Server List: 180px min, 320px max, 220px default
- User List: 180px min, 320px max, 240px default
- Chat Area: 480px minimum for readability

---

## 4. Typography System

### 4.1 Font Selection

**Primary Font Stack:**
```rust
// Proportional (UI elements)
"Inter-Regular"        // 14px base, modern sans-serif
"Inter-Medium"         // 14px, semibold UI elements
"Inter-Bold"           // 16px, headings and emphasis

// Monospace (IRC messages, nicknames)
"JetBrains Mono"       // 14px, excellent readability
"JetBrains Mono Bold"  // 14px, highlighted nicks

// Fallbacks (egui defaults)
"Ubuntu-Light"         // Cross-platform proportional
"Hack"                 // Cross-platform monospace
"Noto Emoji Regular"   // Emoji support
```

**Licensing:** All fonts use SIL OFL 1.1 (redistributable)

**Download Links:**
- Inter: https://github.com/rsms/inter/releases
- JetBrains Mono: https://github.com/JetBrains/JetBrainsMono/releases

### 4.2 Text Style Specifications

```rust
TextStyle::Name("menu_item") →        13px Inter-Regular
TextStyle::Name("section_header") →   11px Inter-Medium, UPPERCASE, letter-spacing: 0.5px
TextStyle::Name("channel_name") →     14px Inter-Regular
TextStyle::Name("channel_unread") →   14px Inter-Bold
TextStyle::Name("timestamp") →        11px JetBrains Mono, opacity: 0.5
TextStyle::Name("irc_nick") →         13px JetBrains Mono Medium
TextStyle::Name("irc_message") →      14px JetBrains Mono
TextStyle::Name("topic") →            12px Inter-Regular, italic
TextStyle::Name("user_count") →       11px Inter-Regular, opacity: 0.6
```

### 4.3 Implementation

```rust
fn setup_fonts(cc: &eframe::CreationContext) {
    let mut fonts = FontDefinitions::default();
    
    // Load bundled fonts
    fonts.font_data.insert(
        "Inter-Regular".to_owned(),
        Arc::new(FontData::from_static(include_bytes!("../fonts/Inter-Regular.ttf")))
    );
    fonts.font_data.insert(
        "Inter-Medium".to_owned(),
        Arc::new(FontData::from_static(include_bytes!("../fonts/Inter-Medium.ttf")))
    );
    fonts.font_data.insert(
        "JetBrainsMono-Regular".to_owned(),
        Arc::new(FontData::from_static(include_bytes!("../fonts/JetBrainsMono-Regular.ttf")))
    );
    
    // Set family priorities
    fonts.families.insert(
        FontFamily::Proportional,
        vec![
            "Inter-Regular".to_owned(),
            "Ubuntu-Light".to_owned(),
            "NotoEmoji-Regular".to_owned(),
        ]
    );
    
    fonts.families.insert(
        FontFamily::Monospace,
        vec![
            "JetBrainsMono-Regular".to_owned(),
            "Hack".to_owned(),
        ]
    );
    
    cc.egui_ctx.set_fonts(fonts);
    
    // Configure text styles
    configure_text_styles(&cc.egui_ctx);
}
```

---

## 5. Color System & Theme

### 5.1 Semantic Color Palette

**Dark Theme (Primary):**
```rust
// Surface layers (background depth)
surface_0:      #0A0A0F    // App background
surface_1:      #13131A    // Sidebar background
surface_2:      #1C1C26    // Message background
surface_3:      #252532    // Hover state
surface_4:      #2E2E3E    // Active selection
surface_5:      #38384A    // Elevated panels
surface_6:      #424256    // Modals/dialogs

// Primary accent (brand color)
accent:         #5865F2    // Discord-inspired blue
accent_hover:   #4752C4
accent_active:  #3C45A5

// Semantic colors
success:        #43B581    // Green (online, success)
warning:        #FAA61A    // Amber (away, warning)
error:          #F04747    // Red (offline, error, mentions)
info:           #00AFF4    // Cyan (info, links)

// Text hierarchy
text_primary:   #FFFFFF    // Main text, 100% opacity
text_secondary: #B9BBBE    // Usernames, ~73% opacity
text_muted:     #72767D    // Timestamps, ~45% opacity
text_disabled:  #4F545C    // Disabled elements, ~30% opacity

// Borders & dividers
border_subtle:  #202225    // Subtle separators
border_medium:  #2F3136    // Clear divisions
border_strong:  #40444B    // Strong emphasis

// Special states
unread:         #FFFFFF    // Bold white text
highlight:      #F04747    // Mention background
highlight_text: #FFFFFF    // Mention text
```

**Light Theme:**
```rust
surface_0:      #FFFFFF
surface_1:      #F6F6F7
surface_2:      #F2F3F5
surface_3:      #E3E5E8
surface_4:      #D4D7DC
surface_5:      #C4C9D0
surface_6:      #B5BBC4

accent:         #5865F2
text_primary:   #060607
text_secondary: #4F5660
text_muted:     #747F8D
```

### 5.2 Implementation

```rust
// In theme.rs
pub struct SlircTheme {
    pub name: String,
    pub surface: [Color32; 7],
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_active: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_disabled: Color32,
    pub border_subtle: Color32,
    pub border_medium: Color32,
    pub border_strong: Color32,
}

impl SlircTheme {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            surface: [
                Color32::from_hex("#0A0A0F").unwrap(),
                Color32::from_hex("#13131A").unwrap(),
                Color32::from_hex("#1C1C26").unwrap(),
                Color32::from_hex("#252532").unwrap(),
                Color32::from_hex("#2E2E3E").unwrap(),
                Color32::from_hex("#38384A").unwrap(),
                Color32::from_hex("#424256").unwrap(),
            ],
            accent: Color32::from_hex("#5865F2").unwrap(),
            accent_hover: Color32::from_hex("#4752C4").unwrap(),
            accent_active: Color32::from_hex("#3C45A5").unwrap(),
            success: Color32::from_hex("#43B581").unwrap(),
            warning: Color32::from_hex("#FAA61A").unwrap(),
            error: Color32::from_hex("#F04747").unwrap(),
            info: Color32::from_hex("#00AFF4").unwrap(),
            text_primary: Color32::WHITE,
            text_secondary: Color32::from_hex("#B9BBBE").unwrap(),
            text_muted: Color32::from_hex("#72767D").unwrap(),
            text_disabled: Color32::from_hex("#4F545C").unwrap(),
            border_subtle: Color32::from_hex("#202225").unwrap(),
            border_medium: Color32::from_hex("#2F3136").unwrap(),
            border_strong: Color32::from_hex("#40444B").unwrap(),
        }
    }
}
```

---

## 6. Component Specifications

### 6.1 Server/Channel List (Left Sidebar)

**Visual Design:**
```
┌─────────────────────┐
│ Libera.Chat     [v] │  ← Header: 48px, surface_2, semibold
├─────────────────────┤
│ CHANNELS            │  ← Section: 11px, UPPERCASE, muted
│                     │
│ # announcements     │  ← Item: 32px height
│ # general       ①   │  ← Active: surface_4, unread badge
│ # rust              │
│ # webdev            │
│                     │
│ DIRECT MESSAGES     │  ← Collapsible section
│ alice               │
│ bob             ●   │  ← Online indicator
└─────────────────────┘
```

**Item States:**
- **Default:** surface_1 background, text_secondary
- **Hover:** surface_3 background, text_primary
- **Active:** surface_4 background, text_primary, 2px accent left border
- **Unread:** text_primary (bold), blue dot or count badge
- **Mention:** error badge with count, text_primary bold

**Code Structure:**
```rust
fn render_channel_item(ui: &mut Ui, channel: &Channel, is_active: bool, theme: &SlircTheme) {
    let item_height = 32.0;
    let padding = vec2(12.0, 6.0);
    
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), item_height),
        Sense::click()
    );
    
    // Background
    let bg_color = if is_active {
        theme.surface[4]
    } else if response.hovered() {
        theme.surface[3]
    } else {
        theme.surface[1]
    };
    ui.painter().rect_filled(rect, 4.0, bg_color);
    
    // Left accent bar for active
    if is_active {
        let bar_rect = Rect::from_min_size(
            rect.min,
            vec2(2.0, rect.height())
        );
        ui.painter().rect_filled(bar_rect, 0.0, theme.accent);
    }
    
    // Channel name
    let text_pos = rect.min + padding;
    let text_color = if is_active || response.hovered() {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    
    ui.painter().text(
        text_pos,
        Align2::LEFT_CENTER,
        format!("# {}", channel.name),
        TextStyle::Name("channel_name".into()),
        text_color
    );
    
    // Unread badge (right-aligned)
    if channel.unread_count > 0 {
        let badge_text = if channel.unread_count > 99 {
            "99+".to_string()
        } else {
            channel.unread_count.to_string()
        };
        
        let badge_color = if channel.has_mention {
            theme.error  // Red for mentions
        } else {
            theme.text_muted  // Gray for unread
        };
        
        // Draw rounded badge
        draw_badge(ui, rect.right_top() + vec2(-12.0, 16.0), &badge_text, badge_color);
    }
    
    if response.clicked() {
        // Switch to this channel
    }
}
```

### 6.2 Message Area

**Layout:**
```
┌────────────────────────────────────────┐
│ #rust - Rust programming discussion   │  ← Topic bar: 48px
├────────────────────────────────────────┤
│                                        │
│  [alice] 14:32                         │  ← Message header
│  Hey everyone! Has anyone tried...    │  ← Message body
│                                        │
│  [bob] 14:35                           │
│  @alice Yes! I actually just...       │
│  It works great for async code        │  ← Grouped message
│                                        │
│  [alice] 14:37                         │
│  Thanks @bob! That's exactly what...  │
│                                        │
│                          [Scroll bar]  │
├────────────────────────────────────────┤
│ Message #rust...              [Send]  │  ← Input: 56px
└────────────────────────────────────────┘
```

**Message Grouping:**
- Group consecutive messages from same user within 5 minutes
- First message: Show full header (nick + timestamp)
- Subsequent messages: Indent without header
- Spacing: 4px between grouped, 16px between different users

**Identicon/Avatar:**
```rust
fn draw_identicon(ui: &mut Ui, username: &str, size: f32) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    username.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Generate deterministic color
    let hue = (hash % 360) as f32;
    let color = Color32::from_rgb(
        hsv_to_rgb(hue, 0.6, 0.8).0,
        hsv_to_rgb(hue, 0.6, 0.8).1,
        hsv_to_rgb(hue, 0.6, 0.8).2,
    );
    
    // Draw circle with initials
    let center = rect.center();
    ui.painter().circle_filled(center, size / 2.0, color);
    
    let initials: String = username.chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    
    ui.painter().text(
        center,
        Align2::CENTER_CENTER,
        initials,
        TextStyle::Name("initials".into()),
        Color32::WHITE
    );
}
```

### 6.3 User List (Right Sidebar)

**Hierarchy:**
```
┌─────────────────┐
│ @ OPERATORS (2) │  ← Collapsible, 24px
│   alice     ●   │  ← 28px, online indicator
│   bob       ●   │
│                 │
│ + VOICED (5)    │
│   charlie   ●   │
│   david     ○   │  ← Offline: muted
│   eve       ⚫  │  ← Away: warning color
│   ...           │
│                 │
│ USERS (142)     │
│   faythe    ●   │
│   ...           │
└─────────────────┘
```

**Status Indicators:**
- **Online:** Green dot (success color)
- **Away:** Amber dot (warning color)
- **Offline:** Gray dot (text_disabled)

---

## 7. Keyboard Shortcuts & Accessibility

### 7.1 Global Shortcuts

```
Navigation:
Ctrl+K          Quick channel switcher
Ctrl+Tab        Next channel
Ctrl+Shift+Tab  Previous channel
Ctrl+1-9        Jump to channel 1-9
Alt+↑/↓         Navigate message history

Actions:
Ctrl+J          Join channel
Ctrl+W          Part/close channel
Ctrl+L          List channels
Ctrl+U          Toggle user list
Ctrl+F          Find in chat

Input:
Tab             Nick completion
↑/↓             Input history
Esc             Clear input / close dialogs

View:
Ctrl++          Zoom in
Ctrl+-          Zoom out
Ctrl+0          Reset zoom
F11             Toggle fullscreen
```

### 7.2 Accessibility Features

**WCAG AA Compliance:**
- Contrast ratio ≥ 4.5:1 for normal text
- Contrast ratio ≥ 3:1 for large text (18px+)
- Focus indicators: 2px accent outline
- Keyboard navigation for all functions

**Screen Reader Support:**
- ARIA labels for all interactive elements
- Semantic HTML structure (when using egui's web backend)
- Announce new messages
- Describe unread counts

**Customization:**
- Font size scaling (80%-200%)
- High contrast mode
- Reduce motion option
- Custom color schemes

---

## 8. Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
**Goal:** Establish core visual system

- [ ] **Task 1.1:** Download and integrate Inter + JetBrains Mono fonts
- [ ] **Task 1.2:** Implement `SlircTheme` struct with dark/light variants
- [ ] **Task 1.3:** Create text style system with semantic names
- [ ] **Task 1.4:** Apply theme to existing components
- [ ] **Task 1.5:** Add theme switcher in preferences

**Acceptance Criteria:**
- Fonts render correctly on Linux/macOS/Windows
- Theme switches without restart
- All text uses semantic styles
- Colors match specification (hex codes)

### Phase 2: Top Menu Bar (Week 2-3)
**Goal:** Traditional desktop menu integration

- [ ] **Task 2.1:** Implement egui menu bar with File/Edit/View/Server/Help
- [ ] **Task 2.2:** Add keyboard shortcut handling system
- [ ] **Task 2.3:** Create shortcut overlay (Ctrl+/)
- [ ] **Task 2.4:** Platform-specific menu adaptations (macOS vs Linux)
- [ ] **Task 2.5:** Menu bar auto-hide option (Windows/Linux)

**Acceptance Criteria:**
- All menu items functional
- Shortcuts work globally
- Menu bar respects platform conventions
- Shortcuts overlay lists all commands

### Phase 3: Sidebar Modernization (Week 3-4)
**Goal:** Modern channel list with sections and badges

- [ ] **Task 3.1:** Refactor panel rendering with proper spacing
- [ ] **Task 3.2:** Implement collapsible sections (Channels/DMs)
- [ ] **Task 3.3:** Add unread badges and mention indicators
- [ ] **Task 3.4:** Create hover/active states with smooth transitions
- [ ] **Task 3.5:** Add server dropdown with connection status
- [ ] **Task 3.6:** Implement channel search/filter

**Acceptance Criteria:**
- 32px item height, 4px rounding
- Unread counts update in real-time
- Mentions show red badge
- Sections expand/collapse with animation
- Active channel has accent border

### Phase 4: Message Area Enhancement (Week 4-5)
**Goal:** Cozy message display with grouping

- [ ] **Task 4.1:** Implement message grouping (5-minute window)
- [ ] **Task 4.2:** Add identicon generation from username hash
- [ ] **Task 4.3:** Create topic bar component (48px)
- [ ] **Task 4.4:** Improve message layout (avatar + content)
- [ ] **Task 4.5:** Add hover actions (reply, react, copy)
- [ ] **Task 4.6:** Implement @mention highlighting

**Acceptance Criteria:**
- Messages group correctly by user + time
- Identicons use deterministic colors
- Topic bar shows channel info
- Mentions highlighted in accent color
- Hover shows action buttons

### Phase 5: User List & Polish (Week 5-6)
**Goal:** Complete right sidebar and refinements

- [ ] **Task 5.1:** Create collapsible user list (240px default)
- [ ] **Task 5.2:** Implement user grouping (Ops/Voiced/Users)
- [ ] **Task 5.3:** Add online status indicators
- [ ] **Task 5.4:** User list search/filter
- [ ] **Task 5.5:** Resize handle for sidebars
- [ ] **Task 5.6:** Save layout preferences

**Acceptance Criteria:**
- User list groups by role
- Status dots use semantic colors
- Collapsible with Ctrl+U
- Resizable 180-320px
- Layout persists across sessions

### Phase 6: Quick Switcher & Search (Week 6-7)
**Goal:** Power user efficiency features

- [ ] **Task 6.1:** Create quick switcher dialog (Ctrl+K)
- [ ] **Task 6.2:** Fuzzy search for channels
- [ ] **Task 6.3:** Implement in-chat search (Ctrl+F)
- [ ] **Task 6.4:** Search result highlighting
- [ ] **Task 6.5:** Recent channels list

**Acceptance Criteria:**
- Ctrl+K opens fuzzy channel search
- Type-ahead filtering works
- Ctrl+F searches current channel
- Results highlight in yellow
- Recent channels shown first

### Phase 7: Accessibility & Testing (Week 7-8)
**Goal:** WCAG AA compliance and polish

- [ ] **Task 7.1:** Verify all color contrasts ≥ 4.5:1
- [ ] **Task 7.2:** Add focus indicators (2px outline)
- [ ] **Task 7.3:** Keyboard navigation for all features
- [ ] **Task 7.4:** High contrast theme option
- [ ] **Task 7.5:** Font size scaling (preferences)
- [ ] **Task 7.6:** Cross-platform testing (Linux/macOS/Windows)

**Acceptance Criteria:**
- All text meets contrast requirements
- Tab navigation reaches all elements
- High contrast mode available
- Font scales 80%-200%
- No visual bugs on any platform

---

## 9. User Research Validation

### 9.1 Comparison Matrix

| Feature | Discord | Slack | Teams | Telegram | **slirc (Target)** |
|---------|---------|-------|-------|----------|-------------------|
| **Menu Bar** | ❌ | ✅ Desktop | ❌ | ✅ Desktop | ✅ All platforms |
| **3-Column Layout** | ✅ | ✅ | ✅ | ❌ | ✅ (2.5 col) |
| **Message Grouping** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Avatars** | ✅ Images | ✅ Images | ✅ Images | ✅ Images | ✅ Identicons |
| **Quick Switcher** | ✅ Ctrl+K | ✅ Ctrl+K | ✅ Ctrl+E | ✅ Ctrl+K | ✅ Ctrl+K |
| **Dark Theme** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Keyboard Nav** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Unread Badges** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **@Mentions** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Font Size** | 14-16px | 15px | 14px | 15px | **16px** |
| **Line Height** | 1.375 | 1.46 | 1.43 | 1.5 | **1.5** |

### 9.2 Meets & Exceeds Analysis

**✅ Meets Standards:**
- Modern color hierarchy (7 surface levels)
- Typography (16px base, proper line height)
- Keyboard shortcuts (matches Discord/Slack conventions)
- Message grouping and visual hierarchy
- Unread/mention indicators

**🚀 Exceeds Standards:**
- **Traditional menu bar:** More discoverable than Discord/Teams
- **IRC-specific features:** Optimized for text, not multimedia
- **Lightweight:** Native Rust (egui), not Electron
- **Open source:** Full transparency and customization
- **Cross-platform:** Linux/macOS/Windows with single codebase

**📊 User-First Wins:**
1. **Progressive disclosure:** Menu bar for discovery, shortcuts for efficiency
2. **Familiar patterns:** Ctrl+K, Ctrl+W, Ctrl+J match user expectations
3. **Accessibility:** WCAG AA compliance from day one
4. **Performance:** 60fps rendering, instant startup
5. **Privacy:** Local-first, no telemetry

---

## 10. Open Questions & Decisions Needed

### 10.1 Design Decisions

**Q1: Emoji Support?**
- **Option A:** Full emoji rendering (Noto Emoji font, ~10MB)
- **Option B:** Text-only emoji (`:smile:` → `:smile:`)
- **Option C:** Hybrid (common emoji rendered, rare ones as text)
- **Recommendation:** Option C

**Q2: Animations?**
- **Option A:** No animations (instant state changes)
- **Option B:** Subtle animations (120-200ms easing)
- **Option C:** Full animations with preferences toggle
- **Recommendation:** Option C (default enabled)

**Q3: Message Density?**
- **Option A:** Cozy (avatar + spacing, like Discord)
- **Option B:** Compact (traditional IRC, tight spacing)
- **Option C:** User preference toggle
- **Recommendation:** Option C (default Cozy)

### 10.2 Technical Decisions

**Q4: Font Bundling Strategy?**
- **Option A:** Always bundle Inter + JetBrains Mono (~1MB)
- **Option B:** Try system fonts first, bundle as fallback
- **Option C:** Optional download on first run
- **Recommendation:** Option A (consistency)

**Q5: Theme Customization?**
- **Option A:** Fixed themes (Dark/Light only)
- **Option B:** Custom color picker for all colors
- **Option C:** Preset themes + accent color picker
- **Recommendation:** Option C

**Q6: Window Management?**
- **Option A:** Single window, tabs for servers
- **Option B:** Multiple windows, one per server
- **Option C:** User choice (preferences)
- **Recommendation:** Option A (simpler)

---

## 11. Success Metrics

### 11.1 Quantitative Goals

- **Performance:** Maintain 60fps with 10k+ messages
- **Accessibility:** 100% keyboard navigable, WCAG AA contrast
- **Cross-platform:** Zero visual bugs on Linux/macOS/Windows
- **File size:** < 15MB binary (including fonts)
- **Startup time:** < 500ms to first render

### 11.2 Qualitative Goals

- **Discoverability:** New users find features without docs
- **Efficiency:** Power users access functions via keyboard
- **Aesthetics:** "Looks like a modern app, not legacy software"
- **Familiarity:** "Feels like Discord/Slack but for IRC"
- **Trust:** Open source, privacy-respecting, lightweight

---

## 12. References

### 12.1 Research Sources

- Discord Desktop App (v0.0.315+)
- Slack Desktop (v4.35+)
- Microsoft Teams (v1.6+)
- Telegram Desktop (v4.12+)
- Element Desktop (v1.11+)
- HexChat (v2.16+) - traditional IRC reference

### 12.2 Design Systems

- Discord Design Language (unofficial analysis)
- Slack Design Guidelines
- Microsoft Fluent 2
- Material Design 3
- Apple Human Interface Guidelines

### 12.3 Standards

- WCAG 2.1 Level AA
- W3C Keyboard Navigation
- Platform Menu Bar Guidelines (macOS HIG, Windows UX)

---

## Appendix A: Font Download Instructions

```bash
# Create fonts directory
mkdir -p fonts

# Download Inter (v4.0)
cd fonts
wget https://github.com/rsms/inter/releases/download/v4.0/Inter-4.0.zip
unzip Inter-4.0.zip
mv "Inter Desktop/"*.ttf .
rm -rf "Inter Desktop/" Inter-4.0.zip

# Download JetBrains Mono (v2.304)
wget https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip
unzip JetBrainsMono-2.304.zip
mv fonts/ttf/JetBrainsMono-*.ttf .
rm -rf fonts/ JetBrainsMono-2.304.zip

# Verify
ls -lh *.ttf
# Should see: Inter-*.ttf and JetBrainsMono-*.ttf
```

---

## Appendix B: egui Style Configuration

```rust
// Complete egui visual style setup
fn configure_egui_style(ctx: &egui::Context, theme: &SlircTheme) {
    let mut style = (*ctx.style()).clone();
    
    // Spacing (8pt grid)
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.button_padding = vec2(12.0, 6.0);
    style.spacing.window_margin = Margin::same(8.0);
    style.spacing.menu_margin = Margin::same(8.0);
    style.spacing.indent = 20.0;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_outer_margin = 0.0;
    
    // Rounding (modern, subtle)
    style.visuals.window_rounding = 8.0.into();
    style.visuals.menu_rounding = 6.0.into();
    style.visuals.widgets.noninteractive.rounding = 4.0.into();
    style.visuals.widgets.inactive.rounding = 4.0.into();
    style.visuals.widgets.hovered.rounding = 4.0.into();
    style.visuals.widgets.active.rounding = 4.0.into();
    
    // Colors
    style.visuals.dark_mode = matches!(theme.name.as_str(), "Dark");
    style.visuals.override_text_color = Some(theme.text_primary);
    style.visuals.panel_fill = theme.surface[1];
    style.visuals.window_fill = theme.surface[0];
    style.visuals.extreme_bg_color = theme.surface[0];
    style.visuals.faint_bg_color = theme.surface[2];
    
    // Widget colors
    style.visuals.widgets.noninteractive.bg_fill = theme.surface[1];
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text_secondary);
    
    style.visuals.widgets.inactive.bg_fill = theme.surface[2];
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text_secondary);
    
    style.visuals.widgets.hovered.bg_fill = theme.surface[3];
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.text_primary);
    
    style.visuals.widgets.active.bg_fill = theme.surface[4];
    style.visuals.widgets.active.fg_stroke = Stroke::new(2.0, theme.accent);
    
    // Selection
    style.visuals.selection.bg_fill = theme.accent.linear_multiply(0.3);
    style.visuals.selection.stroke = Stroke::new(1.0, theme.accent);
    
    // Hyperlinks
    style.visuals.hyperlink_color = theme.info;
    
    // Apply
    ctx.set_style(style);
}
```

---

**End of Document**

*This plan represents a comprehensive, research-backed approach to modernizing slirc-client's UI/UX while maintaining IRC's efficiency and adding contemporary chat application standards.*
