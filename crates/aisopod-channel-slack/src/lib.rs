//! Slack channel plugin for aisopod.
//!
//! This crate provides a channel plugin implementation for Slack,
//! enabling the bot to receive and send messages via the Slack Web API and Socket Mode.
//!
//! # Features
//!
//! - Socket Mode WebSocket connection for receiving real-time events
//! - Support for DMs, channels, and threads
//! - Message normalization to shared `IncomingMessage` type
//! - Self-message filtering to avoid loops
//! - Channel and thread filtering
//! - Bot token authentication with `auth.test`
//! - App token authentication for Socket Mode connection

mod connection;
mod receive;
mod socket_mode;

use aisopod_channel::adapters::{AccountConfig, AccountSnapshot, ChannelConfigAdapter};
use aisopod_channel::message::{IncomingMessage, Media, MessageContent, MessagePart, PeerInfo, PeerKind, SenderInfo, OutgoingMessage};
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

// Re-export modules
pub use connection::{SlackClientHandle, create_client};
pub use receive::{normalize_message, should_filter_message, process_slack_message};
pub use socket_mode::{SlackSocketModeConnection, SocketModeEvent, start_socket_mode_task};

/// Configuration for a Slack bot account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackAccountConfig {
    /// The bot token from Slack Developer Portal (starts with xoxb-)
    pub bot_token: String,
    /// Optional app token for Socket Mode connection (starts with xapp-)
    pub app_token: Option<String>,
    /// Optional signing secret for request verification
    pub signing_secret: Option<String>,
    /// Optional list of allowed channel IDs (if empty, all channels are allowed)
    pub allowed_channels: Option<Vec<String>>,
    /// Optional list of allowed user IDs (if empty, all users are allowed)
    pub allowed_users: Option<Vec<String>>,
    /// Whether messages require a bot mention to be received
    #[serde(default = "default_mention_required")]
    pub mention_required: bool,
    /// Maximum number of reconnection attempts
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: usize,
}

fn default_mention_required() -> bool {
    false
}

fn default_max_reconnect_attempts() -> usize {
    5
}

impl Default for SlackAccountConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            app_token: None,
            signing_secret: None,
            allowed_channels: None,
            allowed_users: None,
            mention_required: false,
            max_reconnect_attempts: 5,
        }
    }
}

/// A Slack account wraps client configuration with its settings.
#[derive(Clone)]
pub struct SlackAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: SlackAccountConfig,
    /// The bot's user ID (fetched from auth.test on startup)
    pub bot_user_id: Option<String>,
}

impl SlackAccount {
    /// Create a new SlackAccount with the given configuration.
    pub fn new(id: String, config: SlackAccountConfig) -> Self {
        Self {
            id,
            config,
            bot_user_id: None,
        }
    }

    /// Set the bot user ID after authentication.
    pub fn set_bot_user_id(&mut self, user_id: String) {
        self.bot_user_id = Some(user_id);
    }
}

/// A Slack channel with connection handles.
#[derive(Clone)]
pub struct SlackChannelWithConnection {
    /// The account information
    pub account: SlackAccount,
    /// The Socket Mode connection handle
    pub connection: SlackSocketModeConnection,
}

impl SlackChannelWithConnection {
    /// Create a new SlackChannelWithConnection.
    pub fn new(account: SlackAccount, connection: SlackSocketModeConnection) -> Self {
        Self { account, connection }
    }

    /// Get the account ID.
    pub fn id(&self) -> &str {
        &self.account.id
    }

    /// Get the connection.
    pub fn connection(&self) -> &SlackSocketModeConnection {
        &self.connection
    }
}

/// A channel plugin implementation for Slack with Socket Mode support.
#[derive(Clone)]
pub struct SlackChannel {
    /// Vector of Slack accounts with their connections
    accounts: Vec<SlackChannelWithConnection>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
}

impl SlackChannel {
    /// Creates a new Slack channel with the given configuration.
    ///
    /// This method validates the bot token by checking if it's non-empty.
    /// The actual authentication and Socket Mode connection happens when `start()` is called.
    ///
    /// # Arguments
    ///
    /// * `config` - The Slack account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(SlackChannel)` - The channel if bot token is valid
    /// * `Err(anyhow::Error)` - An error if the bot token is invalid
    pub async fn new(config: SlackAccountConfig, account_id: &str) -> Result<Self> {
        // Validate bot token
        if config.bot_token.trim().is_empty() {
            return Err(anyhow::anyhow!("Bot token cannot be empty"));
        }

        let id = format!("slack-{}", account_id);
        let meta = ChannelMeta {
            label: "Slack".to_string(),
            docs_url: Some("https://api.slack.com/docs".to_string()),
            ui_hints: serde_json::json!({
                "bot_token_field": "bot_token",
                "app_token_field": "app_token"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group, ChatType::Channel, ChatType::Thread],
            supports_media: true,
            supports_reactions: true,
            supports_threads: true,
            supports_typing: true,
            supports_voice: false,
            max_message_length: Some(4000),
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Audio,
                MediaType::Video,
                MediaType::Document,
            ],
        };

        // Create an account without a connection (connection will be added when start() is called)
        let account = SlackAccount::new(account_id.to_string(), config);

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
    pub fn add_account(&mut self, account: SlackChannelWithConnection) {
        self.accounts.push(account);
    }

    /// Remove an account by its ID.
    pub fn remove_account(&mut self, account_id: &str) -> bool {
        let len = self.accounts.len();
        self.accounts.retain(|a| a.id() != account_id);
        len != self.accounts.len()
    }

    /// Get an account with connection by its ID.
    pub fn get_account_with_connection(&self, account_id: &str) -> Option<&SlackChannelWithConnection> {
        self.accounts.iter().find(|a| a.id() == account_id)
    }

    /// Get an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&SlackAccount> {
        self.accounts.iter()
            .find(|a| a.id() == account_id)
            .map(|a| &a.account)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut SlackAccount> {
        self.accounts.iter_mut()
            .find(|a| a.id() == account_id)
            .map(|a| &mut a.account)
    }

    /// Starts the Slack Socket Mode connection and stores the connection handles.
    ///
    /// This creates Socket Mode connections for each account and stores them in the channel.
    /// The background tasks are spawned to connect to Slack via WebSocket and listen for incoming events.
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
        let accounts_to_connect: Vec<SlackAccount> = match account_id {
            Some(id) => {
                self.get_account(id)
                    .cloned()
                    .map(|a| vec![a])
                    .unwrap_or_default()
            }
            None => {
                // Get accounts from SlackChannelWithConnection
                self.accounts.iter().map(|a| a.account.clone()).collect()
            }
        };

        if accounts_to_connect.is_empty() {
            return Err(anyhow::anyhow!("No accounts found to connect"));
        }

        // Create shutdown signal
        let shutdown = Arc::new(tokio::sync::Notify::new());
        self.shutdown_signal = Some(shutdown.clone());

        let mut tasks: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>> = Vec::new();

        for account in accounts_to_connect {
            let account_id = account.id.clone();
            let config = account.config.clone();
            let shutdown_clone = shutdown.clone();

            // Create Socket Mode connection for this account
            let connection = SlackSocketModeConnection::new(&config, &account_id).await?;

            // Store the account with connection
            let account_with_connection = SlackChannelWithConnection::new(account, connection.clone());
            self.accounts.push(account_with_connection);

            // Start the socket mode task
            let task = start_socket_mode_task(connection, shutdown_clone, account_id);
            tasks.push(Box::pin(task));
        }

        // Combine all tasks into a single future
        let task = async move {
            info!("Starting Slack Socket Mode for {} accounts", tasks.len());

            // Wait for all tasks to complete
            futures_util::future::join_all(tasks).await;

            info!("Slack Socket Mode shutdown complete");
        };

        Ok(task)
    }

    /// Stops the Slack Socket Mode connection gracefully.
    pub async fn stop(&mut self) {
        if let Some(shutdown) = &self.shutdown_signal {
            shutdown.notify_one();
        }
    }

    /// Send a message to Slack.
    ///
    /// This method sends an `OutgoingMessage` to the specified Slack channel.
    /// It supports text and other message content types.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID to send from
    /// * `message` - The outgoing message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_message(
        &self,
        account_id: &str,
        message: &OutgoingMessage,
    ) -> Result<()> {
        // Get the account
        let account = self.get_account(account_id)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_id))?;

        // Extract channel ID from target
        let channel_id = message.target.peer.id.clone();

        // Build the message text
        let text = self.content_to_string_from_message(message);

        // Send using the connection
        let account_with_conn = self.get_account_with_connection(account_id)
            .ok_or_else(|| anyhow::anyhow!("Account connection not found: {}", account_id))?;

        account_with_conn.connection().send_message(&channel_id, &text).await
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

/// Start a socket mode task with a specific shutdown signal.
async fn start_socket_mode_task_for_connection(
    _connection: SlackSocketModeConnection,
    _shutdown: Arc<tokio::sync::Notify>,
    _account_id: String,
) {
    // The connection will be stopped by the shutdown signal when notify_one is called
}

#[async_trait::async_trait]
impl aisopod_channel::plugin::ChannelPlugin for SlackChannel {
    fn id(&self) -> &str {
        "slack"
    }

    fn meta(&self) -> &ChannelMeta {
        &self.meta
    }

    fn capabilities(&self) -> &ChannelCapabilities {
        &self.capabilities
    }

    fn config(&self) -> &dyn ChannelConfigAdapter {
        // Return a dummy implementation for now
        unimplemented!("SlackChannel config adapter not yet implemented")
    }

    fn security(&self) -> Option<&dyn aisopod_channel::adapters::SecurityAdapter> {
        // Return None for now - can be implemented later with proper security adapter
        None
    }
}

/// Register a Slack channel with the given configuration.
///
/// This function creates a new SlackChannel and registers it with the
/// channel registry. It validates the bot token and sets up the channel
/// for message receiving.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `account_id` - Unique identifier for this account instance
/// * `config` - The Slack account configuration
///
/// # Returns
///
/// * `Ok(SlackChannel)` - The created channel
/// * `Err(anyhow::Error)` - An error if channel creation fails
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    account_id: &str,
    config: SlackAccountConfig,
) -> Result<SlackChannel> {
    let channel = SlackChannel::new(config, account_id).await?;
    let channel_clone = channel.clone();
    registry.register(Arc::new(channel));
    Ok(channel_clone)
}
