//! Core ChannelPlugin trait and related types.
//!
//! This module defines the `ChannelPlugin` trait which all channel implementations
//! must implement. It provides a unified interface for accessing channel metadata,
//! capabilities, and configuration.

use crate::types::{ChannelCapabilities, ChannelMeta};
use crate::adapters::{ChannelConfigAdapter, SecurityAdapter};
use crate::message::{IncomingMessage, OutgoingMessage};
use crate::Result;
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

    /// Returns the security adapter for this channel if available.
    ///
    /// The security adapter provides methods for enforcing security policies,
    /// such as sender allowlists and group mention requirements.
    /// Returns `None` if the channel doesn't implement security checks.
    fn security(&self) -> Option<&dyn SecurityAdapter>;

    /// Connect to the channel service.
    ///
    /// This method establishes the connection to the channel's backend service.
    /// Implementations should handle authentication, WebSocket connections,
    /// and any other setup required for communication.
    ///
    /// The default implementation is a no-op. Implementations that need
    /// connection management should override this method.
    async fn connect(&mut self) -> Result<()> {
        Ok(())
    }

    /// Send a message through the channel.
    ///
    /// # Arguments
    ///
    /// * `msg` - The outgoing message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(Error)` - An error occurred while sending
    ///
    /// The default implementation returns an error indicating send is not implemented.
    async fn send(&self, _msg: OutgoingMessage) -> Result<()> {
        Err(anyhow::anyhow!("Send is not implemented for this channel"))
    }

    /// Receive a message from the channel.
    ///
    /// This method should block until a message is available or an error occurs.
    /// Implementations should use WebSocket events, polling, or other appropriate
    /// mechanisms to receive incoming messages.
    ///
    /// # Returns
    ///
    /// * `Ok(IncomingMessage)` - A received message
    /// * `Err(Error)` - An error occurred while receiving
    ///
    /// The default implementation returns an error indicating receive is not implemented.
    async fn receive(&mut self) -> Result<IncomingMessage> {
        Err(anyhow::anyhow!("Receive is not implemented for this channel"))
    }

    /// Disconnect from the channel service.
    ///
    /// This method closes the connection to the channel's backend service,
    /// cleaning up any resources like WebSocket connections.
    ///
    /// The default implementation is a no-op. Implementations that need
    /// connection management should override this method.
    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }
}
