//! Matrix channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Matrix,
//! enabling the bot to receive and send messages via the Matrix protocol.
//!
//! # Features
//!
//! - Connection to Matrix homeserver
//! - Password, access token, and SSO authentication
//! - Room and direct message support
//! - End-to-end encryption (optional)
//! - Message filtering by allowed users
//! - Group mention requirements
//! - Multi-account support
//!
//! # Example
//!
//! ```no_run
//! use aisopod_channel_matrix::{MatrixAccountConfig, MatrixAuth, register};
//! use aisopod_channel::ChannelRegistry;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let mut registry = ChannelRegistry::new();
//!
//!     let config = MatrixAccountConfig {
//!         homeserver_url: "https://matrix.org".to_string(),
//!         auth: MatrixAuth::Password {
//!             username: "bot".to_string(),
//!             password: "password".to_string(),
//!         },
//!         enable_e2ee: true,
//!         rooms: vec!["!room:matrix.org".to_string()],
//!         state_store_path: Some(PathBuf::from("/tmp/matrix-state")),
//!         ..Default::default()
//!     };
//!
//!     register(&mut registry, config, "matrix-main").await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! The Matrix channel supports multiple authentication methods:
//!
//! ## Password Authentication
//!
//! ```toml
//! [[channels]]
//! type = "matrix"
//! account_id = "matrix-main"
//! enabled = true
//!
//! [channels.credentials]
//! homeserver_url = "https://matrix.org"
//! type = "password"
//! username = "bot"
//! password = "password"
//!
//! [channels.rooms]
//! rooms = ["!room:matrix.org"]
//! enable_e2ee = true
//! ```
//!
//! ## Access Token Authentication
//!
//! ```toml
//! [channels.credentials]
//! type = "token"
//! access_token = "your_access_token_here"
//! ```
//!
//! ## SSO Authentication
//!
//! ```toml
//! [channels.credentials]
//! type = "sso"
//! token = "your_sso_token_here"
//! ```
//!
//! # End-to-End Encryption
//!
//! E2EE is enabled by default. To disable:
//!
//! ```toml
//! [channels.rooms]
//! enable_e2ee = false
//! ```
//!
//! # Security
//!
//! You can configure an allowed users list:
//!
//! ```toml
//! [channels.rooms]
//! allowed_users = ["@user1:matrix.org", "@user2:matrix.org"]
//! requires_mention_in_group = true
//! ```

mod channel;
mod client;
mod config;
mod encryption;

// Re-export common types
pub use crate::channel::{
    matrix_event_to_incoming_message, register, MatrixAccount, MatrixChannel,
    MatrixChannelConfigAdapter, MatrixSecurityAdapter,
};
pub use crate::client::MatrixClient;
pub use crate::config::{MatrixAccountConfig, MatrixAuth};
pub use crate::encryption::{setup_e2ee, E2EEConfig};

// Re-export types from aisopod-channel for convenience
pub use aisopod_channel::message::{
    IncomingMessage, Media, MessageContent, MessagePart, MessageTarget, OutgoingMessage, PeerInfo,
    PeerKind, SenderInfo,
};
pub use aisopod_channel::plugin::ChannelPlugin;
pub use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
pub use aisopod_channel::ChannelRegistry;

/// Result type for Matrix channel operations.
pub type MatrixResult<T> = std::result::Result<T, anyhow::Error>;
