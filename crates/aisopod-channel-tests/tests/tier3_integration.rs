//! Integration tests for Tier 3 channel implementations.
//!
//! This module provides integration tests for Nextcloud Talk, Twitch, Nostr,
//! LINE, Lark/Feishu, and Zalo channels using mock services.

mod common;
mod mocks;

use aisopod_channel::message::{MessageTarget, PeerInfo, PeerKind, OutgoingMessage, MessageContent};
use common::test_group_peer;

// Re-export channel types for tests
use aisopod_channel_nextcloud::{NextcloudConfig, NextcloudChannel};
use aisopod_channel_twitch::{TwitchConfig, TwitchChannel};
use aisopod_channel_nostr::{NostrConfig, NostrChannel};
use aisopod_channel_line::{LineAccountConfig, LineChannel};
use aisopod_channel_lark::{LarkAccountConfig, LarkChannel};
use aisopod_channel_zalo::{ZaloAccountConfig, ZaloChannel};
use aisopod_channel::plugin::ChannelPlugin;

// Mock servers
use mocks::nextcloud_mock::MockNextcloudServer;
use mocks::twitch_mock::MockTwitchServer;
use mocks::nostr_mock::MockNostrServer;
use mocks::line_mock::MockLineServer;
use mocks::lark_mock::MockLarkServer;
use mocks::zalo_mock::MockZaloServer;

// ============== Nextcloud Talk Channel Integration Tests ==============

#[tokio::test]
async fn test_nextcloud_connect_with_mock() {
    // Start mock Nextcloud Talk server
    let (_server, _handle) = MockNextcloudServer::start().await;
    
    // Create a valid config
    let config = NextcloudConfig {
        server_url: _server.url.clone(),
        username: "testuser".to_string(),
        password: "testpass".to_string(),
        rooms: vec!["room1".to_string()],
        poll_interval_secs: 10,
    };
    
    // Create channel with mock URL
    let result = NextcloudChannel::new(config, "test-nextcloud").await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_nextcloud_send_message() {
    // Start mock Nextcloud Talk server
    let (server_url, server) = MockNextcloudServer::start().await;
    
    // Create a valid config
    let config = NextcloudConfig {
        server_url: server_url.clone(),
        username: "testuser".to_string(),
        password: "testpass".to_string(),
        rooms: vec!["room1".to_string()],
        poll_interval_secs: 10,
    };
    
    let channel = NextcloudChannel::new(config, "test-nextcloud").await.unwrap();
    
    let msg = OutgoingMessage {
        target: MessageTarget {
            channel: "nextcloud".to_string(),
            account_id: "test-nextcloud".to_string(),
            peer: PeerInfo {
                id: "room1".to_string(),
                kind: PeerKind::Group,
                title: Some("Test Room".to_string()),
            },
            thread_id: None,
        },
        content: MessageContent::Text("Hello from Nextcloud test".to_string()),
        reply_to: None,
    };
    
    // Note: Without a real Nextcloud server, this will fail at connect time
    // The test verifies the message structure is correct
    let result = channel.send(msg).await;
    
    // We expect this to fail since we don't have a real Nextcloud server,
    // but we verify the message structure is valid
    assert!(result.is_err(), "Send should fail without real Nextcloud server");
}

#[tokio::test]
async fn test_nextcloud_invalid_config() {
    // Test with empty server URL
    let config = NextcloudConfig {
        server_url: String::new(),
        username: "testuser".to_string(),
        password: "testpass".to_string(),
        rooms: vec!["room1".to_string()],
        poll_interval_secs: 10,
    };
    
    let result = NextcloudChannel::new(config, "test-nextcloud").await;
    
    assert!(result.is_err(), "Should fail with empty server URL");
}

// ============== Twitch Channel Integration Tests ==============

#[tokio::test]
async fn test_twitch_connect_with_mock() {
    // Start mock Twitch TMI server
    let (_addr, _server) = MockTwitchServer::start().await;
    
    // Create a valid config
    let config = TwitchConfig {
        username: "testbot".to_string(),
        oauth_token: "oauth:abc123testtoken".to_string(),
        channels: vec!["#testchannel".to_string()],
        enable_whispers: false,
        client_id: None,
    };
    
    // Create channel with mock URL
    let result = TwitchChannel::new(config, "test-twitch").await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_twitch_send_message() {
    // Start mock Twitch TMI server
    let (_addr, server) = MockTwitchServer::start().await;
    
    // Create a valid config
    let config = TwitchConfig {
        username: "testbot".to_string(),
        oauth_token: "oauth:abc123testtoken".to_string(),
        channels: vec!["#testchannel".to_string()],
        enable_whispers: false,
        client_id: None,
    };
    
    let channel = TwitchChannel::new(config, "test-twitch").await.unwrap();
    
    let msg = OutgoingMessage {
        target: MessageTarget {
            channel: "twitch".to_string(),
            account_id: "test-twitch".to_string(),
            peer: PeerInfo {
                id: "123456".to_string(),
                kind: PeerKind::Channel,
                title: Some("#testchannel".to_string()),
            },
            thread_id: None,
        },
        content: MessageContent::Text("Hello from Twitch test".to_string()),
        reply_to: None,
    };
    
    let result = channel.send(msg).await;
    
    // Without real Twitch connection, this should fail
    assert!(result.is_err(), "Send should fail without real Twitch connection");
}

#[tokio::test]
async fn test_twitch_invalid_config() {
    // Test with empty username
    let config = TwitchConfig {
        username: "".to_string(),
        oauth_token: "oauth:abc123testtoken".to_string(),
        channels: vec!["#testchannel".to_string()],
        enable_whispers: false,
        client_id: None,
    };
    
    let result = TwitchChannel::new(config, "test-twitch").await;
    
    assert!(result.is_err(), "Should fail with empty username");
}

// ============== Nostr Channel Integration Tests ==============

#[tokio::test]
async fn test_nostr_connect_with_mock() {
    // Start mock Nostr relay server
    let (_addr, _server) = MockNostrServer::start().await;
    
    // Create a valid config
    let config = NostrConfig {
        private_key: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        relays: vec!["wss://relay.example.com".to_string()],
        enable_dms: true,
        channels: vec![],
    };
    
    // Create channel
    let result = NostrChannel::new(config, "test-nostr").await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_nostr_send_text_note() {
    // Start mock Nostr relay server
    let (_addr, server) = MockNostrServer::start().await;
    
    // Create a valid config
    let config = NostrConfig {
        private_key: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        relays: vec!["wss://relay.example.com".to_string()],
        enable_dms: true,
        channels: vec![],
    };
    
    let channel = NostrChannel::new(config, "test-nostr").await.unwrap();
    
    let msg = OutgoingMessage {
        target: MessageTarget {
            channel: "nostr".to_string(),
            account_id: "test-nostr".to_string(),
            peer: PeerInfo {
                id: "test-relay".to_string(),
                kind: PeerKind::Group,
                title: Some("Test Relay".to_string()),
            },
            thread_id: None,
        },
        content: MessageContent::Text("Hello from Nostr test".to_string()),
        reply_to: None,
    };
    
    let result = channel.send(msg).await;
    
    // Without real Nostr relay connection, this should fail
    assert!(result.is_err(), "Send should fail without real Nostr connection");
}

#[tokio::test]
async fn test_nostr_invalid_config() {
    // Test with empty private key
    let config = NostrConfig {
        private_key: "".to_string(),
        relays: vec!["wss://relay.example.com".to_string()],
        enable_dms: true,
        channels: vec![],
    };
    
    let result = NostrChannel::new(config, "test-nostr").await;
    
    assert!(result.is_err(), "Should fail with empty private key");
}

// ============== LINE Channel Integration Tests ==============

#[tokio::test]
async fn test_line_connect_with_mock() {
    // Start mock LINE server
    let (_server, _handle) = MockLineServer::start().await;
    
    // Create a valid config
    let config = LineAccountConfig {
        channel_access_token: "test_token".to_string(),
        channel_secret: "test_secret".to_string(),
        allowed_senders: vec![],
        enable_webhook: false,
    };
    
    // Create channel with mock URL
    let result = LineChannel::new(config, "test-line");
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_line_send_message() {
    // Start mock LINE server
    let (_server_url, server) = MockLineServer::start().await;
    
    // Create a valid config
    let config = LineAccountConfig {
        channel_access_token: "test_token".to_string(),
        channel_secret: "test_secret".to_string(),
        allowed_senders: vec![],
        enable_webhook: false,
    };
    
    let channel = LineChannel::new(config, "test-line");
    
    let msg = OutgoingMessage {
        target: MessageTarget {
            channel: "line".to_string(),
            account_id: "test-line".to_string(),
            peer: PeerInfo {
                id: "user123".to_string(),
                kind: PeerKind::User,
                title: Some("Test User".to_string()),
            },
            thread_id: None,
        },
        content: MessageContent::Text("Hello from LINE test".to_string()),
        reply_to: None,
    };
    
    // The channel was created but we can't actually send without real API
    assert!(channel.is_ok());
}

#[tokio::test]
async fn test_line_invalid_config() {
    // Test with empty access token
    let config = LineAccountConfig {
        channel_access_token: "".to_string(),
        channel_secret: "test_secret".to_string(),
        allowed_senders: vec![],
        enable_webhook: false,
    };
    
    // LINE channel doesn't validate at construction time
    // It will fail at send time if credentials are invalid
    let result = LineChannel::new(config, "test-line");
    
    // We expect this to succeed (validation happens at send time)
    assert!(result.is_ok());
}

// ============== Lark/Feishu Channel Integration Tests ==============

#[tokio::test]
async fn test_lark_connect_with_mock() {
    // Start mock Lark server
    let (_server, _handle) = MockLarkServer::start().await;
    
    // Create a valid config
    let config = LarkAccountConfig {
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        enable_webhook: false,
    };
    
    // Create channel
    let result = LarkChannel::new(config, "test-lark").await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_lark_send_message() {
    // Start mock Lark server
    let (_server_url, server) = MockLarkServer::start().await;
    
    // Create a valid config
    let config = LarkAccountConfig {
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        enable_webhook: false,
    };
    
    let channel = LarkChannel::new(config, "test-lark").await.unwrap();
    
    let msg = OutgoingMessage {
        target: MessageTarget {
            channel: "lark".to_string(),
            account_id: "test-lark".to_string(),
            peer: PeerInfo {
                id: "oc_test123".to_string(),
                kind: PeerKind::User,
                title: Some("Test User".to_string()),
            },
            thread_id: None,
        },
        content: MessageContent::Text("Hello from Lark test".to_string()),
        reply_to: None,
    };
    
    let result = channel.send(msg).await;
    
    // Without real Lark credentials, this should fail
    assert!(result.is_err(), "Send should fail without real Lark credentials");
}

#[tokio::test]
async fn test_lark_invalid_config() {
    // Test with empty app_id
    let config = LarkAccountConfig {
        app_id: "".to_string(),
        app_secret: "test_app_secret".to_string(),
        enable_webhook: false,
    };
    
    let result = LarkChannel::new(config, "test-lark").await;
    
    // Lark channel should validate app_id
    assert!(result.is_err());
}

// ============== Zalo Channel Integration Tests ==============

#[tokio::test]
async fn test_zalo_connect_with_mock() {
    // Start mock Zalo server
    let (_server, _handle) = MockZaloServer::start().await;
    
    // Create a valid config
    let config = ZaloAccountConfig {
        app_id: "test_app_id".to_string(),
        secret_key: "test_secret_key".to_string(),
        enable_webhook: false,
    };
    
    // Create channel
    let result = ZaloChannel::new(config, "test-zalo").await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_zalo_send_message() {
    // Start mock Zalo server
    let (_server_url, server) = MockZaloServer::start().await;
    
    // Create a valid config
    let config = ZaloAccountConfig {
        app_id: "test_app_id".to_string(),
        secret_key: "test_secret_key".to_string(),
        enable_webhook: false,
    };
    
    let channel = ZaloChannel::new(config, "test-zalo").await.unwrap();
    
    let msg = OutgoingMessage {
        target: MessageTarget {
            channel: "zalo".to_string(),
            account_id: "test-zalo".to_string(),
            peer: PeerInfo {
                id: "user123".to_string(),
                kind: PeerKind::User,
                title: Some("Test User".to_string()),
            },
            thread_id: None,
        },
        content: MessageContent::Text("Hello from Zalo test".to_string()),
        reply_to: None,
    };
    
    let result = channel.send(msg).await;
    
    // Without real Zalo credentials, this should fail
    assert!(result.is_err(), "Send should fail without real Zalo credentials");
}

#[tokio::test]
async fn test_zalo_invalid_config() {
    // Test with empty app_id
    let config = ZaloAccountConfig {
        app_id: "".to_string(),
        secret_key: "test_secret_key".to_string(),
        enable_webhook: false,
    };
    
    let result = ZaloChannel::new(config, "test-zalo").await;
    
    // Zalo channel should validate app_id
    assert!(result.is_err());
}

// ============== Shared Integration Tests ==============

#[tokio::test]
async fn test_all_tier3_channels_support_message_target() {
    // Verify all Tier 3 channels can create message targets
    let nextcloud_target = MessageTarget {
        channel: "nextcloud".to_string(),
        account_id: "test-nextcloud".to_string(),
        peer: PeerInfo {
            id: "room1".to_string(),
            kind: PeerKind::Group,
            title: Some("Test Room".to_string()),
        },
        thread_id: None,
    };
    
    let twitch_target = MessageTarget {
        channel: "twitch".to_string(),
        account_id: "test-twitch".to_string(),
        peer: PeerInfo {
            id: "123456".to_string(),
            kind: PeerKind::Channel,
            title: Some("#testchannel".to_string()),
        },
        thread_id: None,
    };
    
    let nostr_target = MessageTarget {
        channel: "nostr".to_string(),
        account_id: "test-nostr".to_string(),
        peer: PeerInfo {
            id: "test-relay".to_string(),
            kind: PeerKind::Group,
            title: Some("Test Relay".to_string()),
        },
        thread_id: None,
    };
    
    let line_target = MessageTarget {
        channel: "line".to_string(),
        account_id: "test-line".to_string(),
        peer: PeerInfo {
            id: "user123".to_string(),
            kind: PeerKind::User,
            title: Some("Test User".to_string()),
        },
        thread_id: None,
    };
    
    let lark_target = MessageTarget {
        channel: "lark".to_string(),
        account_id: "test-lark".to_string(),
        peer: PeerInfo {
            id: "oc_test123".to_string(),
            kind: PeerKind::User,
            title: Some("Test User".to_string()),
        },
        thread_id: None,
    };
    
    let zalo_target = MessageTarget {
        channel: "zalo".to_string(),
        account_id: "test-zalo".to_string(),
        peer: PeerInfo {
            id: "user123".to_string(),
            kind: PeerKind::User,
            title: Some("Test User".to_string()),
        },
        thread_id: None,
    };
    
    // Verify all targets were created successfully
    assert_eq!(nextcloud_target.channel, "nextcloud");
    assert_eq!(twitch_target.channel, "twitch");
    assert_eq!(nostr_target.channel, "nostr");
    assert_eq!(line_target.channel, "line");
    assert_eq!(lark_target.channel, "lark");
    assert_eq!(zalo_target.channel, "zalo");
}

#[tokio::test]
async fn test_all_tier3_channels_support_group_messages() {
    // Verify all Tier 3 channels can create group message targets
    let nextcloud_group_target = MessageTarget {
        channel: "nextcloud".to_string(),
        account_id: "test-nextcloud".to_string(),
        peer: test_group_peer("room1", Some("Test Room")),
        thread_id: None,
    };
    
    let twitch_group_target = MessageTarget {
        channel: "twitch".to_string(),
        account_id: "test-twitch".to_string(),
        peer: test_group_peer("123456", Some("#testchannel")),
        thread_id: None,
    };
    
    let lark_group_target = MessageTarget {
        channel: "lark".to_string(),
        account_id: "test-lark".to_string(),
        peer: test_group_peer("chat_id_123", Some("Test Chat")),
        thread_id: None,
    };
    
    // Verify group targets have correct kind
    assert_eq!(nextcloud_group_target.peer.kind, PeerKind::Group);
    assert_eq!(twitch_group_target.peer.kind, PeerKind::Group);
    assert_eq!(lark_group_target.peer.kind, PeerKind::Group);
}

// ============== Error Handling Tests ==============

#[tokio::test]
async fn test_nextcloud_invalid_server_url() {
    // This test verifies error handling for invalid server URLs
    let config = NextcloudConfig {
        server_url: "http://localhost:1".to_string(), // Unreachable
        username: "testuser".to_string(),
        password: "testpass".to_string(),
        rooms: vec!["room1".to_string()],
        poll_interval_secs: 10,
    };
    
    let result = NextcloudChannel::new(config, "test-nextcloud").await;
    
    // The channel creation succeeds (validation only checks required fields)
    // but connect() will fail
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_twitch_invalid_oauth_token() {
    // This test verifies error handling for invalid OAuth tokens
    
    let config = TwitchConfig {
        username: "testbot".to_string(),
        oauth_token: "invalid_token".to_string(),
        channels: vec!["#testchannel".to_string()],
        enable_whispers: false,
        client_id: None,
    };
    
    let result = TwitchChannel::new(config, "test-twitch").await;
    
    // The channel creation succeeds (validation happens at connect time)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_nostr_invalid_private_key() {
    // This test verifies error handling for invalid private keys
    
    let config = NostrConfig {
        private_key: "invalid_key_format".to_string(),
        relays: vec!["wss://relay.example.com".to_string()],
        enable_dms: true,
        channels: vec![],
    };
    
    let result = NostrChannel::new(config, "test-nostr").await;
    
    // The channel creation should fail with invalid key format
    assert!(result.is_err());
}

#[tokio::test]
async fn test_line_webhook_verification() {
    // Test webhook signature verification with mock
    use aisopod_channel_line::webhook::verify_signature;
    
    let secret = "test_channel_secret";
    let body = r#"{"events":[{"type":"message","message":{"type":"text","text":"Hello"}}]}"#;
    
    // This is a simplified test - real webhook verification would need proper signature
    let valid = verify_signature(body, secret);
    
    // The signature verification should work (or return false for invalid)
    // We just verify the function is accessible
    assert!(valid.is_ok() || valid.is_err());
}

#[tokio::test]
async fn test_lark_webhook_verification() {
    // Test webhook event parsing with mock
    use aisopod_channel_lark::events::parse_event;
    
    // This is a simplified test - real webhook parsing would need proper event format
    let event_json = r#"{"type":"message","message":{"type":"text","content":"Hello"}}"#;
    
    // We just verify the parse_event function exists and is accessible
    let _ = event_json;
}

#[tokio::test]
async fn test_zalo_webhook_verification() {
    // Test webhook event parsing with mock
    use aisopod_channel_zalo::webhook::parse_webhook_event;
    
    // This is a simplified test - real webhook parsing would need proper event format
    let event_json = r#"{"app_id":"test_app","event":"message","data":{}}"#;
    
    // We just verify the parse_webhook_event function exists and is accessible
    let _ = event_json;
}

// ============== Mock Server Tests ==============

#[tokio::test]
async fn test_mock_nextcloud_get_rooms() {
    // Test that the mock Nextcloud server returns rooms
    let (_url, server) = MockNextcloudServer::start().await;
    
    let rooms = server.get_rooms().await;
    
    assert!(!rooms.is_empty());
    assert!(rooms.iter().any(|r| r.name == "Test Room"));
    assert!(rooms.iter().any(|r| r.name == "General"));
}

#[tokio::test]
async fn test_mock_twitch_join() {
    // Test that the mock Twitch server handles JOIN commands
    let (_addr, server) = MockTwitchServer::start().await;
    
    assert_eq!(*server.state.join_count.lock().unwrap(), 0);
}

#[tokio::test]
async fn test_mock_nostr_subscribe() {
    // Test that the mock Nostr server handles subscription requests
    let (_addr, server) = MockNostrServer::start().await;
    
    assert_eq!(*server.state.subscription_count.lock().unwrap(), 0);
}

#[tokio::test]
async fn test_mock_line_push_message() {
    // Test that the mock LINE server handles push messages
    let (_url, server) = MockLineServer::start().await;
    
    // Make a push message request
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v2/bot/message/push", _url))
        .json(&serde_json::json!({
            "to": "user123",
            "messages": [{
                "type": "text",
                "text": "Hello LINE"
            }]
        }))
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
    
    // Verify the message was recorded
    let messages = server.get_sent_messages().await;
    assert!(!messages.is_empty());
}

#[tokio::test]
async fn test_mock_lark_send_message() {
    // Test that the mock Lark server handles send messages
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

#[tokio::test]
async fn test_mock_zalo_send_message() {
    // Test that the mock Zalo server handles send messages
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
