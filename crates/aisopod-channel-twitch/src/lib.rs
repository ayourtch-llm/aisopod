//! Twitch channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Twitch,
//! enabling the bot to connect to Twitch chat via the TMI WebSocket interface,
//! join channels, send and receive messages, and support whispers.
//!
//! # Features
//!
//! - Connect to Twitch chat via TMI WebSocket
//! - Channel messaging (send and receive)
//! - Whisper (private message) support
//! - Moderator and subscriber status detection
//! - OAuth authentication validation
//! - Multiple channel joins
//!
//! # Example
//!
//! ```no_run
//! use aisopod_channel_twitch::{TwitchConfig, register};
//! use aisopod_channel::ChannelRegistry;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let mut registry = ChannelRegistry::new();
//!     
//!     let config = TwitchConfig {
//!         username: "aisopod-bot".to_string(),
//!         oauth_token: "oauth:abc123...".to_string(),
//!         channels: vec!["#aisopod".to_string(), "#general".to_string()],
//!         enable_whispers: false,
//!         client_id: Some("your_client_id".to_string()),
//!     };
//!     
//!     register(&mut registry, config, "twitch-main").await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! The Twitch channel is configured using a `TwitchConfig` struct:
//!
//! ```toml
//! [twitch]
//! username = "aisopod-bot"
//! oauth_token = "oauth:abc123..."
//! channels = ["#aisopod", "#general"]
//! enable_whispers = false
//! client_id = "your_client_id"
//! ```
//!
//! # Connection Lifecycle
//!
//! 1. Create a `TwitchChannel` with your configuration
//! 2. Call `connect()` to establish connection to Twitch TMI
//! 3. Join channels using `join_channel()` or via config
//! 4. Messages are sent via `send_message()` and received via `receive()`
//! 5. Call `disconnect()` to gracefully close the connection

mod auth;
mod badges;
mod channel;
mod config;
mod tmi;

// Re-export common types
pub use crate::auth::{validate_token, validate_token_blocking, is_token_expired, token_has_scopes, AuthError, TokenInfo};
pub use crate::badges::{Badge, parse_badges, is_moderator, is_subscriber, is_broadcaster, is_vip, parse_badge_info};
pub use crate::channel::{register, TwitchAccount, TwitchChannel};
pub use crate::config::TwitchConfig;
pub use crate::tmi::{TmiClient, TwitchMessage, TwitchTags, parse_irc_line};

// Re-export types from aisopod-channel for convenience
pub use aisopod_channel::message::{IncomingMessage, OutgoingMessage, MessageTarget, Media, MessageContent, MessagePart, PeerInfo, PeerKind, SenderInfo};
pub use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
pub use aisopod_channel::plugin::ChannelPlugin;
pub use aisopod_channel::ChannelRegistry;

/// Result type for Twitch channel operations.
pub type TwitchResult<T> = std::result::Result<T, anyhow::Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twitch_config_default() {
        let config = TwitchConfig {
            username: "testbot".to_string(),
            oauth_token: "oauth:abc123".to_string(),
            channels: vec!["#test".to_string()],
            enable_whispers: false,
            client_id: None,
        };
        
        assert!(config.validate().is_ok());
        assert_eq!(config.username, "testbot");
        assert!(config.channels.contains(&"#test".to_string()));
    }

    #[test]
    fn test_twitch_config_serialization() {
        let config = TwitchConfig {
            username: "testbot".to_string(),
            oauth_token: "oauth:abc123".to_string(),
            channels: vec!["#test".to_string(), "#another".to_string()],
            enable_whispers: true,
            client_id: Some("client123".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("testbot"));
        assert!(json.contains("#test"));
        assert!(json.contains("#another"));
        assert!(json.contains("true"));
        assert!(json.contains("client123"));
    }

    #[test]
    fn test_badge_parsing() {
        let badges = parse_badges("moderator/1,subscriber/12");
        assert_eq!(badges.len(), 2);
        assert_eq!(badges[0].name, "moderator");
        assert_eq!(badges[0].version, "1");
        assert_eq!(badges[1].name, "subscriber");
        assert_eq!(badges[1].version, "12");
    }

    #[test]
    fn test_moderator_detection() {
        let badges = parse_badges("moderator/1,subscriber/12");
        assert!(is_moderator(&badges));
    }

    #[test]
    fn test_subscriber_detection() {
        let badges = parse_badges("moderator/1,subscriber/12");
        assert!(is_subscriber(&badges));
    }
}
