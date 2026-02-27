//! LINE channel configuration types.
//!
//! This module defines the configuration structures for the LINE channel plugin,
//! including account configuration and webhook settings.

use serde::{Deserialize, Serialize};

/// Configuration for a LINE bot account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineAccountConfig {
    /// Channel access token (long-lived or stateless)
    pub channel_access_token: String,
    /// Channel secret (for webhook signature verification)
    pub channel_secret: String,
    /// Optional list of allowed user IDs (if empty, all users are allowed)
    pub allowed_users: Option<Vec<String>>,
    /// Optional list of allowed group IDs (if empty, all groups are allowed)
    pub allowed_groups: Option<Vec<String>>,
}

impl Default for LineAccountConfig {
    fn default() -> Self {
        Self {
            channel_access_token: String::new(),
            channel_secret: String::new(),
            allowed_users: None,
            allowed_groups: None,
        }
    }
}

impl LineAccountConfig {
    /// Create a new LineAccountConfig with the given token and secret.
    pub fn new(channel_access_token: String, channel_secret: String) -> Self {
        Self {
            channel_access_token,
            channel_secret,
            allowed_users: None,
            allowed_groups: None,
        }
    }

    /// Check if a sender ID is allowed based on the configuration.
    pub fn is_sender_allowed(&self, sender_id: &str) -> bool {
        if let Some(ref allowed_users) = self.allowed_users {
            allowed_users.iter().any(|id| id == sender_id)
        } else {
            true
        }
    }

    /// Check if a group ID is allowed based on the configuration.
    pub fn is_group_allowed(&self, group_id: &str) -> bool {
        if let Some(ref allowed_groups) = self.allowed_groups {
            allowed_groups.iter().any(|id| id == group_id)
        } else {
            true
        }
    }
}
