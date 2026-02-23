//! Integration tests for WhatsApp channel
//!
//! These tests verify the end-to-end message flow for WhatsApp using
//! mock API servers.

use std::sync::Arc;
use tokio::time::Duration;
use tracing_test::traced_test;
use reqwest::Client;

use crate::integration::mock_servers::create_whatsapp_mock_server;

#[tokio::test]
#[traced_test]
async fn test_whatsapp_receive_message() {
    // Test message receive normalization
    use aisopod_channel::message::{IncomingMessage, SenderInfo, PeerInfo, PeerKind, MessageContent};
    use chrono::Utc;
    
    let message = IncomingMessage {
        id: "msg123".to_string(),
        channel: "whatsapp".to_string(),
        account_id: "test-account".to_string(),
        sender: SenderInfo {
            id: "15551234567".to_string(),
            display_name: None,
            username: None,
            is_bot: false,
        },
        peer: PeerInfo {
            id: "15559999999".to_string(),
            kind: PeerKind::User,
            title: None,
        },
        content: MessageContent::Text("Hello, WhatsApp!".to_string()),
        reply_to: None,
        timestamp: Utc::now(),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
    };
    
    assert_eq!(message.id, "msg123");
    assert_eq!(message.channel, "whatsapp");
    assert_eq!(message.sender.id, "15551234567");
}

#[tokio::test]
#[traced_test]
async fn test_whatsapp_send_text() {
    // Setup
    let server = create_whatsapp_mock_server(Default::default()).await;
    
    // Test logic
    let client = Client::new();
    let response = client
        .post(format!("{}/v1/{}/messages", server.uri(), "123456789"))
        .json(&serde_json::json!({
            "to": "15559999999",
            "messaging_product": "whatsapp",
            "messages": [{
                "type": "text",
                "text": {
                    "body": "Hello, world!"
                }
            }]
        }))
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
}

#[tokio::test]
#[traced_test]
async fn test_whatsapp_send_media() {
    // Test media send structure (actual media upload would be tested with real API)
    let server = create_whatsapp_mock_server(Default::default()).await;
    
    // Test logic - media message structure
    let client = Client::new();
    let response = client
        .post(format!("{}/v1/{}/messages", server.uri(), "123456789"))
        .json(&serde_json::json!({
            "to": "15559999999",
            "messaging_product": "whatsapp",
            "messages": [{
                "type": "image",
                "image": {
                    "media_id": "media_id_123"
                }
            }]
        }))
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
}

#[tokio::test]
#[traced_test]
async fn test_whatsapp_allowed_number_filter() {
    // Test message filtering based on allowed numbers
    use aisopod_channel::message::{IncomingMessage, SenderInfo, PeerInfo, PeerKind, MessageContent};
    use chrono::Utc;
    
    // Create a message from a non-allowed number
    let message = IncomingMessage {
        id: "msg123".to_string(),
        channel: "whatsapp".to_string(),
        account_id: "test-account".to_string(),
        sender: SenderInfo {
            id: "15559999999".to_string(), // Not in allowed list
            display_name: None,
            username: None,
            is_bot: false,
        },
        peer: PeerInfo {
            id: "15559999999".to_string(),
            kind: PeerKind::User,
            title: None,
        },
        content: MessageContent::Text("Hello".to_string()),
        reply_to: None,
        timestamp: Utc::now(),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
    };
    
    // Simulate filtering logic
    let allowed_numbers = vec!["15551234567".to_string(), "15552345678".to_string()];
    let is_allowed = allowed_numbers.contains(&message.sender.id);
    
    assert!(!is_allowed); // Should be filtered out
}

#[tokio::test]
#[traced_test]
async fn test_whatsapp_rate_limiter() {
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
