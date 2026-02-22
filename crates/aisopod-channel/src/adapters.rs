//! Adapter interface traits for optional channel capabilities.
//!
//! This module defines 13 adapter traits that channel plugins can implement
//! to provide optional functionality beyond the core messaging capabilities.
//!
//! Each adapter trait represents a specific capability domain, from onboarding
//! to device pairing. Channel plugins implement only the adapters they support.

use crate::message::{Media, MessageTarget, SenderInfo};
use crate::types::{ChannelMeta, ChannelCapabilities};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Supporting Types
// ============================================================================

/// Context for the onboarding wizard flow.
#[derive(Debug, Clone)]
pub struct OnboardingContext {
    /// Directory for storing configuration files.
    pub config_dir: PathBuf,
    /// Optional parent directory for channel-specific configs.
    pub channel_config_dir: Option<PathBuf>,
}

/// Configuration for an account instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    /// Unique identifier for this account.
    pub id: String,
    /// Channel type this account connects to (e.g., "telegram", "discord").
    pub channel: String,
    /// JSON-encoded credentials and settings.
    pub credentials: serde_json::Value,
    /// Whether this account is enabled.
    pub enabled: bool,
}

/// Health status of a channel connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelHealth {
    /// Connection is healthy and fully operational.
    Healthy,
    /// Connection is degraded but partially functional.
    Degraded(String),
    /// Connection is disconnected with optional reason.
    Disconnected(String),
}

/// Information about a group or channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    /// Unique identifier for the group.
    pub id: String,
    /// Human-readable name of the group.
    pub name: String,
}

/// Information about a group member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberInfo {
    /// Unique identifier for the member.
    pub id: String,
    /// Display name of the member.
    pub display_name: String,
}

/// Snapshot of an account's current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSnapshot {
    /// Unique identifier for the account.
    pub id: String,
    /// Channel type this account connects to.
    pub channel: String,
    /// Whether this account is enabled.
    pub enabled: bool,
    /// Whether this account is currently connected.
    pub connected: bool,
}

/// Authentication token for API access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// The token string.
    pub token: String,
    /// Optional expiration timestamp.
    pub expires_at: Option<DateTime<Utc>>,
}

/// Pairing code for device registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingCode {
    /// The pairing code string.
    pub code: String,
    /// Expiration timestamp for the pairing code.
    pub expires_at: DateTime<Utc>,
    /// Optional URL for QR code display.
    pub qr_url: Option<String>,
}

// ============================================================================
// Adapter Traits
// ============================================================================

/// Adapter for CLI onboarding wizard.
///
/// This trait provides methods for guiding users through the process of
/// configuring a new channel account via an interactive CLI wizard.
#[async_trait]
pub trait OnboardingAdapter: Send + Sync {
    /// Run the onboarding setup wizard and return the resulting account config.
    ///
    /// This method should interactively guide the user through all necessary
    /// configuration steps for setting up a new channel account.
    ///
    /// # Arguments
    /// * `ctx` - The onboarding context with configuration directory paths.
    ///
    /// # Returns
    /// * `Ok(AccountConfig)` - The configured account if successful.
    /// * `Err(anyhow::Error)` - An error if onboarding fails or is cancelled.
    async fn setup_wizard(&self, ctx: &OnboardingContext) -> Result<AccountConfig, anyhow::Error>;
}

/// Adapter for outbound message delivery.
///
/// This trait provides methods for sending messages and media content
/// through the channel.
#[async_trait]
pub trait OutboundAdapter: Send + Sync {
    /// Send a text message to the specified target.
    ///
    /// # Arguments
    /// * `target` - The message target specifying where to send.
    /// * `text` - The plain text content to send.
    ///
    /// # Returns
    /// * `Ok(())` - Message was sent successfully.
    /// * `Err(anyhow::Error)` - An error if sending fails.
    async fn send_text(&self, target: &MessageTarget, text: &str) -> Result<(), anyhow::Error>;

    /// Send media content to the specified target.
    ///
    /// # Arguments
    /// * `target` - The message target specifying where to send.
    /// * `media` - The media content to send.
    ///
    /// # Returns
    /// * `Ok(())` - Media was sent successfully.
    /// * `Err(anyhow::Error)` - An error if sending fails.
    async fn send_media(&self, target: &MessageTarget, media: &Media) -> Result<(), anyhow::Error>;
}

/// Adapter for gateway connection lifecycle management.
///
/// This trait provides methods for managing the underlying connection
/// to the channel's API (WebSocket, polling, or other protocols).
#[async_trait]
pub trait GatewayAdapter: Send + Sync {
    /// Establish a connection for the given account.
    ///
    /// # Arguments
    /// * `account` - The account configuration to use for connection.
    ///
    /// # Returns
    /// * `Ok(())` - Connection was established successfully.
    /// * `Err(anyhow::Error)` - An error if connection fails.
    async fn connect(&self, account: &AccountConfig) -> Result<(), anyhow::Error>;

    /// Close the connection for the given account.
    ///
    /// # Arguments
    /// * `account` - The account configuration whose connection to close.
    ///
    /// # Returns
    /// * `Ok(())` - Disconnection was successful.
    /// * `Err(anyhow::Error)` - An error if disconnection fails.
    async fn disconnect(&self, account: &AccountConfig) -> Result<(), anyhow::Error>;

    /// Check if the given account is currently connected.
    ///
    /// # Arguments
    /// * `account` - The account configuration to check.
    ///
    /// # Returns
    /// * `true` - The account has an active connection.
    /// * `false` - The account is not connected.
    fn is_connected(&self, account: &AccountConfig) -> bool;
}

/// Adapter for channel health monitoring.
///
/// This trait provides methods for checking the health and status
/// of channel connections.
#[async_trait]
pub trait StatusAdapter: Send + Sync {
    /// Perform a health check on the given account.
    ///
    /// # Arguments
    /// * `account` - The account configuration to check.
    ///
    /// # Returns
    /// * `Ok(ChannelHealth)` - The current health status.
    /// * `Err(anyhow::Error)` - An error if the check fails.
    async fn health_check(&self, account: &AccountConfig) -> Result<ChannelHealth, anyhow::Error>;
}

/// Adapter for typing indicators.
///
/// This trait provides methods for indicating when the bot is typing,
/// which is useful for providing a more natural conversational experience.
#[async_trait]
pub trait TypingAdapter: Send + Sync {
    /// Send a typing indicator to the target.
    ///
    /// This indicates that the bot is currently composing a response.
    /// The indicator typically disappears after a short timeout or
    /// when a message is actually sent.
    ///
    /// # Arguments
    /// * `target` - The message target where typing should be indicated.
    ///
    /// # Returns
    /// * `Ok(())` - Typing indicator was sent successfully.
    /// * `Err(anyhow::Error)` - An error if sending fails.
    async fn send_typing(&self, target: &MessageTarget) -> Result<(), anyhow::Error>;
}

/// Adapter for message reactions.
///
/// This trait provides methods for adding and removing emoji reactions
/// to messages, allowing for richer interaction patterns.
#[async_trait]
pub trait MessagingAdapter: Send + Sync {
    /// Add an emoji reaction to a message.
    ///
    /// # Arguments
    /// * `message_id` - The identifier of the message to react to.
    /// * `emoji` - The emoji to add as a reaction.
    ///
    /// # Returns
    /// * `Ok(())` - Reaction was added successfully.
    /// * `Err(anyhow::Error)` - An error if adding fails.
    async fn react(&self, message_id: &str, emoji: &str) -> Result<(), anyhow::Error>;

    /// Remove an emoji reaction from a message.
    ///
    /// # Arguments
    /// * `message_id` - The identifier of the message to unreact from.
    /// * `emoji` - The emoji to remove from the reaction list.
    ///
    /// # Returns
    /// * `Ok(())` - Reaction was removed successfully.
    /// * `Err(anyhow::Error)` - An error if removal fails.
    async fn unreact(&self, message_id: &str, emoji: &str) -> Result<(), anyhow::Error>;
}

/// Adapter for thread and reply support.
///
/// This trait provides methods for creating new threads and replying
/// within existing threads in channels that support threading.
#[async_trait]
pub trait ThreadingAdapter: Send + Sync {
    /// Create a new thread in the specified parent message.
    ///
    /// # Arguments
    /// * `parent_id` - The identifier of the parent message to thread under.
    /// * `title` - The title for the new thread.
    ///
    /// # Returns
    /// * `Ok(String)` - The identifier of the newly created thread.
    /// * `Err(anyhow::Error)` - An error if creation fails.
    async fn create_thread(&self, parent_id: &str, title: &str) -> Result<String, anyhow::Error>;

    /// Reply to an existing thread.
    ///
    /// # Arguments
    /// * `thread_id` - The identifier of the thread to reply to.
    /// * `text` - The text content for the reply.
    ///
    /// # Returns
    /// * `Ok(())` - Reply was sent successfully.
    /// * `Err(anyhow::Error)` - An error if sending fails.
    async fn reply_in_thread(&self, thread_id: &str, text: &str) -> Result<(), anyhow::Error>;
}

/// Adapter for group and user directory discovery.
///
/// This trait provides methods for listing and discovering groups,
/// channels, and their members.
#[async_trait]
pub trait DirectoryAdapter: Send + Sync {
    /// List all accessible groups for the given account.
    ///
    /// # Arguments
    /// * `account` - The account configuration to use for the request.
    ///
    /// # Returns
    /// * `Ok(Vec<GroupInfo>)` - List of groups the user can access.
    /// * `Err(anyhow::Error)` - An error if the request fails.
    async fn list_groups(&self, account: &AccountConfig) -> Result<Vec<GroupInfo>, anyhow::Error>;

    /// List all members of a specific group.
    ///
    /// # Arguments
    /// * `group_id` - The identifier of the group to list members for.
    ///
    /// # Returns
    /// * `Ok(Vec<MemberInfo>)` - List of group members.
    /// * `Err(anyhow::Error)` - An error if the request fails.
    async fn list_members(&self, group_id: &str) -> Result<Vec<MemberInfo>, anyhow::Error>;
}

/// Adapter for security and access control.
///
/// This trait provides methods for enforcing security policies,
/// such as sender allowlists and group mention requirements.
#[async_trait]
pub trait SecurityAdapter: Send + Sync {
    /// Check if the given sender is allowed to interact with the bot.
    ///
    /// # Arguments
    /// * `sender` - Information about the sender to check.
    ///
    /// # Returns
    /// * `true` - The sender is allowed.
    /// * `false` - The sender is not allowed.
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool;

    /// Check if messages in group chats require the bot to be mentioned.
    ///
    /// # Returns
    /// * `true` - The bot must be mentioned to receive messages in groups.
    /// * `false` - The bot can respond to messages without being mentioned.
    fn requires_mention_in_group(&self) -> bool;
}

/// Adapter for heartbeat/keep-alive mechanisms.
///
/// This trait provides methods for maintaining persistent connections
/// through periodic heartbeat signals.
#[async_trait]
pub trait HeartbeatAdapter: Send + Sync {
    /// Send a heartbeat signal for the given account.
    ///
    /// This method should be called periodically to maintain the
    /// connection with the channel's API.
    ///
    /// # Arguments
    /// * `account` - The account configuration to send heartbeat for.
    ///
    /// # Returns
    /// * `Ok(())` - Heartbeat was sent successfully.
    /// * `Err(anyhow::Error)` - An error if sending fails.
    async fn heartbeat(&self, account: &AccountConfig) -> Result<(), anyhow::Error>;

    /// Get the recommended heartbeat interval for this channel.
    ///
    /// # Returns
    /// * `Duration` - The recommended interval between heartbeats.
    fn heartbeat_interval(&self) -> std::time::Duration;
}

/// Adapter for channel configuration management.
///
/// This trait provides methods for managing channel accounts, including
/// listing, enabling, disabling, and deleting accounts.
#[async_trait]
pub trait ChannelConfigAdapter: Send + Sync {
    /// List all configured account IDs for this channel.
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of all account IDs.
    /// * `Err(anyhow::Error)` - An error if listing fails.
    fn list_accounts(&self) -> Result<Vec<String>, anyhow::Error>;

    /// Resolve an account by its ID to a full snapshot.
    ///
    /// # Arguments
    /// * `id` - The account ID to resolve.
    ///
    /// # Returns
    /// * `Ok(AccountSnapshot)` - The resolved account state.
    /// * `Err(anyhow::Error)` - An error if resolution fails.
    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot, anyhow::Error>;

    /// Enable an account by its ID.
    ///
    /// # Arguments
    /// * `id` - The account ID to enable.
    ///
    /// # Returns
    /// * `Ok(())` - Account was enabled successfully.
    /// * `Err(anyhow::Error)` - An error if enabling fails.
    fn enable_account(&self, id: &str) -> Result<(), anyhow::Error>;

    /// Disable an account by its ID.
    ///
    /// # Arguments
    /// * `id` - The account ID to disable.
    ///
    /// # Returns
    /// * `Ok(())` - Account was disabled successfully.
    /// * `Err(anyhow::Error)` - An error if disabling fails.
    fn disable_account(&self, id: &str) -> Result<(), anyhow::Error>;

    /// Delete an account by its ID.
    ///
    /// # Arguments
    /// * `id` - The account ID to delete.
    ///
    /// # Returns
    /// * `Ok(())` - Account was deleted successfully.
    /// * `Err(anyhow::Error)` - An error if deletion fails.
    fn delete_account(&self, id: &str) -> Result<(), anyhow::Error>;
}

/// Adapter for authentication and token management.
///
/// This trait provides methods for authenticating with the channel's API
/// and managing authentication tokens.
#[async_trait]
pub trait AuthAdapter: Send + Sync {
    /// Authenticate with the given account configuration.
    ///
    /// # Arguments
    /// * `config` - The account configuration with credentials.
    ///
    /// # Returns
    /// * `Ok(AuthToken)` - The authentication token if successful.
    /// * `Err(anyhow::Error)` - An error if authentication fails.
    async fn authenticate(&self, config: &AccountConfig) -> Result<AuthToken, anyhow::Error>;

    /// Refresh an expired authentication token.
    ///
    /// # Arguments
    /// * `token` - The current authentication token to refresh.
    ///
    /// # Returns
    /// * `Ok(AuthToken)` - The new authentication token.
    /// * `Err(anyhow::Error)` - An error if refresh fails.
    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken, anyhow::Error>;
}

/// Adapter for device pairing and registration.
///
/// This trait provides methods for pairing new devices with the channel,
/// typically used for QR code-based setup flows.
#[async_trait]
pub trait PairingAdapter: Send + Sync {
    /// Initiate a device pairing flow.
    ///
    /// # Returns
    /// * `Ok(PairingCode)` - The pairing code and related info.
    /// * `Err(anyhow::Error)` - An error if pairing initialization fails.
    async fn initiate_pairing(&self) -> Result<PairingCode, anyhow::Error>;

    /// Complete the pairing process with the given code.
    ///
    /// # Arguments
    /// * `code` - The pairing code entered by the user.
    ///
    /// # Returns
    /// * `Ok(AccountConfig)` - The configured account if pairing succeeds.
    /// * `Err(anyhow::Error)` - An error if pairing fails.
    async fn complete_pairing(&self, code: &str) -> Result<AccountConfig, anyhow::Error>;
}
