//! Mock Lark/Feishu Open Platform API server for integration testing.
//!
//! This module provides a mock implementation of a Lark/Feishu Open Platform API server
//! that responds to the Lark API endpoints, allowing integration
//! tests to verify Lark channel behavior without requiring a real Lark app.

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

/// State for the Lark mock server
#[derive(Clone, Default)]
pub struct MockLarkState {
    /// List of messages sent
    pub sent_messages: Arc<Mutex<Vec<LarkMessage>>>,
    /// Count of message send calls
    pub send_count: Arc<Mutex<usize>>,
    /// Webhook events received
    pub webhook_events: Arc<Mutex<Vec<serde_json::Value>>>,
    /// Tenant access tokens issued
    pub access_tokens: Arc<Mutex<Vec<String>>>,
    /// Last event received
    pub last_event: Arc<Mutex<Option<serde_json::Value>>>,
}

impl MockLarkState {
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

/// A Lark message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LarkMessage {
    pub receive_id: String,
    #[serde(rename = "msg_type")]
    pub message_type: String,
    pub content: serde_json::Value,
}

/// A mock Lark/Feishu API server
pub struct MockLarkServer {
    /// The server URL
    pub url: String,
    /// The state of the mock server
    pub state: MockLarkState,
    /// The join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
    /// Channel to signal server shutdown
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockLarkServer {
    /// Starts a mock Lark/Feishu API server on a random port
    ///
    /// # Returns
    ///
    /// A tuple of (server URL, MockLarkServer instance)
    pub async fn start() -> (String, Self) {
        let state = MockLarkState::new();
        let state_clone = state.clone();

        // Create a channel to signal shutdown
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let app = Router::new()
            .route("/open-apis/auth/v3/tenant_access_token/internal", post(mock_tenant_token))
            .route("/open-apis/auth/v3/app_access_token/internal", post(mock_app_token))
            .route("/open-apis/im/v1/messages", post(mock_send_message))
            .route("/open-apis/im/v1/messages/:message_id", get(mock_get_message))
            .route("/webhook/issue", post(mock_webhook_issue))
            .route("/webhook/event", post(mock_webhook_event))
            .with_state(state_clone);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local addr");
        let server_url = format!("http://{}", addr);

        info!("Starting mock Lark server at {}", server_url);

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
                eprintln!("Mock Lark server error: {}", e);
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
    pub async fn get_sent_messages(&self) -> Vec<LarkMessage> {
        self.state.sent_messages.lock().unwrap().clone()
    }

    /// Get the count of messages sent
    pub async fn get_send_count(&self) -> usize {
        *self.state.send_count.lock().unwrap()
    }
}

/// Mock tenant access token endpoint
async fn mock_tenant_token() -> Json<serde_json::Value> {
    info!("Mock Lark tenant access token endpoint called");

    Json(serde_json::json!({
        "code": 0,
        "msg": "success",
        "tenant_access_token": "test_access_token_12345",
        "expire": 7200
    }))
}

/// Mock app access token endpoint
async fn mock_app_token() -> Json<serde_json::Value> {
    info!("Mock Lark app access token endpoint called");

    Json(serde_json::json!({
        "code": 0,
        "msg": "success",
        "app_access_token": "test_app_token_12345",
        "expire": 7200
    }))
}

/// Mock send message endpoint
async fn mock_send_message(
    State(state): State<MockLarkState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("Mock Lark send message endpoint called");

    // Update send counter
    {
        let mut count = state.send_count.lock().unwrap();
        *count += 1;
    }

    // Extract message data
    let receive_id = payload.get("receive_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let msg_type = payload.get("msg_type").and_then(|v| v.as_str()).unwrap_or("text");
    let content = payload.get("content").cloned().unwrap_or_else(|| serde_json::json!(""));

    let message = LarkMessage {
        receive_id,
        message_type: msg_type.to_string(),
        content,
    };

    let mut messages = state.sent_messages.lock().unwrap();
    messages.push(message);

    // Return success response
    Ok(Json(serde_json::json!({
        "code": 0,
        "msg": "success",
        "data": {
            "message_id": format!("msg_{}", uuid::Uuid::new_v4())
        }
    })))
}

/// Mock get message endpoint
async fn mock_get_message(
    axum::extract::Path(message_id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    info!("Mock Lark get message endpoint called for: {}", message_id);

    Json(serde_json::json!({
        "code": 0,
        "msg": "success",
        "data": {
            "message": {
                "message_id": message_id,
                "sender": {
                    "sender_id": {
                        "union_id": "test_union_id",
                        "user_id": "test_user_id",
                        "open_id": "test_open_id"
                    },
                    "sender_type": "user",
                    "sender_info": {
                        "union_id": "test_union_id",
                        "user_id": "test_user_id",
                        "open_id": "test_open_id",
                        "name": "Test User",
                        "email": "test@example.com"
                    }
                },
                "create_time": chrono::Utc::now().timestamp() * 1000,
                "message_type": "text",
                "content": "Test message"
            }
        }
    }))
}

/// Mock webhook issue endpoint
async fn mock_webhook_issue(
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    info!("Mock Lark webhook issue endpoint called");

    let webhook = payload.get("webhook").and_then(|v| v.as_str()).unwrap_or("");

    Json(serde_json::json!({
        "code": 0,
        "msg": "success",
        "data": {
            "webhook": webhook,
            "event_types": ["im.message.receive_v1"]
        }
    }))
}

/// Mock webhook event endpoint
async fn mock_webhook_event(
    State(state): State<MockLarkState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("Mock Lark webhook event endpoint called");

    // Store the webhook event
    {
        let mut events = state.webhook_events.lock().unwrap();
        events.push(payload.clone());
        
        let mut last = state.last_event.lock().unwrap();
        *last = Some(payload);
    }

    // Return success response
    Ok(Json(serde_json::json!({
        "code": 0,
        "msg": "success"
    })))
}

impl Default for MockLarkServer {
    fn default() -> Self {
        panic!("MockLarkServer must be started with MockLarkServer::start()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_lark_server_starts() {
        let (_url, server) = MockLarkServer::start().await;
        assert!(server.url.len() > 0);
        assert!(server.state.send_count.lock().unwrap().eq(&0));
    }

    #[tokio::test]
    async fn test_mock_lark_send_message() {
        let (_url, server) = MockLarkServer::start().await;

        // Make a send message request
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/open-apis/im/v1/messages", _url))
            .json(&serde_json::json!({
                "receive_id": "oc_test123",
                "msg_type": "text",
                "content": "{\"text\":\"Hello Lark\"}"
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
