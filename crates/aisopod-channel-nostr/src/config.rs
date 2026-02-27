//! Nostr channel configuration.
//!
//! This module defines the configuration structure for Nostr channel accounts.

use serde::Deserialize;

/// Configuration for a Nostr channel account.
#[derive(Debug, Deserialize, Clone)]
pub struct NostrConfig {
    /// Private key in nsec or hex format
    pub private_key: String,
    /// Relay URLs to connect to
    pub relays: Vec<String>,
    /// Enable NIP-04 encrypted DMs
    #[serde(default = "default_true")]
    pub enable_dms: bool,
    /// Public channels to follow (by event ID or pubkey)
    #[serde(default)]
    pub channels: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl NostrConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.private_key.is_empty() {
            return Err(anyhow::anyhow!("Private key cannot be empty"));
        }
        if self.relays.is_empty() {
            return Err(anyhow::anyhow!("At least one relay is required"));
        }
        for relay in &self.relays {
            if !relay.starts_with("ws://") && !relay.starts_with("wss://") {
                return Err(anyhow::anyhow!(
                    "Relay URL must start with ws:// or wss://: {}",
                    relay
                ));
            }
        }
        Ok(())
    }
}
