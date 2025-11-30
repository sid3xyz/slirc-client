# Copilot Instructions for slirc-client

Modern, native IRC client built with [egui](https://github.com/emilk/egui) and [slirc-proto](https://github.com/sid3xyz/slirc-proto). Released to the public domain under [The Unlicense](../LICENSE).

## Quick Reference

```bash
cargo build --release         # Build release binary
cargo test                    # Run all tests (75 passing)
cargo clippy -- -D warnings   # Lint (zero warnings policy)
cargo fmt -- --check          # Format check
cargo run -- --help           # Run with help
```

## Project Constraints

| Constraint | Requirement |
|------------|-------------|
| MSRV | Rust 1.70+ |
| Error handling | Use `?` propagation, avoid `unwrap()` in production code |
| Testing | Maintain ≥75 tests, write tests for new features |
| UI Framework | egui 0.31 with eframe (native, not web) |
| Protocol | Use `slirc-proto` for all IRC parsing/serialization |
| Async Runtime | Tokio for backend thread only, not in UI |

## Architecture

### Dual-Thread Design

```
┌─────────────────┐         ┌──────────────────────┐
│   Main Thread   │         │   Backend Thread     │
│   (egui UI)     │◄───────►│   (Tokio Runtime)    │
│   Synchronous   │ channels│   Async Network I/O  │
└─────────────────┘         └──────────────────────┘
```

| Component | Location | Pattern |
|-----------|----------|---------|
| **UI Layer** | `src/app.rs`, `src/ui/` | egui immediate mode, state in `SlircApp` |
| **Backend** | `src/backend.rs` | Tokio runtime, handles IRC connections |
| **Communication** | `crossbeam-channel` | `AppEvent` (UI→Backend), `BackendEvent` (Backend→UI) |
| **State** | `src/buffer.rs`, `src/config.rs` | Buffers for channels/PMs, network config |
| **Protocol** | `src/protocol.rs` | Wrapper around `slirc-proto` types |
| **Commands** | `src/commands.rs` | IRC command parsing (/join, /msg, etc.) |
| **Formatting** | `src/ui/messages.rs` | mIRC color codes, bold, italic rendering |
| **Logging** | `src/logging.rs` | File-based chat logs in XDG_DATA_HOME |

### Key Files

- **`src/main.rs`** - Entry point, eframe setup
- **`src/app.rs`** - Main application state, UI orchestration
- **`src/backend.rs`** - Async network I/O, connection management
- **`src/buffer.rs`** - Message buffer abstraction (channels, PMs, system)
- **`src/ui/mod.rs`** - UI modules (panels, messages, dialogs, toolbar, theme)
- **`src/events.rs`** - Event types for UI↔Backend communication
- **`src/config.rs`** - Network configuration, persistence
- **`src/validation.rs`** - RFC 2812 validation (nicks, channels)

## Coding Standards

### Error Handling

```rust
// ✅ Good: Propagate errors with ?
pub fn connect_to_server(config: &NetworkConfig) -> Result<Connection, Error> {
    let stream = TcpStream::connect(&config.address)?;
    let transport = setup_transport(stream)?;
    Ok(Connection::new(transport))
}

// ❌ Bad: Using unwrap() in production code
let stream = TcpStream::connect(&config.address).unwrap(); // Never do this

// ✅ Good: Handle UI errors gracefully
if let Err(e) = self.backend_tx.send(event) {
    self.add_system_message(&format!("Backend error: {}", e));
}
```

### UI State Management

```rust
// ✅ Good: State in SlircApp, no globals
pub struct SlircApp {
    buffers: HashMap<String, Buffer>,
    active_buffer: Option<String>,
    config: Config,
    // ...
}

// ❌ Bad: Mutable static state
static mut GLOBAL_STATE: Option<State> = None; // Avoid
```

### Async vs Sync Boundary

```rust
// ✅ Good: Async in backend thread only
async fn handle_connection(mut transport: ZeroCopyTransport<TlsStream>) {
    while let Some(msg) = transport.next().await {
        // Process message
    }
}

// ✅ Good: Sync in UI thread
impl SlircApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // No async/await here, use channels
        self.handle_backend_events();
        self.render_ui(ctx);
    }
}
```

### Testing Patterns

```rust
// ✅ Good: Test public APIs
#[test]
fn test_join_command_parsing() {
    let cmd = parse_command("/join #rust").unwrap();
    assert_eq!(cmd, Command::Join("#rust".to_string()));
}

// ✅ Good: Integration tests
#[tokio::test]
async fn test_tls_connection() {
    let config = NetworkConfig {
        address: "irc.libera.chat:6697".to_string(),
        use_tls: true,
        // ...
    };
    let result = connect_with_tls(&config).await;
    assert!(result.is_ok());
}
```

## UI/UX Guidelines

### Design System (See `/docs/MODERN_UI_DESIGN_PLAN.md`)

**Active Design Migration:**
- **Target:** Modern chat UI (Discord/Slack-inspired)
- **Layout:** 2.5-column (Sidebar + Chat + Optional User List)
- **Typography:** Inter + JetBrains Mono (16px base font)
- **Colors:** 7-level surface hierarchy, semantic colors
- **Menu Bar:** Traditional horizontal menu (File/Edit/View/Server/Help)

**Implementation Status:**
- ✅ Phase 0: Basic UI with egui
- 🚧 Phase 1: Foundation (fonts, theme system) - **IN PROGRESS**
- ⏳ Phase 2: Top menu bar
- ⏳ Phase 3-7: Sidebar, messages, user list, quick switcher, polish

### Current UI Components

| Component | File | Description |
|-----------|------|-------------|
| Main window | `ui/mod.rs` | Layout orchestration |
| Sidebar | `ui/panels.rs` | Buffer/channel list |
| Messages | `ui/messages.rs` | Chat rendering with mIRC formatting |
| Input | `ui/panels.rs` | Message input with history |
| Dialogs | `ui/dialogs.rs` | Network manager, join, preferences |
| Toolbar | `ui/toolbar.rs` | Connect/disconnect, quick actions |
| Theme | `ui/theme.rs` | Color scheme, dark/light mode |

### egui Best Practices

```rust
// ✅ Good: Use egui's layout helpers
ui.horizontal(|ui| {
    ui.label("Nick:");
    ui.text_edit_singleline(&mut self.nick);
});

// ✅ Good: Handle input validation
let response = ui.text_edit_singleline(&mut self.channel_input);
if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
    if validate_channel_name(&self.channel_input) {
        self.join_channel();
    }
}

// ✅ Good: Maintain state between frames
ui.ctx().request_repaint_after(Duration::from_secs(1));
```

## Feature Implementation Checklist

When adding new features:

- [ ] Update `src/events.rs` if new event types needed
- [ ] Add backend handling in `src/backend.rs` for network operations
- [ ] Add UI rendering in appropriate `src/ui/*.rs` file
- [ ] Write tests (unit + integration if applicable)
- [ ] Update help text in dialogs if user-facing
- [ ] Add logging for debugging (`log::debug!`, `log::error!`)
- [ ] Validate input with `src/validation.rs` functions
- [ ] Update `README.md` if it's a major feature

## IRC Protocol Notes

### Using slirc-proto

```rust
// ✅ Good: Use slirc-proto types
use slirc_proto::{Message, Command, IrcCodec};

// Parsing (zero-copy)
let msg = MessageRef::parse(b":nick!user@host PRIVMSG #channel :hello")?;

// Serialization
let join = Message::join("#rust", None);
let mut buf = String::new();
join.write_to(&mut buf)?;

// Transport
let codec = IrcCodec::new();
let mut framed = Framed::new(stream, codec);
framed.send(message).await?;
```

### Common IRC Commands

| Command | Format | Handler |
|---------|--------|---------|
| JOIN | `/join #channel [key]` | `commands::parse_command()` |
| PART | `/part [#channel]` | Uses active buffer if no channel |
| PRIVMSG | `/msg target message` | For PMs |
| ACTION | `/me does something` | CTCP ACTION |
| NICK | `/nick newnick` | Change nickname |
| TOPIC | `/topic [new topic]` | Set/view channel topic |
| QUIT | `/quit [reason]` | Disconnect |

## Testing Requirements

### Coverage Targets

| Module | Current | Target | Notes |
|--------|---------|--------|-------|
| `events.rs` | 93.4% | 95%+ | Test event serialization |
| `commands.rs` | 71.7% | 85%+ | Test all IRC commands |
| `validation.rs` | 96.3% | 95%+ | Already excellent |
| `ui/messages.rs` | High | 90%+ | Test mIRC formatting |
| `backend.rs` | Integration | N/A | Tested via integration tests |

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test commands::tests

# Integration tests only
cargo test --test integration_tests

# With output
cargo test -- --nocapture

# Coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

## Dependencies

### Core Dependencies

- **eframe** (0.31) - egui framework for native apps
- **tokio** (1.x) - Async runtime (backend thread only)
- **slirc-proto** - IRC protocol library (our own)
- **crossbeam-channel** - Thread-safe channels for UI↔Backend
- **tokio-rustls** - TLS support for encrypted connections
- **keyring** - Secure password storage (system keyring)

### Feature Flags

Currently, all features are compiled by default. Future:
- `tls` - TLS/SSL support (may become optional)
- `keyring` - Secure password storage (may become optional)

## Security Considerations

- **TLS:** Always validate certificates, use `tokio-rustls` + `webpki-roots`
- **Passwords:** Store in system keyring via `keyring` crate, never in plain text config
- **Input validation:** Use `validation.rs` for all user input (nicks, channels)
- **Command injection:** Parse commands with `commands.rs`, never pass raw strings to shell

## Performance Tips

- **Buffer rendering:** Use `egui::ScrollArea` with `max_height()` to limit redraws
- **Message history:** Limit displayed messages (e.g., last 1000) per buffer
- **Backend events:** Batch process events in `handle_backend_events()`
- **Logging:** Use `log::debug!` liberally, control with `RUST_LOG=debug`

## Common Patterns

### Adding a New IRC Command

1. Add enum variant to `Command` in `src/commands.rs`
2. Update `parse_command()` with regex for `/newcommand`
3. Add `AppEvent::SendCommand` variant if needed
4. Handle in `backend.rs` → send to IRC server
5. Write test in `src/commands.rs`

### Adding a New UI Dialog

1. Add boolean flag to `SlircApp` (e.g., `show_my_dialog`)
2. Create rendering function in `src/ui/dialogs.rs`
3. Call from `app.rs` when flag is true
4. Add menu item or button to toggle flag

### Adding a New Buffer Type

1. Extend `BufferType` in `src/buffer.rs`
2. Update `Buffer::new()` constructor
3. Add rendering logic in `src/ui/messages.rs`
4. Handle creation in `backend.rs` when events occur

## Debugging

### Enable Logging

```bash
RUST_LOG=debug cargo run
RUST_LOG=slirc_client=trace cargo run  # More verbose
```

### Useful Log Points

- `backend.rs`: Connection lifecycle, message send/receive
- `app.rs`: Event processing, state changes
- `commands.rs`: Command parsing
- `ui/messages.rs`: Message rendering, formatting

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| UI freezes | Blocking call in main thread | Move to backend with `backend_tx.send()` |
| Messages not showing | Not polling `backend_rx` | Check `handle_backend_events()` |
| TLS fails | Certificate validation | Check time sync, use `webpki-roots` |
| Keyring errors | No keyring on system | Handle gracefully, store in config as fallback |

## Release Checklist

Before releasing:

- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt`
- [ ] Update version in `Cargo.toml`
- [ ] Update `README.md` with new features
- [ ] Test on Linux (primary target)
- [ ] Build release binary: `cargo build --release`
- [ ] Tag release: `git tag v0.x.0`

## Resources

- **slirc-proto docs:** https://github.com/sid3xyz/slirc-proto
- **egui docs:** https://docs.rs/egui/
- **eframe docs:** https://docs.rs/eframe/
- **IRC RFCs:** RFC 1459, RFC 2812
- **Design docs:** `/docs/MODERN_UI_DESIGN_PLAN.md`, `/LAYOUT_SPECIFICATION.md`

---

**Remember:** This is a native desktop app. Keep the UI thread responsive, do all I/O in the backend thread, and use channels for communication. Test thoroughly, handle errors gracefully, and maintain the zero-unwrap policy in production code.
