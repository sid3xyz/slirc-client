//! Input validation for IRC protocol compliance


/// Validates an IRC channel name according to RFC 2812
pub fn validate_channel_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Channel name cannot be empty".to_string());
    }
    
    // Channel must start with # or &
    if !name.starts_with('#') && !name.starts_with('&') {
        return Err("Channel name must start with # or &".to_string());
    }
    
    // Maximum length per RFC 2812 is 50 characters
    if name.len() > 50 {
        return Err("Channel name too long (max 50 characters)".to_string());
    }
    
    // Channel names cannot contain spaces, commas, or control characters
    if name.contains(|c: char| c.is_control() || c == ' ' || c == ',' || c == '\x07') {
        return Err("Channel name contains invalid characters".to_string());
    }
    
    Ok(())
}

/// Validate IRC nickname according to RFC 2812
#[allow(dead_code)]
pub fn validate_nickname(nick: &str) -> Result<(), String> {
    if nick.is_empty() {
        return Err("Nickname cannot be empty".to_string());
    }
    
    // Maximum length per RFC 2812 is 9 characters (though many servers allow more)
    if nick.len() > 30 {
        return Err("Nickname too long (max 30 characters)".to_string());
    }
    
    // First character must be a letter
    let first_char = nick.chars().next()
        .ok_or_else(|| "Nickname cannot be empty".to_string())?;
    if !first_char.is_alphabetic() && first_char != '[' && first_char != ']' 
        && first_char != '{' && first_char != '}' && first_char != '\\' 
        && first_char != '|' && first_char != '_' && first_char != '^' {
        return Err("Nickname must start with a letter or special character".to_string());
    }
    
    // Rest can be alphanumeric or special characters
    for c in nick.chars() {
        if !c.is_alphanumeric() && !"-[]{}\\|_^".contains(c) {
            return Err(format!("Invalid character '{}' in nickname", c));
        }
    }
    
    Ok(())
}

/// Validates a server address (host:port format)
pub fn validate_server_address(addr: &str) -> Result<(String, u16), String> {
    if addr.is_empty() {
        return Err("Server address cannot be empty".to_string());
    }
    
    let parts: Vec<&str> = addr.split(':').collect();
    
    match parts.as_slice() {
        [host] => {
            // No port specified, use default
            if host.is_empty() {
                return Err("Hostname cannot be empty".to_string());
            }
            Ok((host.to_string(), 6667))
        }
        [host, port] => {
            if host.is_empty() {
                return Err("Hostname cannot be empty".to_string());
            }
            
            let port_num = port.parse::<u16>()
                .map_err(|_| format!("Invalid port number: {}", port))?;
            
            if port_num == 0 {
                return Err("Port number must be greater than 0".to_string());
            }
            
            Ok((host.to_string(), port_num))
        }
        _ => Err("Invalid server format. Use 'host:port' or 'host'".to_string())
    }
}

/// Validates an IRC message (PRIVMSG/NOTICE text)
pub fn validate_message(msg: &str) -> Result<(), String> {
    if msg.is_empty() {
        return Err("Message cannot be empty".to_string());
    }
    
    // Maximum IRC message length is 512 bytes, but we need to account for protocol overhead
    // A safe limit for the actual message content is about 400 characters
    if msg.len() > 400 {
        return Err("Message too long (max 400 characters)".to_string());
    }
    
    // Messages cannot contain CR or LF
    if msg.contains('\r') || msg.contains('\n') {
        return Err("Message cannot contain newline characters".to_string());
    }
    
    Ok(())
}

/// Sanitizes a message by removing or replacing invalid characters
pub fn sanitize_message(msg: &str) -> String {
    msg.chars()
        .filter(|&c| c != '\r' && c != '\n' && c != '\0')
        .take(400)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_channel_name() {
        assert!(validate_channel_name("#test").is_ok());
        assert!(validate_channel_name("&local").is_ok());
        assert!(validate_channel_name("#rust-lang").is_ok());
        
        assert!(validate_channel_name("").is_err());
        assert!(validate_channel_name("test").is_err()); // Missing #
        assert!(validate_channel_name("#test channel").is_err()); // Space
        assert!(validate_channel_name("#test,other").is_err()); // Comma
        assert!(validate_channel_name(&"#".to_string().repeat(51)).is_err()); // Too long
    }

    #[test]
    fn test_validate_nickname() {
        assert!(validate_nickname("alice").is_ok());
        assert!(validate_nickname("Bob123").is_ok());
        assert!(validate_nickname("user_name").is_ok());
        assert!(validate_nickname("[guest]").is_ok());
        
        assert!(validate_nickname("").is_err());
        assert!(validate_nickname("123user").is_err()); // Starts with number
        assert!(validate_nickname("user name").is_err()); // Space
        assert!(validate_nickname(&"a".repeat(31)).is_err()); // Too long
    }

    #[test]
    fn test_validate_server_address() {
        assert_eq!(validate_server_address("irc.example.com:6667").unwrap(), 
                   ("irc.example.com".to_string(), 6667));
        assert_eq!(validate_server_address("irc.example.com").unwrap(), 
                   ("irc.example.com".to_string(), 6667));
        assert_eq!(validate_server_address("192.168.1.1:6697").unwrap(), 
                   ("192.168.1.1".to_string(), 6697));
        
        assert!(validate_server_address("").is_err());
        assert!(validate_server_address(":6667").is_err());
        assert!(validate_server_address("host:abc").is_err());
        assert!(validate_server_address("host:0").is_err());
    }

    #[test]
    fn test_validate_message() {
        assert!(validate_message("Hello, world!").is_ok());
        assert!(validate_message("Test message with 日本語").is_ok());
        
        assert!(validate_message("").is_err());
        assert!(validate_message("Line1\nLine2").is_err());
        assert!(validate_message("Line1\rLine2").is_err());
        assert!(validate_message(&"x".repeat(401)).is_err());
    }

    #[test]
    fn test_sanitize_message() {
        assert_eq!(sanitize_message("Hello, world!"), "Hello, world!");
        assert_eq!(sanitize_message("Line1\nLine2"), "Line1Line2");
        assert_eq!(sanitize_message("CR\rLF"), "CRLF");
        assert_eq!(sanitize_message(&"x".repeat(500)), "x".repeat(400));
    }
}
