use crossbeam_channel::{Receiver, Sender};
use slirc_proto::mode::{ChannelMode, Mode};
use slirc_proto::{CapSubCommand, Command, Isupport, Message, Prefix, Transport};
use slirc_proto::sasl::{encode_plain, SaslMechanism};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::time::timeout;
use rustls::RootCertStore;
use tokio_rustls::TlsConnector;

use crate::protocol::{BackendAction, GuiEvent, UserInfo};

/// Connection registration state machine for IRCv3 CAP negotiation
#[derive(Debug, Clone, PartialEq)]
enum RegistrationState {
    /// Initial state after transport connected, haven't sent CAP LS yet
    Initial,
    /// Sent CAP LS 302, waiting for server CAP LS response (may be multi-line)
    CapLsSent,
    /// Received all CAP LS lines, sent CAP REQ, waiting for ACK/NAK
    CapReqSent,
    /// SASL authentication in progress
    SaslAuth(SaslSubState),
    /// Sent CAP END (or skipping CAP), waiting for registration to complete
    Registering,
    /// Fully registered (received 001 RPL_WELCOME)
    Registered,
}

/// SASL sub-states within the authentication flow
#[derive(Debug, Clone, PartialEq)]
enum SaslSubState {
    /// Sent AUTHENTICATE <mechanism>, waiting for server "+" challenge
    MechanismSent,
    /// Sent credentials, waiting for 903 success or 904 failure
    CredentialsSent,
}

/// Server capabilities discovered and enabled during negotiation
#[derive(Debug, Default)]
struct ServerCaps {
    /// All capabilities advertised by server in CAP LS
    available: HashSet<String>,
    /// Capabilities we've successfully enabled via CAP ACK
    enabled: HashSet<String>,
    /// SASL mechanisms if server advertised "sasl=PLAIN,EXTERNAL,..."
    sasl_mechanisms: Vec<SaslMechanism>,
    /// Whether we're still receiving multi-line CAP LS (* prefix)
    cap_ls_more: bool,
}

/// Pending registration info saved while doing CAP negotiation
#[derive(Debug, Clone)]
struct PendingRegistration {
    nickname: String,
    username: String,
    realname: String,
    sasl_password: Option<String>,
}

/// Create a TLS connector with webpki root certificates
pub(crate) fn create_tls_connector() -> Result<TlsConnector, String> {
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

pub fn run_backend(action_rx: Receiver<BackendAction>, event_tx: Sender<GuiEvent>) {
    // Create a Tokio runtime for this thread
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            let _ = event_tx.send(GuiEvent::Error(format!("Failed to create Tokio runtime: {}", e)));
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
                match action {
                    BackendAction::Connect {
                        server,
                        port,
                        nickname,
                        username,
                        realname,
                        use_tls,
                        auto_reconnect,
                        sasl_password,
                    } => {
                        current_nick = nickname.clone();
                        
                        // Save connection parameters for potential reconnect
                        last_connection_params = Some((
                            server.clone(),
                            port,
                            nickname.clone(),
                            username.clone(),
                            realname.clone(),
                            use_tls,
                            auto_reconnect,
                        ));

                        // Reset CAP negotiation state
                        reg_state = RegistrationState::Initial;
                        server_caps = ServerCaps::default();
                        pending_reg = Some(PendingRegistration {
                            nickname: nickname.clone(),
                            username: username.clone(),
                            realname: realname.clone(),
                            sasl_password,
                        });

                        // Try to connect
                        let addr = format!("{}:{}", server, port);
                        let protocol = if use_tls { "TLS" } else { "TCP" };
                        let _ = event_tx.send(GuiEvent::RawMessage(format!(
                            "Connecting to {} via {}...", addr, protocol
                        )));

                        match TcpStream::connect(&addr).await {
                            Ok(stream) => {
                                let transport_inst = if use_tls {
                                    // TLS connection
                                    let connector = match create_tls_connector() {
                                        Ok(c) => c,
                                        Err(e) => {
                                            let _ = event_tx.send(GuiEvent::Error(format!(
                                                "Failed to create TLS connector: {}",
                                                e
                                            )));
                                            continue;
                                        }
                                    };
                                    
                                    // Extract hostname for SNI (remove port if present)
                                    let hostname = server.split(':').next().unwrap_or(&server);
                                    let server_name = match rustls::pki_types::ServerName::try_from(hostname.to_string()) {
                                        Ok(name) => name,
                                        Err(e) => {
                                            let _ = event_tx.send(GuiEvent::Error(format!(
                                                "Invalid server name for TLS: {}",
                                                e
                                            )));
                                            continue;
                                        }
                                    };
                                    
                                    // Perform TLS handshake
                                    let tls_stream = match connector.connect(server_name, stream).await {
                                        Ok(s) => s,
                                        Err(e) => {
                                            let _ = event_tx.send(GuiEvent::Error(format!(
                                                "TLS handshake failed: {}",
                                                e
                                            )));
                                            continue;
                                        }
                                    };
                                    
                                    // Create client TLS transport
                                    match Transport::client_tls(tls_stream) {
                                        Ok(t) => t,
                                        Err(e) => {
                                            let _ = event_tx.send(GuiEvent::Error(format!(
                                                "Failed to create TLS transport: {}",
                                                e
                                            )));
                                            continue;
                                        }
                                    }
                                } else {
                                    // Plain TCP connection
                                    match Transport::tcp(stream) {
                                        Ok(t) => t,
                                        Err(e) => {
                                            let _ = event_tx.send(GuiEvent::Error(format!(
                                                "Failed to create transport: {}",
                                                e
                                            )));
                                            continue;
                                        }
                                    }
                                };

                                let mut transport_inst = transport_inst;

                                // Start IRCv3 CAP negotiation
                                let _ = event_tx.send(GuiEvent::RawMessage(
                                    "Starting CAP negotiation...".to_string()
                                ));
                                
                                // Send CAP LS 302 (version 302 for modern features)
                                let cap_ls = Message::from(Command::CAP(
                                    None,
                                    CapSubCommand::LS,
                                    Some("302".to_string()),
                                    None,
                                ));
                                if let Err(e) = transport_inst.write_message(&cap_ls).await {
                                    let _ = event_tx.send(GuiEvent::Error(format!(
                                        "Failed to send CAP LS: {}",
                                        e
                                    )));
                                    continue;
                                }
                                
                                reg_state = RegistrationState::CapLsSent;
                                transport = Some(transport_inst);
                            }
                            Err(e) => {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Connection failed: {}", e)));
                            }
                        }
                    }

                    BackendAction::Disconnect => {
                        if let Some(ref mut t) = transport {
                            let quit_msg = Message::quit_with_message("Leaving");
                            let _ = t.write_message(&quit_msg).await;
                        }
                        transport = None;
                        last_connection_params = None; // Clear on manual disconnect
                        let _ = event_tx.send(GuiEvent::Disconnected("User disconnected".into()));
                    }

                    BackendAction::Join(channel) => {
                        if let Some(ref mut t) = transport {
                            let join_msg = Message::join(&channel);
                            if let Err(e) = t.write_message(&join_msg).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to join: {}", e)));
                            }
                        }
                    }

                    BackendAction::Part { channel, message } => {
                        if let Some(ref mut t) = transport {
                            let part_msg = if let Some(msg) = message {
                                Message::part_with_message(&channel, &msg)
                            } else {
                                Message::part(&channel)
                            };
                            if let Err(e) = t.write_message(&part_msg).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to part: {}", e)));
                            }
                        }
                    }
                    BackendAction::Nick(newnick) => {
                        if let Some(ref mut t) = transport {
                            let nick_msg = Message::nick(&newnick);
                            if let Err(e) = t.write_message(&nick_msg).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to send NICK: {}", e)));
                            } else {
                                let old_nick = current_nick.clone();
                                current_nick = newnick.clone();
                                let _ = event_tx.send(GuiEvent::NickChanged {
                                    old: old_nick,
                                    new: newnick.clone(),
                                });
                            }
                        } else {
                            let _ = event_tx.send(GuiEvent::Error("Not connected".into()));
                        }
                    }
                    BackendAction::Whois(target) => {
                        if let Some(ref mut t) = transport {
                            let whois = Message::from(slirc_proto::command::Command::WHOIS(
                                None,
                                target.clone(),
                            ));
                            if let Err(e) = t.write_message(&whois).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to send WHOIS: {}", e)));
                            }
                        } else {
                            let _ = event_tx.send(GuiEvent::Error("Not connected".into()));
                        }
                    }
                    BackendAction::SetTopic { channel, topic } => {
                        if let Some(ref mut t) = transport {
                            let topic_cmd = Message::from(slirc_proto::command::Command::TOPIC(
                                channel.clone(),
                                Some(topic.clone()),
                            ));
                            if let Err(e) = t.write_message(&topic_cmd).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to set topic: {}", e)));
                            }
                        }
                    }
                    BackendAction::Kick {
                        channel,
                        nick,
                        reason,
                    } => {
                        if let Some(ref mut t) = transport {
                            let kick_msg = if let Some(r) = reason {
                                Message::kick_with_reason(&channel, &nick, &r)
                            } else {
                                Message::kick(&channel, &nick)
                            };
                            if let Err(e) = t.write_message(&kick_msg).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to kick: {}", e)));
                            }
                        }
                    }
                    BackendAction::SetUserMode {
                        channel,
                        nick,
                        mode,
                    } => {
                        if let Some(ref mut t) = transport {
                            // Create a raw MODE command with args: MODE <channel> <mode> <nick>
                            if let Ok(mode_msg) =
                                Message::new(None, "MODE", vec![&channel, &mode, &nick])
                            {
                                if let Err(e) = t.write_message(&mode_msg).await {
                                    let _ = event_tx.send(GuiEvent::Error(format!(
                                        "Failed to set user mode: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }
                    BackendAction::Quit(reason) => {
                        if let Some(ref mut t) = transport {
                            let quit_msg = if let Some(r) = reason {
                                Message::quit_with_message(&r)
                            } else {
                                Message::quit()
                            };
                            let _ = t.write_message(&quit_msg).await;
                        }
                        transport = None;
                        last_connection_params = None; // Clear on manual quit
                        let _ = event_tx.send(GuiEvent::Disconnected("User quit".into()));
                    }

                    BackendAction::List => {
                        if let Some(ref mut t) = transport {
                            if let Ok(list_msg) = Message::new(None, "LIST", vec![]) {
                                if let Err(e) = t.write_message(&list_msg).await {
                                    let _ = event_tx.send(GuiEvent::Error(format!(
                                        "Failed to request channel list: {}",
                                        e
                                    )));
                                }
                            }
                        } else {
                            let _ = event_tx.send(GuiEvent::Error("Not connected".into()));
                        }
                    }

                    BackendAction::SendMessage { target, text } => {
                        if let Some(ref mut t) = transport {
                            let privmsg = Message::privmsg(&target, &text);
                            if let Err(e) = t.write_message(&privmsg).await {
                                let _ = event_tx
                                    .send(GuiEvent::Error(format!("Failed to send: {}", e)));
                            } else {
                                // Echo our own message to the UI
                                let _ = event_tx.send(GuiEvent::MessageReceived {
                                    target: target.clone(),
                                    sender: current_nick.clone(),
                                    text,
                                });
                            }
                        }
                    }
                }
            }

            // Read from the network (with short timeout so we can check for actions)
            if let Some(ref mut t) = transport {
                match timeout(Duration::from_millis(50), t.read_message()).await {
                    Ok(Ok(Some(message))) => {
                        // We do NOT send RawMessage for everything anymore to keep the log clean.
                        // Only send it if we don't handle the message or if it's an error.

                        match &message.command {
                            // PING -> PONG
                            Command::PING(server, _) => {
                                let pong = Message::pong(server);
                                let _ = t.write_message(&pong).await;
                            }

                            // CAP LS/ACK/NAK responses during negotiation
                            Command::CAP(_, subcommand, star_or_caps, maybe_caps) => {
                                match subcommand {
                                    CapSubCommand::LS => {
                                        // Parse capabilities from CAP LS response
                                        // Format: CAP * LS * :cap1 cap2 (multi-line) or CAP * LS :cap1 cap2 (final)
                                        let is_multiline = star_or_caps.as_deref() == Some("*");
                                        let caps_str = if is_multiline {
                                            maybe_caps.as_deref().unwrap_or("")
                                        } else {
                                            star_or_caps.as_deref().unwrap_or("")
                                        };
                                        
                                        // Parse each capability (may include values like sasl=PLAIN,EXTERNAL)
                                        for cap in caps_str.split_whitespace() {
                                            let (cap_name, cap_value) = if let Some(eq_pos) = cap.find('=') {
                                                (&cap[..eq_pos], Some(&cap[eq_pos + 1..]))
                                            } else {
                                                (cap, None)
                                            };
                                            
                                            server_caps.available.insert(cap_name.to_string());
                                            
                                            // Parse SASL mechanisms if present
                                            if cap_name == "sasl" {
                                                if let Some(mechs) = cap_value {
                                                    server_caps.sasl_mechanisms = mechs
                                                        .split(',')
                                                        .map(SaslMechanism::parse)
                                                        .collect();
                                                } else {
                                                    // Server supports SASL but didn't list mechanisms
                                                    server_caps.sasl_mechanisms = vec![SaslMechanism::Plain];
                                                }
                                            }
                                        }
                                        
                                        server_caps.cap_ls_more = is_multiline;
                                        
                                        // If not multiline (final CAP LS), proceed with CAP REQ
                                        if !is_multiline && reg_state == RegistrationState::CapLsSent {
                                            let _ = event_tx.send(GuiEvent::RawMessage(format!(
                                                "Server capabilities: {:?}", server_caps.available
                                            )));
                                            
                                            // Build list of capabilities to request
                                            let mut requested = Vec::new();
                                            
                                            // Always request useful caps if available
                                            let desired_caps = ["multi-prefix", "server-time", "account-notify", "away-notify"];
                                            for cap in &desired_caps {
                                                if server_caps.available.contains(*cap) {
                                                    requested.push(*cap);
                                                }
                                            }
                                            
                                            // Request SASL only if we have a password
                                            let want_sasl = pending_reg.as_ref()
                                                .and_then(|p| p.sasl_password.as_ref())
                                                .is_some()
                                                && server_caps.available.contains("sasl");
                                            
                                            if want_sasl {
                                                requested.push("sasl");
                                            }
                                            
                                            if requested.is_empty() {
                                                // No caps to request, end negotiation
                                                let cap_end = Message::from(Command::CAP(
                                                    None, CapSubCommand::END, None, None
                                                ));
                                                let _ = t.write_message(&cap_end).await;
                                                
                                                // Send NICK/USER
                                                if let Some(ref pr) = pending_reg {
                                                    let nick_msg = Message::nick(&pr.nickname);
                                                    let _ = t.write_message(&nick_msg).await;
                                                    let user_msg = Message::user(&pr.username, &pr.realname);
                                                    let _ = t.write_message(&user_msg).await;
                                                }
                                                reg_state = RegistrationState::Registering;
                                            } else {
                                                // Send CAP REQ
                                                let caps_list = requested.join(" ");
                                                let _ = event_tx.send(GuiEvent::RawMessage(format!(
                                                    "Requesting capabilities: {}", caps_list
                                                )));
                                                
                                                let cap_req = Message::from(Command::CAP(
                                                    None, CapSubCommand::REQ, None, Some(caps_list)
                                                ));
                                                let _ = t.write_message(&cap_req).await;
                                                reg_state = RegistrationState::CapReqSent;
                                            }
                                        }
                                    }
                                    
                                    CapSubCommand::ACK => {
                                        // Server acknowledged our CAP REQ
                                        let acked_caps = star_or_caps.as_deref().unwrap_or("");
                                        for cap in acked_caps.split_whitespace() {
                                            // Handle "-cap" (capability removed) - rare during negotiation
                                            let cap_name = cap.trim_start_matches('-');
                                            if cap.starts_with('-') {
                                                server_caps.enabled.remove(cap_name);
                                            } else {
                                                server_caps.enabled.insert(cap_name.to_string());
                                            }
                                        }
                                        
                                        let _ = event_tx.send(GuiEvent::RawMessage(format!(
                                            "Capabilities enabled: {:?}", server_caps.enabled
                                        )));
                                        
                                        // If SASL is enabled and we have a password, start SASL
                                        let want_sasl = pending_reg.as_ref()
                                            .and_then(|p| p.sasl_password.as_ref())
                                            .is_some();
                                            
                                        if server_caps.enabled.contains("sasl") && want_sasl {
                                            // Start SASL PLAIN authentication
                                            let auth_msg = Message::from(Command::AUTHENTICATE("PLAIN".to_string()));
                                            let _ = t.write_message(&auth_msg).await;
                                            reg_state = RegistrationState::SaslAuth(SaslSubState::MechanismSent);
                                        } else {
                                            // No SASL, end CAP and register
                                            let cap_end = Message::from(Command::CAP(
                                                None, CapSubCommand::END, None, None
                                            ));
                                            let _ = t.write_message(&cap_end).await;
                                            
                                            if let Some(ref pr) = pending_reg {
                                                let nick_msg = Message::nick(&pr.nickname);
                                                let _ = t.write_message(&nick_msg).await;
                                                let user_msg = Message::user(&pr.username, &pr.realname);
                                                let _ = t.write_message(&user_msg).await;
                                            }
                                            reg_state = RegistrationState::Registering;
                                        }
                                    }
                                    
                                    CapSubCommand::NAK => {
                                        // Server rejected some/all caps - that's okay, proceed
                                        let _ = event_tx.send(GuiEvent::RawMessage(
                                            "Some capabilities were not supported".to_string()
                                        ));
                                        
                                        // End CAP and register
                                        let cap_end = Message::from(Command::CAP(
                                            None, CapSubCommand::END, None, None
                                        ));
                                        let _ = t.write_message(&cap_end).await;
                                        
                                        if let Some(ref pr) = pending_reg {
                                            let nick_msg = Message::nick(&pr.nickname);
                                            let _ = t.write_message(&nick_msg).await;
                                            let user_msg = Message::user(&pr.username, &pr.realname);
                                            let _ = t.write_message(&user_msg).await;
                                        }
                                        reg_state = RegistrationState::Registering;
                                    }
                                    
                                    _ => {
                                        // NEW/DEL/LIST not expected during registration
                                    }
                                }
                            }
                            
                            // AUTHENTICATE challenge from server during SASL
                            Command::AUTHENTICATE(challenge) => {
                                if reg_state == RegistrationState::SaslAuth(SaslSubState::MechanismSent)
                                    && challenge == "+"
                                {
                                    // Server ready for credentials
                                    if let Some(ref pr) = pending_reg {
                                        if let Some(ref password) = pr.sasl_password {
                                            // Encode PLAIN credentials: \0username\0password
                                            let encoded = encode_plain(&pr.username, password);
                                            let auth_msg = Message::from(Command::AUTHENTICATE(encoded));
                                            let _ = t.write_message(&auth_msg).await;
                                            reg_state = RegistrationState::SaslAuth(SaslSubState::CredentialsSent);
                                        }
                                    }
                                }
                            }
                            
                            // SASL success (903)
                            Command::Response(code, args) if code.code() == 903 => {
                                let msg = args.last().map(|s| s.as_str()).unwrap_or("Authenticated");
                                let _ = event_tx.send(GuiEvent::SaslResult {
                                    success: true,
                                    message: msg.to_string(),
                                });
                                let _ = event_tx.send(GuiEvent::RawMessage(format!(
                                    "SASL authentication successful: {}", msg
                                )));
                                
                                // End CAP negotiation and register
                                let cap_end = Message::from(Command::CAP(
                                    None, CapSubCommand::END, None, None
                                ));
                                let _ = t.write_message(&cap_end).await;
                                
                                if let Some(ref pr) = pending_reg {
                                    let nick_msg = Message::nick(&pr.nickname);
                                    let _ = t.write_message(&nick_msg).await;
                                    let user_msg = Message::user(&pr.username, &pr.realname);
                                    let _ = t.write_message(&user_msg).await;
                                }
                                reg_state = RegistrationState::Registering;
                            }
                            
                            // SASL failure (904)
                            Command::Response(code, args) if code.code() == 904 => {
                                let msg = args.last().map(|s| s.as_str()).unwrap_or("Authentication failed");
                                let _ = event_tx.send(GuiEvent::SaslResult {
                                    success: false,
                                    message: msg.to_string(),
                                });
                                let _ = event_tx.send(GuiEvent::Error(format!(
                                    "SASL authentication failed: {}", msg
                                )));
                                
                                // End CAP negotiation and try to register anyway
                                let cap_end = Message::from(Command::CAP(
                                    None, CapSubCommand::END, None, None
                                ));
                                let _ = t.write_message(&cap_end).await;
                                
                                if let Some(ref pr) = pending_reg {
                                    let nick_msg = Message::nick(&pr.nickname);
                                    let _ = t.write_message(&nick_msg).await;
                                    let user_msg = Message::user(&pr.username, &pr.realname);
                                    let _ = t.write_message(&user_msg).await;
                                }
                                reg_state = RegistrationState::Registering;
                            }
                            
                            // RPL_LOGGEDIN (900) - Successfully authenticated account
                            Command::Response(code, args) if code.code() == 900 => {
                                let account = args.get(2).map(|s| s.as_str()).unwrap_or("account");
                                let _ = event_tx.send(GuiEvent::RawMessage(format!(
                                    "Logged in as {}", account
                                )));
                            }

                            // RPL_WELCOME (001) - Registration complete
                            Command::Response(code, _) if code.code() == 1 => {
                                reg_state = RegistrationState::Registered;
                                pending_reg = None;
                                let _ = event_tx.send(GuiEvent::Connected);
                            }

                            // RPL_ISUPPORT (005) - Server capabilities
                            Command::Response(code, args) if code.code() == 5 => {
                                // Parse ISUPPORT tokens using slirc-proto
                                let params: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                                let isupport = Isupport::parse_params(&params);
                                
                                // Extract useful info and send to UI
                                let _ = event_tx.send(GuiEvent::ServerInfo {
                                    network: isupport.network().map(|s| s.to_string()),
                                    casemapping: isupport.casemapping().map(|s| s.to_string()),
                                });
                            }

                            // RPL_TOPIC (332)
                            Command::Response(code, args) if code.code() == 332 => {
                                if args.len() >= 3 {
                                    let _ = event_tx.send(GuiEvent::Topic {
                                        channel: args[1].clone(),
                                        topic: args[2].clone(),
                                    });
                                }
                            }

                            // RPL_NAMREPLY (353)
                            Command::Response(code, args) if code.code() == 353 => {
                                if args.len() >= 4 {
                                    let channel = args[2].clone();
                                    let mut names: Vec<UserInfo> = Vec::new();
                                    for s in args[3].split_whitespace() {
                                        let mut chars = s.chars();
                                        let prefix = chars
                                            .next()
                                            .filter(|c| matches!(c, '@' | '+' | '%' | '&' | '~'));
                                        let nick = if prefix.is_some() {
                                            chars.as_str().to_string()
                                        } else {
                                            s.to_string()
                                        };
                                        names.push(UserInfo { nick, prefix });
                                    }
                                    let _ = event_tx.send(GuiEvent::Names { channel, names });
                                }
                            }

                            // RPL_LIST (322) - channel list item
                            Command::Response(code, args) if code.code() == 322 => {
                                if args.len() >= 4 {
                                    let channel = args[1].clone();
                                    let user_count = args[2].parse::<usize>().unwrap_or(0);
                                    let topic = args[3].clone();
                                    let _ = event_tx.send(GuiEvent::ChannelListItem {
                                        channel,
                                        user_count,
                                        topic,
                                    });
                                }
                            }

                            // RPL_LISTEND (323) - end of channel list
                            Command::Response(code, _) if code.code() == 323 => {
                                let _ = event_tx.send(GuiEvent::ChannelListEnd);
                            }

                            // RPL_MOTD (372) and RPL_MOTDSTART (375)
                            Command::Response(code, args)
                                if code.code() == 372 || code.code() == 375 =>
                            {
                                if let Some(text) = args.last() {
                                    let _ = event_tx.send(GuiEvent::Motd(text.clone()));
                                }
                            }

                            // PRIVMSG
                            Command::PRIVMSG(target, text) => {
                                let sender =
                                    message.source_nickname().unwrap_or("unknown").to_string();
                                let _ = event_tx.send(GuiEvent::MessageReceived {
                                    target: target.clone(),
                                    sender,
                                    text: text.clone(),
                                });
                            }

                            // NOTICE
                            Command::NOTICE(target, text) => {
                                let sender =
                                    message.source_nickname().unwrap_or("server").to_string();
                                let _ = event_tx.send(GuiEvent::MessageReceived {
                                    target: target.clone(),
                                    sender: format!("-{}-", sender),
                                    text: text.clone(),
                                });
                            }

                            // JOIN
                            Command::JOIN(channel, _, _) => {
                                let nick = message.source_nickname().unwrap_or("").to_string();
                                if nick == current_nick {
                                    let _ = event_tx.send(GuiEvent::JoinedChannel(channel.clone()));
                                } else {
                                    let _ = event_tx.send(GuiEvent::UserJoined {
                                        channel: channel.clone(),
                                        nick,
                                    });
                                }
                            }

                            // PART
                            Command::PART(channel, msg) => {
                                let nick = message.source_nickname().unwrap_or("").to_string();
                                if nick == current_nick {
                                    let _ = event_tx.send(GuiEvent::PartedChannel(channel.clone()));
                                } else {
                                    let _ = event_tx.send(GuiEvent::UserParted {
                                        channel: channel.clone(),
                                        nick,
                                        message: msg.clone(),
                                    });
                                }
                            }

                            // NICK change (someone changed their nick)
                            Command::NICK(newnick) => {
                                let oldnick = message.source_nickname().unwrap_or("").to_string();
                                // Update internal state if it was our nick
                                if oldnick == current_nick {
                                    current_nick = newnick.clone();
                                }
                                let _ = event_tx.send(GuiEvent::NickChanged {
                                    old: oldnick.clone(),
                                    new: newnick.clone(),
                                });
                            }

                            // QUIT - user left the server
                            Command::QUIT(msg) => {
                                if let Some(Prefix::Nickname(nick, _, _)) = &message.prefix {
                                    let _ = event_tx.send(GuiEvent::UserQuit {
                                        nick: nick.to_string(),
                                        message: msg.clone(),
                                    });
                                }
                            }

                            // ERROR from server
                            Command::ERROR(msg) => {
                                let _ = event_tx.send(GuiEvent::Error(msg.clone()));
                            }

                            // Channel mode changes (e.g. +o/-o): update UI user prefixes
                            Command::ChannelMODE(channel, modes) => {
                                for m in modes {
                                    match m {
                                        Mode::Plus(ChannelMode::Oper, Some(nick)) => {
                                            let _ = event_tx.send(GuiEvent::UserMode {
                                                channel: channel.clone(),
                                                nick: nick.clone(),
                                                prefix: Some('@'),
                                                added: true,
                                            });
                                        }
                                        Mode::Minus(ChannelMode::Oper, Some(nick)) => {
                                            let _ = event_tx.send(GuiEvent::UserMode {
                                                channel: channel.clone(),
                                                nick: nick.clone(),
                                                prefix: Some('@'),
                                                added: false,
                                            });
                                        }
                                        Mode::Plus(ChannelMode::Voice, Some(nick)) => {
                                            let _ = event_tx.send(GuiEvent::UserMode {
                                                channel: channel.clone(),
                                                nick: nick.clone(),
                                                prefix: Some('+'),
                                                added: true,
                                            });
                                        }
                                        Mode::Minus(ChannelMode::Voice, Some(nick)) => {
                                            let _ = event_tx.send(GuiEvent::UserMode {
                                                channel: channel.clone(),
                                                nick: nick.clone(),
                                                prefix: Some('+'),
                                                added: false,
                                            });
                                        }
                                        Mode::Plus(ChannelMode::Halfop, Some(nick)) => {
                                            let _ = event_tx.send(GuiEvent::UserMode {
                                                channel: channel.clone(),
                                                nick: nick.clone(),
                                                prefix: Some('%'),
                                                added: true,
                                            });
                                        }
                                        Mode::Minus(ChannelMode::Halfop, Some(nick)) => {
                                            let _ = event_tx.send(GuiEvent::UserMode {
                                                channel: channel.clone(),
                                                nick: nick.clone(),
                                                prefix: Some('%'),
                                                added: false,
                                            });
                                        }
                                        Mode::Plus(ChannelMode::Admin, Some(nick)) => {
                                            let _ = event_tx.send(GuiEvent::UserMode {
                                                channel: channel.clone(),
                                                nick: nick.clone(),
                                                prefix: Some('&'),
                                                added: true,
                                            });
                                        }
                                        Mode::Minus(ChannelMode::Admin, Some(nick)) => {
                                            let _ = event_tx.send(GuiEvent::UserMode {
                                                channel: channel.clone(),
                                                nick: nick.clone(),
                                                prefix: Some('&'),
                                                added: false,
                                            });
                                        }
                                        Mode::Plus(ChannelMode::Founder, Some(nick)) => {
                                            let _ = event_tx.send(GuiEvent::UserMode {
                                                channel: channel.clone(),
                                                nick: nick.clone(),
                                                prefix: Some('~'),
                                                added: true,
                                            });
                                        }
                                        Mode::Minus(ChannelMode::Founder, Some(nick)) => {
                                            let _ = event_tx.send(GuiEvent::UserMode {
                                                channel: channel.clone(),
                                                nick: nick.clone(),
                                                prefix: Some('~'),
                                                added: false,
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            // For other messages, we might want to log them if they are interesting
                            _ => {}
                        }
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
