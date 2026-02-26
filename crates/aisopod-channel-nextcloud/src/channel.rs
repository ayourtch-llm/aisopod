//! Nextcloud Talk channel plugin implementation.
//!
//! This module implements the `ChannelPlugin` trait for Nextcloud Talk,
//! enabling the bot to participate in Nextcloud Talk rooms.

use crate::api::{NextcloudTalkApi, TalkMessage};
use crate::config::NextcloudConfig;
use crate::polling::MessagePoller;
use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, ChannelConfigAdapter, SecurityAdapter,
};
use aisopod_channel::message::{IncomingMessage, MessageContent, MessagePart, MessageTarget, PeerInfo, PeerKind, SenderInfo};
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use aisopod_channel::plugin::ChannelPlugin;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, warn};

/// A Nextcloud Talk account wraps the configuration with its state.
#[derive(Clone)]
pub struct NextcloudAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: NextcloudConfig,
    /// Whether this account is currently connected
    pub connected: bool,
}

impl NextcloudAccount {
    /// Create a new NextcloudAccount with the given configuration.
    pub fn new(id: String, config: NextcloudConfig) -> Self {
        Self {
            id,
            config,
            connected: false,
        }
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.config.server_url.is_empty()
            && !self.config.username.is_empty()
            && !self.config.password.is_empty()
    }

    /// Get the API client for this account.
    pub fn get_api(&self) -> Result<NextcloudTalkApi> {
        NextcloudTalkApi::new(
            &self.config.server_url,
            &self.config.username,
            &self.config.password,
        )
    }
}

/// Nextcloud Talk channel plugin implementation.
///
/// This struct manages Nextcloud Talk connections and room participation.
/// It implements the `ChannelPlugin` trait to integrate with the aisopod system.
pub struct NextcloudChannel {
    /// Vector of Nextcloud Talk accounts
    accounts: Vec<NextcloudAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// Config adapter - stored as a field to avoid lifetime issues
    config_adapter: NextcloudChannelConfigAdapter,
    /// Security adapter - stored as a field to avoid lifetime issues
    security_adapter: Option<NextcloudSecurityAdapter>,
}

impl NextcloudChannel {
    /// Creates a new Nextcloud Talk channel with the given configuration.
    ///
    /// This method validates the configuration and creates the channel,
    /// but does not connect to the Nextcloud server yet. Call `connect()`
    /// to establish the connection.
    ///
    /// # Arguments
    ///
    /// * `config` - The Nextcloud Talk configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(NextcloudChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: NextcloudConfig, account_id: &str) -> Result<Self> {
        // Validate configuration
        if config.server_url.is_empty() {
            return Err(anyhow::anyhow!("Nextcloud server URL is required"));
        }
        if config.username.is_empty() {
            return Err(anyhow::anyhow!("Username is required"));
        }
        if config.password.is_empty() {
            return Err(anyhow::anyhow!("Password is required"));
        }

        let account = NextcloudAccount::new(account_id.to_string(), config.clone());

        // Validate the configuration by attempting to create API client
        if let Err(e) = account.get_api() {
            return Err(anyhow::anyhow!(
                "Failed to create API client: {}",
                e
            ));
        }

        let id = format!("nextcloud-{}", account_id);
        let meta = ChannelMeta {
            label: "Nextcloud Talk".to_string(),
            docs_url: Some("https://docs.nextcloud.com/server/latest/admin_manual/configuration_server/talk_configuration.html".to_string()),
            ui_hints: serde_json::json!({
                "server_url_field": "server_url",
                "username_field": "username",
                "password_field": "password",
                "rooms_field": "rooms"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![
                ChatType::Group,
                ChatType::Channel,
            ],
            supports_media: true,
            supports_reactions: true,
            supports_threads: true,
            supports_typing: false,
            supports_voice: false,
            max_message_length: Some(10000), // Nextcloud Talk supports long messages
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Audio,
                MediaType::Video,
                MediaType::Document,
            ],
        };
        let accounts = vec![account];
        let config_adapter = NextcloudChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(NextcloudSecurityAdapter::new(accounts.clone()));

        Ok(Self {
            accounts,
            id,
            meta,
            capabilities,
            shutdown_signal: None,
            config_adapter,
            security_adapter,
        })
    }

    /// Get all configured account IDs for this channel.
    pub fn list_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.id.clone()).collect()
    }

    /// Get an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&NextcloudAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut NextcloudAccount> {
        self.accounts.iter_mut().find(|a| a.id == account_id)
    }

    /// Connect to the Nextcloud Talk server.
    pub async fn connect(&mut self) -> Result<()> {
        for account in &mut self.accounts {
            info!(
                "Connecting to Nextcloud server {} as {}",
                account.config.server_url, account.config.username
            );

            let api = account.get_api()?;
            
            // Get list of rooms to verify connection
            match api.get_rooms().await {
                Ok(rooms) => {
                    info!("Connected to Nextcloud, found {} rooms", rooms.len());
                    
                    // Print available rooms for reference
                    for room in &rooms {
                        info!("  - {} (token: {})", room.name, room.token);
                    }
                    
                    account.connected = true;
                }
                Err(e) => {
                    error!("Failed to get rooms: {}", e);
                    account.connected = false;
                    return Err(anyhow::anyhow!(
                        "Failed to connect to Nextcloud: {}",
                        e
                    ));
                }
            }
        }

        Ok(())
    }

    /// Disconnect from the Nextcloud Talk server.
    pub async fn disconnect(&mut self) -> Result<()> {
        for account in &mut self.accounts {
            if account.connected {
                info!(
                    "Disconnecting from Nextcloud server {}",
                    account.config.server_url
                );
                account.connected = false;
            }
        }

        Ok(())
    }

    /// Send a message to a room.
    ///
    /// # Arguments
    ///
    /// * `room_token` - The room token to send the message to
    /// * `message` - The message content to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_to_room(&self, room_token: &str, message: &str) -> Result<()> {
        for account in &self.accounts {
            if account.connected {
                let api = account.get_api()?;
                return api.send_message(room_token, message).await;
            }
        }
        
        Err(anyhow::anyhow!("No connected accounts available"))
    }

    /// Create a message poller for this channel.
    pub fn create_poller(&self) -> Result<MessagePoller> {
        let account = self.accounts.first()
            .ok_or_else(|| anyhow::anyhow!("No accounts configured"))?;
        
        let api = account.get_api()?;
        
        Ok(MessagePoller::new(
            api,
            account.config.rooms.clone(),
            account.config.poll_interval_secs,
        ))
    }
}

/// ChannelConfigAdapter implementation for NextcloudChannel.
#[derive(Clone)]
pub struct NextcloudChannelConfigAdapter {
    /// Reference to the channel accounts
    accounts: Vec<NextcloudAccount>,
}

impl NextcloudChannelConfigAdapter {
    /// Create a new NextcloudChannelConfigAdapter.
    pub fn new(accounts: Vec<NextcloudAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for NextcloudChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts.iter()
            .find(|a| a.id == id)
            .map(|a| AccountSnapshot {
                id: a.id.clone(),
                channel: "nextcloud".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, _id: &str) -> Result<()> {
        warn!("enable_account not yet implemented for Nextcloud");
        Ok(())
    }

    fn disable_account(&self, _id: &str) -> Result<()> {
        warn!("disable_account not yet implemented for Nextcloud");
        Ok(())
    }

    fn delete_account(&self, _id: &str) -> Result<()> {
        warn!("delete_account not yet implemented for Nextcloud");
        Ok(())
    }
}

/// Security adapter for NextcloudChannel.
#[derive(Clone)]
pub struct NextcloudSecurityAdapter {
    /// Reference to the channel accounts
    accounts: Vec<NextcloudAccount>,
}

impl NextcloudSecurityAdapter {
    /// Create a new NextcloudSecurityAdapter.
    pub fn new(accounts: Vec<NextcloudAccount>) -> Self {
        Self { accounts }
    }
}

impl SecurityAdapter for NextcloudSecurityAdapter {
    fn is_allowed_sender(&self, _sender: &SenderInfo) -> bool {
        // Nextcloud Talk allows all users in the room to send messages
        true
    }

    fn requires_mention_in_group(&self) -> bool {
        // Nextcloud Talk doesn't require mentioning the bot
        false
    }
}

#[async_trait]
impl ChannelPlugin for NextcloudChannel {
    fn id(&self) -> &str {
        &self.id
    }

    fn meta(&self) -> &ChannelMeta {
        &self.meta
    }

    fn capabilities(&self) -> &ChannelCapabilities {
        &self.capabilities
    }

    fn config(&self) -> &dyn ChannelConfigAdapter {
        &self.config_adapter
    }

    fn security(&self) -> Option<&dyn SecurityAdapter> {
        self.security_adapter.as_ref().map(|a| a as &dyn SecurityAdapter)
    }

    async fn connect(&mut self) -> Result<()> {
        self.connect().await
    }

    async fn send(&self, msg: aisopod_channel::message::OutgoingMessage) -> Result<()> {
        // Extract the room token from the target
        let room_token = msg.target.peer.id.clone();
        let message = match &msg.content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Media(media) => {
                // For media messages, create a placeholder text
                format!("[Media: {}]", media.url.as_deref().unwrap_or("unknown"))
            }
            MessageContent::Mixed(parts) => {
                parts.iter()
                    .map(|part| match part {
                        MessagePart::Text(text) => text.clone(),
                        MessagePart::Media(media) => {
                            format!("[Media: {}]", media.url.as_deref().unwrap_or("unknown"))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        };

        self.send_to_room(&room_token, &message).await
    }

    async fn receive(&mut self) -> Result<aisopod_channel::message::IncomingMessage> {
        // This is a simplified implementation - in production, you'd want
        // a proper polling or WebSocket-based approach
        let mut poller = self.create_poller()?;
        
        loop {
            match poller.poll_once().await {
                Ok(messages) => {
                    if !messages.is_empty() {
                        let (room, talk_msg) = messages.first()
                            .ok_or_else(|| anyhow::anyhow!("No messages received"))?;
                        
                        // Convert Nextcloud Talk message to aisopod IncomingMessage
                        let incoming = IncomingMessage {
                            id: format!("nc-{}", talk_msg.id),
                            channel: self.id.clone(),
                            account_id: self.accounts.first()
                                .map(|a| a.id.clone())
                                .unwrap_or_default(),
                            sender: SenderInfo {
                                id: talk_msg.actor_id.clone(),
                                display_name: talk_msg.actor_display_name.clone(),
                                username: Some(talk_msg.actor_id.clone()),
                                is_bot: false,
                            },
                            peer: PeerInfo {
                                id: room.clone(),
                                kind: PeerKind::Group,
                                title: None,
                            },
                            content: MessageContent::Text(talk_msg.message.clone()),
                            reply_to: None,
                            timestamp: DateTime::<Utc>::from_timestamp(talk_msg.timestamp, 0)
                                .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?,
                            metadata: serde_json::json!({
                                "actor_type": talk_msg.actor_type,
                                "chat_id": talk_msg.chat_id
                            }),
                        };
                        
                        return Ok(incoming);
                    }
                }
                Err(e) => {
                    warn!("Polling failed: {}", e);
                }
            }
            
            // Wait before polling again
            tokio::time::sleep(poller.poll_interval()).await;
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.disconnect().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_validation() {
        // Test empty config fails
        let config = NextcloudConfig::default();
        let result = NextcloudChannel::new(config, "test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_server_url() {
        let config = NextcloudConfig {
            server_url: String::new(),
            username: "user".to_string(),
            password: "pass".to_string(),
            rooms: vec![],
            poll_interval_secs: 10,
        };
        
        let result = NextcloudChannel::new(config, "test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_username() {
        let config = NextcloudConfig {
            server_url: "https://cloud.example.com".to_string(),
            username: String::new(),
            password: "pass".to_string(),
            rooms: vec![],
            poll_interval_secs: 10,
        };
        
        let result = NextcloudChannel::new(config, "test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_password() {
        let config = NextcloudConfig {
            server_url: "https://cloud.example.com".to_string(),
            username: "user".to_string(),
            password: String::new(),
            rooms: vec![],
            poll_interval_secs: 10,
        };
        
        let result = NextcloudChannel::new(config, "test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_valid_config() {
        let config = NextcloudConfig {
            server_url: "https://cloud.example.com".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            rooms: vec!["room1".to_string()],
            poll_interval_secs: 10,
        };
        
        // This should succeed (API validation happens at connect time)
        let result = NextcloudChannel::new(config, "test").await;
        assert!(result.is_ok());
    }
}
