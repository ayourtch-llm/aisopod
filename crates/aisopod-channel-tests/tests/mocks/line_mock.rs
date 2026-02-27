//! Mock LINE Messaging API server for integration testing.
//!
//! This module provides a mock implementation of a LINE Messaging API server
//! that responds to the LINE API endpoints, allowing integration
//! tests to verify LINE channel behavior without requiring a real LINE account.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tracing::info;

/// State for the LINE mock server
#[derive(Clone, Default)]
pub struct MockLineState {
    /// List of messages sent
    pub sent_messages: Arc<Mutex<Vec<LineMessage>>>,
    /// Count of push message calls
    pub push_count: Arc<Mutex<usize>>,
    /// Count of reply message calls
    pub reply_count: Arc<Mutex<usize>>,
    /// Webhook events received
    pub webhook_events: Arc<Mutex<Vec<serde_json::Value>>>,
    /// Last reply token received
    pub last_reply_token: Arc<Mutex<Option<String>>>,
}

impl MockLineState {
    pub fn new() -> Self {
        Self {
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            push_count: Arc::new(Mutex::new(0)),
            reply_count: Arc::new(Mutex::new(0)),
            webhook_events: Arc::new(Mutex::new(Vec::new())),
            last_reply_token: Arc::new(Mutex::new(None)),
        }
    }
}

/// A LINE message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineMessage {
    pub to: String,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_content_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_image_url: Option<String>,
}

/// A mock LINE Messaging API server
pub struct MockLineServer {
    /// The server URL
    pub url: String,
    /// The state of the mock server
    pub state: MockLineState,
    /// The join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
    /// Channel to signal server shutdown
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockLineServer {
    /// Starts a mock LINE Messaging API server on a random port
    ///
    /// # Returns
    ///
    /// A tuple of (server URL, MockLineServer instance)
    pub async fn start() -> (String, Self) {
        let state = MockLineState::new();
        let state_clone = state.clone();

        // Create a channel to signal shutdown
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let app = Router::new()
            .route("/v2/bot/message/push", post(mock_push_message))
            .route("/v2/bot/message/reply", post(mock_reply_message))
            .route("/v2/bot/message/multicast", post(mock_multicast))
            .route("/v2/bot/message/broadcast", post(mock_broadcast))
            .route("/v2/bot/profile/:user_id", get(mock_get_profile))
            .route("/webhook", post(mock_webhook))
            .with_state(state_clone);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local addr");
        let server_url = format!("http://{}", addr);

        info!("Starting mock LINE server at {}", server_url);

        let handle = tokio::spawn(async move {
            let server = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    tokio::select! {
                        _ = shutdown_rx => {},
                        _ = std::future::pending::<()>() => {},
                    }
                })
                .await;

            if let Err(e) = server {
                eprintln!("Mock LINE server error: {}", e);
            }
        });

        let server = Self {
            url: server_url.clone(),
            state,
            _handle: handle,
            _shutdown_tx: Some(shutdown_tx),
        };

        (server_url, server)
    }

    /// Get all sent messages
    pub async fn get_sent_messages(&self) -> Vec<LineMessage> {
        self.state.sent_messages.lock().unwrap().clone()
    }

    /// Get the count of push messages sent
    pub async fn get_push_count(&self) -> usize {
        *self.state.push_count.lock().unwrap()
    }

    /// Get the count of reply messages sent
    pub async fn get_reply_count(&self) -> usize {
        *self.state.reply_count.lock().unwrap()
    }
}

/// Mock push message endpoint
async fn mock_push_message(
    State(state): State<MockLineState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("Mock LINE push message endpoint called");

    // Update push counter
    {
        let mut count = state.push_count.lock().unwrap();
        *count += 1;
    }

    // Extract message data
    let to = payload.get("to").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let messages: Vec<&serde_json::Value> = payload.get("messages").and_then(|v| v.as_array()).map_or(vec![], |v| v.iter().collect());
    
    for msg in &messages {
        let message_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("text").to_string();
        let text = msg.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());

        let line_msg = LineMessage {
            to: to.clone(),
            message_type: message_type.to_string(),
            text,
            original_content_url: None,
            preview_image_url: None,
        };

        let mut messages = state.sent_messages.lock().unwrap();
        messages.push(line_msg);
    }

    // Return success response
    Ok(Json(serde_json::json!({
        "status": 200,
        "message": "ok"
    })))
}

/// Mock reply message endpoint
async fn mock_reply_message(
    State(state): State<MockLineState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("Mock LINE reply message endpoint called");

    // Update reply counter
    {
        let mut count = state.reply_count.lock().unwrap();
        *count += 1;
    }

    // Extract reply token
    let reply_token = payload.get("replyToken").and_then(|v| v.as_str()).unwrap_or("").to_string();
    {
        let mut token = state.last_reply_token.lock().unwrap();
        *token = Some(reply_token);
    }

    // Extract message data
    let messages: Vec<&serde_json::Value> = payload.get("messages").and_then(|v| v.as_array()).map_or(vec![], |v| v.iter().collect());
    
    let to = payload.get("to").and_then(|v| v.as_str()).unwrap_or("").to_string();
    
    for msg in &messages {
        let message_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("text").to_string();
        let text = msg.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());

        let line_msg = LineMessage {
            to: to.clone(),
            message_type: message_type.to_string(),
            text,
            original_content_url: None,
            preview_image_url: None,
        };

        let mut messages = state.sent_messages.lock().unwrap();
        messages.push(line_msg);
    }

    // Return success response
    Ok(Json(serde_json::json!({
        "status": 200,
        "message": "ok"
    })))
}

/// Mock multicast endpoint
async fn mock_multicast(
    State(state): State<MockLineState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("Mock LINE multicast endpoint called");

    // Extract message data
    let to: Vec<&serde_json::Value> = payload.get("to").and_then(|v| v.as_array()).map_or(vec![], |v| v.iter().collect());
    let messages: Vec<&serde_json::Value> = payload.get("messages").and_then(|v| v.as_array()).map_or(vec![], |v| v.iter().collect());
    
    // For multicast, we count the number of recipients
    let recipient_count = to.len();
    let mut push_count = state.push_count.lock().unwrap();
    *push_count += recipient_count;

    for recipient in &to {
        let recipient_str = recipient.as_str().unwrap_or("").to_string();
        for msg in &messages {
            let message_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("text").to_string();
            let text = msg.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());

            let line_msg = LineMessage {
                to: recipient_str.clone(),
                message_type: message_type.to_string(),
                text,
                original_content_url: None,
                preview_image_url: None,
            };

            let mut messages = state.sent_messages.lock().unwrap();
            messages.push(line_msg);
        }
    }

    Ok(Json(serde_json::json!({
        "status": 200,
        "message": "ok"
    })))
}

/// Mock broadcast endpoint
async fn mock_broadcast(
    State(state): State<MockLineState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("Mock LINE broadcast endpoint called");

    // Extract message data
    let messages: Vec<&serde_json::Value> = payload.get("messages").and_then(|v| v.as_array()).map_or(vec![], |v| v.iter().collect());
    
    // For broadcast, count as multiple push messages
    let mut push_count = state.push_count.lock().unwrap();
    *push_count += messages.len();

    // Broadcast doesn't have a specific recipient, so we use a dummy target
    let to = "broadcast_target".to_string();
    for msg in &messages {
        let message_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("text").to_string();
        let text = msg.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());

        let line_msg = LineMessage {
            to: to.clone(),
            message_type: message_type.to_string(),
            text,
            original_content_url: None,
            preview_image_url: None,
        };

        let mut messages = state.sent_messages.lock().unwrap();
        messages.push(line_msg);
    }

    Ok(Json(serde_json::json!({
        "status": 200,
        "message": "ok"
    })))
}

/// Mock get profile endpoint
async fn mock_get_profile(
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    info!("Mock LINE get profile endpoint called for user: {}", user_id);

    Json(serde_json::json!({
        "displayName": "Test User",
        "userId": user_id,
        "pictureUrl": "https://example.com/profile.png",
        "statusMessage": "Online",
        "language": "en"
    }))
}

/// Mock webhook endpoint
async fn mock_webhook(
    State(state): State<MockLineState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("Mock LINE webhook endpoint called");

    // Store the webhook event
    {
        let mut events = state.webhook_events.lock().unwrap();
        if let Some(events_array) = payload.get("events").and_then(|v| v.as_array()) {
            for event in events_array {
                events.push(event.clone());
            }
        }
    }

    // Return success response
    Ok(Json(serde_json::json!({
        "status": 200,
        "message": "ok"
    })))
}

impl Default for MockLineServer {
    fn default() -> Self {
        panic!("MockLineServer must be started with MockLineServer::start()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_line_server_starts() {
        let (_url, server) = MockLineServer::start().await;
        assert!(server.url.len() > 0);
        assert!(server.state.push_count.lock().unwrap().eq(&0));
    }

    #[tokio::test]
    async fn test_mock_line_push_message() {
        let (_url, server) = MockLineServer::start().await;

        // Make a push message request
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/v2/bot/message/push", _url))
            .json(&serde_json::json!({
                "to": "user123",
                "messages": [{
                    "type": "text",
                    "text": "Hello LINE"
                }]
            }))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());
        
        // Verify the message was recorded
        let messages = server.get_sent_messages().await;
        assert!(!messages.is_empty());
    }
}
