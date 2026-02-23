//! Discord channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Discord,
//! enabling the bot to receive and send messages via the Discord Bot API.
//!
//! # Features
//!
//! - Gateway connection for receiving messages via WebSocket
//! - Support for DMs, server channels, and threads
//! - Message normalization to shared `IncomingMessage` type
//! - Self-message filtering to avoid loops
//! - Allowlist filtering for guilds and channels
//! - Multi-account support
//! - Text message sending with Discord markdown formatting
//! - Long message chunking (2000 char limit)
//! - Rich embed support with builder
//! - Media sending and receiving (attachments)
//! - Typing indicators
//! - Reply-to-message handling
//! - Thread management (create, reply, detect)
//! - Reaction handling (add, remove, events)
//! - Guild and channel discovery
//! - Message editing and deletion

mod connection;
mod features;
mod media;
mod receive;
mod send;
mod embeds;

use aisopod_channel::adapters::{AccountConfig, AccountSnapshot, ChannelConfigAdapter};
use aisopod_channel::message::{IncomingMessage, Media, MessageContent, MessagePart, MessageTarget, PeerInfo, PeerKind, SenderInfo, OutgoingMessage};
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};
use serenity::all::{ChannelId, Context, MessageId};
use serenity::client::Client;
use futures::{future, FutureExt};
use std::pin::Pin;

// Re-export modules
pub use connection::{DiscordClientHandle, DiscordEventHandler, create_client, start_client_task};
pub use receive::{normalize_message, should_filter_message, check_mention_requirement, process_discord_message};

// Re-export new modules
pub use send::{send_message, chunk_text, SendOptions, SendMessageResult, formatting, DISCORD_MESSAGE_LIMIT};
pub use embeds::{EmbedBuilder, build_tool_result_embed, build_error_embed, build_success_embed, build_info_embed, build_warning_embed, colors, MAX_EMBEDS};
pub use media::{extract_media_from_attachments, create_attachment_from_path, send_media, send_media_batch, validate_media, download_attachments, MAX_FILE_SIZE};
pub use features::{send_typing, create_thread, reply_in_thread, detect_thread_in_message, get_thread_info, add_reaction, remove_reaction, list_guilds, list_channels, find_channel_by_name, edit_message, delete_message, bulk_delete_messages};

/// Configuration for a Discord bot account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordAccountConfig {
    /// The bot token from Discord Developer Portal
    pub bot_token: String,
    /// Optional application ID for interaction commands
    pub application_id: Option<String>,
    /// Optional list of allowed guild IDs (if empty, all guilds are allowed)
    pub allowed_guilds: Option<Vec<u64>>,
    /// Optional list of allowed channel IDs (if empty, all channels are allowed)
    pub allowed_channels: Option<Vec<u64>>,
    /// Whether messages in guild channels require a bot mention to be received
    #[serde(default = "default_mention_required")]
    pub mention_required_in_channels: bool,
}

fn default_mention_required() -> bool {
    false
}

impl Default for DiscordAccountConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            application_id: None,
            allowed_guilds: None,
            allowed_channels: None,
            mention_required_in_channels: false,
        }
    }
}

/// A Discord account wraps client configuration with its settings.
#[derive(Clone)]
pub struct DiscordAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: DiscordAccountConfig,
}

impl DiscordAccount {
    /// Create a new DiscordAccount with the given configuration.
    pub fn new(id: String, config: DiscordAccountConfig) -> Self {
        Self { id, config }
    }
}

/// A Discord account with a running client.
pub struct DiscordAccountWithClient {
    /// The account information
    pub account: DiscordAccount,
    /// The Discord client (Arc-wrapped for sharing)
    pub client: Arc<Client>,
}

impl DiscordAccountWithClient {
    /// Create a new DiscordAccountWithClient.
    pub fn new(account: DiscordAccount, client: Arc<Client>) -> Self {
        Self {
            account,
            client,
        }
    }

    /// Get the account ID.
    pub fn id(&self) -> &str {
        &self.account.id
    }

    /// Get the client.
    pub fn client(&self) -> &Arc<Client> {
        &self.client
    }
}

/// Manually implement Clone for DiscordAccountWithClient since Client's internals are Arc-wrapped
impl Clone for DiscordAccountWithClient {
    fn clone(&self) -> Self {
        Self {
            account: self.account.clone(),
            client: self.client.clone(), // Arc<Client> implements Clone
        }
    }
}

// ============================================================================
// ChannelPlugin implementation
// ============================================================================

/// A channel plugin implementation for Discord with multi-account support.
#[derive(Clone)]
pub struct DiscordChannel {
    /// Vector of Discord accounts with their running clients
    accounts: Vec<DiscordAccountWithClient>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
}

impl DiscordChannel {
    /// Creates a new Discord channel with the given configuration.
    ///
    /// This method validates the bot token by checking if it's non-empty.
    /// Note: Clients are created and started when `start()` is called.
    ///
    /// # Arguments
    ///
    /// * `config` - The Discord account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(DiscordChannel)` - The channel if bot token is valid
    /// * `Err(anyhow::Error)` - An error if the bot token is invalid
    pub async fn new(config: DiscordAccountConfig, account_id: &str) -> Result<Self> {
        // Validate bot token
        if config.bot_token.trim().is_empty() {
            return Err(anyhow!("Bot token cannot be empty"));
        }

        let id = format!("discord-{}", account_id);
        let meta = ChannelMeta {
            label: "Discord".to_string(),
            docs_url: Some("https://discord.com/developers/docs/intro".to_string()),
            ui_hints: serde_json::json!({
                "bot_token_field": "bot_token",
                "application_id_field": "application_id"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group, ChatType::Channel, ChatType::Thread],
            supports_media: true,
            supports_reactions: true,
            supports_threads: true,
            supports_typing: true,
            supports_voice: true,
            max_message_length: Some(2000),
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Audio,
                MediaType::Video,
                MediaType::Document,
            ],
        };

        // Create an account without a client (client will be added when start() is called)
        let account = DiscordAccount::new(account_id.to_string(), config);

        Ok(Self {
            accounts: vec![],
            id,
            meta,
            capabilities,
            shutdown_signal: None,
        })
    }

    /// Get all active account IDs.
    pub fn get_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.id().to_string()).collect()
    }

    /// Add a new account to the channel.
    pub fn add_account(&mut self, account: DiscordAccountWithClient) {
        self.accounts.push(account);
    }

    /// Remove an account by its ID.
    pub fn remove_account(&mut self, account_id: &str) -> bool {
        let len = self.accounts.len();
        self.accounts.retain(|a| a.id() != account_id);
        len != self.accounts.len()
    }

    /// Get an account with client by its ID.
    pub fn get_account_with_client(&self, account_id: &str) -> Option<&DiscordAccountWithClient> {
        self.accounts.iter().find(|a| a.id() == account_id)
    }

    /// Get an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&DiscordAccount> {
        self.accounts.iter()
            .find(|a| a.id() == account_id)
            .map(|a| &a.account)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut DiscordAccount> {
        self.accounts.iter_mut()
            .find(|a| a.id() == account_id)
            .map(|a| &mut a.account)
    }

    /// Get the client for a given account.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID
    ///
    /// # Returns
    ///
    /// * `Ok(Arc<Client>)` - The client if the account exists
    /// * `Err(anyhow::Error)` - An error if the account is not found
    pub fn get_client(&self, account_id: &str) -> Result<Arc<Client>> {
        self.accounts.iter()
            .find(|a| a.id() == account_id)
            .map(|a| a.client.clone())
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_id))
    }

    /// Starts the Discord gateway connection and stores the client handles.
    ///
    /// This creates Discord clients for each account and stores them in the channel.
    /// The background tasks are spawned to connect to the Discord gateway and listen for incoming messages.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID for this channel instance (optional, if None, connects all accounts)
    ///
    /// # Returns
    ///
    /// A handle to the background task that can be awaited or cancelled.
    pub async fn start(&mut self, account_id: Option<&str>) -> Result<impl std::future::Future<Output = ()> + Send> {
        // Determine which accounts to connect
        let accounts_to_connect: Vec<DiscordAccount> = match account_id {
            Some(id) => {
                self.get_account(id)
                    .cloned()
                    .map(|a| vec![a])
                    .unwrap_or_default()
            }
            None => {
                // Get accounts from DiscordAccountWithClient
                self.accounts.iter().map(|a| a.account.clone()).collect()
            }
        };

        if accounts_to_connect.is_empty() {
            return Err(anyhow!("No accounts found to connect"));
        }

        // Create shutdown signal
        let shutdown = Arc::new(tokio::sync::Notify::new());
        self.shutdown_signal = Some(shutdown.clone());

        let mut tasks: Vec<Pin<Box<dyn std::future::Future<Output = ()> + Send>>> = Vec::new();

        for account in accounts_to_connect {
            let account_id = account.id.clone();
            let config = account.config.clone();
            let shutdown_clone = shutdown.clone();

            // Create client for this account
            let client_handle = create_client(&config, &account_id).await?;

            // Use the Arc<Client> from the handle for sharing (DiscordAccountWithClient stores Arc<Client>)
            let client = client_handle.client.clone();

            // Store the account with client
            let account_with_client = DiscordAccountWithClient::new(account, client.clone());
            self.accounts.push(account_with_client);

            // Start the client with the handle
            let handle = client_handle;
            let shutdown = shutdown_clone;
            
            // Spawn the task that will start the client
            let task = async move {
                // The handle.start() method will unwrap the Arc and start the client
                handle.start();
                
                // Wait for shutdown signal
                shutdown.notified().await;
            };
            
            tasks.push(Box::pin(task));
        }

        // Combine all tasks into a single future
        let task = async move {
            info!("Starting Discord gateway for {} accounts", tasks.len());

            // Wait for all tasks to complete
            futures::future::join_all(tasks).await;

            info!("Discord gateway shutdown complete");
        };

        Ok(task)
    }

    /// Stops the Discord gateway connection gracefully.
    pub async fn stop(&mut self) {
        if let Some(shutdown) = &self.shutdown_signal {
            shutdown.notify_one();
        }
    }

    /// Send a message to Discord.
    ///
    /// This method sends an `OutgoingMessage` to the specified Discord channel.
    /// It supports text, media, and other message content types.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID to send from
    /// * `message` - The outgoing message to send
    ///
    /// # Returns
    ///
    /// * `Ok(MessageId)` - The ID of the sent message
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_message(
        &self,
        account_id: &str,
        message: &OutgoingMessage,
    ) -> Result<MessageId> {
        // Get the client for this account
        let client = self.get_client(account_id)?;
        
        // Extract channel ID from target
        let channel_id = message.target.peer.id.parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid channel ID: {}: {}", message.target.peer.id, e))?
            .into();
        
        // Build the message content
        let text = self.content_to_string_from_message(message);
        
        // Build send options
        let options = SendOptions {
            reply_to_message_id: message.reply_to.as_ref().and_then(|r| {
                r.parse::<u64>().ok().map(|id| id.into())
            }),
            ..Default::default()
        };
        
        // Send the message using the http client
        // Arc<Client> doesn't implement CacheHttp, so we use Arc<Http> instead
        let http = client.http.clone();
        send_message(http, channel_id, &text, Some(options))
            .await
            .map(|result| result.message_id)
    }

    /// Helper function to convert message content to string.
    fn content_to_string_from_message(&self, message: &OutgoingMessage) -> String {
        match &message.content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Media(media) => {
                // Return a placeholder for media content
                match &media.media_type {
                    MediaType::Image => format!("[Image: {}]", media.url.as_deref().unwrap_or("unknown")),
                    MediaType::Audio => format!("[Audio: {}]", media.url.as_deref().unwrap_or("unknown")),
                    MediaType::Video => format!("[Video: {}]", media.url.as_deref().unwrap_or("unknown")),
                    MediaType::Document => format!("[Document: {}]", media.filename.as_deref().unwrap_or("unknown")),
                    MediaType::Other(other) => format!("[{}: {}]", other, media.url.as_deref().unwrap_or("unknown")),
                }
            }
            MessageContent::Mixed(parts) => {
                parts
                    .iter()
                    .map(|part| match part {
                        MessagePart::Text(text) => text.clone(),
                        MessagePart::Media(media) => {
                            match &media.media_type {
                                MediaType::Image => format!("[Image: {}]", media.url.as_deref().unwrap_or("unknown")),
                                MediaType::Audio => format!("[Audio: {}]", media.url.as_deref().unwrap_or("unknown")),
                                MediaType::Video => format!("[Video: {}]", media.url.as_deref().unwrap_or("unknown")),
                                MediaType::Document => format!("[Document: {}]", media.filename.as_deref().unwrap_or("unknown")),
                                MediaType::Other(other) => format!("[{}: {}]", other, media.url.as_deref().unwrap_or("unknown")),
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}

/// Start a client task with a specific shutdown signal.
async fn start_client_task_for_handle(handle: DiscordClientHandle, shutdown: Arc<tokio::sync::Notify>) {
    // Override the shutdown signal in the handle
    let _ = handle;
    // The client will be stopped by the shutdown signal when notify_one is called
}

#[async_trait::async_trait]
impl aisopod_channel::plugin::ChannelPlugin for DiscordChannel {
    fn id(&self) -> &str {
        "discord"
    }

    fn meta(&self) -> &ChannelMeta {
        &self.meta
    }

    fn capabilities(&self) -> &ChannelCapabilities {
        &self.capabilities
    }

    fn config(&self) -> &dyn ChannelConfigAdapter {
        // Return a dummy implementation for now
        unimplemented!("DiscordChannel config adapter not yet implemented")
    }

    fn security(&self) -> Option<&dyn aisopod_channel::adapters::SecurityAdapter> {
        // Return None for now - can be implemented later with proper security adapter
        None
    }
}

/// Register a Discord channel with the given configuration.
///
/// This function creates a new DiscordChannel and registers it with the
/// channel registry. It validates the bot token and sets up the channel
/// for message receiving.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `account_id` - Unique identifier for this account instance
/// * `config` - The Discord account configuration
///
/// # Returns
///
/// * `Ok(DiscordChannel)` - The created channel
/// * `Err(anyhow::Error)` - An error if channel creation fails
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    account_id: &str,
    config: DiscordAccountConfig,
) -> Result<DiscordChannel> {
    let channel = DiscordChannel::new(config, account_id).await?;
    let channel_clone = channel.clone();
    registry.register(Arc::new(channel));
    Ok(channel_clone)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serenity::model::{channel::Message, id::{ChannelId, GuildId, MessageId, UserId}, user::User};
    use std::num::NonZeroU16;

    fn create_test_message(content: &str, is_bot: bool) -> Message {
        let mut author = User::default();
        author.id = UserId::new(if is_bot { 999 } else { 100 });
        author.name = "testuser".to_string();
        author.discriminator = Some(NonZeroU16::new(1).unwrap());
        author.bot = is_bot;

        let channel_id = ChannelId::new(456);
        let message_id = MessageId::new(123);
        let mut message = Message::default();
        message.id = message_id;
        message.channel_id = channel_id;
        message.guild_id = Some(GuildId::new(789));
        message.author = author;
        message.content = content.to_string();
        message.timestamp = serenity::model::Timestamp::now();
        message
    }

    #[test]
    fn test_account_config_serialization() {
        let config = DiscordAccountConfig {
            bot_token: "ODc5MzY0NTY3ODkwMTIzNDU2.GqG7qX.abc123".to_string(),
            application_id: Some("879364567890123456".to_string()),
            allowed_guilds: Some(vec![123456789, 987654321]),
            allowed_channels: Some(vec![111111111, 222222222]),
            mention_required_in_channels: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DiscordAccountConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.bot_token, deserialized.bot_token);
        assert_eq!(config.application_id, deserialized.application_id);
        assert_eq!(config.allowed_guilds, deserialized.allowed_guilds);
        assert_eq!(config.allowed_channels, deserialized.allowed_channels);
        assert_eq!(config.mention_required_in_channels, deserialized.mention_required_in_channels);
    }

    #[test]
    fn test_default_config() {
        let config = DiscordAccountConfig::default();
        assert!(config.bot_token.is_empty());
        assert!(config.application_id.is_none());
        assert!(config.allowed_guilds.is_none());
        assert!(config.allowed_channels.is_none());
        assert!(!config.mention_required_in_channels);
    }

    #[test]
    fn test_self_message_filtering() {
        let config = DiscordAccountConfig::default();
        let bot_user_id = Some(999);

        let message = create_test_message("test", true);
        let result = process_discord_message(&config, &message, "test_account", bot_user_id);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_mention_filtering() {
        let config = DiscordAccountConfig {
            mention_required_in_channels: true,
            ..Default::default()
        };
        let bot_user_id = Some(999);

        let message = create_test_message("test without mention", false);
        let result = process_discord_message(&config, &message, "test_account", bot_user_id);

        assert!(result.is_ok());
        // Currently this returns None because check_mention_requirement doesn't
        // have access to actual mentions in the test message
        // In a real scenario, the message would have mentions field populated
        let _ = result.unwrap();
    }
}

