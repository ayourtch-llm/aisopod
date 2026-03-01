//! Connection management for Discord channel.
//!
//! This module handles the gateway connection lifecycle using serenity,
//! including client initialization, event handler setup, and graceful shutdown.

use crate::DiscordAccountConfig;
use anyhow::Result;
use async_trait::async_trait;
use serenity::{
    client::{Client, EventHandler},
    model::{
        gateway::{GatewayIntents, Ready},
        prelude::Message,
    },
    prelude::Context,
};
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{error, info, warn};

/// A handle to a running Discord client task.
pub struct DiscordClientHandle {
    /// The account ID
    pub account_id: String,
    /// The serenity client (Arc-wrapped for sharing)
    pub client: Arc<Client>,
    /// Shutdown notification for graceful termination
    pub shutdown: Arc<Notify>,
}

impl DiscordClientHandle {
    /// Create a new DiscordClientHandle.
    pub fn new(account_id: String, client: Arc<Client>) -> Self {
        Self {
            account_id,
            client,
            shutdown: Arc::new(Notify::new()),
        }
    }

    /// Start the client in a background task.
    pub fn start(self) {
        let account_id = self.account_id.clone();
        let client = self.client;
        let shutdown = self.shutdown.clone();

        tokio::spawn(async move {
            info!("Starting Discord client task for account {}", account_id);

            tokio::select! {
                biased;
                _ = shutdown.notified() => {
                    info!("Shutdown signal received for Discord client {}", account_id);
                }
                // Start the client
                result = async move {
                    // We need to unwrap the Arc to get Client, but only if there's one reference
                    // Since we just received the Arc as self, there should be only one reference
                    // Use Arc::try_unwrap to get the inner Client
                    match Arc::try_unwrap(client) {
                        Ok(mut inner_client) => {
                            // We have ownership of Client, can call start()
                            inner_client.start().await
                        }
                        Err(_) => {
                            // There are multiple references, we can't unwrap
                            // This shouldn't happen in normal usage
                            Err(serenity::Error::Other("Cannot start client - multiple Arc references. Client must be stored directly, not Arc-wrapped."))
                        }
                    }
                } => {
                    if let Err(e) = result {
                        error!("Discord client error for account {}: {}", account_id, e);
                    }
                }
            }

            info!("Discord client task stopped for account {}", account_id);
        });
    }
}

/// Discord event handler that processes incoming messages.
#[derive(Clone)]
pub struct DiscordEventHandler {
    /// The account configuration
    pub config: DiscordAccountConfig,
    /// The account ID
    pub account_id: String,
}

impl DiscordEventHandler {
    /// Create a new DiscordEventHandler.
    pub fn new(config: DiscordAccountConfig, account_id: String) -> Self {
        Self { config, account_id }
    }
}

#[async_trait]
impl EventHandler for DiscordEventHandler {
    /// Handle the ready event when the client connects.
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Discord bot connected as {}", ready.user.tag());
    }

    /// Handle incoming messages.
    async fn message(&self, ctx: Context, msg: Message) {
        // Skip messages from the bot itself
        if msg.author.bot {
            return;
        }

        // Check mention requirement if configured
        if self.config.mention_required_in_channels {
            if !self.is_mentioned(&msg) {
                warn!(
                    "Message filtered: mention required but bot not mentioned in channel {}",
                    msg.channel_id
                );
                return;
            }
        }

        // Log the message for now
        let content = msg.content.trim();
        if !content.is_empty() {
            info!(
                "Discord message received from {}: {}",
                msg.author.tag(),
                content
            );
        }
    }
}

impl DiscordEventHandler {
    /// Check if the bot is mentioned in the message.
    fn is_mentioned(&self, msg: &Message) -> bool {
        // Check if the bot's user ID is in the mentions list
        for user in &msg.mentions {
            if let Some(app_id) = &self.config.application_id {
                if let Ok(bot_id) = app_id.parse::<u64>() {
                    if user.id.get() == bot_id {
                        return true;
                    }
                }
            }
        }
        // Fallback: check if user ID matches the bot's ID from ready event
        false
    }
}

/// Create a Discord client with the given configuration.
///
/// # Arguments
///
/// * `config` - The Discord account configuration
/// * `account_id` - Unique identifier for this account instance
///
/// # Returns
///
/// * `Ok(DiscordClientHandle)` - The client handle with shutdown capability
/// * `Err(anyhow::Error)` - An error if client creation fails
pub async fn create_client(
    config: &DiscordAccountConfig,
    account_id: &str,
) -> Result<DiscordClientHandle> {
    // Validate bot token is not empty
    if config.bot_token.trim().is_empty() {
        return Err(anyhow::anyhow!("Bot token cannot be empty"));
    }

    // Set up gateway intents
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create event handler
    let event_handler = DiscordEventHandler::new(config.clone(), account_id.to_string());

    // Create client builder
    let client = Client::builder(&config.bot_token, intents)
        .event_handler(event_handler)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create Discord client: {}", e))?;

    // Wrap in Arc for sharing
    Ok(DiscordClientHandle::new(
        account_id.to_string(),
        Arc::new(client),
    ))
}

/// Start the Discord client in a background task.
///
/// # Arguments
///
/// * `handle` - The Discord client handle
pub fn start_client_task(handle: DiscordClientHandle) {
    handle.start();
}
