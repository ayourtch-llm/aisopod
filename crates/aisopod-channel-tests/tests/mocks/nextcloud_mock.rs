//! Mock Nextcloud Talk server for integration testing.
//!
//! This module provides a mock implementation of a Nextcloud Talk server
//! that responds to the OCS API endpoints, allowing integration
//! tests to verify Nextcloud Talk channel behavior without requiring
//! a real Nextcloud server.

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

/// State for the Nextcloud mock server
#[derive(Clone, Default)]
pub struct MockNextcloudState {
    /// List of rooms
    pub rooms: Arc<Mutex<Vec<NextcloudRoom>>>,
    /// List of messages
    pub messages: Arc<Mutex<Vec<NextcloudMessage>>>,
    /// Count of send message calls
    pub send_message_calls: Arc<Mutex<usize>>,
    /// Count of get rooms calls
    pub get_rooms_calls: Arc<Mutex<usize>>,
}

impl MockNextcloudState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(vec![
                NextcloudRoom {
                    token: "room1".to_string(),
                    name: "Test Room".to_string(),
                    display_name: Some("Test Room".to_string()),
                    room_type: 1,
                    description: Some("A test room".to_string()),
                    password: false,
                },
                NextcloudRoom {
                    token: "room2".to_string(),
                    name: "General".to_string(),
                    display_name: Some("General".to_string()),
                    room_type: 2,
                    description: Some("General discussion".to_string()),
                    password: false,
                },
            ])),
            messages: Arc::new(Mutex::new(Vec::new())),
            send_message_calls: Arc::new(Mutex::new(0)),
            get_rooms_calls: Arc::new(Mutex::new(0)),
        }
    }
}

/// A Nextcloud Talk room
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NextcloudRoom {
    pub token: String,
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(rename = "type")]
    pub room_type: i32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub password: bool,
}

/// A Nextcloud Talk message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NextcloudMessage {
    pub id: i64,
    pub actor_id: String,
    pub message: String,
    pub timestamp: i64,
    #[serde(default)]
    pub actor_type: String,
    #[serde(default)]
    pub actor_display_name: Option<String>,
    #[serde(default)]
    pub chat_id: Option<String>,
}

/// A mock Nextcloud Talk server
pub struct MockNextcloudServer {
    /// The server URL
    pub url: String,
    /// The state of the mock server
    pub state: MockNextcloudState,
    /// The join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
    /// Channel to signal server shutdown
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockNextcloudServer {
    /// Starts a mock Nextcloud Talk server on a random port
    ///
    /// # Returns
    ///
    /// A tuple of (server URL, MockNextcloudServer instance)
    pub async fn start() -> (String, Self) {
        let state = MockNextcloudState::new();
        let state_clone = state.clone();

        // Create a channel to signal shutdown
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let app = Router::new()
            .route("/ocs/v2.php/apps/spreed/api/v4/room", get(mock_get_rooms))
            .route("/ocs/v2.php/apps/spreed/api/v1/chat/:room_token", get(mock_get_messages))
            .route(
                "/ocs/v2.php/apps/spreed/api/v1/chat/:room_token",
                post(mock_send_message),
            )
            .with_state(state_clone);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local addr");
        let server_url = format!("http://{}", addr);

        info!("Starting mock Nextcloud Talk server at {}", server_url);

        let server_url_for_closure = server_url.clone();
        let handle = tokio::spawn(async move {
            let _server_url = server_url_for_closure;
            let server = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    tokio::select! {
                        _ = shutdown_rx => {},
                        _ = std::future::pending::<()>() => {},
                    }
                })
                .await;

            if let Err(e) = server {
                eprintln!("Mock Nextcloud server error: {}", e);
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

    /// Get a list of rooms from the mock server
    pub async fn get_rooms(&self) -> Vec<NextcloudRoom> {
        self.state.rooms.lock().unwrap().clone()
    }

    /// Get messages for a specific room
    pub async fn get_messages(&self, room_token: &str) -> Vec<NextcloudMessage> {
        let all_messages = self.state.messages.lock().unwrap();
        all_messages
            .iter()
            .filter(|m| m.chat_id.as_deref() == Some(room_token))
            .cloned()
            .collect()
    }

    /// Add a message to the mock server
    pub async fn add_message(&self, message: NextcloudMessage) {
        let mut messages = self.state.messages.lock().unwrap();
        messages.push(message);
    }

    /// Get the count of send message calls
    pub async fn get_send_count(&self) -> usize {
        *self.state.send_message_calls.lock().unwrap()
    }
}

/// Mock get rooms endpoint
async fn mock_get_rooms(State(state): State<MockNextcloudState>) -> Json<serde_json::Value> {
    info!("Mock Nextcloud get rooms endpoint called");

    // Update get rooms counter
    {
        let mut count = state.get_rooms_calls.lock().unwrap();
        *count += 1;
    }

    let rooms = state.rooms.lock().unwrap();
    
    Json(serde_json::json!({
        "ocs": {
            "meta": {
                "status": "ok",
                "statusCode": 200
            },
            "data": {
                "rooms": rooms.iter().map(|r| {
                    serde_json::json!({
                        "token": r.token,
                        "name": r.name,
                        "displayName": r.display_name,
                        "type": r.room_type,
                        "description": r.description,
                        "password": r.password
                    })
                }).collect::<Vec<_>>()
            }
        }
    }))
}

/// Mock get messages endpoint
async fn mock_get_messages(
    axum::extract::Path(room_token): axum::extract::Path<String>,
    State(state): State<MockNextcloudState>,
) -> Json<serde_json::Value> {
    info!("Mock Nextcloud get messages endpoint called for room: {}", room_token);

    let messages = state.messages.lock().unwrap();
    
    Json(serde_json::json!({
        "ocs": {
            "meta": {
                "status": "ok",
                "statusCode": 200
            },
            "data": {
                "messages": messages.iter().filter(|m| m.chat_id.as_deref() == Some(&room_token)).map(|m| {
                    serde_json::json!({
                        "id": m.id,
                        "actorId": m.actor_id,
                        "message": m.message,
                        "timestamp": m.timestamp,
                        "actorType": m.actor_type,
                        "actorDisplayName": m.actor_display_name,
                        "chatId": m.chat_id
                    })
                }).collect::<Vec<_>>()
            }
        }
    }))
}

/// Mock send message endpoint
#[axum::debug_handler]
async fn mock_send_message(
    axum::extract::Path(room_token): axum::extract::Path<String>,
    State(state): State<MockNextcloudState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    info!(
        "Mock Nextcloud send message endpoint called for room: {}",
        room_token
    );

    // Update send message counter
    {
        let mut count = state.send_message_calls.lock().unwrap();
        *count += 1;
    }

    // Extract message text
    let message_text = payload.get("message").and_then(|m| m.as_str()).unwrap_or("");

    // Create a mock message
    let message = NextcloudMessage {
        id: chrono::Utc::now().timestamp(),
        actor_id: "testbot".to_string(),
        message: message_text.to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        actor_type: "user".to_string(),
        actor_display_name: Some("Test Bot".to_string()),
        chat_id: Some(room_token.clone()),
    };

    let mut messages = state.messages.lock().unwrap();
    messages.push(message);

    Json(serde_json::json!({
        "ocs": {
            "meta": {
                "status": "ok",
                "statusCode": 200
            },
            "data": {
                "id": chrono::Utc::now().timestamp(),
                "actorId": "testbot",
                "message": message_text,
                "timestamp": chrono::Utc::now().timestamp(),
                "actorType": "user",
                "actorDisplayName": "Test Bot",
                "chatId": room_token
            }
        }
    }))
}

impl Default for MockNextcloudServer {
    fn default() -> Self {
        panic!("MockNextcloudServer must be started with MockNextcloudServer::start()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_nextcloud_server_starts() {
        let (_url, server) = MockNextcloudServer::start().await;
        assert!(server.url.len() > 0);
        assert!(server.state.get_rooms_calls.lock().unwrap().eq(&0));
    }

    #[tokio::test]
    async fn test_mock_nextcloud_get_rooms() {
        let (_url, server) = MockNextcloudServer::start().await;
        
        let rooms = server.get_rooms().await;
        assert!(!rooms.is_empty());
        assert!(rooms.iter().any(|r| r.name == "Test Room"));
    }
}
