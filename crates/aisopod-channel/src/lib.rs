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

pub mod plugin;
pub mod types;

pub use plugin::ChannelPlugin;
pub use types::{ChannelCapabilities, ChannelConfigAdapter, ChannelMeta, ChatType, MediaType};
