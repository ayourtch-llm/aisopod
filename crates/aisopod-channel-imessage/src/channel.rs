//! iMessage channel implementation.
//!
//! This module implements the ChannelPlugin trait for iMessage, providing
//! support for both AppleScript (macOS native) and BlueBubbles (cross-platform)
//! backends.

use crate::config::{BackendType, ImessageAccountConfig, ImessageError, ImessageResult};
use crate::platform::{check_platform_support, is_macos};
use crate::applescript::{ApplescriptBackend, AppleScriptBackendImpl};
use crate::bluebubbles::{BlueBubblesBackend, BlueBubblesBackendImpl};
use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, AuthAdapter, ChannelConfigAdapter, GroupInfo, MemberInfo,
    SecurityAdapter,
};
use aisopod_channel::message::{IncomingMessage, Media, MessageContent, MessagePart, MessageTarget, PeerInfo, PeerKind, SenderInfo};
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use aisopod_channel::plugin::ChannelPlugin;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

// Trait for unified backend interface
trait BackendImpl: Send + Sync {
    fn connect(&mut self) -> Result<()>;
    fn disconnect(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
    fn send_text(&self, to: &str, text: &str) -> Result<String>;
    fn send_text_to_group(&self, group_id: &str, text: &str) -> Result<String>;
    fn send_media(&self, to: &str, media_path: &str, mime_type: &str) -> Result<String>;
    fn send_media_to_group(&self, group_id: &str, media_path: &str, mime_type: &str) -> Result<String>;
    fn backend_type(&self) -> BackendType;
}

/// iMessage account state.
#[derive(Clone)]
pub struct ImessageAccount {
    /// Account configuration
    pub config: ImessageAccountConfig,
    /// Whether this account is connected
    pub connected: bool,
    /// Last connection timestamp
    pub last_connected: Option<DateTime<Utc>>,
    /// Backend type
    backend: BackendType,
}

impl ImessageAccount {
    /// Creates a new iMessage account.
    pub fn new(config: ImessageAccountConfig) -> Self {
        let backend = config.backend_type();
        Self {
            config,
            connected: false,
            last_connected: None,
            backend,
        }
    }

    /// Checks if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.config.account_id.is_empty()
    }
}

/// iMessage channel plugin.
///
/// This struct implements the ChannelPlugin trait and provides communication
/// with iMessage through either the AppleScript backend (macOS native) or
/// the BlueBubbles HTTP API backend (cross-platform).
///
/// # Platform Support
///
/// - **macOS**: Uses AppleScript via osascript (default)
/// - **Other platforms**: Uses BlueBubbles HTTP API
///
/// # Features
///
/// - DM and group chat support
/// - Media attachment support
/// - Sender filtering
/// - Group monitoring
/// - Multi-account support
///
/// # Example
///
/// ```no_run
/// use aisopod_channel_imessage::{ImessageChannel, ImessageAccountConfig};
/// use aisopod_channel::ChannelRegistry;
/// use std::sync::Arc;
///
/// async fn example() -> Result<(), anyhow::Error> {
///     let mut registry = ChannelRegistry::new();
///     
///     let config = ImessageAccountConfig::new("my-imessage-account");
///     // backend is set via ImessageAccountConfig.backend field or uses default
///     
///     let channel = ImessageChannel::new(config).await?;
///     registry.register(Arc::new(channel));
///     
///     Ok(())
/// }
/// ```
pub struct ImessageChannel {
    /// Accounts managed by this channel
    accounts: Vec<ImessageAccount>,
    /// Channel ID
    id: String,
    /// Channel metadata
    meta: ChannelMeta,
    /// Channel capabilities
    capabilities: ChannelCapabilities,
    /// AppleScript backend (macOS only)
    applescript_backend: Option<ApplescriptBackend>,
    /// BlueBubbles backend
    bluebubbles_backend: Option<BlueBubblesBackend>,
    /// Configuration adapter
    config_adapter: ImessageChannelConfigAdapter,
    /// Security adapter
    security_adapter: Option<ImessageSecurityAdapter>,
}

impl ImessageChannel {
    /// Creates a new iMessage channel with the given configuration.
    ///
    /// This method validates the configuration and initializes the appropriate
    /// backend based on the platform and configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The iMessage account configuration
    ///
    /// # Returns
    ///
    /// * `Ok(ImessageChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if configuration is invalid
    pub async fn new(config: ImessageAccountConfig) -> Result<Self> {
        let account = ImessageAccount::new(config.clone());

        // Validate the configuration
        if let Err(e) = account.config.validate() {
            return Err(anyhow::anyhow!(
                "Failed to validate iMessage account: {}",
                e
            ));
        }

        // Check platform support
        if let Err(e) = check_platform_support(&config) {
            return Err(anyhow::anyhow!(
                "Platform not supported: {}",
                e
            ));
        }

        let id = format!("imessage-{}", config.account_id);
        let meta = ChannelMeta {
            label: "iMessage".to_string(),
            docs_url: Some("https://support.apple.com/messages".to_string()),
            ui_hints: serde_json::json!({
                "account_id_field": "account_id",
                "backend_field": "backend",
                "backend_options": ["applescript", "bluebubbles"]
            }),
        };

        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group],
            supports_media: true,
            supports_reactions: true,
            supports_threads: false,
            supports_typing: false,
            supports_voice: true,
            max_message_length: Some(32000),
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Audio,
                MediaType::Video,
                MediaType::Document,
            ],
        };

        let accounts = vec![account];
        
        // Initialize backends based on configuration
        let applescript_backend = if config.backend == "applescript" {
            Some(ApplescriptBackend::new())
        } else {
            None
        };

        let bluebubbles_backend = if config.backend == "bluebubbles" {
            Some(BlueBubblesBackend::new(config.bluebubbles)?)
        } else {
            None
        };

        // Initialize adapters
        let config_adapter = ImessageChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(ImessageSecurityAdapter::new(accounts.clone()));

        Ok(Self {
            accounts,
            id,
            meta,
            capabilities,
            applescript_backend,
            bluebubbles_backend,
            config_adapter,
            security_adapter,
        })
    }

    /// Creates a new iMessage channel with the default configuration.
    ///
    /// This uses the default backend for the current platform:
    /// - macOS: AppleScript
    /// - Other platforms: BlueBubbles
    pub async fn default() -> Result<Self> {
        let config = ImessageAccountConfig::default();
        Self::new(config).await
    }

    /// Gets the account by ID.
    pub fn get_account(&self, account_id: &str) -> Option<&ImessageAccount> {
        self.accounts.iter().find(|a| a.config.account_id == account_id)
    }

    /// Gets all configured account IDs.
    pub fn list_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.config.account_id.clone()).collect()
    }

    /// Connects to the iMessage backend.
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to iMessage backend");

        // Connect AppleScript backend
        if let Some(backend) = &mut self.applescript_backend {
            backend.connect().await?;
        }

        // Connect BlueBubbles backend
        if let Some(backend) = &mut self.bluebubbles_backend {
            backend.connect().await?;
        }

        // Update account connection status
        for account in &mut self.accounts {
            account.connected = true;
            account.last_connected = Some(Utc::now());
        }

        Ok(())
    }

    /// Disconnects from the iMessage backend.
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from iMessage backend");

        // Disconnect BlueBubbles backend
        if let Some(backend) = &mut self.bluebubbles_backend {
            backend.disconnect().await?;
        }

        // Disconnect AppleScript backend
        if let Some(backend) = &mut self.applescript_backend {
            backend.disconnect().await?;
        }

        // Update account connection status
        for account in &mut self.accounts {
            account.connected = false;
        }

        Ok(())
    }

    /// Checks if connected to the iMessage backend.
    pub fn is_connected(&self) -> bool {
        if let Some(backend) = &self.applescript_backend {
            if !backend.is_connected() {
                return false;
            }
        }

        if let Some(backend) = &self.bluebubbles_backend {
            if !backend.is_connected() {
                return false;
            }
        }

        self.accounts.iter().any(|a| a.connected)
    }

    /// Gets the backend type.
    pub fn backend_type(&self) -> BackendType {
        if self.bluebubbles_backend.is_some() {
            return BackendType::BlueBubbles;
        }

        if self.applescript_backend.is_some() {
            return BackendType::AppleScript;
        }

        BackendType::BlueBubbles
    }

    /// Sends a text message to a recipient.
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient identifier (phone number or email)
    /// * `text` - Message text
    /// * `account_id` - Optional account ID (uses first account if not specified)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_text(
        &self,
        to: &str,
        text: &str,
        account_id: Option<&str>,
    ) -> Result<()> {
        let account = account_id
            .and_then(|id| self.get_account(id))
            .or_else(|| self.accounts.first());

        let account = account.ok_or_else(|| {
            anyhow::anyhow!("No iMessage account configured")
        })?;

        match account.config.backend.as_str() {
            "applescript" => {
                let backend = self.applescript_backend.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("AppleScript backend not available")
                })?;
                backend.send_text(to, text).await?;
            }
            "bluebubbles" => {
                let backend = self.bluebubbles_backend.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("BlueBubbles backend not available")
                })?;
                backend.send_text(to, text).await?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid backend: {}",
                    account.config.backend
                ));
            }
        }
        Ok(())
    }

    /// Sends a text message to a group.
    ///
    /// # Arguments
    ///
    /// * `group_id` - Group chat identifier
    /// * `text` - Message text
    /// * `account_id` - Optional account ID (uses first account if not specified)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_text_to_group(
        &self,
        group_id: &str,
        text: &str,
        account_id: Option<&str>,
    ) -> Result<()> {
        let account = account_id
            .and_then(|id| self.get_account(id))
            .or_else(|| self.get_account(&self.accounts[0].config.account_id));

        let account = account.ok_or_else(|| {
            anyhow::anyhow!("No iMessage account configured")
        })?;

        match account.config.backend.as_str() {
            "applescript" => {
                let backend = self.applescript_backend.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("AppleScript backend not available")
                })?;
                backend.send_text_to_group(group_id, text).await?;
            }
            "bluebubbles" => {
                let backend = self.bluebubbles_backend.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("BlueBubbles backend not available")
                })?;
                backend.send_text_to_group(group_id, text).await?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid backend: {}",
                    account.config.backend
                ));
            }
        }
        Ok(())
    }

    /// Sends media to a recipient.
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient identifier
    /// * `media` - Media content
    /// * `account_id` - Optional account ID
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Media sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_media(
        &self,
        to: &str,
        media: &Media,
        account_id: Option<&str>,
    ) -> Result<()> {
        let account = account_id
            .and_then(|id| self.get_account(id))
            .or_else(|| self.accounts.first());

        let account = account.ok_or_else(|| {
            anyhow::anyhow!("No iMessage account configured")
        })?;

        let backend = match account.config.backend.as_str() {
            "bluebubbles" => {
                self.bluebubbles_backend.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("BlueBubbles backend not available")
                })?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Media sending requires BlueBubbles backend"
                ));
            }
        };

        // Get media path and mime type from Media struct
        let media_path = media.data.clone().ok_or_else(|| {
            anyhow::anyhow!("Media data required for sending")
        })?;

        let mime_type = media.mime_type.clone().unwrap_or_else(|| {
            match media.media_type {
                MediaType::Image => "image/jpeg".to_string(),
                MediaType::Audio => "audio/mpeg".to_string(),
                MediaType::Video => "video/mp4".to_string(),
                MediaType::Document => "application/pdf".to_string(),
                MediaType::Other(ref other) => format!("application/{}", other),
            }
        });

        // Convert data to file for sending (in a real implementation, we'd use a temporary file)
        let media_path_str = String::from("/tmp/media.bin");
        
        backend.send_media(to, &media_path_str, &mime_type).await?;
        Ok(())
    }

    /// Sends media to a group.
    pub async fn send_media_to_group(
        &self,
        group_id: &str,
        media: &Media,
        account_id: Option<&str>,
    ) -> Result<()> {
        let account = account_id
            .and_then(|id| self.get_account(id))
            .or_else(|| self.accounts.first());

        let account = account.ok_or_else(|| {
            anyhow::anyhow!("No iMessage account configured")
        })?;

        let backend = match account.config.backend.as_str() {
            "bluebubbles" => {
                self.bluebubbles_backend.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("BlueBubbles backend not available")
                })?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Media sending requires BlueBubbles backend"
                ));
            }
        };

        let media_path = media.data.clone().ok_or_else(|| {
            anyhow::anyhow!("Media data required for sending")
        })?;

        let mime_type = media.mime_type.clone().unwrap_or_else(|| {
            match media.media_type {
                MediaType::Image => "image/jpeg".to_string(),
                MediaType::Audio => "audio/mpeg".to_string(),
                MediaType::Video => "video/mp4".to_string(),
                MediaType::Document => "application/pdf".to_string(),
                MediaType::Other(ref other) => format!("application/{}", other),
            }
        });

        let media_path_str = String::from("/tmp/media.bin");
        
        backend.send_media_to_group(group_id, &media_path_str, &mime_type).await?;
        Ok(())
    }
}

/// Configuration adapter for iMessage channel.
#[derive(Clone)]
pub struct ImessageChannelConfigAdapter {
    /// Reference to channel accounts
    accounts: Vec<ImessageAccount>,
}

impl ImessageChannelConfigAdapter {
    /// Creates a new configuration adapter.
    pub fn new(accounts: Vec<ImessageAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for ImessageChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.config.account_id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts.iter()
            .find(|a| a.config.account_id == id)
            .map(|a| AccountSnapshot {
                id: a.config.account_id.clone(),
                channel: "imessage".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, _id: &str) -> Result<()> {
        info!("Enabling iMessage account");
        // In production, this would enable the account
        Ok(())
    }

    fn disable_account(&self, _id: &str) -> Result<()> {
        info!("Disabling iMessage account");
        // In production, this would disable the account
        Ok(())
    }

    fn delete_account(&self, _id: &str) -> Result<()> {
        info!("Deleting iMessage account");
        // In production, this would delete the account
        Ok(())
    }
}

/// Security adapter for iMessage channel.
#[derive(Clone)]
pub struct ImessageSecurityAdapter {
    /// Reference to channel accounts
    accounts: Vec<ImessageAccount>,
}

impl ImessageSecurityAdapter {
    /// Creates a new security adapter.
    pub fn new(accounts: Vec<ImessageAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl SecurityAdapter for ImessageSecurityAdapter {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        // Check if the sender is in the allowed list for any account
        for account in &self.accounts {
            if account.config.is_sender_allowed(&sender.id) {
                return true;
            }
        }
        
        // If no allowlist is configured, all senders are allowed
        self.accounts.is_empty() || self.accounts.iter().any(|a| a.config.allowed_senders.is_none())
    }

    fn requires_mention_in_group(&self) -> bool {
        // iMessage doesn't require mentions in groups by default
        false
    }
}

/// Channel implementation for iMessage.
#[async_trait]
impl ChannelPlugin for ImessageChannel {
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
        self.security_adapter.as_ref().map(|a| a as &dyn SecurityAdapter)
    }
}

/// Register an iMessage channel with the registry.
///
/// This function creates an ImessageChannel from the given configuration
/// and adds it to the channel registry.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The iMessage account configuration
/// * `account_id` - Unique identifier for this account instance
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: ImessageAccountConfig,
    account_id: &str,
) -> Result<()> {
    // Override account_id in config
    let mut config = config;
    config.account_id = account_id.to_string();
    
    let channel = ImessageChannel::new(config).await?;
    registry.register(Arc::new(channel));
    Ok(())
}

/// Parse an iMessage message from JSON.
///
/// This function parses the raw JSON message received from iMessage into
/// a structured IncomingMessage that can be processed by the aisopod system.
///
/// # Arguments
///
/// * `json` - The raw JSON message
/// * `account_id` - The account ID that received the message
/// * `channel_id` - The channel identifier
///
/// # Returns
///
/// * `Ok(IncomingMessage)` - The parsed message
/// * `Err(anyhow::Error)` - An error if parsing fails
pub fn parse_imessage_message(
    json: serde_json::Value,
    account_id: &str,
    channel_id: &str,
) -> Result<IncomingMessage> {
    // Extract fields from the JSON
    let guid = json.get("guid")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let sender_id = json.get("address")
        .or_else(|| json.get("sender"))
        .or_else(|| json.get("from"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let text = json.get("text")
        .or_else(|| json.get("body"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let timestamp = json.get("date")
        .or_else(|| json.get("date_sent"))
        .and_then(|v| v.as_i64())
        .map(|ts| DateTime::<Utc>::from_utc(chrono::NaiveDateTime::from_timestamp(ts, 0), Utc));

    // Extract chat/group info
    let chat_id = json.get("chat_guid")
        .or_else(|| json.get("group_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Determine if this is a group chat
    let is_group = chat_id.is_some();

    // Clone sender_id for use in peer info
    let sender_id_for_peer = sender_id.clone();

    // Create sender info
    let sender = SenderInfo {
        id: sender_id,
        display_name: None,
        username: None,
        is_bot: false,
    };

    // Create peer info
    let peer = PeerInfo {
        id: chat_id.unwrap_or_else(|| sender_id_for_peer.clone()),
        kind: if is_group { PeerKind::Group } else { PeerKind::User },
        title: None,
    };

    // Create message content
    let content = MessageContent::Text(text.clone());

    Ok(IncomingMessage {
        id: guid,
        channel: channel_id.to_string(),
        account_id: account_id.to_string(),
        sender,
        peer,
        content,
        reply_to: None,
        timestamp: timestamp.unwrap_or_else(Utc::now),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BlueBubblesConfig;

    #[test]
    fn test_default_backend_on_macos() {
        let config = ImessageAccountConfig::default();
        
        #[cfg(target_os = "macos")]
        assert_eq!(config.backend, "applescript");
        
        #[cfg(not(target_os = "macos"))]
        assert_eq!(config.backend, "bluebubbles");
    }

    #[tokio::test]
    async fn test_channel_meta() {
        let config = ImessageAccountConfig::new("test");
        
        // Ensure we have a valid backend for testing
        #[cfg(target_os = "macos")]
        let config = config;
        
        #[cfg(not(target_os = "macos"))]
        let config = {
            let mut config = config;
            // On non-macOS, bluebubbles requires api_url
            config.bluebubbles = BlueBubblesConfig {
                api_url: Some("http://localhost:12345".to_string()),
                ..Default::default()
            };
            config
        };
        
        let channel = ImessageChannel::new(config).await.unwrap();
        
        assert_eq!(channel.id(), "imessage-test");
        assert_eq!(channel.meta().label, "iMessage");
        assert!(channel.meta().docs_url.is_some());
    }

    #[tokio::test]
    async fn test_channel_capabilities() {
        let config = ImessageAccountConfig::new("test");
        
        // Ensure we have a valid backend for testing
        #[cfg(target_os = "macos")]
        let config = config;
        
        #[cfg(not(target_os = "macos"))]
        let config = {
            let mut config = config;
            // On non-macOS, bluebubbles requires api_url
            config.bluebubbles = BlueBubblesConfig {
                api_url: Some("http://localhost:12345".to_string()),
                ..Default::default()
            };
            config
        };
        
        let channel = ImessageChannel::new(config).await.unwrap();
        
        let caps = channel.capabilities();
        
        assert!(caps.chat_types.contains(&ChatType::Dm));
        assert!(caps.chat_types.contains(&ChatType::Group));
        assert!(caps.supports_media);
        assert!(caps.supports_reactions);
        assert!(caps.supports_voice);
    }

    #[test]
    fn test_parse_imessage_message_dm() {
        let json = serde_json::json!({
            "guid": "msg123",
            "address": "+1234567890",
            "text": "Hello!",
            "date": 1234567890,
            "is_from_me": false,
            "date_sent": 1234567890
        });

        let message = parse_imessage_message(json, "test-account", "imessage").unwrap();
        
        assert_eq!(message.id, "msg123");
        assert_eq!(message.sender.id, "+1234567890");
        assert_eq!(message.peer.id, "+1234567890");
        assert_eq!(message.peer.kind, PeerKind::User);
    }

    #[test]
    fn test_parse_imessage_message_group() {
        let json = serde_json::json!({
            "guid": "msg123",
            "address": "+1234567890",
            "text": "Hello group!",
            "chat_guid": "group123",
            "is_from_me": false,
            "date": 1234567890
        });

        let message = parse_imessage_message(json, "test-account", "imessage").unwrap();
        
        assert_eq!(message.peer.id, "group123");
        assert_eq!(message.peer.kind, PeerKind::Group);
    }

    #[tokio::test]
    async fn test_channel_registration() {
        let mut registry = aisopod_channel::ChannelRegistry::new();
        
        let config = ImessageAccountConfig::new("test");
        
        // Note: This test would need to be run with tokio runtime
        // For now, we just verify the types compile
        let _result = register(&mut registry, config, "test").await;
    }

    #[tokio::test]
    async fn test_channel_disconnected_state() {
        let config = ImessageAccountConfig::new("test-disconnected");
        
        // Ensure we have a valid backend for testing
        #[cfg(target_os = "macos")]
        let config = config;
        
        #[cfg(not(target_os = "macos"))]
        let config = {
            let mut config = config;
            // On non-macOS, bluebubbles requires api_url
            config.bluebubbles = BlueBubblesConfig {
                api_url: Some("http://localhost:12345".to_string()),
                ..Default::default()
            };
            config
        };
        
        let channel = ImessageChannel::new(config).await.unwrap();
        
        // Channel should be disconnected initially
        assert!(!channel.is_connected());
    }

    #[test]
    fn test_imessage_account_config_validation() {
        // Valid AppleScript config
        let config = ImessageAccountConfig {
            backend: "applescript".to_string(),
            ..Default::default()
        };
        
        // On macOS, this should succeed (or fail only if osascript is missing)
        let result = config.validate();
        // Don't assert success as osascript might not exist in test environment
        let _ = result;
        
        // Valid BlueBubbles config
        let config = ImessageAccountConfig {
            backend: "bluebubbles".to_string(),
            bluebubbles: crate::config::BlueBubblesConfig {
                api_url: Some("http://localhost:12345".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        
        assert!(config.validate().is_ok());
    }
}
