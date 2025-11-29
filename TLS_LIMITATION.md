# TLS Support - Known Limitation

## Issue

TLS support in slirc-client is currently **disabled** due to an incompatibility with the slirc-proto library.

## Technical Details

**Problem**: The `slirc-proto` library is designed for IRC **servers** and uses `tokio_rustls::server::TlsStream` for TLS connections. However, IRC **clients** require `tokio_rustls::client::TlsStream`.

**Location**: `src/backend.rs` line 69

**Error Message**:
```
error[E0308]: mismatched types
expected struct `tokio_rustls::server::TlsStream<tokio::net::TcpStream>`
found struct `tokio_rustls::client::TlsStream<tokio::net::TcpStream>`
```

**Current Workaround**: The code detects when TLS is requested and displays an error message to the user:
```rust
if use_tls {
    let _ = event_tx.send(GuiEvent::Error(
        "TLS support is not yet available. Please use plain TCP connection.".to_string()
    ));
    continue;
}
```

## Implementation Status

### ✅ Completed
- TLS UI checkbox in connection dialog and toolbar
- `use_tls` field propagated through all relevant structs (Network, BackendAction, SlircApp, NetworkForm)
- TLS dependencies added (tokio-rustls, rustls, webpki-roots)
- TLS connector implementation with Mozilla root certificates
- User-facing error message when TLS is attempted

### ❌ Blocked
- Actual TLS connection establishment (library incompatibility)
- Testing TLS connections

## Potential Solutions

### Option 1: Fork/Patch slirc-proto
Fork the `slirc-proto` library and add support for client-side TLS streams. This would require:
- Adding a new `Transport::client_tls()` method that accepts `tokio_rustls::client::TlsStream`
- Or making the existing `Transport::tls()` generic over TLS stream types

### Option 2: Custom Transport Layer
Implement a custom IRC message framing layer for TLS connections, bypassing `slirc-proto`'s Transport abstraction:
- Wrap `tokio_rustls::client::TlsStream` with our own framing codec
- Use `tokio_util::codec::Framed` with `slirc_proto::IrcCodec`
- Manually handle the async read/write loop

### Option 3: Upstream Contribution
File an issue or pull request with the `slirc-proto` repository to add client TLS support. This is the cleanest long-term solution.

## Recommended Action

**Short term**: Document the limitation and ensure users can connect via plain TCP.

**Long term**: Submit a pull request to `slirc-proto` adding client TLS support, or implement Option 2 if upstream is unresponsive.

## References

- slirc-proto repository: https://github.com/sid3xyz/slirc-proto
- Related code: `src/backend.rs` lines 14-30 (TLS connector), lines 69-94 (connection logic)
- Issue tracking: TBD (create GitHub issue)

## Testing

Once TLS is functional, ensure these test cases pass:
- [ ] Connect to irc.libera.chat:6697 with TLS enabled
- [ ] Verify certificate validation works correctly
- [ ] Test connection failure with invalid certificates
- [ ] Verify TLS checkbox state persists in network settings
- [ ] Test auto-port selection (6697 for TLS, 6667 for plain)
