//! Mock Zalo OA API server for integration testing.
//!
//! This module provides a mock implementation of a Zalo OA API server
//! that responds to the Zalo API endpoints, allowing integration
//! tests to verify Zalo channel behavior without requiring a real Zalo account.

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

/// State for the Zalo mock server
#[derive(Clone, Default)]
pub struct MockZaloState {
    /// List of messages sent
    pub sent_messages: Arc<Mutex<Vec<ZaloMessage>>>,
    /// Count of message send calls
    pub send_count: Arc<Mutex<usize>>,
    /// Webhook events received
    pub webhook_events: Arc<Mutex<Vec<serde_json::Value>>>,
    /// Access tokens issued
    pub access_tokens: Arc<Mutex<Vec<String>>>,
    /// Last event received
    pub last_event: Arc<Mutex<Option<serde_json::Value>>>,
}

impl MockZaloState {
    pub fn new() -> Self {
        Self {
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            send_count: Arc::new(Mutex::new(0)),
            webhook_events: Arc::new(Mutex::new(Vec::new())),
            access_tokens: Arc::new(Mutex::new(vec![
                "test_access_token_12345".to_string(),
            ])),
            last_event: Arc::new(Mutex::new(None)),
        }
    }
}

/// A Zalo message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZaloMessage {
    pub recipient: ZaloRecipient,
    pub message: ZaloMessageContent,
}

/// Zalo recipient
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZaloRecipient {
    #[serde(rename = "user_id")]
    pub user_id: String,
}

/// Zalo message content
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZaloMessageContent {
    pub attachment: ZaloAttachment,
}

/// Zalo attachment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZaloAttachment {
    #[serde(rename = "type")]
    pub type_field: String,
    pub payload: serde_json::Value,
}

/// A mock Zalo OA API server
pub struct MockZaloServer {
    /// The server URL
    pub url: String,
    /// The state of the mock server
    pub state: MockZaloState,
    /// The join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
    /// Channel to signal server shutdown
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockZaloServer {
    /// Starts a mock Zalo OA API server on a random port
    ///
    /// # Returns
    ///
    /// A tuple of (server URL, MockZaloServer instance)
    pub async fn start() -> (String, Self) {
        let state = MockZaloState::new();
        let state_clone = state.clone();

        // Create a channel to signal shutdown
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let app = Router::new()
            .route("/oauth/access_token", post(mock_access_token))
            .route("/oa/message", post(mock_send_message))
            .route("/oa/message/batch", post(mock_send_batch))
            .route("/webhook", post(mock_webhook))
            .with_state(state_clone);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local addr");
        let server_url = format!("http://{}", addr);

        info!("Starting mock Zalo server at {}", server_url);

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
                eprintln!("Mock Zalo server error: {}", e);
            }
        });

        let server = Self {
            url: server_url,
            state,
            _handle: handle,
            _shutdown_tx: Some(shutdown_tx),
        };

        (server_url, server)
    }

    /// Get all sent messages
    pub async fn get_sent_messages(&self) -> Vec<ZaloMessage> {
        self.state.sent_messages.lock().unwrap().clone()
    }

    /// Get the count of messages sent
    pub async fn get_send_count(&self) -> usize {
        *self.state.send_count.lock().unwrap()
    }
}

/// Mock access token endpoint
async fn mock_access_token(
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    info!("Mock Zalo access token endpoint called");

    let app_id = payload.get("app_id").and_then(|v| v.as_str()).unwrap_or("test_app");
    let secret = payload.get("secret").and_then(|v| v.as_str()).unwrap_or("test_secret");

    Json(serde_json::json!({
        "access_token": format!("{}_token", app_id),
        "expires_in": 3600,
        "app_id": app_id
    }))
}

/// Mock send message endpoint
async fn mock_send_message(
    State(state): State<MockZaloState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("Mock Zalo send message endpoint called");

    // Update send counter
    {
        let mut count = state.send_count.lock().unwrap();
        *count += 1;
    }

    // Extract message data
    let recipient = payload.get("recipient").cloned().unwrap_or_else(|| {
        serde_json::json!({
            "user_id": "test_user"
        })
    });
    let message = payload.get("message").cloned().unwrap_or_else(|| {
        serde_json::json!({
            "attachment": {
                "type": "text",
                "payload": {
                    "text": "Test message"
                }
            }
        })
    });

    let zalo_msg = ZaloMessage {
        recipient: serde_json::from_value(recipient).unwrap_or_default(),
        message: serde_json::from_value(message).unwrap_or_default(),
    };

    let mut messages = state.sent_messages.lock().unwrap();
    messages.push(zalo_msg);

    // Return success response
    Ok(Json(serde_json::json!({
        "error": 0,
        "message": "Success",
        "data": {
            "message_id": format!("msg_{}", uuid::Uuid::new_v4())
        }
    })))
}

/// Mock send batch endpoint
async fn mock_send_batch(
    State(state): State<MockZaloState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("Mock Zalo send batch endpoint called");

    // Extract messages
    let messages_data = payload.get("messages").and_then(|v| v.as_array()).unwrap_or(&[]);
    
    // Count batch messages
    let mut batch_count = state.send_count.lock().unwrap();
    *batch_count += messages_data.len();

    for msg in messages_data {
        let recipient = msg.get("recipient").cloned().unwrap_or_else(|| {
            serde_json::json!({
                "user_id": "test_user"
            })
        });
        let message = msg.get("message").cloned().unwrap_or_else(|| {
            serde_json::json!({
                "attachment": {
                    "type": "text",
                    "payload": {
                        "text": "Batch message"
                    }
                }
            })
        });

        let zalo_msg = ZaloMessage {
            recipient: serde_json::from_value(recipient).unwrap_or_default(),
            message: serde_json::from_value(message).unwrap_or_default(),
        };

        let mut messages = state.sent_messages.lock().unwrap();
        messages.push(zalo_msg);
    }

    Ok(Json(serde_json::json!({
        "error": 0,
        "message": "Success",
        "data": {
            "results": messages_data.iter().enumerate().map(|(i, _)| {
                serde_json::json!({
                    "message_id": format!("msg_batch_{}", i),
                    "user_id": "test_user"
                })
            }).collect::<Vec<_>>()
        }
    })))
}

/// Mock webhook endpoint
async fn mock_webhook(
    State(state): State<MockZaloState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("Mock Zalo webhook endpoint called");

    // Store the webhook event
    {
        let mut events = state.webhook_events.lock().unwrap();
        events.push(payload.clone());
        
        let mut last = state.last_event.lock().unwrap();
        *last = Some(payload);
    }

    // Return success response
    Ok(Json(serde_json::json!({
        "error": 0,
        "message": "Success"
    })))
}

impl Default for MockZaloServer {
    fn default() -> Self {
        panic!("MockZaloServer must be started with MockZaloServer::start()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_zalo_server_starts() {
        let (_url, server) = MockZaloServer::start().await;
        assert!(server.url.len() > 0);
        assert!(server.state.send_count.lock().unwrap().eq(&0));
    }

    #[tokio::test]
    async fn test_mock_zalo_send_message() {
        let (_url, server) = MockZaloServer::start().await;

        // Make a send message request
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/oa/message", _url))
            .json(&serde_json::json!({
                "recipient": {
                    "user_id": "test_user_123"
                },
                "message": {
                    "attachment": {
                        "type": "text",
                        "payload": {
                            "text": "Hello Zalo"
                        }
                    }
                }
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
