//! Integration tests for Telegram channel
//!
//! These tests verify the end-to-end message flow for Telegram using
//! mock API servers.

use tokio::time::Duration;
use tracing_test::traced_test;
use reqwest::Client;

use crate::integration::mock_servers::create_telegram_mock_server;

#[tokio::test]
#[traced_test]
async fn test_telegram_connect_and_receive() {
    // Setup
    let server = create_telegram_mock_server(Default::default()).await;
    
    // Test logic
    let client = Client::new();
    let response: reqwest::Response = client
        .post(format!("{}/getUpdates", server.uri()))
        .json(&serde_json::json!({"offset": 0}))
        .send()
        .await
        .unwrap();
    
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    
    assert!(json["ok"].as_bool().unwrap_or(false));
    assert!(json["result"].as_array().unwrap_or(&vec![]).is_empty());
}

#[tokio::test]
#[traced_test]
async fn test_telegram_send_text() {
    // Setup
    let server = create_telegram_mock_server(Default::default()).await;
    
    // Test logic
    let client = Client::new();
    let response: reqwest::Response = client
        .post(format!("{}/sendMessage", server.uri()))
        .json(&serde_json::json!({
            "chat_id": "12345",
            "text": "Hello, world!",
            "parse_mode": "MarkdownV2"
        }))
        .send()
        .await
        .unwrap();
    
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    
    assert!(json["ok"].as_bool().unwrap_or(false));
}

#[tokio::test]
#[traced_test]
async fn test_telegram_send_media() {
    // Setup
    let server = create_telegram_mock_server(Default::default()).await;
    
    // Test logic
    let client = Client::new();
    let response: reqwest::Response = client
        .post(format!("{}/sendPhoto", server.uri()))
        .json(&serde_json::json!({
            "chat_id": "12345",
            "photo": "https://example.com/image.jpg"
        }))
        .send()
        .await
        .unwrap();
    
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    
    assert!(json["ok"].as_bool().unwrap_or(false));
}

#[tokio::test]
#[traced_test]
async fn test_telegram_reconnect() {
    // Test the connection manager's reconnection logic
    use aisopod_channel::util::connection::ConnectionManager;
    
    let manager = ConnectionManager::new();
    
    // Initial state
    assert_eq!(manager.state(), aisopod_channel::util::connection::ConnectionState::Disconnected);
    
    // Record a failed connection
    manager.record_connect_failed();
    assert_eq!(manager.state(), aisopod_channel::util::connection::ConnectionState::Failed);
    
    // Record a reconnect attempt
    manager.record_reconnect_attempt();
    assert_eq!(manager.state(), aisopod_channel::util::connection::ConnectionState::Reconnecting);
    
    // Record a successful connection
    manager.record_connect();
    assert_eq!(manager.state(), aisopod_channel::util::connection::ConnectionState::Connected);
}

#[tokio::test]
#[traced_test]
async fn test_telegram_rate_limiter() {
    // Test the rate limiter
    use aisopod_channel::util::rate_limit::{RateLimiter, Platform, RateLimitConfig, RateLimit};
    
    // Create a rate limiter with strict limits for testing
    let config = RateLimitConfig {
        global_limit: RateLimit::new(2, Duration::from_secs(1)),
        per_chat_limit: RateLimit::new(2, Duration::from_secs(1)),
    };
    let limiter = RateLimiter::with_config(config);
    
    // First two requests should succeed
    assert!(limiter.try_acquire(None).await.is_ok());
    assert!(limiter.try_acquire(None).await.is_ok());
    
    // Third request should fail
    assert!(limiter.try_acquire(None).await.is_err());
}
