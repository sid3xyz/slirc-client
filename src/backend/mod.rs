/// Backend submodules for connection and message handling
///
/// This module breaks down the large backend logic into focused components:
/// - `connection`: TLS and TCP connection establishment
/// - `handlers`: IRC message routing and event generation
/// - `main_loop`: Core event loop and CAP negotiation state machine
mod connection;
mod handlers;
mod main_loop;

// Re-export the main backend entry points
pub use main_loop::run_backend;

#[cfg(test)]
pub use main_loop::create_tls_connector;
