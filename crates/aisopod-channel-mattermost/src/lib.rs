//! Mattermost channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Mattermost,
//! enabling the bot to receive and send messages via the Mattermost REST API
//! and WebSocket event streaming.
//!
//! # Features
//!
//! - REST API client for sending messages
//! - WebSocket event streaming for receiving messages in real-time
//! - Bot token and personal access token authentication
//! - Password authentication support
//! - Channel and DM messaging
//! - Self-hosted server URL configuration
//! - Multi-account support
//!
//! # Example
//!
//! ```no_run
//! use aisopod_channel_mattermost::{MattermostConfig, MattermostAuth, register};
//! use aisopod_channel::ChannelRegistry;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let mut registry = ChannelRegistry::new();
//!
//!     let config = MattermostConfig::new("https://mattermost.example.com".to_string())
//!         .with_auth(MattermostAuth::BotToken {
//!             token: "your-bot-token".to_string(),
//!         })
//!         .with_channels(vec!["general".to_string()]);
//!
//!     register(&mut registry, config, "mattermost-main").await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! ```toml
//! [[channels]]
//! type = "mattermost"
//! account_id = "mattermost-main"
//! enabled = true
//!
//! [channels.credentials]
//! server_url = "https://mattermost.example.com"
//! team = "myteam"
//! channels = ["general", "random"]
//!
//! [channels.credentials.auth]
//! type = "bot"
//! token = "your-bot-token"
//! ```
//!
//! # Authentication Methods
//!
//! ## Bot Token
//!
//! ```toml
//! [channels.credentials.auth]
//! type = "bot"
//! token = "your-bot-token"
//! ```
//!
//! ## Personal Access Token
//!
//! ```toml
//! [channels.credentials.auth]
//! type = "personal"
//! token = "your-personal-token"
//! ```
//!
//! ## Username/Password
//!
//! ```toml
//! [channels.credentials.auth]
//! type = "password"
//! username = "your-username"
//! password = "your-password"
//! ```

mod api;
mod auth;
mod channel;
mod config;
mod websocket;

// Re-export common types
pub use crate::api::{ApiError, Channel, ChannelType, MattermostApi, Post, User};
pub use crate::auth::{AuthError, AuthResult};
pub use crate::channel::{
    register, MattermostAccount, MattermostAccountWithConnections, MattermostChannel,
};
pub use crate::config::{MattermostAuth, MattermostConfig};
pub use crate::websocket::{MattermostEvent, MattermostWebSocket, WebSocketError};

// Re-export types from aisopod-channel for convenience
pub use aisopod_channel::message::{
    IncomingMessage, Media, MessageContent, MessagePart, MessageTarget, OutgoingMessage, PeerInfo,
    PeerKind, SenderInfo,
};
pub use aisopod_channel::plugin::ChannelPlugin;
pub use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};

/// Result type for Mattermost channel operations.
pub type MattermostResult<T> = std::result::Result<T, anyhow::Error>;
