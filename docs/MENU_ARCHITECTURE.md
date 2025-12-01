# Menu Bar & Keyboard Shortcuts Architecture

**Date:** November 30, 2025
**Status:** Phase 2 Complete
**Related:** MODERN_UI_DESIGN_PLAN.md, AUDIT_AND_FORWARD_PATH.md

---

## Overview

The slirc-client menu system implements a traditional horizontal menu bar inspired by modern chat applications (Discord, Slack, Teams) while respecting desktop platform conventions. The architecture emphasizes:

- **Discoverability**: Traditional menu bar for new users
- **Efficiency**: Keyboard shortcuts for power users
- **Consistency**: Centralized shortcut registry (single source of truth)
- **Accessibility**: Help overlay (Ctrl+/) for learning shortcuts

**Key Components:**
- `src/ui/menu.rs` (303 lines): Menu rendering and MenuAction pattern
- `src/ui/shortcuts.rs` (332 lines): Centralized keyboard shortcut registry
- `src/app.rs`: Integration and action handling

---

## Architecture Components

### MenuAction Pattern

The menu system uses an enum-based action pattern to decouple UI rendering from business logic:

```rust
// src/ui/menu.rs
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    NetworkManager,
    Help,
    ChannelBrowser,
}

// Menu rendering returns Option<MenuAction>
pub fn render_menu_bar(...) -> Option<MenuAction> {
    let mut menu_action: Option<MenuAction> = None;

    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Network Manager...").clicked() {
                menu_action = Some(MenuAction::NetworkManager);
                ui.close_menu();
            }
        });
    });

    menu_action
}
```

**Benefits:**
- Type-safe action dispatch
- Clear separation of concerns (rendering vs. handling)
- Easy to test and extend
- No callback hell

### ShortcutRegistry

Centralized keyboard shortcut definitions with category organization:

```rust
// src/ui/shortcuts.rs
pub struct ShortcutRegistry {
    shortcuts: Vec<Shortcut>,
}

pub struct Shortcut {
    pub category: ShortcutCategory,
    pub key_text: &'static str,
    pub description: &'static str,
    pub action_id: &'static str,
}

pub enum ShortcutCategory {
    File,
    Edit,
    View,
    Server,
    Window,
    Navigation,
}
```

**Benefits:**
- Single source of truth for all shortcuts
- Automatic help overlay generation
- Easy to maintain consistency
- Prevents duplicate key bindings

---

## Menu Structure

### File Menu
```
New Connection...         Ctrl+N    (future)
─────────────────
Open Logs...              Ctrl+O    (future)
Save Chat Log...          Ctrl+S    (future)
─────────────────
Preferences...            Ctrl+,    (future)
─────────────────
Quit                      Ctrl+Q    (future)
```

### Edit Menu
```
Cut                       Ctrl+X
Copy                      Ctrl+C
Paste                     Ctrl+V
Select All                Ctrl+A
```

### View Menu
```
Quick Switcher            Ctrl+K
─────────────────
✓ Show Channel List       Ctrl+B
✓ Show User List          Ctrl+U
─────────────────
Theme                     ▸
  ● Dark
  ○ Light
─────────────────
Toggle Full Screen        Ctrl+Shift+F
```

### Server Menu (IRC-specific)
```
Join Channel...           Ctrl+J    (future)
Part Channel              Ctrl+W    (future)
─────────────────
List Channels...          Ctrl+L    (future)
─────────────────
Disconnect                Ctrl+D    (future)
Reconnect                 Ctrl+R    (future)
```

### Window Menu
```
Minimize                  Ctrl+M
Close Window              Ctrl+W    (future)
─────────────────
Bring All to Front
```

### Help Menu
```
Keyboard Shortcuts        Ctrl+/ or F1
IRC Commands...           (future)
─────────────────
About slirc               (future)
```

---

## Integration Points

### 1. Import in app.rs (line 24)

```rust
use crate::ui::shortcuts::ShortcutRegistry;
```

### 2. Struct Fields (lines 60-61)

```rust
pub struct SlircApp {
    // ... existing fields

    // Keyboard shortcuts registry
    pub shortcuts: ShortcutRegistry,
    pub show_shortcuts_help: bool,
}
```

### 3. Initialization in new() (lines 124-125)

```rust
impl SlircApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // ... existing initialization

        Self {
            // ... other fields
            shortcuts: ShortcutRegistry::new(),
            show_shortcuts_help: false,
        }
    }
}
```

### 4. Keyboard Handlers in update() (lines 289-307)

```rust
impl eframe::App for SlircApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ... existing update logic

        ctx.input(|i| {
            // Ctrl+/ or F1: Toggle shortcuts help overlay
            if (i.modifiers.ctrl && i.key_pressed(egui::Key::Slash))
                || i.key_pressed(egui::Key::F1)
            {
                self.show_shortcuts_help = !self.show_shortcuts_help;
            }

            // Ctrl+M: Minimize window
            if i.modifiers.ctrl && i.key_pressed(egui::Key::M) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }

            // Ctrl+Shift+F: Toggle fullscreen
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::F) {
                let current = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!current));
            }

            // Ctrl+B: Toggle channel list
            if i.modifiers.ctrl && i.key_pressed(egui::Key::B) {
                self.show_channel_list = !self.show_channel_list;
            }
        });
    }
}
```

### 5. Overlay Rendering (line 373)

```rust
// Shortcuts help overlay (Ctrl+/ or F1)
self.shortcuts.render_help_overlay(ctx, &mut self.show_shortcuts_help);
```

### 6. MenuAction Handling (line 481)

```rust
match menu_action {
    Some(ui::menu::MenuAction::NetworkManager) => {
        self.dialogs.open_network_manager(self.state.networks.clone());
    }
    Some(ui::menu::MenuAction::Help) => {
        self.show_shortcuts_help = true;
    }
    Some(ui::menu::MenuAction::ChannelBrowser) => {
        self.dialogs.open_channel_browser();
    }
    None => {}
}
```

---

## ShortcutRegistry API

### new() - Initialize Registry

```rust
let registry = ShortcutRegistry::new();
```

Creates a new registry with all 20 shortcuts across 6 categories.

### by_category() - Get Shortcuts by Category

```rust
let file_shortcuts = registry.by_category(ShortcutCategory::File);
for shortcut in file_shortcuts {
    println!("{}: {}", shortcut.key_text, shortcut.description);
}
```

Returns a vector of shortcuts filtered by category.

### find() - Find Shortcut by Action ID

```rust
if let Some(shortcut) = registry.find("file.connect") {
    println!("Connect shortcut: {}", shortcut.key_text);
}
```

Returns `Option<&Shortcut>` for a specific action ID.

### render_help_overlay() - Display Help Dialog

```rust
// In app.rs update() method
self.shortcuts.render_help_overlay(ctx, &mut self.show_shortcuts_help);
```

Renders a centered modal window with all shortcuts organized by category. Automatically called from the main update loop.

---

## Adding New Shortcuts

### Step 1: Add to ShortcutRegistry::new()

Edit `src/ui/shortcuts.rs`:

```rust
impl ShortcutRegistry {
    pub fn new() -> Self {
        let shortcuts = vec![
            // ... existing shortcuts

            // Add your new shortcut
            Shortcut {
                category: ShortcutCategory::File,
                key_text: "Ctrl+N",
                description: "New connection",
                action_id: "file.connect",
            },
        ];

        Self { shortcuts }
    }
}
```

### Step 2: Add Keyboard Handler in app.rs

Edit `src/app.rs` in the `update()` method:

```rust
ctx.input(|i| {
    // ... existing handlers

    // Ctrl+N: New connection dialog
    if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
        self.dialogs.open_network_manager(self.state.networks.clone());
    }
});
```

### Step 3: Add Menu Item in menu.rs

Edit `src/ui/menu.rs`:

```rust
ui.menu_button("File", |ui| {
    ui.horizontal(|ui| {
        if ui.button("New Connection...").clicked() {
            menu_action = Some(MenuAction::NetworkManager);
            ui.close_menu();
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new("Ctrl+N")
                    .size(11.0)
                    .color(ui.visuals().weak_text_color())
            );
        });
    });
});
```

### Step 4: Test

1. Compile: `cargo build`
2. Run: `cargo run`
3. Test keyboard shortcut: Press Ctrl+N
4. Test menu: Click File → New Connection...
5. Test help overlay: Press Ctrl+/, verify shortcut appears

---

## Design Decisions

### Why MenuAction Enum Over Callbacks?

**Considered Alternatives:**
- Direct callbacks: `on_clicked(|| { ... })`
- Event bus pattern: `event_tx.send(Event::Connect)`

**Chosen: MenuAction Enum**

**Reasons:**
1. **Type Safety**: Compiler ensures all actions are handled
2. **Clarity**: Clear data flow (menu → action → handler)
3. **Testing**: Easy to test action dispatch without UI
4. **Debugging**: Actions visible in debugger
5. **Refactoring**: Find all usages of an action

**Trade-off**: Requires match statement in app.rs, but this is minimal boilerplate for significant benefits.

### Why Centralized Registry?

**Considered Alternatives:**
- Shortcuts defined inline where used
- Per-component shortcut definitions
- Configuration file-based shortcuts

**Chosen: Centralized ShortcutRegistry**

**Reasons:**
1. **Single Source of Truth**: One place to see all shortcuts
2. **Prevent Conflicts**: Easy to detect duplicate key bindings
3. **Help Overlay**: Automatic generation from registry
4. **Consistency**: Ensures uniform naming and categorization
5. **Documentation**: Self-documenting via the registry

**Trade-off**: Adds indirection (registry → handler), but worth it for maintainability.

### Help Overlay UX

**Design Choices:**
- **Trigger**: Ctrl+/ (universal "help" shortcut) + F1 (traditional help key)
- **Layout**: Centered modal window (not sidebar) for focus
- **Organization**: 6 categories in 2-column grid
- **Dismissal**: Click outside, press Esc, or toggle Ctrl+/ again
- **Styling**: Matches theme, high contrast for readability

**Inspiration**: Discord (Ctrl+/), Slack (Ctrl+/), VS Code (Ctrl+K Ctrl+S)

---

## Code Examples

### Complete Example: Adding "Reconnect" Shortcut

**1. Update ShortcutRegistry** (`src/ui/shortcuts.rs`):

```rust
Shortcut {
    category: ShortcutCategory::Server,
    key_text: "Ctrl+R",
    description: "Reconnect to server",
    action_id: "server.reconnect",
},
```

**2. Add Keyboard Handler** (`src/app.rs`):

```rust
// In update() method, inside ctx.input(|i| { ... })
if i.modifiers.ctrl && i.key_pressed(egui::Key::R) {
    if !self.state.is_connected {
        self.do_connect();
    }
}
```

**3. Add Menu Item** (`src/ui/menu.rs`):

```rust
ui.menu_button("Server", |ui| {
    // ... other items

    ui.horizontal(|ui| {
        if ui.add_enabled(!is_connected, egui::Button::new("Reconnect")).clicked() {
            action_tx.send(BackendAction::Connect {
                server: /* ... */,
                nickname: /* ... */,
                use_tls: /* ... */,
            }).ok();
            ui.close_menu();
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new("Ctrl+R")
                    .size(11.0)
                    .color(ui.visuals().weak_text_color())
            );
        });
    });
});
```

### Complete Example: Handling MenuAction

```rust
// In app.rs, after render_menu_bar() call
if let Some(menu_action) = ui::menu::render_menu_bar(/* ... */) {
    match menu_action {
        ui::menu::MenuAction::NetworkManager => {
            self.dialogs.open_network_manager(self.state.networks.clone());
        }
        ui::menu::MenuAction::Help => {
            self.show_shortcuts_help = true;
        }
        ui::menu::MenuAction::ChannelBrowser => {
            self.dialogs.open_channel_browser();
        }
    }
}
```

---

## Testing Guidelines

### Unit Tests

Shortcuts are tested in `src/ui/shortcuts.rs`:

```rust
#[test]
fn test_registry_initialization() {
    let registry = ShortcutRegistry::new();
    assert!(!registry.all().is_empty());
}

#[test]
fn test_find_shortcut() {
    let registry = ShortcutRegistry::new();
    let shortcut = registry.find("file.connect");
    assert!(shortcut.is_some());
    assert_eq!(shortcut.unwrap().key_text, "Ctrl+N");
}
```

### Integration Tests

Test keyboard shortcuts in running application:

1. Launch app: `cargo run`
2. Press shortcut (e.g., Ctrl+/)
3. Verify expected action occurs
4. Check help overlay lists the shortcut

### Checklist for New Shortcuts

- [ ] Added to ShortcutRegistry::new()
- [ ] Keyboard handler in app.rs update()
- [ ] Menu item in menu.rs (if applicable)
- [ ] Help text matches key_text exactly
- [ ] No conflicts with existing shortcuts
- [ ] Tested in running application
- [ ] Help overlay displays correctly
- [ ] Works in both dark and light themes

---

## Future Enhancements

### Customizable Shortcuts

Allow users to remap shortcuts via preferences:

```rust
// Future: src/config.rs
pub struct ShortcutConfig {
    pub action_id: String,
    pub key_combination: String,
}

// User can override defaults
let custom_shortcuts = load_shortcut_config();
registry.override_shortcuts(custom_shortcuts);
```

### Shortcut Conflicts Detection

Automatically detect and warn about conflicts:

```rust
impl ShortcutRegistry {
    pub fn validate(&self) -> Vec<String> {
        // Return list of conflicting shortcuts
    }
}
```

### Platform-Specific Shortcuts

Handle Cmd on macOS, Ctrl on Windows/Linux:

```rust
#[cfg(target_os = "macos")]
const MODIFIER: &str = "Cmd";

#[cfg(not(target_os = "macos"))]
const MODIFIER: &str = "Ctrl";
```

---

## References

- **MODERN_UI_DESIGN_PLAN.md**: Menu structure specification
- **AUDIT_AND_FORWARD_PATH.md**: Phase 2 implementation roadmap
- **egui documentation**: https://docs.rs/egui/latest/egui/
- **Discord keyboard shortcuts**: Ctrl+/ for help
- **Slack keyboard shortcuts**: Ctrl+/ for help
- **VS Code keyboard shortcuts**: Ctrl+K Ctrl+S for shortcuts panel

---

**End of Documentation**
