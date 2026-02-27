//! Mock Matrix homeserver for integration testing.
//!
//! This module provides a mock implementation of a Matrix homeserver
//! that responds with predefined JSON-RPC responses, allowing integration
//! tests to verify Matrix channel behavior without requiring a real homeserver.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tracing::info;

/// State for the Matrix mock server
#[derive(Clone, Default)]
pub struct MockMatrixState {
    /// Queue of incoming messages to return from sync
    pub sync_queue: Arc<Mutex<Vec<serde_json::Value>>>,
    /// Count of send message calls
    pub send_message_calls: Arc<Mutex<usize>>,
    /// Last received text message
    pub last_text_message: Arc<Mutex<Option<String>>>,
    /// Login attempts
    pub login_attempts: Arc<Mutex<usize>>,
}

impl MockMatrixState {
    pub fn new() -> Self {
        Self {
            sync_queue: Arc::new(Mutex::new(Vec::new())),
            send_message_calls: Arc::new(Mutex::new(0)),
            last_text_message: Arc::new(Mutex::new(None)),
            login_attempts: Arc::new(Mutex::new(0)),
        }
    }
}

/// A mock Matrix homeserver
pub struct MockMatrixServer {
    /// The server URL
    pub url: String,
    /// The state of the mock server
    pub state: MockMatrixState,
    /// The join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
    /// Channel to signal server shutdown
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockMatrixServer {
    /// Starts a mock Matrix homeserver on a random port
    ///
    /// # Returns
    ///
    /// A tuple of (server URL, MockMatrixServer instance)
    pub async fn start() -> (String, Self) {
        let state = MockMatrixState::new();
        let state_clone = state.clone();

        // Create a channel to signal shutdown
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let app = Router::new()
            .route("/_matrix/client/v3/login", post(mock_login))
            .route("/_matrix/client/v3/sync", get(mock_sync))
            .route(
                "/_matrix/client/v3/rooms/:room_id/send/:event_type/:txn_id",
                put(mock_send_event),
            )
            .with_state(state_clone);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local addr");
        let server_url = format!("http://{}", addr);

        info!("Starting mock Matrix homeserver at {}", server_url);

        let handle = tokio::spawn(async move {
            let server = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    // Wait for shutdown signal or never if channel is dropped
                    tokio::select! {
                        _ = shutdown_rx => {},
                        _ = std::future::pending::<()>() => {},
                    }
                })
                .await;

            if let Err(e) = server {
                eprintln!("Mock Matrix server error: {}", e);
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
}

/// Mock login endpoint
async fn mock_login(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    info!("Mock Matrix login endpoint called");

    // Return a successful login response
    Json(serde_json::json!({
        "access_token": "mock_access_token_12345",
        "device_id": "DEVICE_ID",
        "home_server": "localhost",
        "user_id": "@testuser:localhost",
        "expires_in_ms": 3600000
    }))
}

/// Mock sync endpoint
async fn mock_sync(
    Query(query): axum::extract::Query<serde_json::Value>,
    State(state): State<MockMatrixState>,
) -> Json<serde_json::Value> {
    info!("Mock Matrix sync endpoint called");

    // Get sync token if provided
    let _sync_token = query.get("since").and_then(|v| v.as_str());

    // Return empty sync response for tests
    // Tests can add messages to sync_queue before calling sync
    let messages = {
        let mut queue = state.sync_queue.lock().unwrap();
        std::mem::take(&mut *queue)
    };

    Json(serde_json::json!({
        "next_batch": "s12345_67890",
        "rooms": {
            "join": {}
        },
        "presence": {
            "events": []
        }
    }))
}

/// Mock send event endpoint
async fn mock_send_event(
    Path((room_id, event_type, txn_id)): Path<(String, String, String)>,
    Json(payload): Json<serde_json::Value>,
    State(state): State<MockMatrixState>,
) -> Json<serde_json::Value> {
    info!(
        "Mock Matrix send event endpoint called: room={}, event_type={}, txn_id={}",
        room_id, event_type, txn_id
    );

    // Update send message counter
    {
        let mut count = state.send_message_calls.lock().unwrap();
        *count += 1;
    }

    // Extract and store the message text if present
    if let Some(content) = payload.get("content").and_then(|c| c.get("body")) {
        if let Some(text) = content.as_str() {
            let mut last_msg = state.last_text_message.lock().unwrap();
            *last_msg = Some(text.to_string());
        }
    }

    // Return success response
    Json(serde_json::json!({
        "event_id": format!("$event_{}_{}", txn_id, chrono::Utc::now().timestamp())
    }))
}

impl Default for MockMatrixServer {
    fn default() -> Self {
        panic!("MockMatrixServer must be started with MockMatrixServer::start()");
    }
}

/// Helper function to add a message to the sync queue
pub async fn add_sync_message(server: &MockMatrixServer, message: &str) {
    let msg = serde_json::json!({
        "type": "m.room.message",
        "sender": "@otheruser:localhost",
        "content": {
            "msgtype": "m.text",
            "body": message
        },
        "room_id": "!testroom:localhost",
        "event_id": "$event1",
        "origin_server_ts": chrono::Utc::now().timestamp() * 1000
    });

    let mut queue = server.state.sync_queue.lock().unwrap();
    queue.push(msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_matrix_server_starts() {
        let (_url, server) = MockMatrixServer::start().await;
        assert!(server.url.len() > 0);
        assert!(server.state.send_message_calls.lock().unwrap().eq(&0));
    }

    #[tokio::test]
    async fn test_mock_matrix_login() {
        let (_url, _server) = MockMatrixServer::start().await;

        // Test login by making an HTTP request
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/_matrix/client/v3/login", _url))
            .json(&serde_json::json!({
                "type": "m.login.password",
                "identifier": {
                    "type": "m.id.user",
                    "user": "testuser"
                },
                "password": "testpass"
            }))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());
    }
}
