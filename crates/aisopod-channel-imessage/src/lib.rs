//! iMessage channel for macOS
//!
//! This crate provides an iMessage channel plugin for the aisopod system,
//! supporting both native macOS AppleScript backend and cross-platform
//! BlueBubbles HTTP API backend.
//!
//! # Features
//!
//! - DM and group chat support
//! - Media attachment support
//! - Platform-specific backends (AppleScript on macOS, BlueBubbles elsewhere)
//! - Multi-account support
//! - Sender filtering and security
//!
//! # Platform Support
//!
//! - **macOS**: Uses AppleScript via osascript (default)
//! - **Other platforms**: Uses BlueBubbles HTTP API
//!
//! # Example
//!
//! ```no_run
//! use aisopod_channel_imessage::{ImessageChannel, ImessageAccountConfig};
//! use aisopod_channel::ChannelRegistry;
//!
//! async fn example() -> Result<(), anyhow::Error> {
//!     let mut registry = ChannelRegistry::new();
//!     
//!     let config = ImessageAccountConfig::new("my-account");
//!     
//!     ImessageChannel::new(config).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod applescript;
pub mod bluebubbles;
pub mod channel;
pub mod config;
pub mod platform;

// Re-export types for convenience
pub use config::{ImessageAccountConfig, ImessageError, ImessageResult, BackendType};
pub use channel::{ImessageChannel, ImessageAccount, register, parse_imessage_message};
pub use applescript::ApplescriptBackend;
pub use bluebubbles::BlueBubblesBackend;

// Re-export message types from aisopod-channel
pub use aisopod_channel::message::{IncomingMessage, OutgoingMessage, MessageTarget, Media, MessageContent, MessagePart, PeerInfo, PeerKind, SenderInfo};
pub use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
