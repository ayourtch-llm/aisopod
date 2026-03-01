//! Twitch channel plugin implementation.
//!
//! This module implements the `ChannelPlugin` trait for Twitch,
//! enabling the bot to receive and send messages via Twitch chat.

use crate::auth::{validate_token, TokenInfo};
use crate::badges::{is_moderator, is_subscriber};
use crate::config::TwitchConfig;
use crate::tmi::TmiClient;
use crate::tmi::TwitchMessage as TmiMessage;
use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, ChannelConfigAdapter, SecurityAdapter,
};
use aisopod_channel::message::{
    IncomingMessage, MessageContent, MessagePart, MessageTarget, PeerInfo, PeerKind, SenderInfo,
};
use aisopod_channel::plugin::ChannelPlugin;
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// A Twitch account wraps the configuration with its connection state.
#[derive(Clone)]
pub struct TwitchAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: TwitchConfig,
    /// The TMI client connection
    pub client: Option<Arc<Mutex<TmiClient>>>,
    /// Whether this account is currently connected
    pub connected: bool,
    /// The validated token info (if available)
    pub token_info: Option<TokenInfo>,
}

impl TwitchAccount {
    /// Create a new TwitchAccount with the given configuration.
    pub fn new(id: String, config: TwitchConfig) -> Self {
        Self {
            id,
            config,
            client: None,
            connected: false,
            token_info: None,
        }
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.config.username.is_empty() && !self.config.oauth_token.is_empty()
    }

    /// Get the username for this account.
    pub fn username(&self) -> &str {
        &self.config.username
    }
}

/// Twitch channel plugin implementation.
///
/// This struct manages connections to Twitch chat via the TMI WebSocket interface.
/// It implements the `ChannelPlugin` trait to integrate with the aisopod system.
pub struct TwitchChannel {
    /// Vector of Twitch accounts
    accounts: Vec<TwitchAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// Config adapter - stored as a field to avoid lifetime issues
    config_adapter: TwitchChannelConfigAdapter,
    /// Security adapter - stored as a field to avoid lifetime issues
    security_adapter: Option<TwitchSecurityAdapter>,
}

impl TwitchChannel {
    /// Creates a new Twitch channel with the given configuration.
    ///
    /// This method creates the channel but does not connect to Twitch yet.
    /// Call `connect()` to establish connections.
    ///
    /// # Arguments
    ///
    /// * `config` - The Twitch configuration
    /// * `channel_id` - Unique identifier for this channel instance
    ///
    /// # Returns
    ///
    /// * `Ok(TwitchChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: TwitchConfig, channel_id: &str) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        let id = format!("twitch-{}", channel_id);

        // Create account for this channel
        let account_id = format!("{}-primary", channel_id);
        let account = TwitchAccount::new(account_id, config.clone());

        // Validate the OAuth token if client_id is provided
        let token_info = if let Some(ref client_id) = config.client_id {
            match validate_token(&config.oauth_token, client_id).await {
                Ok(info) => {
                    info!(
                        "Twitch OAuth token validated successfully for user {}",
                        info.login
                    );
                    Some(info)
                }
                Err(e) => {
                    warn!("Failed to validate Twitch OAuth token: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Update account with validated token info
        let mut account = account;
        account.token_info = token_info;

        let meta = ChannelMeta {
            label: "Twitch".to_string(),
            docs_url: Some("https://www.twitch.tv".to_string()),
            ui_hints: serde_json::json!({
                "username_field": "username",
                "oauth_token_field": "oauth_token"
            }),
        };

        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group],
            supports_media: false,
            supports_reactions: false,
            supports_threads: false,
            supports_typing: false,
            supports_voice: false,
            max_message_length: Some(500), // Twitch chat limit
            supported_media_types: vec![],
        };

        // Initialize adapters
        let config_adapter = TwitchChannelConfigAdapter::new(vec![account.clone()]);
        let security_adapter = Some(TwitchSecurityAdapter::new(&account));

        Ok(Self {
            accounts: vec![account],
            id,
            meta,
            capabilities,
            shutdown_signal: None,
            config_adapter,
            security_adapter,
        })
    }

    /// Get the primary account ID for this channel.
    pub fn primary_account_id(&self) -> &str {
        &self.accounts[0].id
    }

    /// Get an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&TwitchAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut TwitchAccount> {
        self.accounts.iter_mut().find(|a| a.id == account_id)
    }

    /// Connect to Twitch TMI and join configured channels.
    pub async fn connect(&mut self) -> Result<()> {
        let account = self
            .accounts
            .first_mut()
            .ok_or_else(|| anyhow::anyhow!("No Twitch accounts configured"))?;

        info!("Connecting to Twitch TMI as {}", account.config.username);

        let mut client =
            TmiClient::connect(&account.config.username, &account.config.oauth_token).await?;

        // Enable whispers if configured
        if account.config.enable_whispers {
            client.enable_whispers();
        }

        // Join configured channels
        for channel in &account.config.channels {
            info!("Joining channel {}", channel);
            if let Err(e) = client.join_channel(channel).await {
                warn!("Failed to join channel {}: {}", channel, e);
            }
        }

        account.client = Some(Arc::new(Mutex::new(client)));
        account.connected = true;

        Ok(())
    }

    /// Disconnect from Twitch TMI.
    pub async fn disconnect(&mut self) -> Result<()> {
        let account = self
            .accounts
            .first_mut()
            .ok_or_else(|| anyhow::anyhow!("No Twitch accounts configured"))?;

        if let Some(ref client) = account.client {
            // The TMI client will be dropped, which closes the connection
            account.client = None;
        }

        account.connected = false;

        Ok(())
    }

    /// Send a message to a channel or user.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID to use for sending (ignored for single-account channels)
    /// * `target` - The target (channel like "#channelname" or username for whispers)
    /// * `message` - The message content to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_message(&self, account_id: &str, target: &str, message: &str) -> Result<()> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", account_id))?;

        let client = account
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to Twitch"))?;

        let mut cli = client.lock().await;

        // Check if this is a whisper
        if target.starts_with('@') || !target.starts_with('#') {
            // Send as whisper
            let username = target.trim_start_matches('@');
            cli.send_whisper(username, message).await?;
        } else {
            // Send to channel
            cli.send_message(target, message).await?;
        }

        Ok(())
    }

    /// Receive messages from Twitch TMI.
    ///
    /// This method returns incoming messages from Twitch chat.
    /// The caller should handle messages from the primary account.
    ///
    /// # Returns
    ///
    /// * `Ok(IncomingMessage)` - An incoming message
    /// * `Err(anyhow::Error)` - An error if receiving fails
    pub async fn receive(&mut self) -> Result<IncomingMessage> {
        let mut account = self
            .accounts
            .first_mut()
            .ok_or_else(|| anyhow::anyhow!("No Twitch accounts configured"))?;

        let client = account
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to Twitch"))?;

        let mut cli = client.lock().await;
        let tmi_msg = cli.read_message().await?;
        let account_id = account.id.clone();
        let channel_id = self.id.clone();

        // Convert TMI message to aisopod IncomingMessage
        Ok(convert_tmi_message(
            &channel_id,
            &account_id,
            &account,
            &tmi_msg,
        ))
    }
}

/// Convert a TMI message to an aisopod IncomingMessage.
fn convert_tmi_message(
    channel_id: &str,
    account_id: &str,
    account: &TwitchAccount,
    msg: &TmiMessage,
) -> IncomingMessage {
    let is_whisper = msg.is_whisper;
    let channel = if is_whisper {
        // For whispers, the username is in the channel field
        format!("@{}", msg.username)
    } else {
        msg.channel.clone()
    };

    let sender = SenderInfo {
        id: msg.tags.user_id.clone(),
        display_name: Some(msg.tags.display_name.clone()),
        username: Some(msg.username.clone()),
        is_bot: msg.tags.is_mod || msg.tags.is_subscriber,
    };

    let target = if is_whisper {
        MessageTarget {
            channel: channel_id.to_string(),
            account_id: account_id.to_string(),
            peer: PeerInfo {
                id: msg.tags.user_id.clone(),
                kind: PeerKind::User,
                title: Some(msg.username.clone()),
            },
            thread_id: None,
        }
    } else {
        MessageTarget {
            channel: channel_id.to_string(),
            account_id: account_id.to_string(),
            peer: PeerInfo {
                id: msg.tags.user_id.clone(),
                kind: PeerKind::Channel,
                title: Some(msg.channel.clone()),
            },
            thread_id: None,
        }
    };

    let content = MessageContent::Text(msg.text.clone());

    IncomingMessage {
        id: format!("twitch-{}-{}", msg.username, Utc::now().timestamp()),
        channel: channel_id.to_string(),
        account_id: account_id.to_string(),
        sender,
        peer: target.peer,
        content,
        timestamp: Utc::now(),
        reply_to: None,
        metadata: serde_json::json!({
            "is_whisper": is_whisper,
            "badges": msg.tags.badges,
            "display_name": msg.tags.display_name,
        }),
    }
}

/// Parse Twitch badges from string to Badge struct.
fn parse_twitch_badges(badges: &[String]) -> Vec<crate::badges::Badge> {
    badges
        .iter()
        .map(|b| crate::badges::Badge::new(b.clone(), "1"))
        .collect()
}

/// ChannelConfigAdapter implementation for TwitchChannel.
#[derive(Clone)]
pub struct TwitchChannelConfigAdapter {
    /// Reference to the channel accounts
    accounts: Vec<TwitchAccount>,
}

impl TwitchChannelConfigAdapter {
    /// Create a new TwitchChannelConfigAdapter.
    pub fn new(accounts: Vec<TwitchAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for TwitchChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| AccountSnapshot {
                id: a.id.clone(),
                channel: "twitch".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, id: &str) -> Result<()> {
        info!("Enabling Twitch account {}", id);
        // In production, this would enable the account
        Ok(())
    }

    fn disable_account(&self, id: &str) -> Result<()> {
        info!("Disabling Twitch account {}", id);
        // In production, this would disable the account
        Ok(())
    }

    fn delete_account(&self, id: &str) -> Result<()> {
        info!("Deleting Twitch account {}", id);
        // In production, this would delete the account
        Ok(())
    }
}

/// SecurityAdapter implementation for TwitchChannel.
///
/// This adapter checks if senders are allowed to interact with the bot
/// based on Twitch-specific logic (moderators, subscribers, etc.).
#[derive(Clone)]
pub struct TwitchSecurityAdapter {
    /// The account configuration
    account: TwitchAccount,
}

impl TwitchSecurityAdapter {
    /// Create a new TwitchSecurityAdapter.
    pub fn new(account: &TwitchAccount) -> Self {
        Self {
            account: account.clone(),
        }
    }
}

#[async_trait]
impl SecurityAdapter for TwitchSecurityAdapter {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        // For Twitch, allow all senders who have access to the channel
        // (moderators, subscribers, or anyone in public chat)
        // A more sophisticated implementation could check against an allowlist

        // If the sender is a bot (moderator), always allow
        if sender.is_bot {
            return true;
        }

        // If the channel is public, allow subscribers and regular users
        // (We don't have a way to know if the channel is subscriber-only here,
        // so we'll allow all users for now)
        true
    }

    fn requires_mention_in_group(&self) -> bool {
        // Twitch doesn't require mentions in channels
        false
    }
}

#[async_trait]
impl ChannelPlugin for TwitchChannel {
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

    /// Returns the security adapter for this channel if available.
    fn security(&self) -> Option<&dyn SecurityAdapter> {
        self.security_adapter
            .as_ref()
            .map(|a| a as &dyn SecurityAdapter)
    }
}

/// Register a Twitch channel with the channel registry.
///
/// This is a convenience function for creating and registering a Twitch channel.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The Twitch configuration
/// * `channel_id` - Unique identifier for this channel instance
///
/// # Returns
///
/// * `Ok(())` - Channel was registered successfully
/// * `Err(anyhow::Error)` - An error if registration fails
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: TwitchConfig,
    channel_id: &str,
) -> Result<()> {
    let channel = TwitchChannel::new(config, channel_id).await?;
    registry.register(Arc::new(channel));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twitch_config_validation() {
        let config = TwitchConfig {
            username: "testbot".to_string(),
            oauth_token: "oauth:abc123".to_string(),
            channels: vec!["#test".to_string()],
            enable_whispers: false,
            client_id: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_twitch_config_validation_empty_username() {
        let config = TwitchConfig {
            username: "".to_string(),
            oauth_token: "oauth:abc123".to_string(),
            channels: vec!["#test".to_string()],
            enable_whispers: false,
            client_id: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_twitch_config_validation_no_channels() {
        let config = TwitchConfig {
            username: "testbot".to_string(),
            oauth_token: "oauth:abc123".to_string(),
            channels: vec![],
            enable_whispers: false,
            client_id: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_parse_twitch_badges() {
        let badge_strings = vec!["moderator".to_string(), "subscriber".to_string()];
        let badges = parse_twitch_badges(&badge_strings);

        assert_eq!(badges.len(), 2);
        assert_eq!(badges[0].name, "moderator");
        assert_eq!(badges[1].name, "subscriber");
    }

    #[test]
    fn test_is_moderator_with_badge() {
        let badges = vec![
            crate::badges::Badge::new("moderator", "1"),
            crate::badges::Badge::new("subscriber", "12"),
        ];

        assert!(is_moderator(&badges));
    }
}
