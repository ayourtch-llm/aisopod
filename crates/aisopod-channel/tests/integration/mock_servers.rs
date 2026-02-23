//! Mock API servers for testing channel implementations.
//!
//! This module provides mock servers that simulate the APIs of various
//! messaging platforms (Telegram, Discord, WhatsApp, Slack) for integration
//! testing purposes.

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, path_regex};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// ============================================================================
// Telegram Mock Server
// ============================================================================

/// State for the Telegram mock server
#[derive(Clone, Default)]
pub struct TelegramMockState {
    /// Queue of updates to return from getUpdates
    pub updates_queue: Arc<Mutex<Vec<serde_json::Value>>>,
    /// Count of sendMessage calls
    pub send_message_calls: Arc<Mutex<usize>>,
    /// Count of sendPhoto calls
    pub send_photo_calls: Arc<Mutex<usize>>,
    /// Last received text message
    pub last_text_message: Arc<Mutex<Option<String>>>,
    /// Last received photo data
    pub last_photo_data: Arc<Mutex<Option<Vec<u8>>>>,
    /// Rate limit simulation (429 after this many requests)
    pub rate_limit_after: Arc<Mutex<usize>>,
}

impl TelegramMockState {
    pub fn new() -> Self {
        Self {
            updates_queue: Arc::new(Mutex::new(Vec::new())),
            send_message_calls: Arc::new(Mutex::new(0)),
            send_photo_calls: Arc::new(Mutex::new(0)),
            last_text_message: Arc::new(Mutex::new(None)),
            last_photo_data: Arc::new(Mutex::new(None)),
            rate_limit_after: Arc::new(Mutex::new(100)), // Default: no rate limiting
        }
    }

    pub fn with_rate_limit_after(self, n: usize) -> Self {
        *self.rate_limit_after.lock().unwrap() = n;
        self
    }
}

/// Create a mock Telegram Bot API server
pub async fn create_telegram_mock_server(_state: Arc<TelegramMockState>) -> MockServer {
    let mock_server = MockServer::start().await;
    
    // Mock /getMe - returns simple response
    Mock::given(method("POST"))
        .and(path("/getMe"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "result": {
                "id": 123456789,
                "username": "testbot",
                "first_name": "Test",
                "last_name": "Bot"
            }
        })))
        .mount(&mock_server)
        .await;
    
    // Mock /getUpdates - returns empty updates
    Mock::given(method("POST"))
        .and(path("/getUpdates"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "result": []
        })))
        .mount(&mock_server)
        .await;
    
    // Mock /sendMessage - returns success
    Mock::given(method("POST"))
        .and(path("/sendMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "result": {
                "message_id": 1,
                "chat": {
                    "id": "12345",
                    "type": "private"
                },
                "text": "test"
            }
        })))
        .mount(&mock_server)
        .await;
    
    // Mock /sendPhoto - returns success
    Mock::given(method("POST"))
        .and(path("/sendPhoto"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "result": {
                "message_id": 2,
                "chat": {
                    "id": "12345",
                    "type": "private"
                },
                "photo": []
            }
        })))
        .mount(&mock_server)
        .await;
    
    mock_server
}

// ============================================================================
// Discord Mock Server
// ============================================================================

/// State for the Discord mock server
#[derive(Clone, Default)]
pub struct DiscordMockState {
    /// Received messages
    pub received_messages: Arc<Mutex<Vec<DiscordSendMessage>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiscordSendMessage {
    pub channel_id: String,
    pub content: Option<String>,
    pub embed: Option<serde_json::Value>,
}

impl DiscordMockState {
    pub fn new() -> Self {
        Self {
            received_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Create a mock Discord REST API server
pub async fn create_discord_rest_mock_server(_state: Arc<DiscordMockState>) -> MockServer {
    let mock_server = MockServer::start().await;
    
    // Mock POST /channels/{id}/messages - returns success
    // Use regex matcher to match /channels/{id}/messages pattern
    Mock::given(method("POST"))
        .and(path_regex(r"^/channels/[0-9]+/messages$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "123456789",
            "channel_id": "123456789",
            "content": null,
            "embeds": [],
            "type": 0
        })))
        .mount(&mock_server)
        .await;
    
    mock_server
}

// ============================================================================
// WhatsApp Mock Server
// ============================================================================

/// State for the WhatsApp mock server
#[derive(Clone, Default)]
pub struct WhatsAppMockState {
    /// Received messages
    pub received_messages: Arc<Mutex<Vec<WhatsAppSendMessage>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WhatsAppSendMessage {
    pub to: String,
    pub messaging_product: String,
    pub preview_url: Option<bool>,
    pub text: Option<WhatsAppTextMessage>,
    pub media: Option<WhatsAppMediaMessage>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WhatsAppTextMessage {
    pub body: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WhatsAppMediaMessage {
    pub media_id: String,
}

impl WhatsAppMockState {
    pub fn new() -> Self {
        Self {
            received_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Create a mock WhatsApp Business API server
pub async fn create_whatsapp_mock_server(_state: Arc<WhatsAppMockState>) -> MockServer {
    let mock_server = MockServer::start().await;
    
    // Mock POST /v1/{phone_number_id}/messages - returns success
    Mock::given(method("POST"))
        .and(path("/v1/123456789/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "messages": [{
                "id": "wamid.HBgNNTU1MDAwMDAwMBUA"
            }]
        })))
        .mount(&mock_server)
        .await;
    
    mock_server
}

// ============================================================================
// Slack Mock Server
// ============================================================================

/// State for the Slack mock server
#[derive(Clone, Default)]
pub struct SlackMockState {
    /// Received messages
    pub received_messages: Arc<Mutex<Vec<SlackSendMessage>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SlackSendMessage {
    pub channel: String,
    pub text: Option<String>,
    pub blocks: Option<Vec<serde_json::Value>>,
    pub thread_ts: Option<String>,
}

impl SlackMockState {
    pub fn new() -> Self {
        Self {
            received_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Create a mock Slack Web API server
pub async fn create_slack_mock_server(_state: Arc<SlackMockState>) -> MockServer {
    let mock_server = MockServer::start().await;
    
    // Mock /auth.test - returns success
    Mock::given(method("POST"))
        .and(path("/auth.test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "url": "https://test.slack.com",
            "team": "Test Team",
            "user": "testuser",
            "team_id": "T123456",
            "user_id": "U123456",
            "bot_id": "B123456"
        })))
        .mount(&mock_server)
        .await;
    
    // Mock /apps.connections.open - returns WebSocket URL
    Mock::given(method("POST"))
        .and(path("/apps.connections.open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "url": "wss://mock.slack.com/websocket",
            "sockets": [{
                "url": "wss://mock.slack.com/websocket"
            }]
        })))
        .mount(&mock_server)
        .await;
    
    // Mock /chat.postMessage - returns success
    Mock::given(method("POST"))
        .and(path("/chat.postMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "channel": "C123456",
            "ts": "1234567890.123456",
            "message": {
                "type": "message",
                "channel": "C123456",
                "ts": "1234567890.123456"
            }
        })))
        .mount(&mock_server)
        .await;
    
    mock_server
}

// ============================================================================
// Server Management
// ============================================================================

/// A running mock server wrapper
pub struct TestServer {
    pub server: MockServer,
}

impl TestServer {
    /// Create a new TestServer from a wiremock MockServer
    pub fn from_wiremock(server: MockServer) -> Self {
        Self { server }
    }
    
    /// Get the server URL
    pub fn url(&self) -> String {
        self.server.uri()
    }
}
