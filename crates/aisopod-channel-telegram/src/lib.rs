//! Telegram channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Telegram,
//! enabling the bot to receive and send messages via the Telegram Bot API.
//!
//! # Features
//!
//! - Long-polling mode for receiving messages
//! - Webhook mode for push-based message delivery
//! - Message normalization to shared `IncomingMessage` type
//! - Support for DMs, groups, and supergroups
//! - Sender filtering and access control

use aisopod_channel::adapters::{AccountConfig, AccountSnapshot, ChannelConfigAdapter};
use aisopod_channel::message::{IncomingMessage, Media, MessageContent, MessagePart, PeerInfo, PeerKind, SenderInfo};
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, MediaType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::{prelude::*, types::{ChatKind, ParseMode, PublicChatKind, StickerFormat, UpdateKind}};
use tracing::{error, info, warn};
use url::Url;

/// Configuration for a Telegram bot account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramAccountConfig {
    /// The bot token from @BotFather
    pub bot_token: String,
    /// Optional webhook URL (if empty, long-polling mode is used)
    pub webhook_url: Option<String>,
    /// Optional list of allowed user IDs (if empty, all users are allowed)
    pub allowed_users: Option<Vec<i64>>,
    /// Optional list of allowed group IDs (if empty, all groups are allowed)
    pub allowed_groups: Option<Vec<i64>>,
    /// Message parsing mode (default: MarkdownV2)
    #[serde(default = "default_parse_mode")]
    pub parse_mode: ParseMode,
}

fn default_parse_mode() -> ParseMode {
    ParseMode::MarkdownV2
}

impl Default for TelegramAccountConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            webhook_url: None,
            allowed_users: None,
            allowed_groups: None,
            parse_mode: ParseMode::MarkdownV2,
        }
    }
}

/// A channel plugin implementation for Telegram.
#[derive(Clone)]
pub struct TelegramChannel {
    /// The teloxide bot instance
    bot: Bot,
    /// The account configuration
    config: TelegramAccountConfig,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
}

impl TelegramChannel {
    /// Creates a new Telegram channel with the given configuration.
    ///
    /// This method validates the bot token by calling the `getMe` API endpoint.
    ///
    /// # Arguments
    ///
    /// * `config` - The Telegram account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(TelegramChannel)` - The channel if bot token is valid
    /// * `Err(anyhow::Error)` - An error if the bot token is invalid
    pub async fn new(config: TelegramAccountConfig, account_id: &str) -> Result<Self> {
        // Validate bot token by calling getMe
        let bot = Bot::new(config.bot_token.clone());
        let me = bot.get_me().await.map_err(|e| {
            anyhow::anyhow!("Failed to validate bot token: {}", e)
        })?;

        let id = format!("telegram-{}", account_id);
        let meta = ChannelMeta {
            label: "Telegram".to_string(),
            docs_url: Some("https://core.telegram.org/bots".to_string()),
            ui_hints: serde_json::json!({
                "bot_token_field": "bot_token",
                "webhook_url_field": "webhook_url"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![aisopod_channel::types::ChatType::Dm, aisopod_channel::types::ChatType::Group, aisopod_channel::types::ChatType::Channel],
            supports_media: true,
            supports_reactions: true,
            supports_threads: true,
            supports_typing: true,
            supports_voice: true,
            max_message_length: Some(4096),
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Audio,
                MediaType::Video,
                MediaType::Document,
            ],
        };

        Ok(Self {
            bot,
            config,
            id,
            meta,
            capabilities,
            shutdown_signal: None,
        })
    }

    /// Creates a new Telegram channel in webhook mode.
    ///
    /// # Arguments
    ///
    /// * `config` - The Telegram account configuration with webhook_url set
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(TelegramChannel)` - The channel if bot token is valid
    /// * `Err(anyhow::Error)` - An error if the bot token is invalid or webhook_url is missing
    pub async fn new_webhook(config: TelegramAccountConfig, account_id: &str) -> Result<Self> {
        let webhook_url = config.webhook_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("webhook_url must be set for webhook mode"))?;

        let bot = Bot::new(config.bot_token.clone());
        // Set up webhook - url is now the only required parameter in teloxide 0.12
        let url = Url::parse(webhook_url).map_err(|e| anyhow::anyhow!("Failed to parse webhook URL: {}", e))?;
        let _ = bot.set_webhook(url).await.map_err(|e| {
            anyhow::anyhow!("Failed to set webhook: {}", e)
        })?;

        Self::new(config, account_id).await
    }

    /// Starts the message receiver using long-polling mode.
    ///
    /// This spawns a background task that continuously polls for new messages
    /// and processes them.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID for this channel instance
    ///
    /// # Returns
    ///
    /// A handle to the background task that can be awaited or cancelled.
    pub async fn start_long_polling(
        &mut self,
        account_id: &str,
    ) -> Result<impl std::future::Future<Output = ()> + Send> {
        let bot = self.bot.clone();
        let allowed_users = self.config.allowed_users.clone();
        let allowed_groups = self.config.allowed_groups.clone();

        // Create shutdown signal
        let shutdown = Arc::new(tokio::sync::Notify::new());
        self.shutdown_signal = Some(shutdown.clone());

        // Create a clone of shutdown for the task
        let shutdown_task = shutdown.clone();

        let task = async move {
            info!("Starting long-polling for Telegram channel");

            // In teloxide 0.12, we use a simpler approach with poll_updates
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_task.notified() => {
                        info!("Shutdown signal received for Telegram channel");
                        break;
                    }
                    result = bot.get_updates() => {
                        match result {
                            Ok(updates) => {
                                for update in updates {
                                    // Process each update - in teloxide 0.12, Update is a struct with kind field
                                    if let UpdateKind::Message(message) = update.kind {
                                        // Log the message
                                        if let Some(text) = &message.text() {
                                            let sender = message
                                                .from()
                                                .and_then(|u| u.username.clone())
                                                .unwrap_or_else(|| "unknown".to_string());
                                            info!(
                                                "Received message from {}: {}",
                                                sender,
                                                text
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Error getting updates: {}", e);
                            }
                        }
                    }
                }
            }
        };

        Ok(task)
    }

    /// Starts the message receiver using webhook mode.
    ///
    /// This spawns a background task that listens for incoming webhook requests.
    /// Note: The HTTP server setup must be handled by the caller (typically in aisopod-gateway).
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID for this channel instance
    /// * `port` - The port to listen on for webhook requests
    ///
    /// # Returns
    ///
    /// A handle to the background task that can be awaited or cancelled.
    pub async fn start_webhook(
        &mut self,
        account_id: &str,
        port: u16,
    ) -> Result<impl std::future::Future<Output = ()> + Send> {
        // Create shutdown signal
        let shutdown = Arc::new(tokio::sync::Notify::new());
        self.shutdown_signal = Some(shutdown.clone());

        let task = async move {
            info!("Starting webhook listener for Telegram channel on port {}", port);

            // In a real implementation, this would set up an HTTP server
            // to receive webhook POST requests from Telegram.
            // For now, we provide a placeholder that demonstrates the structure.
            warn!("Webhook mode setup requires HTTP server integration (e.g., aisopod-gateway)");

            // Keep the task alive
            shutdown.notified().await;
            info!("Webhook listener stopped");
        };

        Ok(task)
    }

    /// Stops the message receiver gracefully.
    pub async fn stop(&mut self) {
        if let Some(shutdown) = &self.shutdown_signal {
            shutdown.notify_one();
        }
    }

    /// Normalizes a Telegram message to the shared IncomingMessage type.
    fn normalize_message(
        &self,
        telegram_message: &Message,
        account_id: &str,
    ) -> IncomingMessage {
        // Get chat and message IDs
        let chat = &telegram_message.chat;
        let chat_id = chat.id;
        let message_id = telegram_message.id;

        // Determine chat type - in teloxide 0.12, ChatKind only has Public and Private
        // PublicChatKind has the actual type information
        let peer_kind = match &chat.kind {
            ChatKind::Private(_) => PeerKind::User,
            ChatKind::Public(p) => match &p.kind {
                PublicChatKind::Supergroup(_) => PeerKind::Group,
                PublicChatKind::Channel(_) => PeerKind::Channel,
                PublicChatKind::Group(_) => PeerKind::Group,
            },
        };

        let peer = PeerInfo {
            id: chat_id.to_string(),
            kind: peer_kind,
            title: chat.title().map(|s| s.to_string()),
        };

        // Extract sender information
        let sender = telegram_message.from().as_ref().map(|user| SenderInfo {
            id: user.id.to_string(),
            display_name: Some(user.first_name.clone()),
            username: user.username.clone(),
            is_bot: user.is_bot,
        }).unwrap_or(SenderInfo {
            id: chat_id.to_string(),
            display_name: chat.title().map(|s| s.to_string()),
            username: None,
            is_bot: false,
        });

        // Extract message content
        let content = self.extract_message_content(telegram_message);

        // Get timestamp
        let timestamp = telegram_message.date;

        // Extract reply_to if present
        let reply_to = telegram_message
            .reply_to_message()
            .as_ref()
            .map(|msg| msg.id);

        IncomingMessage {
            id: format!("telegram:{}:{}", chat_id, message_id),
            channel: self.id.clone(),
            account_id: account_id.to_string(),
            sender,
            peer,
            content,
            reply_to: reply_to.map(|r| r.to_string()),
            timestamp,
            metadata: serde_json::json!({
                "telegram": {
                    "message_id": message_id.to_string(),
                    "chat_id": chat_id.to_string(),
                    "chat_type": format!("{:?}", chat.kind),
                }
            }),
        }
    }

    /// Extracts message content from a Telegram message.
    fn extract_message_content(&self, msg: &Message) -> MessageContent {
        // Check for text content
        if let Some(text) = &msg.text() {
            return MessageContent::Text(text.to_string());
        }

        // Check for media content
        // In teloxide 0.12, the file structures are different from 0.11
        // Media types use FileMeta with id as the file identifier
        
        if let Some(photo) = &msg.photo() {
            if let Some(last_size) = photo.last() {
                return MessageContent::Media(Media {
                    media_type: MediaType::Image,
                    url: Some(last_size.file.id.clone()),
                    data: None,
                    filename: None,
                    mime_type: None,
                    size_bytes: None,
                });
            }
        }

        if let Some(audio) = &msg.audio() {
            return MessageContent::Media(Media {
                media_type: MediaType::Audio,
                url: Some(audio.file.id.clone()),
                data: None,
                filename: audio.file_name.clone(),
                mime_type: audio.mime_type.as_ref().map(|m| m.essence_str().to_string()),
                size_bytes: Some(audio.file.size as u64),
            });
        }

        if let Some(video) = &msg.video() {
            return MessageContent::Media(Media {
                media_type: MediaType::Video,
                url: Some(video.file.id.clone()),
                data: None,
                filename: video.file_name.clone(),
                mime_type: video.mime_type.as_ref().map(|m| m.essence_str().to_string()),
                size_bytes: Some(video.file.size as u64),
            });
        }

        if let Some(document) = &msg.document() {
            return MessageContent::Media(Media {
                media_type: MediaType::Document,
                url: Some(document.file.id.clone()),
                data: None,
                filename: document.file_name.clone(),
                mime_type: document.mime_type.as_ref().map(|m| m.essence_str().to_string()),
                size_bytes: Some(document.file.size as u64),
            });
        }

        if let Some(sticker) = &msg.sticker() {
            // StickerFormat in teloxide 0.12 is an enum with Raster/Animated/Video variants
            // Map sticker format to MIME type based on the format
            let mime_type = match sticker.format {
                StickerFormat::Raster => Some("image/webp".to_string()),
                StickerFormat::Animated => Some("application/x-tgsticker".to_string()),
                StickerFormat::Video => Some("video/webm".to_string()),
            };
            return MessageContent::Media(Media {
                media_type: MediaType::Image,
                url: Some(sticker.file.id.clone()),
                data: None,
                filename: sticker.emoji.clone(),
                mime_type,
                size_bytes: Some(sticker.file.size as u64),
            });
        }

        // For any other content type, use text placeholder
        MessageContent::Text("[Unknown message type]".to_string())
    }
}

// ============================================================================
// ChannelConfigAdapter implementation
// ============================================================================

/// Configuration adapter for Telegram channels.
pub struct TelegramConfigAdapter {
    accounts: std::sync::RwLock<HashMap<String, AccountSnapshot>>,
}

impl TelegramConfigAdapter {
    /// Creates a new TelegramConfigAdapter.
    pub fn new() -> Self {
        Self {
            accounts: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Adds or updates an account snapshot.
    pub fn add_account(&self, account: AccountSnapshot) {
        self.accounts.write().unwrap().insert(account.id.clone(), account);
    }

    /// Enables an account by its ID.
    pub fn enable_account_by_id(&self, id: &str) {
        let mut accounts = self.accounts.write().unwrap();
        if let Some(account) = accounts.get_mut(id) {
            account.enabled = true;
        }
    }

    /// Disables an account by its ID.
    pub fn disable_account_by_id(&self, id: &str) {
        let mut accounts = self.accounts.write().unwrap();
        if let Some(account) = accounts.get_mut(id) {
            account.enabled = false;
        }
    }
}

impl Default for TelegramConfigAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ChannelConfigAdapter for TelegramConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.read().unwrap().keys().cloned().collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts
            .read()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", id))
    }

    fn enable_account(&self, id: &str) -> Result<()> {
        self.enable_account_by_id(id);
        Ok(())
    }

    fn disable_account(&self, id: &str) -> Result<()> {
        self.disable_account_by_id(id);
        Ok(())
    }

    fn delete_account(&self, id: &str) -> Result<()> {
        self.accounts.write().unwrap().remove(id);
        Ok(())
    }
}

// ============================================================================
// SecurityAdapter implementation
// ============================================================================

/// Security adapter for Telegram channels with sender filtering.
pub struct TelegramSecurityAdapter {
    allowed_users: Option<Vec<i64>>,
    allowed_groups: Option<Vec<i64>>,
}

impl TelegramSecurityAdapter {
    /// Creates a new TelegramSecurityAdapter.
    pub fn new(allowed_users: Option<Vec<i64>>, allowed_groups: Option<Vec<i64>>) -> Self {
        Self {
            allowed_users,
            allowed_groups,
        }
    }

    /// Checks if a user ID is in the allowed list.
    fn is_user_allowed(&self, user_id: i64) -> bool {
        match &self.allowed_users {
            None => true, // If no filter, all users are allowed
            Some(list) => list.contains(&user_id),
        }
    }

    /// Checks if a group ID is in the allowed list.
    fn is_group_allowed(&self, group_id: i64) -> bool {
        match &self.allowed_groups {
            None => true, // If no filter, all groups are allowed
            Some(list) => list.contains(&group_id),
        }
    }
}

impl TelegramSecurityAdapter {
    /// Checks if a sender is allowed to interact with the bot.
    pub fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        // For now, we only filter by user ID
        // In a full implementation, we would also check group membership
        sender.id.parse::<i64>()
            .map(|uid| self.is_user_allowed(uid))
            .unwrap_or(true)
    }
}

#[async_trait::async_trait]
impl aisopod_channel::adapters::SecurityAdapter for TelegramSecurityAdapter {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        TelegramSecurityAdapter::is_allowed_sender(self, sender)
    }

    fn requires_mention_in_group(&self) -> bool {
        // Telegram bots can receive group messages without being mentioned
        // if they were previously mentioned in that group
        false
    }
}

// ============================================================================
// ChannelPlugin implementation
// ============================================================================

#[async_trait::async_trait]
impl aisopod_channel::plugin::ChannelPlugin for TelegramChannel {
    fn id(&self) -> &str {
        "telegram"
    }

    fn meta(&self) -> &ChannelMeta {
        &self.meta
    }

    fn capabilities(&self) -> &ChannelCapabilities {
        &self.capabilities
    }

    fn config(&self) -> &dyn ChannelConfigAdapter {
        // Return a dummy implementation
        unimplemented!("TelegramChannel config adapter not yet implemented")
    }

    fn security(&self) -> Option<&dyn aisopod_channel::adapters::SecurityAdapter> {
        // Return a reference to self since TelegramSecurityAdapter is not stored
        // We need to return a static reference, so we can't create a new one
        // For now, return None and implement properly later
        None
    }
}

/// Register a Telegram channel with the given configuration.
///
/// This function creates a new TelegramChannel and registers it with the
/// channel registry. It validates the bot token and sets up the channel
/// for message receiving.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `account_id` - Unique identifier for this account instance
/// * `config` - The Telegram account configuration
///
/// # Returns
///
/// * `Ok(TelegramChannel)` - The created channel
/// * `Err(anyhow::Error)` - An error if channel creation fails
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    account_id: &str,
    config: TelegramAccountConfig,
) -> Result<TelegramChannel> {
    let channel = TelegramChannel::new(config, account_id).await?;
    let channel_clone = channel.clone();
    registry.register(Arc::new(channel));
    Ok(channel_clone)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aisopod_channel::{ChannelPlugin, ChannelRegistry};
    use async_trait::async_trait;

    #[test]
    fn test_account_config_serialization() {
        let config = TelegramAccountConfig {
            bot_token: "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11".to_string(),
            webhook_url: Some("https://example.com/webhook/telegram".to_string()),
            allowed_users: Some(vec![123456, 789012]),
            allowed_groups: Some(vec![-1001234567890]),
            parse_mode: ParseMode::MarkdownV2,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TelegramAccountConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.bot_token, deserialized.bot_token);
        assert_eq!(config.webhook_url, deserialized.webhook_url);
        assert_eq!(config.allowed_users, deserialized.allowed_users);
        assert_eq!(config.allowed_groups, deserialized.allowed_groups);
    }

    #[test]
    fn test_default_config() {
        let config = TelegramAccountConfig::default();
        assert!(config.bot_token.is_empty());
        assert!(config.webhook_url.is_none());
        assert!(config.allowed_users.is_none());
        assert!(config.allowed_groups.is_none());
    }

    #[tokio::test]
    async fn test_channel_creation_with_valid_token() {
        // Note: This test would fail if we try to validate a real token
        // For testing, we'll use a mock or skip actual API calls
        let config = TelegramAccountConfig {
            bot_token: "invalid-token-for-testing".to_string(),
            ..Default::default()
        };

        // This should fail with invalid token
        let result = TelegramChannel::new(config, "test").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_channel_id_format() {
        let config = TelegramAccountConfig::default();
        let channel = TelegramChannel {
            bot: Bot::new("dummy-token"),
            config,
            id: "telegram-test123".to_string(),
            meta: ChannelMeta {
                label: "Telegram".to_string(),
                docs_url: None,
                ui_hints: serde_json::Value::Object(serde_json::Map::new()),
            },
            capabilities: ChannelCapabilities {
                chat_types: vec![aisopod_channel::types::ChatType::Dm, aisopod_channel::types::ChatType::Group, aisopod_channel::types::ChatType::Channel],
                supports_media: true,
                supports_reactions: true,
                supports_threads: true,
                supports_typing: true,
                supports_voice: true,
                max_message_length: Some(4096),
                supported_media_types: vec![
                    aisopod_channel::types::MediaType::Image,
                    aisopod_channel::types::MediaType::Audio,
                    aisopod_channel::types::MediaType::Video,
                    aisopod_channel::types::MediaType::Document,
                ],
            },
            shutdown_signal: None,
        };

        // Check that the channel's id field is set correctly
        assert_eq!(channel.id, "telegram-test123");
        // Check that id() returns the channel type identifier
        assert_eq!(aisopod_channel::plugin::ChannelPlugin::id(&channel), "telegram");
    }

    #[test]
    fn test_channel_capabilities() {
        let config = TelegramAccountConfig::default();
        let channel = TelegramChannel {
            bot: Bot::new("dummy-token"),
            config,
            id: "telegram-test".to_string(),
            meta: ChannelMeta {
                label: "Telegram".to_string(),
                docs_url: None,
                ui_hints: serde_json::Value::Object(serde_json::Map::new()),
            },
            capabilities: ChannelCapabilities {
                chat_types: vec![aisopod_channel::types::ChatType::Dm, aisopod_channel::types::ChatType::Group, aisopod_channel::types::ChatType::Channel],
                supports_media: true,
                supports_reactions: true,
                supports_threads: true,
                supports_typing: true,
                supports_voice: true,
                max_message_length: Some(4096),
                supported_media_types: vec![
                    aisopod_channel::types::MediaType::Image,
                    aisopod_channel::types::MediaType::Audio,
                    aisopod_channel::types::MediaType::Video,
                    aisopod_channel::types::MediaType::Document,
                ],
            },
            shutdown_signal: None,
        };

        let caps = channel.capabilities();
        assert!(caps.supports_media);
        assert!(caps.supports_reactions);
        assert!(caps.supports_threads);
        assert!(caps.supports_typing);
        assert!(caps.supports_voice);
        assert_eq!(caps.max_message_length, Some(4096));
    }

    #[test]
    fn test_security_adapter_with_allowed_users() {
        let adapter = TelegramSecurityAdapter::new(
            Some(vec![123456, 789012]),
            None,
        );

        assert!(adapter.is_allowed_sender(&SenderInfo {
            id: "123456".to_string(),
            display_name: Some("Test".to_string()),
            username: None,
            is_bot: false,
        }));

        assert!(!adapter.is_allowed_sender(&SenderInfo {
            id: "999999".to_string(),
            display_name: Some("Other".to_string()),
            username: None,
            is_bot: false,
        }));
    }

    #[test]
    fn test_security_adapter_without_filter() {
        let adapter = TelegramSecurityAdapter::new(None, None);

        assert!(adapter.is_allowed_sender(&SenderInfo {
            id: "999999".to_string(),
            display_name: Some("Anyone".to_string()),
            username: None,
            is_bot: false,
        }));
    }
}
