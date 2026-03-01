//! IRC channel configuration types.
//!
//! This module defines configuration structures for the IRC channel plugin,
//! including server connection settings and authentication options.

use serde::{Deserialize, Serialize};

/// Configuration for IRC servers.
#[derive(Debug, Deserialize, Clone)]
pub struct IrcConfig {
    /// List of IRC server connections
    pub servers: Vec<IrcServerConfig>,
}

/// Configuration for a single IRC server connection.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct IrcServerConfig {
    /// Server hostname (e.g., "irc.libera.chat")
    pub server: String,
    /// Server port (default: 6697 for TLS, 6667 for plain)
    #[serde(default = "default_port")]
    pub port: u16,
    /// Use TLS encryption
    #[serde(default = "default_use_tls")]
    pub use_tls: bool,
    /// Bot nickname
    pub nickname: String,
    /// Optional NickServ password
    pub nickserv_password: Option<String>,
    /// Channels to join (e.g., ["#channel1", "#channel2"])
    pub channels: Vec<String>,
    /// Server password (for password-protected servers)
    pub server_password: Option<String>,
}

fn default_port() -> u16 {
    6697 // Default to TLS port
}

fn default_use_tls() -> bool {
    true // Default to TLS
}

impl Default for IrcServerConfig {
    fn default() -> Self {
        Self {
            server: String::new(),
            port: default_port(),
            use_tls: default_use_tls(),
            nickname: String::new(),
            nickserv_password: None,
            channels: Vec::new(),
            server_password: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irc_server_config_default() {
        let config = IrcServerConfig::default();
        assert_eq!(config.port, 6697);
        assert!(config.use_tls);
    }

    #[test]
    fn test_irc_config_serialization() {
        let config = IrcServerConfig {
            server: "irc.example.com".to_string(),
            port: 6697,
            use_tls: true,
            nickname: "testbot".to_string(),
            nickserv_password: Some("secret".to_string()),
            channels: vec!["#test".to_string()],
            server_password: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("irc.example.com"));
        assert!(json.contains("testbot"));

        let deserialized: IrcServerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.server, config.server);
        assert_eq!(deserialized.nickname, config.nickname);
    }
}
