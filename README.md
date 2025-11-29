# SLIRC Client

A modern, native IRC client built with [egui](https://github.com/emilk/egui) and the [slirc-proto](https://github.com/sid3xyz/slirc-proto) protocol library.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-Unlicense-blue)
![Tests](https://img.shields.io/badge/tests-69%20passing-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-21.23%25-yellow)

## Features

### Core Features
- **🔒 TLS/SSL Support** - Secure encrypted connections (tested with Libera.Chat, OFTC, etc.)
- **🌐 Native GUI** - Cross-platform desktop application using egui/glow
- **📋 Multi-buffer Interface** - Separate buffers for channels, private messages, and system log
- **👥 User Lists** - Live user list for joined channels with prefix indicators (@, +, etc.)
- **📝 Topic Display** - Shows and allows editing of channel topics
- **💾 Network Manager** - Save and manage multiple IRC network configurations
- **🔑 Secure Password Storage** - NickServ passwords stored in system keyring
- **⚡ Quick Connect** - One-click presets for Libera.Chat, OFTC, EFnet, Rizon

### User Experience
- **Timestamps** - All messages displayed with local time
- **Input History** - Up/Down arrows to navigate previous commands
- **Channel Tabs** - Vertical buffer list with unread badges
- **Mention Highlights** - Messages mentioning your nick are highlighted
- **Input Validation** - RFC 2812 compliant channel and nickname validation
- **Command Completion** - Full IRC command support (/join, /part, /msg, /me, /nick, etc.)

### Technical Excellence
- **69 Passing Tests** - Comprehensive test coverage (21.23% overall)
  - Events: 93.4% coverage
  - Commands: 71.7% coverage
  - Validation: 96.3% coverage
- **Zero Unwraps** - Production code uses proper error handling
- **Integration Tested** - TLS, network manager, and protocol handling
- **Modern Rust** - Clean, idiomatic code with async/await

## Screenshots

*Coming soon - GUI screenshots showing network manager, TLS connection, and multi-buffer interface*

## Development

### Architecture

SLIRC Client uses a **dual-thread architecture** to bridge the async network layer with the synchronous GUI:

```
┌─────────────────────────────────────────────────────────────┐
│                      Main Thread (GUI)                       │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                    egui Application                      │ │
│  │  - Renders UI (buffers, user list, input)               │ │
│  │  - Handles user input                                    │ │
│  │  - Consumes GuiEvents each frame                        │ │
│  └─────────────────────────────────────────────────────────┘ │
│         │ BackendAction                    ▲ GuiEvent        │
│         ▼ (crossbeam channel)              │ (crossbeam)     │
└─────────────────────────────────────────────────────────────┘
                          │                  ▲
                          ▼                  │
┌─────────────────────────────────────────────────────────────┐
│                   Backend Thread (Network)                   │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                   Tokio Runtime                          │ │
│  │  - Owns Transport (TCP connection)                      │ │
│  │  - Reads/writes IRC messages                            │ │
│  │  - Parses commands using slirc-proto                    │ │
│  │  - Handles PING/PONG automatically                      │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Communication Protocol

**BackendAction** (UI → Backend):
- `Connect { server, port, nickname, username, realname }` - Initiate connection
- `Disconnect` - Close connection gracefully
- `Join(channel)` - Join an IRC channel
- `Part(channel)` - Leave an IRC channel
- `SendMessage { target, text }` - Send PRIVMSG to channel/user

**GuiEvent** (Backend → UI):
- `Connected` - Registration complete (RPL_WELCOME received)
- `Disconnected(reason)` - Connection closed
- `Error(message)` - Error occurred
- `MessageReceived { target, sender, text }` - PRIVMSG/NOTICE received
- `JoinedChannel(channel)` - Successfully joined a channel
- `PartedChannel(channel)` - Left a channel
- `UserJoined { channel, nick }` - Another user joined
- `UserParted { channel, nick, message }` - Another user left
- `RawMessage(line)` - Raw IRC protocol line (for System log)
- `Motd(line)` - Message of the Day line
- `Topic { channel, topic }` - Channel topic received
- `Names { channel, names }` - Channel user list received

### Why This Architecture?

1. **egui runs on the main thread** - It's a synchronous immediate-mode GUI that redraws every frame
2. **slirc-proto uses Tokio** - The `Transport` type requires an async runtime for network I/O
3. **Lock-free communication** - `crossbeam-channel` provides efficient, non-blocking message passing between threads

This separation ensures:
- The GUI never blocks on network operations
- Network events are processed asynchronously
- Clean separation of concerns between UI and protocol handling

## Dependencies

| Crate | Purpose |
|-------|---------|
| `eframe` | egui framework with native windowing (glow backend) |
| `tokio` | Async runtime for network operations |
| `slirc-proto` | IRC protocol parsing and transport |
| `crossbeam-channel` | Lock-free channels for thread communication |
| `tokio-rustls` | TLS/SSL encryption support |
| `rustls` | Modern TLS implementation |
| `keyring` | Secure password storage in system keyring |
| `serde` | Configuration serialization |

## Building

### Prerequisites

- Rust 1.70 or later
- Linux: X11 or Wayland development libraries

```bash
# Ubuntu/Debian
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libssl-dev

# Fedora
sudo dnf install libxcb-devel libxkbcommon-devel openssl-devel
```

### Build & Run

```bash
# Clone with slirc-proto as sibling directory
git clone https://github.com/sid3xyz/slirc-proto.git
git clone https://github.com/sid3xyz/slirc-client.git

# Build and run
cd slirc-client
cargo run --release
```

## Quick Start

### First-Time Setup

1. Launch SLIRC Client
2. Click the **"Networks"** button in the toolbar
3. Use **Quick Add** to add a popular network (e.g., "Libera.Chat")
4. Click **"Connect"** on your chosen network
5. Watch the System buffer for "✓ Connected and registered!"
6. Use `/join #channel` to join channels

### Manual Connection

1. Enter server address (e.g., `irc.libera.chat:6697`)
2. Enter your nickname
3. Check **"Use TLS"** for secure connections (recommended)
4. Click **"Connect"**

### IRC Commands

| Command | Description | Example |
|---------|-------------|----------|
| `/join <channel>` | Join a channel | `/join #rust` |
| `/part [channel]` | Leave a channel | `/part #channel goodbye!` |
| `/msg <target> <text>` | Send private message | `/msg alice hello` |
| `/nick <newnick>` | Change nickname | `/nick alice_away` |
| `/me <action>` | Send action message | `/me waves hello` |
| `/quit [message]` | Disconnect from server | `/quit See you later!` |
| `/whois <nick>` | Get user information | `/whois bob` |
| `/topic [text]` | View or set topic | `/topic Welcome!` |
| `/kick <nick> [reason]` | Kick user from channel | `/kick spammer Bye` |
| `/help` | Show available commands | `/help` |

### UI & Look-and-feel Improvements

- Your messages: Outgoing messages are left-aligned and colored to help differentiate from other users.

### Quick Checks

Try these to verify the UI features:

1. Connect to an IRC server and join `#straylight`.
2. From another client, change your nick (`/nick newnick`) and verify that the user list updates to show `newnick`.
3. Post a message in channel mentioning your nick to verify message highlight and unread increment when the channel is inactive.
4. Use the left-hand channel tabs to switch buffers and see unread counts cleared.

## Configuration

### Default Settings
- **Server**: `irc.slirc.net:6667`
- **Default Channel**: `#straylight`
- **No Auto-Connect**: Manual connection required
- **Config Location**: `~/.config/slirc-client/settings.json` (Linux)

### Network Manager
Networks are saved in `settings.json` and include:
- Server addresses (with fallback servers)
- Nickname preferences
- Auto-join channels
- TLS settings
- Auto-connect on startup option

### Secure Password Storage
NickServ passwords are stored in your system's secure keyring:
- **Linux**: libsecret (GNOME Keyring, KWallet)
- **macOS**: Keychain
- **Windows**: Credential Manager

Passwords are **never** stored in plain text configuration files.

## Security & Quality

### Security Features
- \u2705 **TLS 1.3 Support** - Modern encryption with certificate validation
- \u2705 **Secure Password Storage** - System keyring integration
- \u2705 **Input Validation** - RFC 2812 compliant sanitization
- \u2705 **No Hardcoded Secrets** - All credentials user-provided
- \u2705 **Certificate Verification** - Mozilla root CA store (webpki-roots)

### Code Quality
- \u2705 **Zero Production Unwraps** - Proper error handling throughout
- \u2705 **69 Passing Tests** - Unit and integration test coverage
- \u2705 **21.23% Test Coverage** - Core business logic well-tested
  - Events: 93.4% coverage
  - Commands: 71.7% coverage  
  - Validation: 96.3% coverage
- \u2705 **Type-Safe Protocol** - Leverages Rust's type system
- \u2705 **Memory Safe** - No unsafe code in production paths

## Project Structure

```
slirc-client/
├── Cargo.toml                 # Dependencies and project metadata
├── README.md                  # This file
├── PHASE2_REPORT.md          # Test coverage report
├── COVERAGE_IMPROVEMENT.md   # Coverage improvement details
├── TLS_LIMITATION.md         # TLS implementation notes (RESOLVED)
└── src/
    ├── main.rs               # Application entry point
    ├── app.rs                # Main application state and UI
    ├── backend.rs            # Async network I/O (Tokio runtime)
    ├── backend_tests.rs      # Backend unit tests (15 tests)
    ├── integration_tests.rs  # Integration tests (16 tests)
    ├── buffer.rs             # Channel message storage
    ├── commands.rs           # IRC command handling (71.7% coverage)
    ├── config.rs             # Settings persistence and keyring
    ├── events.rs             # Event processing (93.4% coverage)
    ├── protocol.rs           # Protocol type definitions
    ├── validation.rs         # Input validation (96.3% coverage)
    └── ui/
        ├── mod.rs            # UI module exports
        ├── dialogs.rs        # Network manager, help, topic editor
        ├── messages.rs       # Message rendering
        ├── panels.rs         # Buffer list, user list
        ├── theme.rs          # Color themes and styling
        └── toolbar.rs        # Top toolbar UI
```

## IRC Commands Handled

| Command | Handling |
|---------|----------|
| `PING` | Auto-responds with PONG |
| `001` (RPL_WELCOME) | Signals connected state |
| `332` (RPL_TOPIC) | Updates channel topic |
| `353` (RPL_NAMREPLY) | Populates user list |
| `372/375` (MOTD) | Displays in System buffer |
| `PRIVMSG` | Routes to appropriate buffer |
| `NOTICE` | Routes to appropriate buffer (sender prefixed with `-`) |
| `JOIN` | Creates buffer or adds user to list |
| `PART` | Removes buffer or user from list |
| `QUIT` | Logged (user tracking TBD) |
| `ERROR` | Displays error in System buffer |

## Completed Features (November 2025)

- ✅ **TLS/SSL Support** - Full TLS 1.3 with certificate validation
- ✅ **Network Manager** - Save/load multiple network configurations
- ✅ **Secure Password Storage** - System keyring integration
- ✅ **Input Validation** - RFC 2812 compliant validation
- ✅ **Comprehensive Testing** - 69 tests with 21% coverage
- ✅ **Configuration Persistence** - JSON-based settings storage

## Roadmap

### Short Term
- [ ] Auto-reconnect on disconnect
- [ ] Tab completion for nicks and channels
- [ ] Custom color themes
- [ ] Message search/filtering
- [ ] Channel logging to disk

### Medium Term
- [ ] SASL authentication
- [ ] IRCv3 capability negotiation
- [ ] WebSocket connections
- [ ] Message history persistence
- [ ] Notification sounds

### Long Term
- [ ] DCC file transfer
- [ ] Plugin system
- [ ] Scripting support
- [ ] Custom emojis/reactions

## License

This project is released under the [Unlicense](LICENSE) - public domain.

## Related Projects

- [slirc-proto](https://github.com/sid3xyz/slirc-proto) - The IRC protocol library powering this client
- [egui](https://github.com/emilk/egui) - The immediate mode GUI library
