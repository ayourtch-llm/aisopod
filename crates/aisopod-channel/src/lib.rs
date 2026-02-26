//! # aisopod-channel
//!
//! Communication channels, message routing, and I/O abstractions.
//!
//! ## Overview
//!
//! This crate provides the core abstractions for channel implementations:
//!
//! - [`ChannelPlugin`] - The main trait that all channel plugins must implement
//! - [`ChannelMeta`] - Metadata about a channel implementation
//! - [`ChannelCapabilities`] - Describes what features a channel supports
//! - [`ChatType`] - Enum for chat conversation types
//! - [`MediaType`] - Enum for media content types
//!
//! ## Channel Registry
//!
//! The crate includes a [`ChannelRegistry`] for managing channel plugins:
//!
//! - [`ChannelRegistry`] - Central registry for channel plugins
//! - [`ChannelAlias`] - Alias mapping for channel IDs
//!
//! ## Adapter Traits
//!
//! The crate also defines 13 optional adapter traits for channel capabilities:
//!
//! - [`OnboardingAdapter`] - CLI onboarding wizard
//! - [`OutboundAdapter`] - Message delivery
//! - [`GatewayAdapter`] - WebSocket/polling connection lifecycle
//! - [`StatusAdapter`] - Health monitoring
//! - [`TypingAdapter`] - Typing indicators
//! - [`MessagingAdapter`] - Message reactions
//! - [`ThreadingAdapter`] - Thread/reply support
//! - [`DirectoryAdapter`] - Group/user discovery
//! - [`SecurityAdapter`] - Security and DM policies
//! - [`HeartbeatAdapter`] - Keep-alive mechanism
//! - [`ChannelConfigAdapter`] - Account management
//! - [`AuthAdapter`] - Token/credential management
//! - [`PairingAdapter`] - Device pairing
//!
//! ## Shared Utilities
//!
//! The crate includes a [`util`] module with shared utilities for all channel
//! implementations:
//!
//! - [`util::NormalizedMarkdown`] - Cross-platform message formatting
//! - [`util::MediaAttachment`] - Media transcoding utilities
//! - [`util::RateLimiter`] - API rate limiting
//! - [`util::ConnectionManager`] - Connection state management
//! - [`util::ChannelError`] - Common error types
//!
//! ## Example
//!
//! ```rust,ignore
//! use aisopod_channel::{ChannelPlugin, ChannelMeta, ChannelCapabilities, ChatType, MediaType};
//!
//! struct MyChannel {
//!     // channel-specific fields
//! }
//!
//! impl ChannelPlugin for MyChannel {
//!     fn id(&self) -> &str {
//!         "my-channel"
//!     }
//!
//!     fn meta(&self) -> &ChannelMeta {
//!         // return metadata
//!     }
//!
//!     fn capabilities(&self) -> &ChannelCapabilities {
//!         // return capabilities
//!     }
//!
//!     fn config(&self) -> &dyn ChannelConfigAdapter {
//!         // return config adapter
//!     }
//! }
//! ```

pub mod adapters;
pub mod channel;
pub mod media;
pub mod message;
pub mod plugin;
pub mod router;
pub mod security;
pub mod types;
pub mod util;

// Re-export message types
pub use message::{
    IncomingMessage, OutgoingMessage, MessageContent, MessagePart, MessageTarget,
    PeerInfo, PeerKind, Media,
};

// Re-export router
pub use router::{MessageRouter, AgentResolver, ConfigAgentResolver};

// Re-export security types
pub use security::{SecurityEnforcer, MentionCheckResult};

// Re-export core types
pub use plugin::ChannelPlugin;
pub use types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};

// Re-export media utilities
pub use media::{
    AudioTranscriber, DocumentExtractor, detect_media_type, detect_mime_type,
    resize_image, convert_image_format, validate_media,
};

// Re-export adapter traits and types from adapters module
pub use adapters::{
    AccountConfig, AccountSnapshot, AuthAdapter, AuthToken, ChannelConfigAdapter,
    ChannelHealth, DirectoryAdapter, GatewayAdapter, GroupInfo, HeartbeatAdapter,
    MemberInfo, MessagingAdapter, OnboardingAdapter, OnboardingContext,
    PairingAdapter, PairingCode, SecurityAdapter, StatusAdapter, ThreadingAdapter,
    TypingAdapter, OutboundAdapter,
};

// Re-export channel registry
pub use channel::{ChannelAlias, ChannelRegistry};

// Re-export shared utilities
pub use util::{
    connection::{ConnectionManager, ConnectionState},
    errors::ChannelError,
    formatting::{from_discord_markdown, from_plain_text, from_slack_mrkdwn, from_telegram_markdown, NormalizedMarkdown},
    media::{ensure_compatible_format, MediaAttachment, Platform},
    rate_limit::{RateLimiter, RateLimitConfig, RateLimitError},
};

// Re-export anyhow Result for convenience
pub use anyhow::Result;
