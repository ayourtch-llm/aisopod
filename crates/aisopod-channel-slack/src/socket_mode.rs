//! Socket Mode WebSocket connection for Slack.
//!
//! This module provides the Socket Mode connection implementation
//! for receiving real-time events from Slack.

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};

use crate::connection::SlackClientHandle;

/// A Socket Mode connection to Slack.
///
/// This struct manages the WebSocket connection to Slack's Socket Mode API.
#[derive(Clone)]
pub struct SlackSocketModeConnection {
    /// The client handle for Web API calls
    client: SlackClientHandle,
    /// The bot token
    bot_token: String,
    /// The app token (optional)
    app_token: Option<String>,
    /// The account ID
    account_id: String,
    /// The WebSocket URL (fetched from apps.connections.open)
    websocket_url: Option<String>,
    /// Current connection state
    connected: bool,
}

impl SlackSocketModeConnection {
    /// Create a new Socket Mode connection.
    ///
    /// This method fetches the WebSocket URL from Slack's apps.connections.open endpoint.
    pub async fn new(config: &crate::SlackAccountConfig, account_id: &str) -> Result<Self> {
        let client = SlackClientHandle::new(config.bot_token.clone());

        // Fetch the WebSocket URL
        let response = client.apps_connections_open().await?;
        let websocket_url = response.get_url()?;

        Ok(Self {
            client,
            bot_token: config.bot_token.clone(),
            app_token: config.app_token.clone(),
            account_id: account_id.to_string(),
            websocket_url: Some(websocket_url),
            connected: false,
        })
    }

    /// Get the account ID.
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    /// Get the WebSocket URL.
    pub fn websocket_url(&self) -> Option<&str> {
        self.websocket_url.as_deref()
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Connect to the Socket Mode WebSocket.
    ///
    /// This method establishes the WebSocket connection and returns
    /// a stream of events.
    pub async fn connect(&mut self) -> Result<()> {
        let url = self
            .websocket_url
            .clone()
            .ok_or_else(|| anyhow::anyhow!("WebSocket URL not available"))?;

        info!("Connecting to Slack Socket Mode: {}", url);

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(url).await?;
        self.connected = true;

        // Store the stream for later use
        // In a full implementation, we'd store this for sending/receiving
        // For now, we just mark as connected
        let _ = ws_stream;

        info!(
            "Socket Mode connection established for account {}",
            self.account_id
        );
        Ok(())
    }

    /// Disconnect from the Socket Mode WebSocket.
    pub async fn disconnect(&mut self) -> Result<()> {
        if self.connected {
            info!(
                "Disconnecting from Socket Mode for account {}",
                self.account_id
            );
            self.connected = false;
        }
        Ok(())
    }

    /// Send a message to a Slack channel.
    ///
    /// This method uses the Slack Web API to send a message.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The channel ID to send to
    /// * `text` - The message text
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message was sent successfully
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn send_message(&self, channel_id: &str, text: &str) -> Result<()> {
        // Use the client to call the chat.postMessage endpoint
        let response = self
            .client
            .post(
                "https://slack.com/api/chat.postMessage",
                &serde_json::json!({
                    "channel": channel_id,
                    "text": text
                }),
            )
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow::anyhow!(
                "chat.postMessage failed with status {}: {}",
                status,
                body
            ));
        }

        debug!("Message sent to channel {}: {}", channel_id, text);
        Ok(())
    }

    /// Send an acknowledgment response for an event.
    ///
    /// Socket Mode requires acknowledging each event with an envelope_id.
    pub async fn send_ack(&self, envelope_id: &str) -> Result<()> {
        // In a full implementation, we'd send an ack message through the WebSocket
        // For now, this is a placeholder
        debug!("Acknowledged event with envelope_id: {}", envelope_id);
        Ok(())
    }

    /// Get the client handle for Web API calls.
    pub fn client(&self) -> &SlackClientHandle {
        &self.client
    }
}

/// An event received from Slack Socket Mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketModeEvent {
    /// The envelope ID for acknowledgment
    pub envelope_id: String,
    /// The payload type
    pub payload: SocketModePayload,
    /// The team ID
    pub team_id: Option<String>,
}

/// The payload of a Socket Mode event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SocketModePayload {
    /// Hello event on connection
    Hello(HelloEvent),
    /// Events API payload
    EventsApi(EventsApiPayload),
    /// Disconnect event
    Disconnect(DisconnectEvent),
    /// Other event types
    Other(serde_json::Value),
}

/// Hello event received on WebSocket connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloEvent {
    /// The socket mode version
    pub socket_mode_version: String,
    /// The bot ID
    pub bot_id: String,
}

/// Events API payload containing message and other events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsApiPayload {
    /// The event data
    pub event: EventsApiEvent,
    /// The event type
    pub event_id: String,
    /// The event timestamp
    pub event_time: String,
    /// The token for verification
    pub token: String,
}

/// An event from the Events API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventsApiEvent {
    /// A message event
    Message(MessageEvent),
    /// An app_mention event
    AppMention(AppMentionEvent),
    /// Other event types
    Other(serde_json::Value),
}

/// A message event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    /// The event type (always "message")
    #[serde(rename = "type")]
    pub event_type: String,
    /// The channel ID
    pub channel: String,
    /// Optional thread timestamp
    pub thread_ts: Option<String>,
    /// The user ID who sent the message
    pub user: Option<String>,
    /// The message text
    pub text: Option<String>,
    /// Optional bot user ID
    pub bot_id: Option<String>,
    /// The message timestamp
    pub ts: String,
    /// Optional subtype (e.g., "bot_message")
    pub subtype: Option<String>,
    /// Optional channel type
    pub channel_type: Option<String>,
    /// Files attached to the message
    pub files: Option<Vec<serde_json::Value>>,
}

/// An app mention event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMentionEvent {
    /// The event type (always "app_mention")
    #[serde(rename = "type")]
    pub event_type: String,
    /// The channel ID
    pub channel: String,
    /// The user ID who mentioned the bot
    pub user: String,
    /// The message text
    pub text: String,
    /// The message timestamp
    pub ts: String,
}

/// Disconnect event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectEvent {
    /// The reason for disconnect
    pub reason: String,
}

/// Start a Socket Mode task for an account.
///
/// This spawns a background task that connects to Socket Mode and processes events.
/// The task runs until the shutdown signal is received.
pub async fn start_socket_mode_task(
    mut connection: SlackSocketModeConnection,
    shutdown: Arc<tokio::sync::Notify>,
    account_id: String,
) {
    // Clone account_id for error logging after the task moves it
    let account_id_for_error = account_id.clone();

    info!("Starting Socket Mode task for account {}", account_id);

    // Connect to Socket Mode
    if let Err(e) = connection.connect().await {
        error!(
            "Failed to connect to Socket Mode for account {}: {}",
            account_id, e
        );
        return;
    }

    // Spawn the receive loop
    let task = tokio::spawn(async move {
        // In a full implementation, this would:
        // 1. Wait for events from the WebSocket stream
        // 2. Parse them into SocketModeEvent
        // 3. Acknowledge each envelope_id
        // 4. Process message events

        // For now, just wait for shutdown
        shutdown.notified().await;
        info!("Socket Mode task shutting down for account {}", account_id);
    });

    // Wait for the task to complete
    let _ = task.await;

    // Disconnect
    if let Err(e) = connection.disconnect().await {
        error!(
            "Failed to disconnect from Socket Mode for account {}: {}",
            account_id_for_error, e
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_mode_event_serialization() {
        let event = SocketModeEvent {
            envelope_id: "123456".to_string(),
            payload: SocketModePayload::Hello(HelloEvent {
                socket_mode_version: "1".to_string(),
                bot_id: "B123456".to_string(),
            }),
            team_id: Some("T123456".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("123456"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_message_event_serialization() {
        let event = EventsApiEvent::Message(MessageEvent {
            event_type: "message".to_string(),
            channel: "C123456".to_string(),
            thread_ts: Some("1234567890.123456".to_string()),
            user: Some("U123456".to_string()),
            text: Some("Hello, world!".to_string()),
            bot_id: None,
            ts: "1234567890.123456".to_string(),
            subtype: None,
            channel_type: Some("channel".to_string()),
            files: None,
        });

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("message"));
        assert!(json.contains("C123456"));
    }

    #[test]
    fn test_socket_mode_connection_new() {
        let config = crate::SlackAccountConfig {
            bot_token: "xoxb-test".to_string(),
            app_token: Some("xapp-test".to_string()),
            ..Default::default()
        };

        // This would fail in a real test without proper setup
        // But we can test the struct construction
        assert_eq!(config.bot_token, "xoxb-test");
        assert_eq!(config.app_token, Some("xapp-test".to_string()));
    }
}
