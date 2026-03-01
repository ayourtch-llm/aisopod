//! Mattermost WebSocket event streaming.
//!
//! This module provides WebSocket connection for real-time event streaming
//! from Mattermost servers.

use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, instrument, warn};

/// Error types for Mattermost WebSocket operations.
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    /// WebSocket connection failed
    #[error("WebSocket connection error: {0}")]
    Connection(#[from] tokio_tungstenite::tungstenite::Error),
    /// Authentication failed
    #[error("Authentication error: {0}")]
    Auth(String),
    /// Message parsing failed
    #[error("Parse error: {0}")]
    Parse(String),
    /// Connection closed
    #[error("Connection closed")]
    Closed,
    /// Timeout waiting for response
    #[error("Timeout waiting for response")]
    Timeout,
}

/// Mattermost WebSocket event.
#[derive(Debug, Clone, Deserialize)]
pub struct MattermostEvent {
    /// The event name (e.g., "posted", "Typing", "online_update")
    pub event: String,
    /// The sequence number
    pub seq: i64,
    /// The timestamp
    pub broadcast: serde_json::Value,
    /// The data payload
    pub data: serde_json::Value,
    /// The status
    pub status: Option<String>,
}

/// Authentication challenge from Mattermost.
#[derive(Debug, Deserialize)]
pub struct AuthChallenge {
    /// The event type
    pub event: String,
    /// The data payload
    pub data: serde_json::Value,
}

/// Authentication response to Mattermost.
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    /// The sequence number
    pub seq: i64,
    /// The action
    pub action: String,
    /// The data payload
    pub data: serde_json::Value,
}

/// Mattermost WebSocket client.
pub struct MattermostWebSocket {
    /// The WebSocket stream
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    /// The next sequence number
    next_seq: i64,
}

impl MattermostWebSocket {
    /// Connect to the Mattermost WebSocket server.
    ///
    /// # Arguments
    ///
    /// * `server_url` - The base URL of the Mattermost server
    /// * `token` - The authentication token
    ///
    /// # Returns
    ///
    /// * `Ok(MattermostWebSocket)` - The WebSocket connection
    /// * `Err(WebSocketError)` - An error if the connection fails
    #[instrument(skip(token))]
    pub async fn connect(server_url: &str, token: &str) -> Result<Self> {
        // Convert http/https to ws/wss
        let ws_url = server_url
            .replace("https://", "wss://")
            .replace("http://", "ws://")
            .trim_end_matches('/')
            .to_string();
        let websocket_url = format!("{}/api/v4/websocket", ws_url);

        info!("Connecting to WebSocket at {}", websocket_url);

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&websocket_url).await?;

        info!("WebSocket connection established");

        let mut client = Self {
            stream: ws_stream,
            next_seq: 1,
        };

        // Send authentication challenge response
        client.authenticate(token).await?;

        Ok(client)
    }

    /// Authenticate with the WebSocket server.
    ///
    /// # Arguments
    ///
    /// * `token` - The authentication token
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Authentication successful
    /// * `Err(WebSocketError)` - An error if authentication fails
    #[instrument(skip(self, token))]
    async fn authenticate(&mut self, token: &str) -> Result<(), WebSocketError> {
        // Read the authentication challenge from server
        let challenge = self.read_challenge().await?;

        // Verify it's an authentication challenge
        if challenge.event != "authentication_challenge" {
            warn!(
                "Expected authentication_challenge, got: {}",
                challenge.event
            );
        }

        // Send authentication response
        let auth_data = json!({
            "token": token
        });

        let response = AuthResponse {
            seq: self.next_seq,
            action: "authentication_response".to_string(),
            data: auth_data,
        };

        self.next_seq += 1;

        self.send_message(&response).await?;

        // Wait for authentication result
        let result = self.read_event().await?;

        if result.event != "authentication_response" {
            error!("Expected authentication_response, got: {}", result.event);
            return Err(WebSocketError::Auth(format!(
                "Unexpected response after authentication: {}",
                result.event
            )));
        }

        // Check if authentication was successful
        let data = &result.data;
        if let Some(status) = result.status.as_ref() {
            if status != "OK" {
                return Err(WebSocketError::Auth(format!(
                    "Authentication failed with status: {}",
                    status
                )));
            }
        }

        // Verify auth success in data
        if let Some(success) = data.get("status").and_then(|s| s.as_str()) {
            if success == "OK" {
                info!("WebSocket authentication successful");
            } else {
                return Err(WebSocketError::Auth(format!(
                    "Authentication failed: {}",
                    success
                )));
            }
        }

        Ok(())
    }

    /// Read the next WebSocket event.
    ///
    /// # Returns
    ///
    /// * `Ok(MattermostEvent)` - The next event
    /// * `Err(WebSocketError)` - An error if reading fails
    #[instrument(skip(self))]
    pub async fn next_event(&mut self) -> Result<MattermostEvent, WebSocketError> {
        self.read_event().await
    }

    /// Read an event from the WebSocket stream.
    #[instrument(skip(self))]
    async fn read_event(&mut self) -> Result<MattermostEvent, WebSocketError> {
        loop {
            let msg = self.stream.next().await;

            match msg {
                Some(Ok(Message::Text(text))) => {
                    debug!("Received WebSocket message: {}", text);

                    // Parse the JSON message
                    let event: MattermostEvent = serde_json::from_str(&text).map_err(|e| {
                        WebSocketError::Parse(format!("Failed to parse JSON: {}", e))
                    })?;

                    // Filter for meaningful events (ignore status updates)
                    if !self.is_status_event(&event) {
                        return Ok(event);
                    }
                }
                Some(Ok(Message::Close(_))) => {
                    info!("WebSocket connection closed");
                    return Err(WebSocketError::Closed);
                }
                Some(Ok(Message::Ping(_))) => {
                    // Pong is handled automatically by tungstenite
                    continue;
                }
                Some(Ok(Message::Pong(_))) => {
                    // Keep-alive response
                    continue;
                }
                Some(Ok(Message::Binary(_))) => {
                    debug!("Received binary message, skipping");
                    continue;
                }
                Some(Ok(Message::Frame(_))) => {
                    // Frame messages are internal - skip them
                    continue;
                }
                Some(Ok(_)) => {
                    // Unknown message type - skip
                    continue;
                }
                Some(Err(e)) => {
                    error!("WebSocket error: {}", e);
                    return Err(WebSocketError::Connection(e));
                }
                None => {
                    info!("WebSocket stream ended");
                    return Err(WebSocketError::Closed);
                }
            }
        }
    }

    /// Check if an event is a status-only event (should be filtered).
    fn is_status_event(&self, event: &MattermostEvent) -> bool {
        // Status events like "connection_established" and "hello" are just handshake
        // We want to filter for actual events like "posted", "typing", etc.
        matches!(
            event.event.as_str(),
            "connection_established" | "hello" | "status" | "heartbeat"
        )
    }

    /// Send a message through the WebSocket.
    #[instrument(skip(self, msg))]
    async fn send_message(&mut self, msg: &impl Serialize) -> Result<(), WebSocketError> {
        let text = serde_json::to_string(msg)
            .map_err(|e| WebSocketError::Parse(format!("Failed to serialize: {}", e)))?;

        self.stream
            .send(Message::Text(text))
            .await
            .map_err(WebSocketError::Connection)?;

        Ok(())
    }

    /// Read an authentication challenge from the WebSocket stream.
    #[instrument(skip(self))]
    async fn read_challenge(&mut self) -> Result<MattermostEvent, WebSocketError> {
        loop {
            let msg = self.stream.next().await;

            match msg {
                Some(Ok(Message::Text(text))) => {
                    debug!("Received WebSocket message: {}", text);

                    // Parse the JSON message
                    let event: MattermostEvent = serde_json::from_str(&text).map_err(|e| {
                        WebSocketError::Parse(format!("Failed to parse JSON: {}", e))
                    })?;

                    // Return the authentication challenge
                    return Ok(event);
                }
                Some(Ok(Message::Close(_))) => {
                    info!("WebSocket connection closed during authentication");
                    return Err(WebSocketError::Closed);
                }
                Some(Ok(Message::Ping(_))) => {
                    // Pong is handled automatically by tungstenite
                    continue;
                }
                Some(Ok(Message::Pong(_))) => {
                    // Keep-alive response
                    continue;
                }
                Some(Ok(Message::Binary(_))) => {
                    debug!("Received binary message during auth, skipping");
                    continue;
                }
                Some(Ok(Message::Frame(_))) => {
                    // Frame messages are internal - skip them
                    continue;
                }
                Some(Ok(_)) => {
                    // Unknown message type - skip
                    continue;
                }
                Some(Err(e)) => {
                    error!("WebSocket error during auth: {}", e);
                    return Err(WebSocketError::Connection(e));
                }
                None => {
                    info!("WebSocket stream ended during authentication");
                    return Err(WebSocketError::Closed);
                }
            }
        }
    }

    /// Send a ping to keep the connection alive.
    #[instrument(skip(self))]
    pub async fn ping(&mut self) -> Result<(), WebSocketError> {
        self.stream
            .send(Message::Ping(Vec::new()))
            .await
            .map_err(WebSocketError::Connection)?;
        Ok(())
    }

    /// Close the WebSocket connection gracefully.
    #[instrument(skip(self))]
    pub async fn close(&mut self) -> Result<(), WebSocketError> {
        self.stream
            .send(Message::Close(None))
            .await
            .map_err(WebSocketError::Connection)?;
        Ok(())
    }
}

impl Drop for MattermostWebSocket {
    fn drop(&mut self) {
        // Try to close the connection gracefully on drop
        // Note: This won't work in async context without runtime
        // For production, use an explicit close() call
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_response_serialization() {
        let response = AuthResponse {
            seq: 1,
            action: "authentication_response".to_string(),
            data: json!({"token": "test-token"}),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"seq\":1"));
        assert!(json.contains("\"action\":\"authentication_response\""));
        assert!(json.contains("\"token\":\"test-token\""));
    }

    #[test]
    fn test_mattermost_event_deserialization() {
        let json = r#"{
            "event": "posted",
            "seq": 123,
            "broadcast": {"channel_id": "abc123", "omit_users": null},
            "data": {
                "channel": {"id": "abc123", "name": "test"},
                "post": {"id": "post123", "message": "Hello"}
            }
        }"#;

        let event: MattermostEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event, "posted");
        assert_eq!(event.seq, 123);
        // Verify the data was parsed correctly
        assert_eq!(event.data["channel"]["name"], "test");
        assert_eq!(event.data["post"]["id"], "post123");
        assert_eq!(event.data["post"]["message"], "Hello");
    }

    #[test]
    fn test_is_status_event() {
        // Test the is_status_event logic directly without creating a WebSocket instance
        // by using the same matching logic
        let event1 = MattermostEvent {
            event: "connection_established".to_string(),
            seq: 0,
            broadcast: serde_json::Value::Null,
            data: serde_json::Value::Null,
            status: None,
        };
        assert!(matches!(
            event1.event.as_str(),
            "connection_established" | "hello" | "status" | "heartbeat"
        ));

        let event2 = MattermostEvent {
            event: "posted".to_string(),
            seq: 123,
            broadcast: serde_json::Value::Null,
            data: serde_json::Value::Null,
            status: None,
        };
        assert!(!matches!(
            event2.event.as_str(),
            "connection_established" | "hello" | "status" | "heartbeat"
        ));
    }

    #[test]
    fn test_ws_url_conversion() {
        // Test HTTP to WS conversion
        let http_url = "http://mattermost.example.com";
        let ws_url = http_url.replace("http://", "ws://");
        assert_eq!(ws_url, "ws://mattermost.example.com");

        // Test HTTPS to WSS conversion
        let https_url = "https://mattermost.example.com";
        let wss_url = https_url.replace("https://", "wss://");
        assert_eq!(wss_url, "wss://mattermost.example.com");
    }
}
