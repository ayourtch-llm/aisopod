//! LINE channel implementation.
//!
//! This module implements the `ChannelPlugin` trait for LINE, enabling
//! the bot to receive and send messages via the LINE Messaging API.
//!
//! # Features
//!
//! - Push and reply message sending
//! - Rich Flex Message support
//! - Webhook-based event receiving
//! - Message filtering and access control
//! - Multi-account support

use crate::api;
use crate::auth;
use crate::config;
use crate::flex;
use crate::webhook;

// Re-export common types
pub use api::{
    BoxComponentBuilder, FlexBlockStyle, FlexBuilder, FlexComponent, FlexContainer,
    FlexContainerType, FlexStyles, LineApi, LineMessage, TextComponentBuilder,
};
pub use auth::{issue_stateless_token, validate_token, TokenManager};
pub use config::LineAccountConfig;
pub use webhook::{
    extract_first_event, extract_reply_token, parse_webhook_body, verify_signature, EventSource,
    MessageContent, MessageEvent, WebhookEventType, WebhookRequestBody,
};

use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, ChannelConfigAdapter, SecurityAdapter,
};
use aisopod_channel::message::{
    IncomingMessage, Media, MessageContent as ChannelMessageContent, MessagePart, MessageTarget,
    PeerInfo, PeerKind, SenderInfo,
};
use aisopod_channel::plugin::ChannelPlugin;
use aisopod_channel::types::{
    ChannelCapabilities, ChannelMeta, ChatType, MediaType, MediaType as ChannelMediaType,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// LINE channel plugin implementation.
///
/// This struct manages LINE connections using the LINE Messaging API.
/// It implements the `ChannelPlugin` trait to integrate with the aisopod system.
pub struct LineChannel {
    /// Vector of LINE accounts
    accounts: Vec<LineAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// The configuration adapter
    config_adapter: LineChannelConfigAdapter,
    /// The security adapter
    security_adapter: Option<LineSecurityAdapter>,
}

impl LineChannel {
    /// Creates a new LINE channel with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The LINE account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(LineChannel)` - The channel
    /// * `Err(anyhow::Error)` - An error if configuration is invalid
    pub fn new(config: LineAccountConfig, account_id: &str) -> Self {
        let account = LineAccount::new(account_id.to_string(), config);

        let id = format!("line-{}", account_id);
        let meta = ChannelMeta {
            label: "LINE".to_string(),
            docs_url: Some("https://line.me".to_string()),
            ui_hints: serde_json::json!({
                "channel_access_token_field": "channel_access_token",
                "channel_secret_field": "channel_secret"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group],
            supports_media: true,
            supports_reactions: true,
            supports_threads: true,
            supports_typing: true,
            supports_voice: true,
            max_message_length: Some(5000),
            supported_media_types: vec![
                ChannelMediaType::Image,
                ChannelMediaType::Video,
                ChannelMediaType::Audio,
                ChannelMediaType::Document,
            ],
        };

        let accounts = vec![account];

        let config_adapter = LineChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(LineSecurityAdapter::new(accounts.clone()));

        Self {
            accounts,
            id,
            meta,
            capabilities,
            shutdown_signal: None,
            config_adapter,
            security_adapter,
        }
    }

    /// Get an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&LineAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut LineAccount> {
        self.accounts.iter_mut().find(|a| a.id == account_id)
    }

    /// Get all active account IDs.
    pub fn get_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.id.clone()).collect()
    }

    /// Add a new account to the channel.
    pub fn add_account(&mut self, account: LineAccount) {
        self.accounts.push(account);
    }

    /// Check if a message should be processed based on security settings.
    pub fn should_process_message(&self, message: &IncomingMessage) -> bool {
        // Check if the sender is in the allowed list for any account
        if let Some(account) = self.get_account(&message.account_id) {
            if !account.config.is_sender_allowed(&message.sender.id) {
                warn!(
                    "Message from {} filtered (not in allowed list)",
                    message.sender.id
                );
                return false;
            }
        }

        true
    }
}

/// A LINE account wraps the configuration with its state.
#[derive(Clone)]
pub struct LineAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: LineAccountConfig,
    /// Whether this account is currently connected
    pub connected: bool,
    /// The timestamp of the last connection
    pub last_connected: Option<DateTime<Utc>>,
}

impl LineAccount {
    /// Create a new LineAccount with the given configuration.
    pub fn new(id: String, config: LineAccountConfig) -> Self {
        Self {
            id,
            config,
            connected: false,
            last_connected: None,
        }
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.config.channel_access_token.is_empty() && !self.config.channel_secret.is_empty()
    }

    /// Create an API client for this account.
    pub fn api(&self) -> LineApi {
        LineApi::new(self.config.channel_access_token.clone())
    }
}

/// ChannelConfigAdapter implementation for LineChannel.
#[derive(Clone)]
pub struct LineChannelConfigAdapter {
    /// Reference to the channel accounts
    accounts: Vec<LineAccount>,
}

impl LineChannelConfigAdapter {
    /// Create a new LineChannelConfigAdapter.
    pub fn new(accounts: Vec<LineAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for LineChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| AccountSnapshot {
                id: a.id.clone(),
                channel: "line".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, _id: &str) -> Result<()> {
        info!("Enabling LINE account");
        // In production, this would enable the account
        Ok(())
    }

    fn disable_account(&self, _id: &str) -> Result<()> {
        info!("Disabling LINE account");
        // In production, this would disable the account
        Ok(())
    }

    fn delete_account(&self, _id: &str) -> Result<()> {
        info!("Deleting LINE account");
        // In production, this would delete the account
        Ok(())
    }
}

/// Security adapter for LineChannel.
#[derive(Clone)]
pub struct LineSecurityAdapter {
    /// Reference to the channel accounts
    accounts: Vec<LineAccount>,
}

impl LineSecurityAdapter {
    /// Create a new LineSecurityAdapter.
    pub fn new(accounts: Vec<LineAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl SecurityAdapter for LineSecurityAdapter {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        // Check if the sender is in the allowed list for any account
        for account in &self.accounts {
            if account.config.is_sender_allowed(&sender.id) {
                return true;
            }
        }

        // If no allowed list is configured, allow all senders
        self.accounts.is_empty()
            || self
                .accounts
                .iter()
                .any(|a| a.config.allowed_users.is_none())
    }

    fn requires_mention_in_group(&self) -> bool {
        // LINE doesn't require mentions in groups by default
        false
    }
}

// ============================================================================
// ChannelPlugin implementation
// ============================================================================

#[async_trait]
impl ChannelPlugin for LineChannel {
    /// Returns the unique identifier for this channel plugin.
    fn id(&self) -> &str {
        &self.id
    }

    /// Returns metadata about this channel implementation.
    fn meta(&self) -> &ChannelMeta {
        &self.meta
    }

    /// Returns the capabilities supported by this channel.
    fn capabilities(&self) -> &ChannelCapabilities {
        &self.capabilities
    }

    /// Returns the configuration adapter for this channel.
    fn config(&self) -> &dyn ChannelConfigAdapter {
        &self.config_adapter
    }

    /// Returns the security adapter for this channel.
    fn security(&self) -> Option<&dyn SecurityAdapter> {
        self.security_adapter
            .as_ref()
            .map(|a| a as &dyn SecurityAdapter)
    }

    /// Connect to the LINE service.
    ///
    /// For LINE, this sets up the connection but doesn't actually start
    /// receiving messages. Message receiving is handled by the gateway.
    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to LINE service");

        for account in &mut self.accounts {
            account.connected = true;
            account.last_connected = Some(Utc::now());
            debug!("Connected account: {}", account.id);
        }

        Ok(())
    }

    /// Send a message through the LINE channel.
    async fn send(&self, msg: aisopod_channel::OutgoingMessage) -> Result<()> {
        let account = self
            .get_account(&msg.target.account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", msg.target.account_id))?;

        if !account.is_enabled() {
            return Err(anyhow::anyhow!("Account {} is not enabled", account.id));
        }

        let api = account.api();

        // Convert the message target to LINE destination
        let destination = match &msg.target.peer.kind {
            PeerKind::User | PeerKind::Channel => &msg.target.peer.id,
            PeerKind::Group | PeerKind::Thread => &msg.target.peer.id,
        };

        // Convert message content to LINE messages
        let line_messages = convert_to_line_messages(&msg.content)?;

        // Send the messages
        for line_message in line_messages {
            api.push_message(destination, vec![line_message]).await?;
        }

        Ok(())
    }

    /// Receive a message from the LINE channel.
    ///
    /// This method is not implemented for LINE. Message receiving should be
    /// handled by the gateway adapter that polls or listens to webhooks.
    async fn receive(&mut self) -> Result<aisopod_channel::IncomingMessage> {
        Err(anyhow::anyhow!(
            "Receive is not implemented for LINE. Use gateway adapter instead."
        ))
    }

    /// Disconnect from the LINE service.
    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from LINE service");

        for account in &mut self.accounts {
            account.connected = false;
        }

        Ok(())
    }
}

// ============================================================================
// Conversion utilities
// ============================================================================

/// Convert aisopod message content to LINE messages.
fn convert_to_line_messages(content: &ChannelMessageContent) -> Result<Vec<api::LineMessage>> {
    let messages = match content {
        ChannelMessageContent::Text(text) => {
            vec![api::LineMessage::Text { text: text.clone() }]
        }
        ChannelMessageContent::Media(media) => convert_media_to_line_messages(media)?,
        ChannelMessageContent::Mixed(parts) => {
            let mut messages = Vec::new();
            for part in parts {
                match part {
                    MessagePart::Text(text) => {
                        messages.push(api::LineMessage::Text { text: text.clone() });
                    }
                    MessagePart::Media(media) => {
                        messages.extend(convert_media_to_line_messages(media)?);
                    }
                }
            }
            messages
        }
    };

    Ok(messages)
}

/// Convert aisopod media to LINE messages.
fn convert_media_to_line_messages(media: &Media) -> Result<Vec<api::LineMessage>> {
    let messages = match media.media_type {
        MediaType::Image => {
            if let Some(url) = &media.url {
                vec![api::LineMessage::Image {
                    original_content_url: url.clone(),
                    preview_image_url: url.clone(),
                }]
            } else {
                return Err(anyhow::anyhow!("Image media requires a URL"));
            }
        }
        MediaType::Video => {
            if let Some(url) = &media.url {
                vec![api::LineMessage::Video {
                    original_content_url: url.clone(),
                    preview_image_url: url.clone(),
                }]
            } else {
                return Err(anyhow::anyhow!("Video media requires a URL"));
            }
        }
        MediaType::Audio => {
            if let Some(url) = &media.url {
                // LINE audio requires duration, use a default of 60 seconds if not provided
                let duration = media.size_bytes.map(|s| s / 1000).unwrap_or(60);
                vec![api::LineMessage::Audio {
                    original_content_url: url.clone(),
                    duration,
                }]
            } else {
                return Err(anyhow::anyhow!("Audio media requires a URL"));
            }
        }
        MediaType::Document => {
            // For documents, we'll send as a text message with the filename
            if let Some(filename) = &media.filename {
                vec![api::LineMessage::Text {
                    text: format!("Document: {}", filename),
                }]
            } else {
                return Err(anyhow::anyhow!("Document media requires a filename"));
            }
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported media type for LINE"));
        }
    };

    Ok(messages)
}

// ============================================================================
// Webhook event conversion
// ============================================================================

/// Convert a LINE webhook message event to an aisopod IncomingMessage.
pub fn webhook_to_incoming_message(
    event: webhook::MessageEvent,
    account_id: &str,
) -> IncomingMessage {
    let (peer, sender) = match &event.source {
        webhook::EventSource::User { user_id } => (
            PeerInfo {
                id: user_id.clone(),
                kind: PeerKind::User,
                title: None,
            },
            SenderInfo {
                id: user_id.clone(),
                display_name: None, // Will be fetched from API if needed
                username: None,
                is_bot: false,
            },
        ),
        webhook::EventSource::Group { group_id, user_id } => {
            let sender_id = user_id.clone().unwrap_or_else(|| "unknown".to_string());
            (
                PeerInfo {
                    id: group_id.clone(),
                    kind: PeerKind::Group,
                    title: None,
                },
                SenderInfo {
                    id: sender_id,
                    display_name: None,
                    username: None,
                    is_bot: false,
                },
            )
        }
        webhook::EventSource::Room { room_id, user_id } => {
            let sender_id = user_id.clone().unwrap_or_else(|| "unknown".to_string());
            (
                PeerInfo {
                    id: room_id.clone(),
                    kind: PeerKind::Channel,
                    title: None,
                },
                SenderInfo {
                    id: sender_id,
                    display_name: None,
                    username: None,
                    is_bot: false,
                },
            )
        }
    };

    let content = match event.message {
        webhook::MessageContent::Text { id, text } => ChannelMessageContent::Text(text),
        webhook::MessageContent::Image { id } => ChannelMessageContent::Media(Media {
            media_type: MediaType::Image,
            url: None,
            data: None,
            filename: None,
            mime_type: Some("image/jpeg".to_string()),
            size_bytes: None,
        }),
        webhook::MessageContent::Video { id } => ChannelMessageContent::Media(Media {
            media_type: MediaType::Video,
            url: None,
            data: None,
            filename: None,
            mime_type: Some("video/mp4".to_string()),
            size_bytes: None,
        }),
        webhook::MessageContent::Audio { id, duration } => ChannelMessageContent::Media(Media {
            media_type: MediaType::Audio,
            url: None,
            data: None,
            filename: None,
            mime_type: Some("audio/m4a".to_string()),
            size_bytes: None,
        }),
        webhook::MessageContent::File {
            id,
            file_name,
            file_size,
        } => ChannelMessageContent::Media(Media {
            media_type: MediaType::Document,
            url: None,
            data: None,
            filename: Some(file_name),
            mime_type: None,
            size_bytes: Some(file_size),
        }),
        webhook::MessageContent::Location {
            id,
            title,
            address,
            latitude,
            longitude,
        } => {
            // For location messages, we create a text message with the location info
            let text = format!(
                "{}: {} (lat: {}, lng: {})",
                title, address, latitude, longitude
            );
            ChannelMessageContent::Text(text)
        }
        webhook::MessageContent::Sticker {
            id,
            package_id,
            sticker_id,
        } => {
            let text = format!("[Sticker: {}/{}]", package_id, sticker_id);
            ChannelMessageContent::Text(text)
        }
    };

    IncomingMessage {
        id: format!("line-{}-{}", account_id, event.reply_token),
        channel: "line".to_string(),
        account_id: account_id.to_string(),
        sender,
        peer,
        content,
        reply_to: None,
        timestamp: DateTime::from_timestamp_millis(event.timestamp as i64)
            .unwrap_or_else(|| Utc::now()),
        metadata: serde_json::json!({
            "replyToken": event.reply_token,
        }),
    }
}

/// Register a LINE channel with the registry.
///
/// This function creates a LineChannel from the given configuration
/// and adds it to the channel registry.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The LINE account configuration
/// * `account_id` - Unique identifier for this account instance
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: LineAccountConfig,
    account_id: &str,
) -> Result<()> {
    let channel = LineChannel::new(config, account_id);
    registry.register(Arc::new(channel));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_account_config_default() {
        let config = LineAccountConfig::default();
        assert!(config.channel_access_token.is_empty());
        assert!(config.channel_secret.is_empty());
        assert!(config.allowed_users.is_none());
    }

    #[test]
    fn test_line_account_config_new() {
        let config = LineAccountConfig::new("test_token".to_string(), "test_secret".to_string());
        assert_eq!(config.channel_access_token, "test_token");
        assert_eq!(config.channel_secret, "test_secret");
    }

    #[test]
    fn test_line_account_enabled() {
        let config = LineAccountConfig::new("test_token".to_string(), "test_secret".to_string());
        let account = LineAccount::new("test".to_string(), config);
        assert!(account.is_enabled());
    }

    #[test]
    fn test_line_account_disabled() {
        let config = LineAccountConfig::default();
        let account = LineAccount::new("test".to_string(), config);
        assert!(!account.is_enabled());
    }

    #[test]
    fn test_is_sender_allowed() {
        let config = LineAccountConfig {
            channel_access_token: "token".to_string(),
            channel_secret: "secret".to_string(),
            allowed_users: Some(vec!["user1".to_string(), "user2".to_string()]),
            allowed_groups: None,
        };

        assert!(config.is_sender_allowed("user1"));
        assert!(config.is_sender_allowed("user2"));
        assert!(!config.is_sender_allowed("user3"));
    }

    #[test]
    fn test_channel_registration() {
        let config = LineAccountConfig::new("test_token".to_string(), "test_secret".to_string());

        // Note: This test would need to be run with tokio runtime
        // For now, we just verify the types compile
        let _channel = LineChannel::new(config, "test-account");
    }
}
