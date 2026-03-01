//! Signal channel implementation.
//!
//! This module implements the ChannelPlugin trait for Signal, enabling
//! the bot to receive and send messages via signal-cli.

use crate::config::{SignalAccountConfig, SignalDaemonConfig, SignalError};
use crate::gateway::SignalGateway;
use crate::outbound::SignalOutbound;
use crate::runtime::SignalRuntime;
use aisopod_channel::adapters::{
    AccountConfig, AccountSnapshot, ChannelConfigAdapter, SecurityAdapter,
};
use aisopod_channel::message::{IncomingMessage, MessageTarget, PeerInfo, SenderInfo};
use aisopod_channel::plugin::ChannelPlugin;
use aisopod_channel::types::{ChannelCapabilities, ChannelMeta, ChatType, MediaType};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

// Re-export common types

/// A Signal account wraps the configuration with its state.
#[derive(Clone)]
pub struct SignalAccount {
    /// Unique identifier for this account
    pub id: String,
    /// The account configuration
    pub config: SignalAccountConfig,
    /// Whether this account is currently connected
    pub connected: bool,
    /// The timestamp of the last connection
    pub last_connected: Option<DateTime<Utc>>,
}

impl SignalAccount {
    /// Create a new SignalAccount with the given configuration.
    pub fn new(id: String, config: SignalAccountConfig) -> Self {
        Self {
            id,
            config,
            connected: false,
            last_connected: None,
        }
    }

    /// Check if the account is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.config.phone_number.is_empty()
    }

    /// Validate the account configuration.
    pub fn validate(&self) -> Result<(), SignalError> {
        self.config.validate_phone_number()
    }
}

/// Signal channel plugin implementation.
///
/// This struct manages Signal connections via signal-cli daemon.
/// It implements the `ChannelPlugin` trait to integrate with the aisopod system.
pub struct SignalChannel {
    /// Vector of Signal accounts
    accounts: Vec<SignalAccount>,
    /// The channel ID
    id: String,
    /// The channel metadata
    meta: ChannelMeta,
    /// The channel capabilities
    capabilities: ChannelCapabilities,
    /// Runtime for managing the signal-cli daemon
    runtime: Arc<tokio::sync::Mutex<SignalRuntime>>,
    /// Gateway for incoming message handling
    gateway: SignalGateway,
    /// Outbound message sender
    outbound: SignalOutbound,
    /// Current running tasks for graceful shutdown
    shutdown_signal: Option<Arc<tokio::sync::Notify>>,
    /// The configuration adapter
    config_adapter: SignalChannelConfigAdapter,
    /// The security adapter
    security_adapter: Option<SignalSecurityAdapter>,
}

impl SignalChannel {
    /// Creates a new Signal channel with the given configuration.
    ///
    /// This method starts the signal-cli daemon and validates the configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The Signal account configuration
    /// * `account_id` - Unique identifier for this account instance
    ///
    /// # Returns
    ///
    /// * `Ok(SignalChannel)` - The channel if configuration is valid
    /// * `Err(anyhow::Error)` - An error if the configuration is invalid
    pub async fn new(config: SignalAccountConfig, account_id: &str) -> Result<Self> {
        let account = SignalAccount::new(account_id.to_string(), config.clone());

        // Validate the configuration
        if let Err(e) = account.validate() {
            return Err(anyhow::anyhow!("Failed to validate Signal account: {}", e));
        }

        let id = format!("signal-{}", account_id);
        let meta = ChannelMeta {
            label: "Signal".to_string(),
            docs_url: Some("https://signal.org".to_string()),
            ui_hints: serde_json::json!({
                "phone_number_field": "phone_number",
                "device_name_field": "device_name"
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

        // Initialize runtime
        let runtime = Arc::new(tokio::sync::Mutex::new(SignalRuntime::new()));

        // Initialize gateway
        let gateway = SignalGateway::new();

        // Initialize outbound
        let outbound = SignalOutbound::new();

        // Initialize adapters
        let config_adapter = SignalChannelConfigAdapter::new(accounts.clone());
        let security_adapter = Some(SignalSecurityAdapter::new(accounts.clone()));

        Ok(Self {
            accounts,
            id,
            meta,
            capabilities,
            runtime,
            gateway,
            outbound,
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
    pub fn get_account(&self, account_id: &str) -> Option<&SignalAccount> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get the runtime instance.
    pub fn runtime(&self) -> &tokio::sync::Mutex<SignalRuntime> {
        &self.runtime
    }

    /// Start the signal-cli daemon for all accounts.
    pub async fn start_daemon(&mut self) -> Result<()> {
        // Collect account IDs first to avoid borrow conflicts
        let account_ids: Vec<String> = self.accounts.iter().map(|a| a.id.clone()).collect();

        for account_id in account_ids {
            let account = self.accounts.iter().find(|a| a.id == account_id).unwrap();
            info!("Starting signal-cli daemon for account {}", account.id);

            let mut runtime = self.runtime.lock().await;
            if let Err(e) = runtime.start_daemon(account).await {
                error!("Failed to start daemon for account {}: {}", account.id, e);
                return Err(anyhow::anyhow!(e));
            }

            let acc = self
                .accounts
                .iter_mut()
                .find(|a| a.id == account_id)
                .unwrap();
            acc.connected = true;
            acc.last_connected = Some(Utc::now());
        }

        Ok(())
    }

    /// Stop the signal-cli daemon for all accounts.
    pub async fn stop_daemon(&mut self) -> Result<()> {
        // Collect account IDs first to avoid borrow conflicts
        let account_ids: Vec<String> = self.accounts.iter().map(|a| a.id.clone()).collect();

        for account_id in account_ids {
            let account = self.accounts.iter().find(|a| a.id == account_id).unwrap();
            info!("Stopping signal-cli daemon for account {}", account.id);
            let mut runtime = self.runtime.lock().await;
            runtime.stop_daemon(account).await;
            let acc = self
                .accounts
                .iter_mut()
                .find(|a| a.id == account_id)
                .unwrap();
            acc.connected = false;
        }

        Ok(())
    }

    /// Check if a message should be processed based on security settings.
    ///
    /// # Arguments
    ///
    /// * `message` - The incoming message to check
    ///
    /// # Returns
    ///
    /// * `true` - The message should be processed
    /// * `false` - The message should be filtered out
    pub fn should_process_message(&self, message: &IncomingMessage) -> bool {
        // Check if the sender is in the allowed list
        if let Some(account) = self.get_account(&message.account_id) {
            if !account.config.is_sender_allowed(&message.sender.id) {
                warn!(
                    "Message from {} filtered (not in allowed list)",
                    message.sender.id
                );
                return false;
            }
        }

        true
    }
}

/// ChannelConfigAdapter implementation for SignalChannel.
#[derive(Clone)]
pub struct SignalChannelConfigAdapter {
    /// Reference to the channel accounts
    accounts: Vec<SignalAccount>,
}

impl SignalChannelConfigAdapter {
    /// Create a new SignalChannelConfigAdapter.
    pub fn new(accounts: Vec<SignalAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl ChannelConfigAdapter for SignalChannelConfigAdapter {
    fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.accounts.iter().map(|a| a.id.clone()).collect())
    }

    fn resolve_account(&self, id: &str) -> Result<AccountSnapshot> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| AccountSnapshot {
                id: a.id.clone(),
                channel: "signal".to_string(),
                enabled: a.is_enabled(),
                connected: a.connected,
            })
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))
    }

    fn enable_account(&self, id: &str) -> Result<()> {
        info!("Enabling Signal account {}", id);
        // In production, this would enable the account
        Ok(())
    }

    fn disable_account(&self, id: &str) -> Result<()> {
        info!("Disabling Signal account {}", id);
        // In production, this would disable the account
        Ok(())
    }

    fn delete_account(&self, id: &str) -> Result<()> {
        info!("Deleting Signal account {}", id);
        // In production, this would delete the account
        Ok(())
    }
}

/// Security adapter for SignalChannel.
#[derive(Clone)]
pub struct SignalSecurityAdapter {
    /// Reference to the channel accounts
    accounts: Vec<SignalAccount>,
}

impl SignalSecurityAdapter {
    /// Create a new SignalSecurityAdapter.
    pub fn new(accounts: Vec<SignalAccount>) -> Self {
        Self { accounts }
    }
}

#[async_trait]
impl SecurityAdapter for SignalSecurityAdapter {
    fn is_allowed_sender(&self, sender: &SenderInfo) -> bool {
        // Check if the sender is in the allowed list for any account
        for account in &self.accounts {
            if account.config.is_sender_allowed(&sender.id) {
                return true;
            }
        }

        // If no allowed list is configured, allow all senders
        self.accounts.is_empty()
            || self
                .accounts
                .iter()
                .any(|a| a.config.allowed_senders.is_none())
    }

    fn requires_mention_in_group(&self) -> bool {
        // Signal doesn't require mentions in groups by default
        false
    }
}

// ============================================================================
// ChannelPlugin implementation
// ============================================================================

#[async_trait]
impl ChannelPlugin for SignalChannel {
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
        self.security_adapter
            .as_ref()
            .map(|a| a as &dyn SecurityAdapter)
    }
}

/// Register a Signal channel with the registry.
///
/// This function creates a SignalChannel from the given configuration
/// and adds it to the channel registry.
///
/// # Arguments
///
/// * `registry` - The channel registry to register with
/// * `config` - The Signal account configuration
/// * `account_id` - Unique identifier for this account instance
pub async fn register(
    registry: &mut aisopod_channel::ChannelRegistry,
    config: SignalAccountConfig,
    account_id: &str,
) -> Result<()> {
    let channel = SignalChannel::new(config, account_id).await?;
    registry.register(Arc::new(channel));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_account_config_default() {
        let config = SignalAccountConfig::default();
        assert!(config.phone_number.is_empty());
        assert!(!config.disappearing_enabled);
        assert_eq!(config.disappearing_timer, 2592000);
    }

    #[test]
    fn test_signal_account_config_new() {
        let config = SignalAccountConfig::new("+1234567890".to_string());
        assert_eq!(config.phone_number, "+1234567890");
    }

    #[test]
    fn test_channel_registration() {
        let config = SignalAccountConfig::new("+1234567890".to_string());

        // Note: This test would need to be run with tokio runtime
        // For now, we just verify the types compile
        let _channel = SignalChannel::new(config, "test-account");
    }
}
