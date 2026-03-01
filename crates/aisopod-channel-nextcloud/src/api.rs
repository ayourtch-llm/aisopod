//! Nextcloud Talk API client.
//!
//! This module provides the API client for interacting with Nextcloud Talk's OCS API.
//! It handles authentication, message sending/receiving, and room management.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};

/// Nextcloud Talk API client.
///
/// This struct provides methods for interacting with Nextcloud Talk's
/// REST API via OCS endpoints. It handles authentication and provides
/// methods for room management and messaging.
#[derive(Clone, Debug)]
pub struct NextcloudTalkApi {
    base_url: String,
    auth: (String, String), // (username, password) for Basic auth
    http: reqwest::Client,
}

impl NextcloudTalkApi {
    /// Create a new Nextcloud Talk API client.
    ///
    /// # Arguments
    ///
    /// * `server_url` - The Nextcloud server URL (e.g., "https://cloud.example.com")
    /// * `username` - The username for authentication
    /// * `password` - The password or app password for authentication
    ///
    /// # Returns
    ///
    /// * `Ok(NextcloudTalkApi)` - The API client if successful
    /// * `Err(anyhow::Error)` - An error if the server URL is invalid
    pub fn new(server_url: &str, username: &str, password: &str) -> Result<Self> {
        // Ensure the URL doesn't have a trailing slash
        let base_url = server_url.trim_end_matches('/').to_string();

        Ok(Self {
            base_url,
            auth: (username.to_string(), password.to_string()),
            http: reqwest::Client::new(),
        })
    }

    /// Get the base URL of the Nextcloud instance.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the HTTP client.
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Get the authentication tuple.
    pub fn auth(&self) -> &(String, String) {
        &self.auth
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
    #[instrument(skip(self, message))]
    pub async fn send_message(&self, room_token: &str, message: &str) -> Result<()> {
        let url = format!(
            "{}/ocs/v2.php/apps/spreed/api/v1/chat/{}",
            self.base_url, room_token
        );

        debug!("Sending message to room {}: {}", room_token, message);

        let response = self
            .http
            .post(&url)
            .basic_auth(&self.auth.0, Some(&self.auth.1))
            .header("OCS-APIRequest", "true")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "message": message }))
            .send()
            .await?;

        if response.status().is_success() {
            debug!("Message sent successfully");
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Failed to send message: status={}, body={}", status, text);
            Err(anyhow!("Failed to send message: HTTP {}", status))
        }
    }

    /// Receive messages from a room.
    ///
    /// # Arguments
    ///
    /// * `room_token` - The room token to receive messages from
    /// * `last_known_id` - The last known message ID (0 for all messages)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<TalkMessage>)` - List of new messages
    /// * `Err(anyhow::Error)` - An error if receiving fails
    #[instrument(skip(self))]
    pub async fn receive_messages(
        &self,
        room_token: &str,
        last_known_id: i64,
    ) -> Result<Vec<TalkMessage>> {
        let url = format!(
            "{}/ocs/v2.php/apps/spreed/api/v1/chat/{}",
            self.base_url, room_token
        );

        debug!(
            "Receiving messages from room {}, last_known_id={}",
            room_token, last_known_id
        );

        let response = self
            .http
            .get(&url)
            .basic_auth(&self.auth.0, Some(&self.auth.1))
            .header("OCS-APIRequest", "true")
            .query(&[
                ("lookIntoFuture", "1"),
                ("lastKnownMessageId", &last_known_id.to_string()),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let ocs_response: OcsResponse<TalkMessages> = response.json().await?;
            debug!("Received {} messages", ocs_response.ocs.data.messages.len());
            Ok(ocs_response.ocs.data.messages)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!(
                "Failed to receive messages: status={}, body={}",
                status, text
            );
            Err(anyhow!("Failed to receive messages: HTTP {}", status))
        }
    }

    /// Get a list of rooms the user has access to.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<TalkRoom>)` - List of rooms
    /// * `Err(anyhow::Error)` - An error if fetching rooms fails
    #[instrument(skip(self))]
    pub async fn get_rooms(&self) -> Result<Vec<TalkRoom>> {
        let url = format!("{}/ocs/v2.php/apps/spreed/api/v4/room", self.base_url);

        debug!("Fetching rooms");

        let response = self
            .http
            .get(&url)
            .basic_auth(&self.auth.0, Some(&self.auth.1))
            .header("OCS-APIRequest", "true")
            .send()
            .await?;

        if response.status().is_success() {
            let ocs_response: OcsResponse<TalkRooms> = response.json().await?;
            debug!("Received {} rooms", ocs_response.ocs.data.rooms.len());
            Ok(ocs_response.ocs.data.rooms)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Failed to fetch rooms: status={}, body={}", status, text);
            Err(anyhow!("Failed to fetch rooms: HTTP {}", status))
        }
    }
}

/// OCS response envelope.
///
/// Nextcloud's OCS API wraps responses in an envelope with metadata.
#[derive(Debug, Deserialize)]
struct OcsResponse<T> {
    ocs: OcsEnvelope<T>,
}

#[derive(Debug, Deserialize)]
struct OcsEnvelope<T> {
    meta: OcsMeta,
    data: T,
}

#[derive(Debug, Deserialize)]
struct OcsMeta {
    status: String,
    #[serde(rename = "statusCode")]
    status_code: i32,
}

/// Messages response from the API.
#[derive(Debug, Deserialize)]
struct TalkMessages {
    messages: Vec<TalkMessage>,
}

/// Room information response from the API.
#[derive(Debug, Deserialize)]
struct TalkRooms {
    rooms: Vec<TalkRoom>,
}

/// A message in Nextcloud Talk.
#[derive(Debug, Deserialize, Clone)]
pub struct TalkMessage {
    /// Unique message ID
    pub id: i64,
    /// ID of the user who sent the message
    pub actor_id: String,
    /// Message content
    pub message: String,
    /// Timestamp when the message was sent (Unix epoch)
    pub timestamp: i64,
    /// Type of actor (user, guest, bot, etc.)
    #[serde(default)]
    pub actor_type: String,
    /// Display name of the actor
    #[serde(default)]
    pub actor_display_name: Option<String>,
    /// The room token this message belongs to
    #[serde(default)]
    pub chat_id: Option<String>,
}

/// A room in Nextcloud Talk.
#[derive(Debug, Deserialize, Clone)]
pub struct TalkRoom {
    /// Unique room token
    pub token: String,
    /// Room name
    pub name: String,
    /// Room display name
    #[serde(default)]
    pub display_name: Option<String>,
    /// Room type (1 = group chat, 2 = public chat)
    #[serde(rename = "type")]
    pub room_type: i32,
    /// Description of the room
    #[serde(default)]
    pub description: Option<String>,
    /// Whether the room is password protected
    #[serde(default)]
    pub password: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = crate::config::NextcloudConfig {
            server_url: "https://cloud.example.com".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            rooms: vec!["room1".to_string(), "room2".to_string()],
            poll_interval_secs: 30,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: crate::config::NextcloudConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.server_url, deserialized.server_url);
        assert_eq!(config.username, deserialized.username);
        assert_eq!(config.rooms.len(), deserialized.rooms.len());
    }

    #[test]
    fn test_api_client_creation() {
        let api = NextcloudTalkApi::new("https://cloud.example.com/", "user", "pass");
        assert!(api.is_ok());

        let api = api.unwrap();
        assert_eq!(api.base_url(), "https://cloud.example.com");
    }
}
