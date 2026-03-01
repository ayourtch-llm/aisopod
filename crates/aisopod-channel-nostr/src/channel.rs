//! Nostr channel plugin implementation.
//!
//! This module implements the `ChannelPlugin` trait for Nostr, enabling
//! the bot to receive and send messages via Nostr relays.

use crate::config::NostrConfig;
use crate::events::NostrEvent;
use crate::keys::NostrKeys;
use crate::nip04;
use crate::relay::RelayPool;
use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, ChannelConfigAdapter, SecurityAdapter,
};
use aisopod_channel::message::{
    IncomingMessage, MessageContent, MessagePart, MessageTarget, PeerInfo, PeerKind, SenderInfo,
};
use aisopod_channel::plugin::ChannelPlugin;
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// A Nostr account wraps the configuration with its connection state.
#[derive(Clone)]
pub struct NostrAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: NostrConfig,
    /// The keys for this account
    pub keys: NostrKeys,
    /// The relay pool (if connected)
    pub relay_pool: Option<Arc<Mutex<RelayPool>>>,
    /// Whether this account is currently connected
    pub connected: bool,
}

impl NostrAccount {
    /// Create a new NostrAccount with the given configuration.
    pub fn new(id: String, config: NostrConfig) -> Result<Self> {
        // Parse the private key (supports nsec or hex format)
        let keys = if config.private_key.starts_with("nsec1") {
            NostrKeys::from_nsec(&config.private_key)
                .map_err(|e| anyhow!("Invalid nsec key: {}", e))?
        } else {
            NostrKeys::from_hex(&config.private_key)
                .map_err(|e| anyhow!("Invalid hex key: {}", e))?
        };

        Ok(Self {
            id,
            config,
            keys,
            relay_pool: None,
            connected: false,
        })
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.config.private_key.is_empty() && !self.config.relays.is_empty()
    }

    /// Get the public key as npub format.
    pub fn npub(&self) -> String {
        self.keys.npub()
    }
}

/// Nostr channel plugin implementation.
///
/// This struct manages Nostr relay connections and message sending/receiving.
/// It implements the `ChannelPlugin` trait to integrate with the aisopod system.
pub struct NostrChannel {
    /// Vector of Nostr accounts (one per configured key)
    accounts: Vec<NostrAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// Config adapter - stored as a field to avoid lifetime issues
    config_adapter: NostrChannelConfigAdapter,
    /// Security adapter - stored as a field to avoid lifetime issues
    security_adapter: Option<NostrSecurityAdapter>,
}

impl NostrChannel {
    /// Creates a new Nostr channel with the given configuration.
    ///
    /// This method creates the channel but does not connect to relays yet.
    /// Call `connect()` to establish connections.
    ///
    /// # Arguments
    ///
    /// * `config` - The Nostr configuration
    /// * `channel_id` - Unique identifier for this channel instance
    ///
    /// # Returns
    ///
    /// * `Ok(NostrChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: NostrConfig, channel_id: &str) -> Result<Self> {
        // Validate the configuration
        config.validate()?;

        // Create account
        let account_id = format!("{}-0", channel_id);
        let account = NostrAccount::new(account_id, config)?;

        let id = format!("nostr-{}", channel_id);
        let meta = ChannelMeta {
            label: "Nostr".to_string(),
            docs_url: Some("https://github.com/nostr-protocol/nostr".to_string()),
            ui_hints: serde_json::json!({
                "private_key_field": "private_key",
                "relays_field": "relays"
            }),
        };
        let capabilities = ChannelCapabilities {
            chat_types: vec![ChatType::Dm, ChatType::Group],
            supports_media: false,
            supports_reactions: false,
            supports_threads: false,
            supports_typing: false,
            supports_voice: false,
            max_message_length: Some(1024), // Nostr event size limit
            supported_media_types: vec![],
        };

        // Initialize adapters
        let config_adapter = NostrChannelConfigAdapter::new(vec![account.clone()]);
        let security_adapter = Some(NostrSecurityAdapter::new());

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

    /// Get all configured account IDs for this channel.
    pub fn list_account_ids(&self) -> Vec<String> {
        self.accounts.iter().map(|a| a.id.clone()).collect()
    }

    /// Get an account by its ID.
    pub fn get_account(&self, account_id: &str) -> Option<&NostrAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get an account by its ID (mutable).
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut NostrAccount> {
        self.accounts.iter_mut().find(|a| a.id == account_id)
    }

    /// Connect to all configured Nostr relays.
    pub async fn connect(&mut self) -> Result<()> {
        for account in &mut self.accounts {
            info!(
                "Connecting to Nostr relays for account {} ({})",
                account.id,
                account.npub()
            );

            // Create and connect relay pool
            let pool = RelayPool::connect(&account.config.relays)
                .await
                .map_err(|e| anyhow!("Failed to connect to relays: {}", e))?;

            let pool = Arc::new(Mutex::new(pool));
            account.relay_pool = Some(pool.clone());
            account.connected = true;

            // Subscribe to events from all relays
            // For now, we'll use a simple filter for all events
            // A more sophisticated implementation would filter by public key or event kind
            let filters = vec![serde_json::json!({
                "kinds": [1, 4],  // text notes and encrypted DMs
            })];

            {
                let mut pool_mut = pool.lock().await;
                if let Err(e) = pool_mut.subscribe(filters).await {
                    warn!("Failed to subscribe on relay: {}", e);
                }
            }

            info!(
                "Connected to {} relays for account {}",
                account.config.relays.len(),
                account.id
            );
        }

        Ok(())
    }

    /// Disconnect from all Nostr relays.
    pub async fn disconnect(&mut self) -> Result<()> {
        for account in &mut self.accounts {
            if let Some(ref pool) = account.relay_pool {
                let mut pool_mut = pool.lock().await;
                pool_mut.disconnect().await;
            }
            account.connected = false;
        }

        Ok(())
    }

    /// Send a text note to the configured relays.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID to use for sending
    /// * `content` - The message content to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_text_note(&self, account_id: &str, content: &str) -> Result<()> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", account_id))?;

        if !account.connected {
            return Err(anyhow::anyhow!("Account {} is not connected", account_id));
        }

        // Create a text note event (kind 1)
        let event = NostrEvent::new_text_note(&account.keys, content)
            .map_err(|e| anyhow!("Failed to create event: {}", e))?;

        // Publish to all relays
        if let Some(ref pool) = account.relay_pool {
            let mut pool_mut = pool.lock().await;
            pool_mut.publish(&event).await?;
        }

        info!(
            "Published text note from {} to {} relays",
            account.npub(),
            account.config.relays.len()
        );

        Ok(())
    }

    /// Send an encrypted DM to a recipient.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID to use for sending
    /// * `recipient_pubkey` - The recipient's public key (hex or npub format)
    /// * `plaintext` - The plaintext message to encrypt
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_dm(
        &self,
        account_id: &str,
        recipient_pubkey: &str,
        plaintext: &str,
    ) -> Result<()> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", account_id))?;

        if !account.connected {
            return Err(anyhow::anyhow!("Account {} is not connected", account_id));
        }

        if !account.config.enable_dms {
            return Err(anyhow::anyhow!(
                "Encrypted DMs are disabled for account {}",
                account_id
            ));
        }

        // Convert recipient pubkey to hex format
        let recipient_pubkey_hex = if recipient_pubkey.starts_with("npub1") {
            // For now, we'll need to convert npub to hex
            // This would require a pubkey parsing function
            return Err(anyhow::anyhow!(
                "Converting npub to hex not yet implemented. Use hex format for recipient_pubkey."
            ));
        } else {
            recipient_pubkey.to_string()
        };

        // Create an encrypted DM event (kind 4)
        let event = NostrEvent::new_dm(&account.keys, &recipient_pubkey_hex, plaintext)
            .map_err(|e| anyhow!("Failed to create DM event: {}", e))?;

        // Publish to all relays
        if let Some(ref pool) = account.relay_pool {
            let mut pool_mut = pool.lock().await;
            pool_mut.publish(&event).await?;
        }

        info!(
            "Published encrypted DM from {} to {}",
            account.npub(),
            recipient_pubkey
        );

        Ok(())
    }

    /// Receive messages from all connected relays.
    ///
    /// This method polls all relay streams and returns incoming messages.
    /// The caller should handle messages from different accounts.
    ///
    /// # Returns
    ///
    /// * `Ok(IncomingMessage)` - An incoming message
    /// * `Err(anyhow::Error)` - An error if receiving fails
    pub async fn receive(&mut self) -> Result<IncomingMessage> {
        // For now, we'll simulate receiving by waiting
        // A full implementation would poll all streams using select!

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Return a dummy message for testing
        // In a real implementation, this would parse Nostr events
        Err(anyhow::anyhow!(
            "receive() not fully implemented - polling all streams needed"
        ))
    }
}

/// ChannelConfigAdapter implementation for NostrChannel.
#[derive(Clone)]
pub struct NostrChannelConfigAdapter {
    /// Reference to the channel accounts
    accounts: Vec<NostrAccount>,
}

impl NostrChannelConfigAdapter {
    /// Create a new NostrChannelConfigAdapter.
    pub fn new(accounts: Vec<NostrAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for NostrChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| AccountSnapshot {
                id: a.id.clone(),
                channel: "nostr".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, id: &str) -> Result<()> {
        info!("Enabling Nostr account {}", id);
        // In production, this would enable the account
        Ok(())
    }

    fn disable_account(&self, id: &str) -> Result<()> {
        info!("Disabling Nostr account {}", id);
        // In production, this would disable the account
        Ok(())
    }

    fn delete_account(&self, id: &str) -> Result<()> {
        info!("Deleting Nostr account {}", id);
        // In production, this would delete the account
        Ok(())
    }
}

/// SecurityAdapter implementation for NostrChannel.
///
/// This adapter checks if senders are allowed to interact with the bot
/// based on the channel configuration.
#[derive(Clone)]
pub struct NostrSecurityAdapter {}

impl NostrSecurityAdapter {
    /// Create a new NostrSecurityAdapter.
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SecurityAdapter for NostrSecurityAdapter {
    fn is_allowed_sender(&self, _sender: &SenderInfo) -> bool {
        // For Nostr, we'll allow all senders
        // A more sophisticated implementation would check against a whitelist
        true
    }

    fn requires_mention_in_group(&self) -> bool {
        // Nostr doesn't require mentions
        false
    }
}

#[async_trait]
impl ChannelPlugin for NostrChannel {
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

/// Register a Nostr channel with the channel registry.
///
/// This is a convenience function for creating and registering a Nostr channel.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The Nostr configuration
/// * `channel_id` - Unique identifier for this channel instance
///
/// # Returns
///
/// * `Ok(())` - Channel was registered successfully
/// * `Err(anyhow::Error)` - An error if registration fails
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: NostrConfig,
    channel_id: &str,
) -> Result<()> {
    let channel = NostrChannel::new(config, channel_id).await?;
    registry.register(Arc::new(channel));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nostr_channel_new() {
        let config = NostrConfig {
            private_key: "0000000000000000000000000000000000000000000000000000000000000001"
                .to_string(),
            relays: vec!["wss://relay.example.com".to_string()],
            enable_dms: true,
            channels: vec![],
        };

        let channel = NostrChannel::new(config, "main").await;
        assert!(channel.is_ok());

        let channel = channel.unwrap();
        assert_eq!(channel.id(), "nostr-main");
        assert_eq!(channel.list_account_ids().len(), 1);
    }

    #[tokio::test]
    async fn test_nostr_channel_multiple_relays() {
        let config = NostrConfig {
            private_key: "0000000000000000000000000000000000000000000000000000000000000001"
                .to_string(),
            relays: vec![
                "wss://relay.example.com".to_string(),
                "wss://relay2.example.com".to_string(),
            ],
            enable_dms: true,
            channels: vec![],
        };

        let channel = NostrChannel::new(config, "multi").await;
        assert!(channel.is_ok());
    }

    #[test]
    fn test_account_validation() {
        let account = NostrAccount::new(
            "test-account".to_string(),
            NostrConfig {
                private_key: "0000000000000000000000000000000000000000000000000000000000000001"
                    .to_string(),
                relays: vec!["wss://relay.example.com".to_string()],
                enable_dms: true,
                channels: vec![],
            },
        );

        assert!(account.is_ok());
        let account = account.unwrap();
        assert!(account.is_enabled());
    }

    #[test]
    fn test_account_disabled_when_no_key() {
        let result = NostrAccount::new(
            "test-account".to_string(),
            NostrConfig {
                private_key: "".to_string(),
                relays: vec!["wss://relay.example.com".to_string()],
                enable_dms: true,
                channels: vec![],
            },
        );

        assert!(result.is_err());
    }
}
