//! IRC channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for IRC,
//! enabling the bot to connect to IRC servers, join channels,
//! send and receive messages (including private messages via PRIVMSG),
//! with support for multiple servers, NickServ authentication, and TLS encryption.
//!
//! # Features
//!
//! - Connect to one or more IRC servers
//! - Channel and DM (PRIVMSG) messaging support
//! - NickServ authentication
//! - TLS-encrypted connections
//! - Multiple simultaneous server connections
//! - Graceful connection management
//!
//! # Example
//!
//! ```no_run
//! use aisopod_channel_irc::{IrcConfig, IrcServerConfig, register};
//! use aisopod_channel::ChannelRegistry;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let mut registry = ChannelRegistry::new();
//!     
//!     let config = IrcConfig {
//!         servers: vec![
//!             IrcServerConfig {
//!                 server: "irc.libera.chat".to_string(),
//!                 port: 6697,
//!                 use_tls: true,
//!                 nickname: "aisopod-bot".to_string(),
//!                 nickserv_password: None,
//!                 channels: vec!["#aisopod".to_string(), "#general".to_string()],
//!                 server_password: None,
//!             },
//!         ],
//!     };
//!     
//!     register(&mut registry, config, "irc-main").await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! The IRC channel is configured using an `IrcConfig` struct containing
//! one or more server configurations:
//!
//! ```toml
//! [[irc.servers]]
//! server = "irc.libera.chat"
//! port = 6697
//! use_tls = true
//! nickname = "aisopod-bot"
//! channels = ["#aisopod", "#general"]
//! nickserv_password = "my-secret-password"
//! server_password = null
//! ```
//!
//! # Connection Lifecycle
//!
//! 1. Create an `IrcChannel` with your configuration
//! 2. Call `connect()` to establish connections to all servers
//! 3. Messages are sent via `send_message()` and received via `receive()`
//! 4. Call `disconnect()` to gracefully close connections
//!
//! # Authentication
//!
//! NickServ authentication is handled automatically if a password is configured.
//! The bot will send an IDENTIFY command to NickServ after connecting.

mod auth;
mod channel;
mod client;
mod config;

// Re-export common types
pub use crate::auth::authenticate_nickserv;
pub use crate::channel::{register, IrcAccount, IrcChannel};
pub use crate::client::IrcConnection;
pub use crate::config::{IrcConfig, IrcServerConfig};

// Re-export types from aisopod-channel for convenience
pub use aisopod_channel::message::{
    IncomingMessage, Media, MessageContent, MessagePart, MessageTarget, OutgoingMessage, PeerInfo,
    PeerKind, SenderInfo,
};
pub use aisopod_channel::plugin::ChannelPlugin;
pub use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
pub use aisopod_channel::ChannelRegistry;

/// Result type for IRC channel operations.
pub type IrcResult<T> = std::result::Result<T, anyhow::Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irc_config_default() {
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
