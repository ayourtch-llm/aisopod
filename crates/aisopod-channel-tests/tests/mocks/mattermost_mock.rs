//! Mock Mattermost server for integration testing.
//!
//! This module provides a mock implementation of a Mattermost server
//! that responds to REST API and WebSocket events, allowing integration
//! tests to verify Mattermost channel behavior without requiring a real server.

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

/// State for the Mattermost mock server
#[derive(Clone, Default)]
pub struct MockMattermostState {
    /// List of received posts
    pub received_posts: Arc<Mutex<Vec<MattermostPost>>>,
    /// Count of API calls
    pub api_calls: Arc<Mutex<usize>>,
    /// Webhook events received
    pub webhook_events: Arc<Mutex<Vec<serde_json::Value>>>,
    /// Last posted message
    pub last_message: Arc<Mutex<Option<String>>>,
}

impl MockMattermostState {
    pub fn new() -> Self {
        Self {
            received_posts: Arc::new(Mutex::new(Vec::new())),
            api_calls: Arc::new(Mutex::new(0)),
            webhook_events: Arc::new(Mutex::new(Vec::new())),
            last_message: Arc::new(Mutex::new(None)),
        }
    }
}

/// A Mattermost post
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MattermostPost {
    pub channel_id: String,
    pub message: String,
    pub root_id: Option<String>,
    pub parent_id: Option<String>,
}

/// A mock Mattermost server
pub struct MockMattermostServer {
    /// The server URL
    pub url: String,
    /// The state of the mock server
    pub state: MockMattermostState,
    /// The join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
    /// Channel to signal server shutdown
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockMattermostServer {
    /// Starts a mock Mattermost server on a random port
    ///
    /// # Returns
    ///
    /// A tuple of (server URL, MockMattermostServer instance)
    pub async fn start() -> (String, Self) {
        let state = MockMattermostState::new();
        let state_clone = state.clone();

        // Create a channel to signal shutdown
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let app = Router::new()
            .route("/api/v4/users/login", post(mock_login))
            .route("/api/v4/teams/:team_id/channels", get(mock_get_channels))
            .route("/api/v4/channels/:channel_id/posts", get(mock_get_posts))
            .route("/api/v4/channels/:channel_id/posts", post(mock_create_post))
            .route("/webhooks/:token", post(mock_webhook))
            .with_state(state_clone);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local addr");
        let server_url = format!("http://{}", addr);

        info!("Starting mock Mattermost server at {}", server_url);

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
                eprintln!("Mock Mattermost server error: {}", e);
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
    info!("Mock Mattermost login endpoint called");

    // Return a successful login response
    Json(serde_json::json!({
        "id": "user123",
        "username": "testuser",
        "auth_token": "mock_auth_token_12345",
        "session_exp": 3600000
    }))
}

/// Mock get channels endpoint
async fn mock_get_channels(
    State(state): State<MockMattermostState>,
) -> Json<serde_json::Value> {
    info!("Mock Mattermost get channels endpoint called");

    // Update API call counter
    {
        let mut count = state.api_calls.lock().unwrap();
        *count += 1;
    }

    Json(serde_json::json!([
        {
            "id": "channel1",
            "name": "test-channel",
            "display_name": "Test Channel",
            "type": "O"
        },
        {
            "id": "channel2",
            "name": "general",
            "display_name": "General",
            "type": "O"
        }
    ]))
}

/// Mock get posts endpoint
async fn mock_get_posts(
    State(state): State<MockMattermostState>,
    axum::extract::Path(channel_id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    info!("Mock Mattermost get posts for channel: {}", channel_id);

    // Update API call counter
    {
        let mut count = state.api_calls.lock().unwrap();
        *count += 1;
    }

    Json(serde_json::json!({
        "posts": {}
    }))
}

/// Mock create post endpoint
async fn mock_create_post(
    State(state): State<MockMattermostState>,
    axum::extract::Path(channel_id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    info!(
        "Mock Mattermost create post for channel: {}",
        channel_id
    );

    // Update API call counter
    {
        let mut count = state.api_calls.lock().unwrap();
        *count += 1;
    }

    // Extract and store the message
    if let Some(message) = payload.get("message").and_then(|m| m.as_str()) {
        let mut last_msg = state.last_message.lock().unwrap();
        *last_msg = Some(message.to_string());
    }

    let post = MattermostPost {
        channel_id,
        message: payload.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string(),
        root_id: None,
        parent_id: None,
    };

    let mut posts = state.received_posts.lock().unwrap();
    posts.push(post);

    Json(serde_json::json!({
        "id": format!("post_{}", chrono::Utc::now().timestamp()),
        "create_at": chrono::Utc::now().timestamp() * 1000,
        "update_at": chrono::Utc::now().timestamp() * 1000,
        "delete_at": 0,
        "user_id": "user123",
        "channel_id": channel_id,
        "message": payload.get("message").and_then(|m| m.as_str()).unwrap_or(""),
        "type": "",
        "props": {}
    }))
}

/// Mock webhook endpoint
async fn mock_webhook(
    axum::extract::Path(token): axum::extract::Path<String>,
    State(state): State<MockMattermostState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    info!("Mock Mattermost webhook called with token: {}", token);

    let mut events = state.webhook_events.lock().unwrap();
    events.push(payload.clone());

    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

impl Default for MockMattermostServer {
    fn default() -> Self {
        panic!("MockMattermostServer must be started with MockMattermostServer::start()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_mattermost_server_starts() {
        let (_url, server) = MockMattermostServer::start().await;
        assert!(server.url.len() > 0);
        assert!(server.state.api_calls.lock().unwrap().eq(&0));
    }

    #[tokio::test]
    async fn test_mock_mattermost_login() {
        let (_url, _server) = MockMattermostServer::start().await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/v4/users/login", _url))
            .json(&serde_json::json!({
                "login_id": "testuser",
                "password": "testpass"
            }))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());
    }
}
