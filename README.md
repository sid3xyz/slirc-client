# slirc-client

An IRC client built with `egui` and `slirc-proto`.

## Features

- **Modern UI**: Built with `egui` for a responsive and immediate-mode GUI.
- **Async Backend**: Uses `tokio` for non-blocking network operations.
- **IRCv3 Support**: Leverages `slirc-proto` for modern IRC features.
- **Secure**: Uses `keyring` for secure password storage.

## Architecture

The client follows a split architecture:

- **Main Thread**: Runs the `egui` UI loop. Handles rendering and user input.
- **Backend Thread**: Runs a `tokio` runtime. Handles TCP connections, TLS, and IRC protocol processing.
- **Communication**: `crossbeam-channel` is used for message passing between the UI and Backend threads.

## Running the Client

```bash
cargo run -p slirc-client
```

## License

Unlicense
