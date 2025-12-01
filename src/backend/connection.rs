//! Connection establishment utilities for IRC client
//!
//! Handles TLS and TCP connection setup with proper error handling.

use rustls::RootCertStore;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

/// Create a TLS connector with webpki root certificates for cross-platform compatibility
pub fn create_tls_connector() -> Result<TlsConnector, String> {
    let mut root_store = RootCertStore::empty();
    
    // Use webpki-roots for cross-platform compatibility
    root_store.extend(
        webpki_roots::TLS_SERVER_ROOTS
            .iter()
            .cloned()
    );
    
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    
    Ok(TlsConnector::from(Arc::new(config)))
}

/// Establish a connection to an IRC server with optional TLS
///
/// # Arguments
/// * `server` - Server hostname or IP address
/// * `port` - Server port
/// * `use_tls` - Whether to use TLS encryption
///
/// # Returns
/// A connected `Transport` instance ready for IRC communication
///
/// # Errors
/// Returns error string if connection fails at any stage (TCP, TLS handshake, transport creation)
pub async fn establish_connection(
    server: &str,
    port: u16,
    use_tls: bool,
) -> Result<slirc_proto::Transport, String> {
    let addr = format!("{}:{}", server, port);
    
    // Establish TCP connection
    let stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("TCP connection failed: {}", e))?;
    
    if use_tls {
        // TLS connection path
        let connector = create_tls_connector()?;
        
        // Extract hostname for SNI (remove port if present)
        let hostname = server.split(':').next().unwrap_or(server);
        let server_name = rustls::pki_types::ServerName::try_from(hostname.to_string())
            .map_err(|e| format!("Invalid server name for TLS: {}", e))?;
        
        // Perform TLS handshake
        let tls_stream = connector
            .connect(server_name, stream)
            .await
            .map_err(|e| format!("TLS handshake failed: {}", e))?;
        
        // Create client TLS transport
        slirc_proto::Transport::client_tls(tls_stream)
            .map_err(|e| format!("Failed to create TLS transport: {}", e))
    } else {
        // Plain TCP connection
        slirc_proto::Transport::tcp(stream)
            .map_err(|e| format!("Failed to create transport: {}", e))
    }
}
