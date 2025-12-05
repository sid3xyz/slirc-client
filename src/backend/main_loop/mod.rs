//! Backend main event loop with CAP negotiation and message routing.

pub mod connection;
pub mod handlers;
pub mod state;

pub use state::{PendingRegistration, RegistrationState, ServerCaps};

use crate::protocol::{BackendAction, GuiEvent};
use crossbeam_channel::{Receiver, Sender};
use slirc_proto::Transport;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time::timeout;

/// Run the backend event loop on a tokio runtime
pub fn run_backend(action_rx: Receiver<BackendAction>, event_tx: Sender<GuiEvent>) {
    // Create a Tokio runtime for this thread
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            let _ = event_tx.send(GuiEvent::Error(format!(
                "Failed to create Tokio runtime: {}",
                e
            )));
            return;
        }
    };

    rt.block_on(async move {
        let mut transport: Option<Transport> = None;
        let mut current_nick = String::new();

        // Connection state for auto-reconnect
        let mut last_connection_params: Option<(String, u16, String, String, String, bool, bool)> = None;

        // CAP negotiation state machine
        let mut reg_state = RegistrationState::Registered; // Start as registered (no connection)
        let mut server_caps = ServerCaps::default();
        let mut pending_reg: Option<PendingRegistration> = None;

        loop {
            // Check for actions from the UI (non-blocking)
            while let Ok(action) = action_rx.try_recv() {
                handlers::handle_backend_action(
                    action,
                    &mut transport,
                    &mut current_nick,
                    &mut last_connection_params,
                    &mut reg_state,
                    &mut server_caps,
                    &mut pending_reg,
                    &event_tx,
                ).await;
            }

            // Read from the network (with short timeout so we can check for actions)
            if let Some(ref mut t) = transport {
                match timeout(Duration::from_millis(50), t.read_message()).await {
                    Ok(Ok(Some(message))) => {
                        handlers::handle_server_message(
                            message,
                            t,
                            &mut current_nick,
                            &mut reg_state,
                            &mut server_caps,
                            &mut pending_reg,
                            &event_tx,
                        ).await;
                    }
                    Ok(Ok(None)) => {
                        // Connection closed
                        transport = None;
                        let _ = event_tx
                            .send(GuiEvent::Disconnected("Connection closed by server".into()));

                        // Note: Auto-reconnect would trigger here if last_connection_params.6 is true
                        // For now, just notify the user
                        if let Some(params) = &last_connection_params {
                            if params.6 { // auto_reconnect flag
                                let _ = event_tx.send(GuiEvent::RawMessage(
                                    "Connection lost. Auto-reconnect enabled (manual reconnect required for now).".to_string()
                                ));
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        let _ = event_tx.send(GuiEvent::Error(format!("Read error: {:?}", e)));
                        transport = None;
                        let _ = event_tx.send(GuiEvent::Disconnected("Read error".into()));
                    }
                    Err(_) => {
                        // Timeout - this is normal, just loop
                    }
                }
            } else {
                // No connection, sleep a bit to avoid busy-looping
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    });
}
