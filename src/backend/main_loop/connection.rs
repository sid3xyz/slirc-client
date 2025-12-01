//! Connection establishment and TLS setup.
//!
//! Re-exports connection functions from the parent backend module.

use slirc_proto::Transport;

/// Establish a connection to an IRC server (TCP or TLS)
pub async fn establish_connection(server: &str, port: u16, use_tls: bool) -> Result<Transport, String> {
    super::super::connection::establish_connection(server, port, use_tls).await
}
