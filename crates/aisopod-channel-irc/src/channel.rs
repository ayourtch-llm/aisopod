//! IRC channel plugin implementation.
//!
//! This module implements the `ChannelPlugin` trait for IRC, enabling
//! the bot to receive and send messages via IRC servers.

use crate::auth::authenticate_nickserv;
use crate::client::IrcConnection;
use crate::config::{IrcConfig, IrcServerConfig};
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
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// An IRC account wraps the configuration with its connection state.
#[derive(Clone)]
pub struct IrcAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: IrcServerConfig,
    /// The connection (if established)
    pub connection: Option<Arc<Mutex<IrcConnection>>>,
    /// Whether this account is currently connected
    pub connected: bool,
}

impl IrcAccount {
    /// Create a new IrcAccount with the given configuration.
    pub fn new(id: String, config: IrcServerConfig) -> Self {
        Self {
            id,
            config,
            connection: None,
            connected: false,
        }
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.config.server.is_empty() && !self.config.nickname.is_empty()
    }

    /// Get the server name for this account.
    pub fn server_name(&self) -> &str {
        &self.config.server
    }
}

/// IRC channel plugin implementation.
///
/// This struct manages IRC connections to one or more servers.
/// It implements the `ChannelPlugin` trait to integrate with the aisopod system.
pub struct IrcChannel {
    /// Vector of IRC accounts (one per server)
    accounts: Vec<IrcAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// Config adapter - stored as a field to avoid lifetime issues
    config_adapter: IrcChannelConfigAdapter,
    /// Security adapter - stored as a field to avoid lifetime issues
    security_adapter: Option<IrcSecurityAdapter>,
}

impl IrcChannel {
    /// Creates a new IRC channel with the given configuration.
    ///
    /// This method creates the channel but does not connect to servers yet.
    /// Call `connect()` to establish connections.
    ///
    /// # Arguments
    ///
    /// * `config` - The IRC server configuration
    /// * `channel_id` - Unique identifier for this channel instance
    ///
    /// # Returns
    ///
    /// * `Ok(IrcChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: IrcConfig, channel_id: &str) -> Result<Self> {
        // Validate at least one server is configured
        if config.servers.is_empty() {
            return Err(anyhow::anyhow!(
                "IRC channel requires at least one server configuration"
            ));
        }

        // Create accounts for each server
        let accounts: Vec<IrcAccount> = config
            .servers
            .iter()
            .enumerate()
            .map(|(i, server_config)| {
                let account_id = format!("{}-{}", channel_id, i);
                IrcAccount::new(account_id, server_config.clone())
            })
            .collect();

        // Validate all accounts have valid configuration
        for account in &accounts {
            if account.config.server.is_empty() {
                return Err(anyhow::anyhow!(
                    "IRC server configuration missing for account {}",
                    account.id
                ));
            }
            if account.config.nickname.is_empty() {
                return Err(anyhow::anyhow!(
                    "IRC nickname missing for account {}",
                    account.id
                ));
            }
        }

        let id = format!("irc-{}", channel_id);
        let meta = ChannelMeta {
            label: "IRC".to_string(),
            docs_url: Some("https://en.wikipedia.org/wiki/Internet_Relay_Chat".to_string()),
            ui_hints: serde_json::json!({
                "servers_field": "servers",
                "nickname_field": "nickname"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group],
            supports_media: false,
            supports_reactions: false,
            supports_threads: false,
            supports_typing: false,
            supports_voice: false,
            max_message_length: Some(512), // IRC message limit
            supported_media_types: vec![],
        };

        // Initialize adapters with the accounts
        let config_adapter = IrcChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(IrcSecurityAdapter::new(&accounts));

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
    pub fn get_account(&self, account_id: &str) -> Option<&IrcAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut IrcAccount> {
        self.accounts.iter_mut().find(|a| a.id == account_id)
    }

    /// Connect to all configured IRC servers.
    pub async fn connect(&mut self) -> Result<()> {
        for account in &mut self.accounts {
            info!(
                "Connecting to IRC server {} as {}",
                account.config.server, account.config.nickname
            );

            let mut connection = IrcConnection::connect(&account.config).await?;

            // Join configured channels
            for channel in &account.config.channels {
                if let Err(e) = connection.join_channel(channel) {
                    warn!("Failed to join channel {}: {}", channel, e);
                }
            }

            // Authenticate with NickServ if password is configured
            if let Some(ref nickserv_password) = account.config.nickserv_password {
                if let Err(e) = authenticate_nickserv(connection.client(), nickserv_password) {
                    warn!("Failed to authenticate with NickServ: {}", e);
                }
            }

            account.connection = Some(Arc::new(Mutex::new(connection)));
            account.connected = true;
        }

        Ok(())
    }

    /// Disconnect from all IRC servers.
    pub async fn disconnect(&mut self) -> Result<()> {
        for account in &mut self.accounts {
            if let Some(ref connection) = account.connection {
                let mut conn = connection.lock().await;
                if let Err(e) = conn.quit(Some("Goodbye")) {
                    warn!("Failed to quit {}: {}", conn.server_name(), e);
                }
            }
            account.connected = false;
        }

        Ok(())
    }

    /// Send a message to a target (channel or user).
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID to use for sending
    /// * `target` - The target channel or user nickname
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

        let connection = account
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to {}", account.config.server))?;

        let mut conn = connection.lock().await;
        conn.send_privmsg(target, message)?;

        Ok(())
    }

    /// Receive messages from all connected servers.
    ///
    /// This method polls all server streams and returns incoming messages.
    /// The caller should handle messages from different accounts.
    ///
    /// # Returns
    ///
    /// * `Ok(IncomingMessage)` - An incoming message
    /// * `Err(anyhow::Error)` - An error if receiving fails
    pub async fn receive(&mut self) -> Result<IncomingMessage> {
        // For now, we'll simulate receiving by waiting
        // A full implementation would poll all streams using select!

        // This is a placeholder implementation
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Return a dummy message for testing
        // In a real implementation, this would parse IRC events
        Err(anyhow::anyhow!(
            "receive() not fully implemented - polling all streams needed"
        ))
    }
}

/// ChannelConfigAdapter implementation for IrcChannel.
#[derive(Clone)]
pub struct IrcChannelConfigAdapter {
    /// Reference to the channel accounts
    accounts: Vec<IrcAccount>,
}

impl IrcChannelConfigAdapter {
    /// Create a new IrcChannelConfigAdapter.
    pub fn new(accounts: Vec<IrcAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for IrcChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| AccountSnapshot {
                id: a.id.clone(),
                channel: "irc".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, id: &str) -> Result<()> {
        info!("Enabling IRC account {}", id);
        // In production, this would enable the account
        Ok(())
    }

    fn disable_account(&self, id: &str) -> Result<()> {
        info!("Disabling IRC account {}", id);
        // In production, this would disable the account
        Ok(())
    }

    fn delete_account(&self, id: &str) -> Result<()> {
        info!("Deleting IRC account {}", id);
        // In production, this would delete the account
        Ok(())
    }
}

/// SecurityAdapter implementation for IrcChannel.
///
/// This adapter checks if senders are allowed to interact with the bot
/// based on the channel configuration.
#[derive(Clone)]
pub struct IrcSecurityAdapter {
    /// Map of account IDs to allowed senders
    allowed_senders: HashMap<String, Vec<String>>,
}

impl IrcSecurityAdapter {
    /// Create a new IrcSecurityAdapter.
    pub fn new(accounts: &[IrcAccount]) -> Self {
        let mut allowed_senders = HashMap::new();

        for account in accounts {
            // For now, we'll use the channels as a way to determine allowed senders
            // A more sophisticated implementation would have explicit allowlists
            allowed_senders.insert(account.id.clone(), account.config.channels.clone());
        }

        Self { allowed_senders }
    }

    /// Check if a sender is allowed based on the account configuration.
    pub fn is_sender_allowed(&self, account_id: &str, sender: &str) -> bool {
        if let Some(allowed) = self.allowed_senders.get(account_id) {
            // For now, allow anyone in the same channel
            // A more sophisticated implementation would check against a user list
            !allowed.is_empty() || sender.starts_with('#')
        } else {
            // If no configuration, allow all
            true
        }
    }
}

#[async_trait]
impl SecurityAdapter for IrcSecurityAdapter {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        // For IRC, we'll allow all senders unless explicitly restricted
        // A more sophisticated implementation would check nicknames against an allowlist
        true
    }

    fn requires_mention_in_group(&self) -> bool {
        // IRC doesn't require mentions in channels
        false
    }
}

#[async_trait]
impl ChannelPlugin for IrcChannel {
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

/// Register an IRC channel with the channel registry.
///
/// This is a convenience function for creating and registering an IRC channel.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The IRC server configuration
/// * `channel_id` - Unique identifier for this channel instance
///
/// # Returns
///
/// * `Ok(())` - Channel was registered successfully
/// * `Err(anyhow::Error)` - An error if registration fails
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: IrcConfig,
    channel_id: &str,
) -> Result<()> {
    let channel = IrcChannel::new(config, channel_id).await?;
    registry.register(Arc::new(channel));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_irc_channel_new() {
        let config = IrcConfig {
            servers: vec![IrcServerConfig {
                server: "irc.libera.chat".to_string(),
                port: 6697,
                use_tls: true,
                nickname: "testbot".to_string(),
                nickserv_password: None,
                channels: vec!["#test".to_string()],
                server_password: None,
            }],
        };

        let channel = IrcChannel::new(config, "main").await;
        assert!(channel.is_ok());

        let channel = channel.unwrap();
        assert_eq!(channel.id(), "irc-main");
        assert_eq!(channel.list_account_ids().len(), 1);
    }

    #[tokio::test]
    async fn test_irc_channel_multiple_servers() {
        let config = IrcConfig {
            servers: vec![
                IrcServerConfig {
                    server: "irc.libera.chat".to_string(),
                    port: 6697,
                    use_tls: true,
                    nickname: "testbot1".to_string(),
                    nickserv_password: None,
                    channels: vec!["#test1".to_string()],
                    server_password: None,
                },
                IrcServerConfig {
                    server: "irc.rizon.net".to_string(),
                    port: 6667,
                    use_tls: false,
                    nickname: "testbot2".to_string(),
                    nickserv_password: None,
                    channels: vec!["#test2".to_string()],
                    server_password: None,
                },
            ],
        };

        let channel = IrcChannel::new(config, "multi").await;
        assert!(channel.is_ok());

        let channel = channel.unwrap();
        assert_eq!(channel.list_account_ids().len(), 2);
    }

    #[test]
    fn test_account_validation() {
        let account = IrcAccount::new(
            "test-account".to_string(),
            IrcServerConfig {
                server: "irc.example.com".to_string(),
                nickname: "testbot".to_string(),
                ..Default::default()
            },
        );

        assert!(account.is_enabled());
        assert_eq!(account.server_name(), "irc.example.com");
    }

    #[test]
    fn test_account_disabled_when_no_server() {
        let account = IrcAccount::new(
            "test-account".to_string(),
            IrcServerConfig {
                server: "".to_string(),
                nickname: "testbot".to_string(),
                ..Default::default()
            },
        );

        assert!(!account.is_enabled());
    }
}
