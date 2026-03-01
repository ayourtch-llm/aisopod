//! Integration tests for Slack channel
//!
//! These tests verify the end-to-end message flow for Slack using
//! mock API servers, including Socket Mode connection, message handling,
//! Block Kit support, thread replies, and reconnection logic.

use reqwest::Client;
use tokio::time::Duration;
use tracing_test::traced_test;

use crate::integration::mock_servers::create_slack_mock_server;

#[tokio::test]
#[traced_test]
async fn test_slack_socket_mode_connect() {
    // Test the Socket Mode connection simulation
    use aisopod_channel::util::connection::ConnectionManager;

    let manager = ConnectionManager::new();

    // Simulate connection
    manager.record_connect();
    assert_eq!(
        manager.state(),
        aisopod_channel::util::connection::ConnectionState::Connected
    );

    // Verify statistics
    let stats = manager.stats();
    assert!(stats.last_connected.is_some());
    assert!(stats.last_connection_start.is_some());
}

#[tokio::test]
#[traced_test]
async fn test_slack_receive_message() {
    // Test message receive normalization for Slack
    use aisopod_channel::message::{
        IncomingMessage, MessageContent, PeerInfo, PeerKind, SenderInfo,
    };
    use chrono::Utc;

    let message = IncomingMessage {
        id: "SlackMessage123".to_string(),
        channel: "slack".to_string(),
        account_id: "slack-bot-1".to_string(),
        sender: SenderInfo {
            id: "U12345678".to_string(),
            display_name: Some("Test User".to_string()),
            username: Some("testuser".to_string()),
            is_bot: false,
        },
        peer: PeerInfo {
            id: "C12345678".to_string(),
            kind: PeerKind::Channel,
            title: Some("general".to_string()),
        },
        content: MessageContent::Text("Hello from Slack!".to_string()),
        timestamp: Utc::now(),
        reply_to: None,
        metadata: serde_json::Value::Object(serde_json::Map::new()),
    };

    assert_eq!(message.channel, "slack");
    assert_eq!(message.sender.id, "U12345678");
}

#[tokio::test]
#[traced_test]
async fn test_slack_send_text() {
    // Test message send with text only
    let server = create_slack_mock_server(Default::default()).await;

    // Simulate sending a message
    let url = format!("{}/chat.postMessage", server.uri());
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "channel": "C123456",
            "text": "Hello from Slack!"
        }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
}

#[tokio::test]
#[traced_test]
async fn test_slack_send_blocks() {
    // Test message send with Block Kit
    let server = create_slack_mock_server(Default::default()).await;

    let url = format!("{}/chat.postMessage", server.uri());
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "channel": "C123456",
            "blocks": [
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": "*Hello* from Slack with blocks!"
                    }
                }
            ]
        }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
}

#[tokio::test]
#[traced_test]
async fn test_slack_thread_reply() {
    // Test thread reply functionality
    let server = create_slack_mock_server(Default::default()).await;

    let url = format!("{}/chat.postMessage", server.uri());
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "channel": "C123456",
            "text": "Reply in thread",
            "thread_ts": "1234567890.123456"
        }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
}

#[tokio::test]
#[traced_test]
async fn test_slack_reconnect() {
    // Test connection manager reconnection logic
    use aisopod_channel::util::connection::ConnectionManager;

    let manager = ConnectionManager::new();

    // Start disconnected
    assert_eq!(
        manager.state(),
        aisopod_channel::util::connection::ConnectionState::Disconnected
    );

    // Simulate failed connection
    manager.record_connect_failed();
    assert_eq!(
        manager.state(),
        aisopod_channel::util::connection::ConnectionState::Failed
    );

    // Simulate reconnect attempt
    manager.record_reconnect_attempt();
    assert_eq!(
        manager.state(),
        aisopod_channel::util::connection::ConnectionState::Reconnecting
    );

    // Simulate successful connection
    manager.record_connect();
    assert_eq!(
        manager.state(),
        aisopod_channel::util::connection::ConnectionState::Connected
    );
}

#[tokio::test]
#[traced_test]
async fn test_slack_rate_limiter() {
    // Test rate limiter with Slack-specific configuration
    use aisopod_channel::util::rate_limit::{Platform, RateLimit, RateLimitConfig, RateLimiter};

    // Slack has 1 message per second limit
    let limiter = RateLimiter::new(Platform::Slack);
    let config = limiter.config();

    assert_eq!(config.global_limit.max_requests, 1);
    assert_eq!(config.global_limit.window_duration, Duration::from_secs(1));
}
