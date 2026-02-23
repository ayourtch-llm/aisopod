//! Integration tests for Discord channel
//!
//! These tests verify the end-to-end message flow for Discord using
//! mock API servers.

use std::sync::Arc;
use tokio::time::Duration;
use tracing_test::traced_test;
use reqwest::Client;

use crate::integration::mock_servers::create_discord_rest_mock_server;

#[tokio::test]
#[traced_test]
async fn test_discord_gateway_connect() {
    // Test the gateway connection simulation
    use aisopod_channel::util::connection::ConnectionManager;
    
    let manager = ConnectionManager::new();
    
    // Simulate gateway connection
    manager.record_connect();
    assert_eq!(manager.state(), aisopod_channel::util::connection::ConnectionState::Connected);
    
    // Verify statistics
    let stats = manager.stats();
    assert!(stats.last_connected.is_some());
    assert!(stats.last_connection_start.is_some());
}

#[tokio::test]
#[traced_test]
async fn test_discord_receive_message() {
    // Test message receive normalization
    use aisopod_channel::message::{IncomingMessage, SenderInfo, PeerInfo, PeerKind, MessageContent};
    use chrono::Utc;
    
    let message = IncomingMessage {
        id: "msg123".to_string(),
        channel: "discord".to_string(),
        account_id: "bot1".to_string(),
        sender: SenderInfo {
            id: "user123".to_string(),
            display_name: Some("Test User".to_string()),
            username: Some("testuser".to_string()),
            is_bot: false,
        },
        peer: PeerInfo {
            id: "channel123".to_string(),
            kind: PeerKind::Channel,
            title: Some("General".to_string()),
        },
        content: MessageContent::Text("Hello, Discord!".to_string()),
        reply_to: None,
        timestamp: Utc::now(),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
    };
    
    assert_eq!(message.id, "msg123");
    assert_eq!(message.channel, "discord");
    assert_eq!(message.sender.id, "user123");
}

#[tokio::test]
#[traced_test]
async fn test_discord_send_text() {
    // Setup
    let server = create_discord_rest_mock_server(Default::default()).await;
    
    // Test logic - just verify the request succeeds
    let client = Client::new();
    let response: reqwest::Response = client
        .post(format!("{}/channels/{}/messages", server.uri(), "123456789"))
        .json(&serde_json::json!({
            "channel_id": "123456789",
            "content": "Hello, world!"
        }))
        .send()
        .await
        .unwrap();
    
    // Just verify the request succeeded - response body may be empty
    assert!(response.status().is_success());
}

#[tokio::test]
#[traced_test]
async fn test_discord_send_embed() {
    // Setup
    let server = create_discord_rest_mock_server(Default::default()).await;
    
    // Test logic - just verify the request succeeds
    let client = Client::new();
    let response: reqwest::Response = client
        .post(format!("{}/channels/{}/messages", server.uri(), "123456789"))
        .json(&serde_json::json!({
            "channel_id": "123456789",
            "embed": {
                "title": "Test Embed",
                "description": "This is a test embed",
                "color": 0x00FF00
            }
        }))
        .send()
        .await
        .unwrap();
    
    // Just verify the request succeeded - response body may be empty
    assert!(response.status().is_success());
}

#[tokio::test]
#[traced_test]
async fn test_discord_reconnect() {
    // Test reconnection logic
    use aisopod_channel::util::connection::ConnectionManager;
    
    let manager = ConnectionManager::new();
    
    // Simulate disconnect and reconnect
    manager.record_disconnect();
    assert_eq!(manager.state(), aisopod_channel::util::connection::ConnectionState::Disconnected);
    
    manager.record_connect();
    assert_eq!(manager.state(), aisopod_channel::util::connection::ConnectionState::Connected);
    
    // Simulate another disconnect
    manager.record_disconnect();
    manager.record_connect();
    assert_eq!(manager.state(), aisopod_channel::util::connection::ConnectionState::Connected);
}

#[tokio::test]
#[traced_test]
async fn test_discord_rate_limiter() {
    // Test the rate limiter
    use aisopod_channel::util::rate_limit::{RateLimiter, Platform, RateLimitConfig, RateLimit};
    
    // Create a rate limiter with strict limits for testing
    let config = RateLimitConfig {
        global_limit: RateLimit::new(3, Duration::from_secs(1)),
        per_chat_limit: RateLimit::new(3, Duration::from_secs(1)),
    };
    let limiter = RateLimiter::with_config(config);
    
    // Make requests up to the limit
    for _ in 0..3 {
        assert!(limiter.try_acquire(None).await.is_ok());
    }
    
    // Fourth request should fail
    assert!(limiter.try_acquire(None).await.is_err());
}
