//! State types for connection registration and CAP negotiation.

use slirc_proto::sasl::SaslMechanism;
use std::collections::HashSet;

/// Connection registration state machine for IRCv3 CAP negotiation
#[derive(Debug, Clone, PartialEq)]
pub enum RegistrationState {
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
pub enum SaslSubState {
    /// Sent AUTHENTICATE <mechanism>, waiting for server "+" challenge
    MechanismSent,
    /// Sent credentials, waiting for 903 success or 904 failure
    CredentialsSent,
}

/// Server capabilities discovered and enabled during negotiation
#[derive(Debug, Default)]
pub struct ServerCaps {
    /// All capabilities advertised by server in CAP LS
    pub available: HashSet<String>,
    /// Capabilities we've successfully enabled via CAP ACK
    pub enabled: HashSet<String>,
    /// SASL mechanisms if server advertised "sasl=PLAIN,EXTERNAL,..."
    pub sasl_mechanisms: Vec<SaslMechanism>,
    /// Whether we're still receiving multi-line CAP LS (* prefix)
    pub cap_ls_more: bool,
}

/// Pending registration info saved while doing CAP negotiation
#[derive(Debug, Clone)]
pub struct PendingRegistration {
    pub nickname: String,
    pub username: String,
    pub realname: String,
    pub sasl_password: Option<String>,
}
