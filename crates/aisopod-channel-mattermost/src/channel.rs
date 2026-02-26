//! Mattermost channel plugin implementation.
//!
//! This module implements the ChannelPlugin trait for Mattermost,
//! enabling the bot to receive and send messages via the Mattermost API.

use crate::api::{ApiError, Channel, ChannelType, MattermostApi, User};
use crate::auth::{authenticate, AuthResult, extract_token, requires_login};
use crate::config::{MattermostAuth, MattermostConfig};
use crate::websocket::{MattermostEvent, MattermostWebSocket};
use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, ChannelConfigAdapter, SecurityAdapter,
};
use aisopod_channel::message::{IncomingMessage, MessageContent, MessagePart, MessageTarget, OutgoingMessage, PeerInfo, PeerKind, SenderInfo};
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use aisopod_channel::plugin::ChannelPlugin;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{debug, error, info, instrument, warn};

/// A Mattermost account with its connection state.
#[derive(Clone)]
pub struct MattermostAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: MattermostConfig,
    /// Whether this account is currently connected
    pub connected: bool,
    /// The timestamp of the last connection
    pub last_connected: Option<DateTime<Utc>>,
    /// The authentication result
    pub auth_result: Option<AuthResult>,
    /// The user ID
    pub user_id: Option<String>,
    /// The username
    pub username: Option<String>,
}

impl MattermostAccount {
    /// Create a new MattermostAccount with the given configuration.
    pub fn new(id: String, config: MattermostConfig) -> Self {
        Self {
            id,
            config,
            connected: false,
            last_connected: None,
            auth_result: None,
            user_id: None,
            username: None,
        }
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.config.server_url.is_empty()
    }
}

/// A Mattermost account with active connections.
#[derive(Clone)]
pub struct MattermostAccountWithConnections {
    /// The account information
    pub account: MattermostAccount,
    /// The API client
    pub api: MattermostApi,
    /// The WebSocket connection (if connected)
    pub websocket: Option<Arc<AsyncMutex<MattermostWebSocket>>>,
    /// Channel ID cache
    pub channel_cache: Arc<tokio::sync::Mutex<HashMap<String, Channel>>>,
    /// User ID cache
    pub user_cache: Arc<tokio::sync::Mutex<HashMap<String, User>>>,
}

impl MattermostAccountWithConnections {
    /// Create a new account with connections.
    pub fn new(account: MattermostAccount, api: MattermostApi) -> Self {
        Self {
            account,
            api,
            websocket: None,
            channel_cache: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            user_cache: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

/// Mattermost channel plugin implementation.
pub struct MattermostChannel {
    /// Vector of Mattermost accounts with their connections
    accounts: Vec<MattermostAccountWithConnections>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// Configuration adapter
    config_adapter: MattermostChannelConfigAdapter,
    /// Security adapter
    security_adapter: Option<MattermostSecurityAdapter>,
}

impl MattermostChannel {
    /// Creates a new Mattermost channel with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The Mattermost account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(MattermostChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: MattermostConfig, account_id: &str) -> Result<Self> {
        // Validate the configuration
        config.validate()?;

        let id = format!("mattermost-{}", account_id);
        let meta = ChannelMeta {
            label: "Mattermost".to_string(),
            docs_url: Some("https://docs.mattermost.com".to_string()),
            ui_hints: serde_json::json!({
                "server_url_field": "server_url",
                "token_field": "auth.token"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group, ChatType::Channel],
            supports_media: true,
            supports_reactions: true,
            supports_threads: true,
            supports_typing: true,
            supports_voice: false,
            max_message_length: Some(8000),
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Audio,
                MediaType::Video,
                MediaType::Document,
            ],
        };

        // Create the API client
        let api = MattermostApi::new(
            config.server_url.clone(),
            extract_token(&config.auth).unwrap_or_default(),
        )?;

        // Create the account with connections
        let account = MattermostAccount::new(account_id.to_string(), config.clone());
        let account_with_conn = MattermostAccountWithConnections::new(account, api);

        // Initialize adapters
        let accounts = vec![account_with_conn];
        let config_adapter = MattermostChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(MattermostSecurityAdapter::new(accounts.clone()));

        Ok(Self {
            accounts: accounts,
            id,
            meta,
            capabilities,
            shutdown_signal: None,
            config_adapter,
            security_adapter,
        })
    }

    /// Resolve a channel name to its ID.
    #[instrument(skip(self))]
    pub async fn resolve_channel_id(&mut self, team_id: &str, channel_name: &str) -> Result<String, ApiError> {
        if let Some(account) = self.accounts.first_mut() {
            {
                let cache = account.channel_cache.lock().await;
                if let Some(channel) = cache.get(channel_name) {
                    return Ok(channel.id.clone());
                }
            }

            // Fetch the channel
            let channel = account.api.get_channel_by_name(team_id, channel_name).await?;
            {
                let mut cache = account.channel_cache.lock().await;
                cache.insert(channel_name.to_string(), channel.clone());
            }

            Ok(channel.id)
        } else {
            Err(ApiError::ApiError("No accounts available".to_string()))
        }
    }

    /// Get the current user's ID.
    #[instrument(skip(self))]
    pub async fn get_current_user_id(&mut self) -> Result<String, ApiError> {
        if let Some(account) = self.accounts.first_mut() {
            if let Some(user_id) = &account.account.user_id {
                return Ok(user_id.clone());
            }

            let user = account.api.get_current_user().await?;
            let user_id = user.id.clone();
            {
                let mut cache = account.user_cache.lock().await;
                cache.insert(user_id.clone(), user);
            }
            account.account.user_id = Some(user_id.clone());

            Ok(user_id)
        } else {
            Err(ApiError::ApiError("No accounts available".to_string()))
        }
    }

    /// Start WebSocket connection for receiving events.
    #[instrument(skip(self))]
    pub async fn start_websocket(&mut self) -> Result<(), anyhow::Error> {
        if let Some(account) = self.accounts.first_mut() {
            if let Some(auth_result) = &account.account.auth_result {
                let server_url = account.account.config.server_url.clone();
                let websocket = MattermostWebSocket::connect(&server_url, &auth_result.token).await?;

                account.websocket = Some(Arc::new(AsyncMutex::new(websocket)));
                account.account.connected = true;
                account.account.last_connected = Some(Utc::now());

                info!("WebSocket connection started for account {}", account.account.id);

                // Start the event listener task
                let websocket_clone = account.websocket.clone().unwrap();
                let shutdown = self.shutdown_signal.clone();

                tokio::spawn(async move {
                    Self::listen_events(websocket_clone, shutdown).await;
                });
            }
        }

        Ok(())
    }

    /// Listen for WebSocket events and process them.
    #[instrument(skip(websocket, shutdown))]
    async fn listen_events(
        websocket: Arc<AsyncMutex<MattermostWebSocket>>,
        shutdown: Option<Arc<tokio::sync::Notify>>,
    ) {
        loop {
            // Check for shutdown signal
            if let Some(shutdown) = &shutdown {
                tokio::select! {
                    _ = shutdown.notified() => {
                        info!("Shutdown signal received, stopping event listener");
                        break;
                    }
                    event = async {
                        let mut ws = websocket.lock().await;
                        ws.next_event().await
                    } => {
                        match event {
                            Ok(event) => {
                                Self::handle_event(event).await;
                            }
                            Err(e) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                        }
                    }
                }
            } else {
                match websocket.lock().await.next_event().await {
                    Ok(event) => {
                        Self::handle_event(event).await;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    /// Handle a WebSocket event.
    #[instrument(skip(event))]
    async fn handle_event(event: MattermostEvent) {
        match event.event.as_str() {
            "posted" => {
                debug!("Received posted event: {:?}", event.data);
                // Process the posted message
            }
            "typing" => {
                debug!("Received typing event: {:?}", event.data);
            }
            "online_update" => {
                debug!("Received online update event: {:?}", event.data);
            }
            "status" => {
                debug!("Received status event: {:?}", event.data);
            }
            _ => {
                debug!("Received unknown event: {}", event.event);
            }
        }
    }
}

/// Channel configuration adapter for Mattermost.
#[derive(Clone)]
pub struct MattermostChannelConfigAdapter {
    accounts: Vec<MattermostAccountWithConnections>,
}

impl MattermostChannelConfigAdapter {
    /// Create a new configuration adapter.
    pub fn new(accounts: Vec<MattermostAccountWithConnections>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for MattermostChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>, anyhow::Error> {
        Ok(self.accounts.iter().map(|a| a.account.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot, anyhow::Error> {
        for account in &self.accounts {
            if account.account.id == id {
                return Ok(AccountSnapshot {
                    id: account.account.id.clone(),
                    channel: "mattermost".to_string(),
                    enabled: account.account.is_enabled(),
                    connected: account.account.connected,
                });
            }
        }
        Err(anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, _id: &str) -> Result<(), anyhow::Error> {
        // TODO: Implement account enabling
        Ok(())
    }

    fn disable_account(&self, _id: &str) -> Result<(), anyhow::Error> {
        // TODO: Implement account disabling
        Ok(())
    }

    fn delete_account(&self, _id: &str) -> Result<(), anyhow::Error> {
        // TODO: Implement account deletion
        Ok(())
    }
}

/// Security adapter for Mattermost.
#[derive(Clone)]
pub struct MattermostSecurityAdapter {
    accounts: Vec<MattermostAccountWithConnections>,
}

impl MattermostSecurityAdapter {
    /// Create a new security adapter.
    pub fn new(accounts: Vec<MattermostAccountWithConnections>) -> Self {
        Self { accounts }
    }

    /// Check if a sender is allowed.
    pub fn is_sender_allowed(&self, sender: &SenderInfo) -> bool {
        // For now, allow all senders
        // TODO: Implement sender allowlist
        let _ = sender;
        true
    }

    /// Check if mentions are required in groups.
    pub fn requires_mention_in_group(&self) -> bool {
        // Mattermost requires mentions in channels by default
        true
    }
}

#[async_trait]
impl SecurityAdapter for MattermostSecurityAdapter {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        self.is_sender_allowed(sender)
    }

    fn requires_mention_in_group(&self) -> bool {
        self.requires_mention_in_group()
    }
}

#[async_trait]
impl ChannelPlugin for MattermostChannel {
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

    async fn connect(&mut self) -> Result<(), anyhow::Error> {
        // Connect each account
        for account in &mut self.accounts {
            // Validate configuration
            if !account.account.config.validate().is_ok() {
                error!("Invalid configuration for account {}", account.account.id);
                continue;
            }

            // Handle authentication based on method
            let server_url = account.account.config.server_url.clone();
            let auth = account.account.config.auth.clone();

            if requires_login(&auth) {
                // For password authentication, perform login
                match authenticate(&auth, &server_url).await {
                    Ok(auth_result) => {
                        account.account.auth_result = Some(auth_result.clone());
                        account.account.username = auth_result.username.clone();
                        account.account.user_id = auth_result.user_id.clone();
                    }
                    Err(e) => {
                        error!("Failed to authenticate account {}: {}", account.account.id, e);
                        continue;
                    }
                }
            }

            // Set up the API client with proper token
            let token = extract_token(&auth).unwrap_or_default();
            account.api = MattermostApi::new(server_url.clone(), token)?;
        }

        // Start WebSocket connections
        self.start_websocket().await?;

        info!("Mattermost channel {} connected", self.id);
        Ok(())
    }

    async fn send(&self, msg: OutgoingMessage) -> Result<(), anyhow::Error> {
        // Extract target and content from the message
        let target = msg.target;
        let content = msg.content;

        // Get the channel ID based on target
        let channel_id = match target.peer.kind {
            PeerKind::Channel | PeerKind::Group => {
                // For channels and groups, use the channel field directly
                target.channel
            }
            PeerKind::User => {
                // For DMs, we need to create or get a direct channel
                // We need to use the current user ID which requires mutable access
                // For now, we'll use an Arc<Mutex> to handle this internally
                // Note: This is a limitation - ideally send would take &mut self
                let peer_id = target.peer.id.clone();
                // Get current user ID - need to clone the accounts to access
                let current_user_id = {
                    let mut accounts = self.accounts.clone();
                    if let Some(account) = accounts.first_mut() {
                        let mut cache = account.user_cache.lock().await;
                        if let Some(user_id) = &account.account.user_id {
                            cache.insert(user_id.clone(), User {
                                id: user_id.clone(),
                                username: account.account.username.clone().unwrap_or_default(),
                                display_name: account.account.username.clone().unwrap_or_default(),
                                email: String::new(),
                                is_bot: false,
                                delete_at: 0,
                            });
                            cache.insert(user_id.clone(), User {
                                id: user_id.clone(),
                                username: account.account.username.clone().unwrap_or_default(),
                                display_name: account.account.username.clone().unwrap_or_default(),
                                email: String::new(),
                                is_bot: false,
                                delete_at: 0,
                            });
                            user_id.clone()
                        } else {
                            // We can't actually get the user ID without mutable self
                            // This is a limitation of the current design
                            return Err(anyhow::anyhow!("Cannot get current user ID in send() - need &mut self"));
                        }
                    } else {
                        return Err(anyhow::anyhow!("No accounts available"));
                    }
                };
                format!("{}__{}", current_user_id, peer_id)
            }
            PeerKind::Thread => {
                // For threads, use the thread_id if provided, otherwise fall back to channel
                target.thread_id.unwrap_or(target.channel)
            }
        };

        // Build the message content
        let message = match content {
            MessageContent::Text(text) => text,
            MessageContent::Media(media) => {
                // For media messages, use a placeholder
                format!("[{:?}: {}]", media.media_type, media.url.as_deref().unwrap_or("unknown"))
            }
            MessageContent::Mixed(parts) => {
                // Convert multi-part message to plain text
                parts
                    .iter()
                    .filter_map(|part| match part {
                        MessagePart::Text(text) => Some(text.clone()),
                        MessagePart::Media(media) => {
                            // Include media as placeholder in the message
                            Some(format!("[{:?}: {}]", media.media_type, media.url.as_deref().unwrap_or("unknown")))
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        };

        // Get an account to send from
        if let Some(account) = self.accounts.first() {
            // Create the post
            account.api.create_post(&channel_id, &message).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No accounts available to send message"))
        }
    }

    async fn receive(&mut self) -> Result<IncomingMessage, anyhow::Error> {
        // This would normally block waiting for messages from WebSocket
        // For now, return an error since we need a proper message queue
        Err(anyhow::anyhow!(
            "Receive not yet implemented - use event listener instead"
        ))
    }

    async fn disconnect(&mut self) -> Result<(), anyhow::Error> {
        // Close WebSocket connections
        for account in &mut self.accounts {
            if let Some(websocket) = &account.websocket {
                let mut ws = websocket.lock().await;
                if let Err(e) = ws.close().await {
                    warn!("Failed to close WebSocket: {}", e);
                }
            }
            account.account.connected = false;
        }

        info!("Mattermost channel {} disconnected", self.id);
        Ok(())
    }
}

/// Register a Mattermost channel with the channel registry.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The Mattermost account configuration
/// * `account_id` - Unique identifier for this account instance
///
/// # Returns
///
/// * `Ok(())` - Registration successful
/// * `Err(anyhow::Error)` - An error if registration fails
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: MattermostConfig,
    account_id: &str,
) -> Result<(), anyhow::Error> {
    let channel = MattermostChannel::new(config, account_id).await?;
    registry.register(Arc::new(channel));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_channel() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string())
            .with_auth(MattermostAuth::BotToken {
                token: "test-token".to_string(),
            });

        let channel = MattermostChannel::new(config, "test").await;
        assert!(channel.is_ok());
    }

    #[tokio::test]
    async fn test_new_channel_validation() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string())
            .with_auth(MattermostAuth::BotToken {
                token: "".to_string(), // Empty token should fail validation
            });

        let channel = MattermostChannel::new(config, "test").await;
        assert!(channel.is_err());
    }

    #[tokio::test]
    async fn test_channel_id_format() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string())
            .with_auth(MattermostAuth::BotToken {
                token: "test-token".to_string(),
            });

        let channel = MattermostChannel::new(config, "main").await.unwrap();
        assert_eq!(channel.id(), "mattermost-main");
    }

    #[tokio::test]
    async fn test_channel_meta() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string())
            .with_auth(MattermostAuth::BotToken {
                token: "test-token".to_string(),
            });

        let channel = MattermostChannel::new(config, "test").await.unwrap();
        let meta = channel.meta();

        assert_eq!(meta.label, "Mattermost");
        assert!(meta.docs_url.is_some());
    }

    #[tokio::test]
    async fn test_channel_capabilities() {
        let config = MattermostConfig::new("https://mattermost.example.com".to_string())
            .with_auth(MattermostAuth::BotToken {
                token: "test-token".to_string(),
            });

        let channel = MattermostChannel::new(config, "test").await.unwrap();
        let caps = channel.capabilities();

        assert!(caps.chat_types.contains(&ChatType::Dm));
        assert!(caps.chat_types.contains(&ChatType::Channel));
        assert!(caps.supports_media);
        assert!(caps.supports_reactions);
        assert!(caps.supports_threads);
        assert!(caps.max_message_length.is_some());
    }
}
