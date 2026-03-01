//! Signal channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Signal,
//! enabling the bot to receive and send messages via signal-cli.
//!
//! # Features
//!
//! - JSON-RPC mode communication with signal-cli daemon
//! - Support for DMs and group messages
//! - Message normalization to shared `IncomingMessage` type
//! - Multi-account support with account-specific configurations
//! - Filtering by allowed phone numbers
//! - Text, image, audio, video, document, and location message support
//! - Media attachment handling
//! - Disappearing message timer detection
//! - Group member management
//!
//! # Requirements
//!
//! - signal-cli version 0.11.0 or later
//! - Rust 1.70 or later
//!
//! # Example
//!
//! ```no_run
//! use aisopod_channel_signal::{SignalAccountConfig, SignalChannel, register};
//! use aisopod_channel::ChannelRegistry;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let mut registry = ChannelRegistry::new();
//!     
//!     let config = SignalAccountConfig::new("+1234567890".to_string());
//!     
//!     register(&mut registry, config, "my-signal-account").await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! The Signal channel uses signal-cli for communication. You need to:
//!
//! 1. Install signal-cli: `cargo install signal-cli`
//! 2. Register your phone number: `signal-cli -u +1234567890 register`
//! 3. Configure the channel in your aisopod config:
//!
//! ```toml
//! [[channels]]
//! type = "signal"
//! account_id = "signal-main"
//! enabled = true
//!
//! [channels.credentials]
//! phone_number = "+1234567890"
//! device_name = "aisopod-bot"
//! disappearing_enabled = true
//! disappearing_timer = 2592000  # 30 days in seconds
//! ```
//!
//! # JSON-RPC Interface
//!
//! The channel communicates with signal-cli via JSON-RPC protocol.
//! signal-cli must be running in daemon mode:
//!
//! ```bash
//! signal-cli -u +1234567890 daemon --json
//! ```
//!
//! # Disappearing Messages
//!
//! The channel supports detecting disappearing message timers:
//!
//! ```rust
//! use aisopod_channel_signal::{SignalGateway, SignalMessage};
//! use serde_json::from_str;
//!
//! let json_str = r#"{
//!     "type": "receive",
//!     "source": "+1234567890",
//!     "timestamp": 1618907555000,
//!     "expires_in": 3600,
//!     "message": {
//!         "body": "This will disappear"
//!     },
//!     "id": "msg123"
//! }"#;
//!
//! let gateway = SignalGateway::new();
//! let signal_msg: SignalMessage = from_str(json_str).expect("Failed to parse");
//!
//! // Check for disappearing message timer
//! if let Some(expires_in) = gateway.extract_disappearing_timer(&signal_msg) {
//!     println!("Message disappears in {} seconds", expires_in);
//! }
//! ```

mod channel;
mod config;
mod gateway;
mod outbound;
mod runtime;

pub use crate::channel::{register, SignalAccount, SignalChannel};
pub use crate::config::utils;
pub use crate::config::{SignalAccountConfig, SignalDaemonConfig, SignalError};
pub use crate::gateway::{
    message_utils, SignalAttachment, SignalGateway, SignalGroup, SignalMessage,
    SignalMessageContent,
};
pub use crate::outbound::SignalOutbound;
pub use crate::runtime::{utils as runtime_utils, SignalRuntime};

// Re-export types from aisopod-channel for convenience
pub use aisopod_channel::message::{
    IncomingMessage, Media, MessageContent, MessagePart, MessageTarget, OutgoingMessage, PeerInfo,
    PeerKind, SenderInfo,
};
pub use aisopod_channel::plugin::ChannelPlugin;
pub use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};

/// Result type for Signal channel operations.
pub type SignalResult<T> = std::result::Result<T, SignalError>;
