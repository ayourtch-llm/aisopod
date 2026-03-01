//! Message polling for Nextcloud Talk.
//!
//! This module provides the message poller that continuously monitors
//! Nextcloud Talk rooms for new messages.

use std::collections::HashMap;
use std::time::Duration;

use crate::api::{NextcloudTalkApi, TalkMessage};
use anyhow::Result;
use tracing::{debug, error, info, instrument, warn};

/// Message poller for Nextcloud Talk rooms.
///
/// This struct handles polling multiple rooms for new messages,
/// tracking the last seen message ID for each room, and returning
/// new messages as they arrive.
pub struct MessagePoller {
    /// The API client for making requests
    api: NextcloudTalkApi,
    /// List of room tokens to poll
    rooms: Vec<String>,
    /// Map of room token to last known message ID
    last_known_ids: HashMap<String, i64>,
    /// Poll interval duration
    poll_interval: Duration,
}

impl MessagePoller {
    /// Create a new message poller.
    ///
    /// # Arguments
    ///
    /// * `api` - The Nextcloud Talk API client
    /// * `rooms` - List of room tokens to poll
    /// * `poll_interval_secs` - Poll interval in seconds
    pub fn new(api: NextcloudTalkApi, rooms: Vec<String>, poll_interval_secs: u64) -> Self {
        let last_known_ids = rooms.iter().map(|r| (r.clone(), 0)).collect();

        Self {
            api,
            rooms,
            last_known_ids,
            poll_interval: Duration::from_secs(poll_interval_secs),
        }
    }

    /// Get the current poll interval.
    pub fn poll_interval(&self) -> Duration {
        self.poll_interval.clone()
    }

    /// Poll all rooms for new messages once.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(String, TalkMessage)>)` - List of new messages with their room tokens
    /// * `Err(anyhow::Error)` - An error if polling fails
    #[instrument(skip(self))]
    pub async fn poll_once(&mut self) -> Result<Vec<(String, TalkMessage)>> {
        let mut new_messages = Vec::new();

        for room in &self.rooms {
            let last_id = *self.last_known_ids.get(room).unwrap_or(&0);

            debug!("Polling room {} for messages after {}", room, last_id);

            match self.api.receive_messages(room, last_id).await {
                Ok(messages) => {
                    if !messages.is_empty() {
                        debug!("Found {} new messages in room {}", messages.len(), room);

                        for msg in &messages {
                            // Update the last known ID
                            if msg.id > self.last_known_ids.get(room).copied().unwrap_or(0) {
                                self.last_known_ids.insert(room.clone(), msg.id);
                            }
                            new_messages.push((room.clone(), msg.clone()));
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to poll room {}: {}", room, e);
                    // Continue polling other rooms even if one fails
                }
            }
        }

        Ok(new_messages)
    }

    /// Get the last known message ID for a room.
    pub fn get_last_known_id(&self, room: &str) -> Option<i64> {
        self.last_known_ids.get(room).copied()
    }

    /// Set the last known message ID for a room.
    pub fn set_last_known_id(&mut self, room: &str, id: i64) {
        self.last_known_ids.insert(room.to_string(), id);
    }

    /// Get the list of rooms being polled.
    pub fn rooms(&self) -> &[String] {
        &self.rooms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::NextcloudTalkApi;

    #[test]
    fn test_poller_creation() {
        // This test just validates the API - actual API calls would require a server
        let config = crate::config::NextcloudConfig::default();
        let api = NextcloudTalkApi::new(&config.server_url, &config.username, &config.password);

        if api.is_ok() {
            let poller = MessagePoller::new(api.unwrap(), vec!["room1".to_string()], 10);
            assert_eq!(poller.rooms(), &["room1".to_string()]);
            assert_eq!(poller.poll_interval().as_secs(), 10);
        }
    }

    #[test]
    fn test_multiple_rooms() {
        let config = crate::config::NextcloudConfig::default();
        let api = NextcloudTalkApi::new(&config.server_url, &config.username, &config.password);

        if api.is_ok() {
            let rooms = vec![
                "room1".to_string(),
                "room2".to_string(),
                "room3".to_string(),
            ];
            let poller = MessagePoller::new(api.unwrap(), rooms.clone(), 15);

            assert_eq!(poller.rooms().len(), 3);
            assert_eq!(poller.poll_interval().as_secs(), 15);
        }
    }
}
