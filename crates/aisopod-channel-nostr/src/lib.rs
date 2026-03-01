//! Nostr channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Nostr,
//! enabling the bot to connect to Nostr relays, post public messages,
//! send and receive encrypted direct messages (NIP-04), and manage
//! cryptographic keys.
//!
//! # Features
//!
//! - Connect to one or more Nostr relays via WebSocket
//! - Public channel posting (kind 1 events)
//! - Encrypted DMs (kind 4 events, NIP-04)
//! - Key management supporting nsec (private) and npub (public) formats
//! - Multiple relay connection management
//! - Event signing and verification
//!
//! # Example
//!
//! ```no_run
//! use aisopod_channel_nostr::{NostrConfig, NostrChannel, register};
//! use aisopod_channel::ChannelRegistry;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let mut registry = ChannelRegistry::new();
//!     
//!     let config = NostrConfig {
//!         private_key: "nsec...".to_string(),  // or hex format
//!         relays: vec![
//!             "wss://relay.example.com".to_string(),
//!             "wss://relay2.example.com".to_string(),
//!         ],
//!         enable_dms: true,
//!         channels: vec!["npub...".to_string()],  // public channels to follow
//!     };
//!     
//!     register(&mut registry, config, "nostr-main").await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! The Nostr channel is configured using a `NostrConfig` struct:
//!
//! ```toml
//! [[nostr.accounts]]
//! private_key = "nsec1..."  # or hex format
//! relays = [
//!     "wss://relay.example.com",
//!     "wss://relay2.example.com"
//! ]
//! enable_dms = true
//! channels = ["npub1...", "eventid..."]
//! ```
//!
//! # Key Formats
//!
//! Nostr keys can be provided in two formats:
//! - **nsec**: Bech32 format for private keys (starts with `nsec1`)
//! - **hex**: 64-character hex string for private keys
//!
//! Public keys (npub) can be used for reading but not for signing.
//!
//! # NIP-04 Encrypted DMs
//!
//! Encrypted direct messages use NIP-04 specification:
//! - ECDH shared secret between sender's private key and recipient's public key
//! - AES-256-CBC encryption with random IV
//! - Format: `base64(ciphertext)?iv=base64(iv)`

mod channel;
mod config;
mod events;
mod keys;
mod nip04;
mod relay;

// Re-export common types
pub use crate::channel::{register, NostrAccount, NostrChannel};
pub use crate::config::NostrConfig;
pub use crate::events::{EventError, NostrEvent};
pub use crate::keys::NostrKeys;
pub use crate::relay::{RelayConnection, RelayPool};

// Re-export types from aisopod-channel for convenience
pub use aisopod_channel::message::{
    IncomingMessage, Media, MessageContent, MessagePart, MessageTarget, OutgoingMessage, PeerInfo,
    PeerKind, SenderInfo,
};
pub use aisopod_channel::plugin::ChannelPlugin;
pub use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
pub use aisopod_channel::ChannelRegistry;

/// Result type for Nostr channel operations.
pub type NostrResult<T> = std::result::Result<T, anyhow::Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = NostrConfig {
            private_key: "test_key".to_string(),
            relays: vec!["wss://relay.example.com".to_string()],
            enable_dms: true,
            channels: vec![],
        };

        assert!(config.enable_dms);
        assert_eq!(config.relays.len(), 1);
    }

    #[test]
    fn test_config_validation_empty_key() {
        let config = NostrConfig {
            private_key: "".to_string(),
            relays: vec!["wss://relay.example.com".to_string()],
            enable_dms: true,
            channels: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_empty_relays() {
        let config = NostrConfig {
            private_key: "test_key".to_string(),
            relays: vec![],
            enable_dms: true,
            channels: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_relay_url() {
        let config = NostrConfig {
            private_key: "test_key".to_string(),
            relays: vec!["http://invalid.com".to_string()],
            enable_dms: true,
            channels: vec![],
        };

        assert!(config.validate().is_err());
    }
}
