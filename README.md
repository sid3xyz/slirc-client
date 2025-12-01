# SLIRC Client

A modern, high-performance native IRC client built with Rust, [egui](https://github.com/emilk/egui), and the [slirc-proto](https://github.com/sid3xyz/slirc-proto) protocol library.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-Unlicense-blue)
![Architecture](https://img.shields.io/badge/architecture-dual--thread-purple)

## Overview

SLIRC Client combines the performance of a native application with modern UI design principles. It uses a dual-thread architecture to ensure the interface remains responsive even during heavy network traffic. The UI is built on a custom design system inspired by modern chat applications, featuring a deep surface hierarchy and semantic coloring.

## Key Features

### 🎨 Modern User Interface

- **Custom Design System**: A 7-level surface hierarchy for depth perception and semantic coloring for UI states.
- **Professional Typography**: Integrated **Inter** for UI text and **JetBrains Mono** for code/monospaced content.
- **Theming**: Built-in Dark and Light modes with high-contrast support.
- **Quick Switcher**: `Ctrl+K` command palette for rapid channel and buffer navigation.
- **Rich Text**: Full support for mIRC color codes, bold, and italic formatting.

### ⚡ Core IRC Capabilities

- **Secure Connections**: Full TLS/SSL support with SASL authentication.
- **Multi-Network**: Manage multiple server configurations with persistent settings.
- **Standard Compliance**: RFC 2812 compliant channel/nickname validation and message handling.
- **Smart Buffers**: Separate buffers for channels, private messages, and system logs with unread indicators.
- **User Management**: Live user lists with prefix indicators (@, +, etc.) and mode tracking.

### 🔒 Security & Persistence

- **Secure Storage**: NickServ and server passwords are stored securely in the system keyring.
- **Config Persistence**: Settings and network configurations are automatically saved and loaded.
- **Logging**: Automatic chat logging to `XDG_DATA_HOME/slirc-client/logs`.

## Architecture

The application follows a strict **Frontend/Backend split**:

1.  **Frontend (Main Thread)**:
    -   Runs the `egui` immediate mode GUI loop.
    -   Handles input, rendering, and state visualization.
    -   Sends `BackendAction` enums to the backend.
    -   Receives `GuiEvent` enums to update local state.

2.  **Backend (Background Thread)**:
    -   Runs a `tokio` async runtime.
    -   Manages TCP/TLS connections via `slirc-proto`.
    -   Handles protocol parsing, keep-alives, and auto-reconnection.
    -   Communicates with the frontend via `crossbeam-channel`.

## Usage

### Prerequisites

- Rust 1.70 or higher
- Linux/macOS/Windows

### Running the Client

From the workspace root:

```bash
cargo run -p slirc-client
```

### Keyboard Shortcuts

| Shortcut  | Action                           |
| --------- | -------------------------------- |
| `Ctrl+K`  | Open Quick Switcher              |
| `Up/Down` | Navigate Input History           |
| `Tab`     | Auto-complete Nicknames/Commands |

## Configuration

Configuration files are stored in your system's standard configuration directory (e.g., `~/.config/slirc-client` on Linux).

- **`settings.json`**: UI preferences and theme settings.
- **`networks.json`**: Saved network configurations.
