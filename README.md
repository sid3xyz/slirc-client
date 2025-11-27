# SLIRC Client

A native IRC client built with [egui](https://github.com/emilk/egui) and the [slirc-proto](https://github.com/sid3xyz/slirc-proto) protocol library.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-Unlicense-blue)

## Features

- **Native GUI** - Cross-platform desktop application using egui/glow
- **Multi-buffer interface** - Separate buffers for channels, private messages, and system log
- **User list** - Live user list for joined channels
- **Topic display** - Shows channel topics
- **Raw message logging** - Full IRC protocol visibility in System buffer
- **PING/PONG handling** - Automatic keep-alive responses
 - **Command input** - Supports /join, /part, /msg, /nick, /quit, /me
 - **Timestamps** - Messages are displayed with local timestamps
 - **Input history** - Use Up/Down to navigate previously sent messages
 - **Channel tabs (left)** - Buffers are shown as vertical tabs with unread badges
 - **Mentions highlight** - Messages that mention your nick are highlighted

## Development Notes (HexChat UI Alignment)

- The UI now uses the left buffer panel as the single navigation surface (Top horizontal tabs removed during Phase 1 refactor).
- Buffer user lists now track nick prefixes (e.g. `@` for ops, `+` for voiced users) and are sorted by prefix rank (owner/admin/op/halfop/voice/regular).
- Messages are left-aligned in the central pane with unified `[time] <nick> Message` format and colors for own messages and mentions.
- The backend parses `NAMES` replies and preserves nick prefixes for accurate user list rendering, and channel `MODE` changes update prefixes.

See `HEXCHAT_UI_PLAN.md` for implementation plan and next steps (Phase 2+).

## Architecture

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

## Usage

1. **Connect**: Enter server address (default: `irc.slirc.net:6667`) and nickname, click "Connect"
2. **Wait for registration**: Watch the System buffer for "✓ Connected and registered!"
3. **Join channels**: Enter channel name (default: `#straylight`) and click "+"
    - You can also join using `/join #channel` via the input
4. **Chat**: Select a channel buffer, type messages, press Enter or click "Send"
    - Use `/join <channel>` to join a channel (e.g., `/join #rust`)
    - Use `/part <channel> [message]` to leave a channel
    - Use `/msg <target> <message>` to send a private message
    - Use `/nick <newnick>` to change your nickname
    - Use `/me <action>` to send a CTCP ACTION (e.g., `/me waves`)
    - Use `/quit [message]` to quit the server gracefully
5. **Switch buffers**: Click buffer names in the left panel
6. **Disconnect**: Click "Disconnect" button
7. **Change Nick**: With a connected session, update the Nick input and click "Change Nick" to request a nick change on the server.

### UI & Look-and-feel Improvements

- **Channel tabs**: Buffer list on the left is now a vertical tab list, showing unread counts and a small `x` button to leave a channel.
- **Unread and Mentions**: Tabs show an unread count; messages mentioning your nick are highlighted in red and mark the tab as `mention`.
- **Your messages**: Outgoing messages are aligned to the right and colored to help differentiate from other users.

### Quick Checks

Try these to verify the UI features:

1. Connect to an IRC server and join `#straylight`.
2. From another client, change your nick (`/nick newnick`) and verify that the user list updates to show `newnick`.
3. Post a message in channel mentioning your nick to verify message highlight and unread increment when the channel is inactive.
4. Use the left-hand channel tabs to switch buffers and see unread counts cleared.

## Default Configuration

- **Server**: `irc.slirc.net:6667`
- **Default Channel**: `#straylight`
- **No auto-connect** - User must click Connect manually

## Project Structure

```
slirc-client/
├── Cargo.toml          # Dependencies and project metadata
├── README.md           # This file
└── src/
    └── main.rs         # Complete application (single-file architecture)
        ├── BackendAction    # UI → Backend message types
        ├── GuiEvent         # Backend → UI message types
        ├── run_backend()    # Tokio runtime and network loop
        ├── Buffer           # Per-channel/query message storage
        ├── SlircApp         # egui application state
        └── main()           # Entry point
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

## Future Enhancements

- [ ] TLS/SSL support (slirc-proto supports it)
- [ ] WebSocket connections
- [ ] SASL authentication
- [ ] Nick completion (Tab)
 - [ ] Command input improvements (Tab completion, extended command parsing)
- [ ] Configuration file
- [ ] Multiple server connections
- [ ] IRCv3 capability negotiation
- [ ] Message history persistence
- [ ] Theming support

## License

This project is released under the [Unlicense](LICENSE) - public domain.

## Related Projects

- [slirc-proto](https://github.com/sid3xyz/slirc-proto) - The IRC protocol library powering this client
- [egui](https://github.com/emilk/egui) - The immediate mode GUI library
