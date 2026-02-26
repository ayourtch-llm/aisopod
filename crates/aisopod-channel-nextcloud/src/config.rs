//! Configuration for Nextcloud Talk channel.
//!
//! This module defines the configuration types for connecting to a Nextcloud
//! Talk instance and managing room subscriptions.

use serde::{Deserialize, Serialize};

/// Configuration for a Nextcloud Talk channel account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextcloudConfig {
    /// Nextcloud server URL (e.g., "https://cloud.example.com")
    pub server_url: String,
    /// Username
    pub username: String,
    /// App password or regular password
    pub password: String,
    /// Rooms to join (by room token)
    pub rooms: Vec<String>,
    /// Poll interval in seconds for new messages
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

fn default_poll_interval() -> u64 {
    10
}

impl Default for NextcloudConfig {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            username: String::new(),
            password: String::new(),
            rooms: Vec::new(),
            poll_interval_secs: default_poll_interval(),
        }
    }
}

/// A room in Nextcloud Talk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalkRoomInfo {
    /// The room token (unique identifier)
    pub token: String,
    /// The room name
    pub name: String,
    /// The room type
    #[serde(rename = "type")]
    pub room_type: i32,
}
