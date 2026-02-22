//! Core ChannelPlugin trait and related types.
//!
//! This module defines the `ChannelPlugin` trait which all channel implementations
//! must implement. It provides a unified interface for accessing channel metadata,
//! capabilities, and configuration.

use crate::types::{ChannelCapabilities, ChannelMeta};
use crate::adapters::ChannelConfigAdapter;
use async_trait::async_trait;

/// The core trait that all channel plugins must implement.
///
/// This trait provides access to channel metadata, supported capabilities,
/// and configuration. It serves as the primary interface between the
/// aisopod system and channel implementations.
///
/// # Lifecycle
///
/// Channel plugins are typically instantiated once per configured account
/// and remain active for the duration of the application session.
#[async_trait]
pub trait ChannelPlugin: Send + Sync {
    /// Returns the unique identifier for this channel plugin.
    ///
    /// The ID should be a lowercase hyphenated string that uniquely identifies
    /// the channel type (e.g., "telegram", "discord", "matrix").
    fn id(&self) -> &str;

    /// Returns metadata about this channel implementation.
    ///
    /// The metadata includes the display label, documentation URL,
    /// and UI hints for configuration.
    fn meta(&self) -> &ChannelMeta;

    /// Returns the capabilities supported by this channel.
    ///
    /// The capabilities describe what features this channel supports,
    /// allowing the system to adapt its behavior and UI accordingly.
    fn capabilities(&self) -> &ChannelCapabilities;

    /// Returns the configuration adapter for this channel.
    ///
    /// The configuration adapter provides methods for managing channel accounts,
    /// including listing, enabling, disabling, and deleting accounts.
    fn config(&self) -> &dyn ChannelConfigAdapter;
}
