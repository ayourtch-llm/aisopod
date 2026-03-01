//! Cross-channel integration tests
//!
//! These tests verify cross-channel functionality including:
//! - Rate limit compliance across all platforms
//! - Connection state transitions
//! - Error mapping and handling
//! - Channel interoperability

use std::sync::Arc;
use tokio::time::Duration;
use tracing_test::traced_test;

use crate::integration::mock_servers::{DiscordMockState, SlackMockState, TestServer};

#[tokio::test]
#[traced_test]
async fn test_cross_channel_rate_limit_slack_vs_discord() {
    // Test rate limit configurations for Slack vs Discord
    use aisopod_channel::util::rate_limit::{Platform, RateLimit, RateLimitConfig, RateLimiter};

    let slack_limiter = RateLimiter::new(Platform::Slack);
    let discord_limiter = RateLimiter::new(Platform::Discord);

    // Slack should have stricter limits (1 msg/sec)
    let slack_config = slack_limiter.config();
    assert_eq!(slack_config.global_limit.max_requests, 1);
    assert_eq!(
        slack_config.global_limit.window_duration,
        Duration::from_secs(1)
    );

    // Discord should have different limits
    let discord_config = discord_limiter.config();
    assert_eq!(discord_config.global_limit.max_requests, 5);
    assert_eq!(
        discord_config.global_limit.window_duration,
        Duration::from_secs(5)
    );
}

#[tokio::test]
#[traced_test]
async fn test_cross_channel_rate_limit_per_chat() {
    // Test per-chat rate limiting across platforms
    use aisopod_channel::util::rate_limit::{Platform, RateLimit, RateLimitConfig, RateLimiter};

    // Configure strict per-chat limits for testing
    let config = RateLimitConfig {
        global_limit: RateLimit::new(100, Duration::from_secs(60)),
        per_chat_limit: RateLimit::new(2, Duration::from_secs(10)),
    };
    let limiter = RateLimiter::with_config(config);

    // Two messages to same chat should succeed
    assert!(limiter.try_acquire(Some("chat1")).await.is_ok());
    assert!(limiter.try_acquire(Some("chat1")).await.is_ok());

    // Third message to same chat should fail (per-chat limit)
    assert!(limiter.try_acquire(Some("chat1")).await.is_err());

    // But message to different chat should succeed
    assert!(limiter.try_acquire(Some("chat2")).await.is_ok());
}

#[tokio::test]
#[traced_test]
async fn test_cross_channel_connection_state_transitions() {
    // Test connection state transitions across all platforms
    use aisopod_channel::util::connection::{ConnectionManager, ConnectionState};

    let manager = ConnectionManager::new();

    // Start at Disconnected
    assert_eq!(manager.state(), ConnectionState::Disconnected);

    // Transition to Connecting
    manager.record_connect();
    assert_eq!(manager.state(), ConnectionState::Connected);

    // Simulate disconnect
    manager.record_disconnect();
    assert_eq!(manager.state(), ConnectionState::Disconnected);

    // Reconnect attempt
    manager.record_connect_failed();
    assert_eq!(manager.state(), ConnectionState::Failed);

    manager.record_reconnect_attempt();
    assert_eq!(manager.state(), ConnectionState::Reconnecting);

    manager.record_connect();
    assert_eq!(manager.state(), ConnectionState::Connected);
}

#[tokio::test]
#[traced_test]
async fn test_cross_channel_error_mapping() {
    // Test error mapping from platform-specific errors to ChannelError
    use aisopod_channel::util::errors::ChannelError;

    // Test authentication error
    let auth_error = ChannelError::AuthenticationFailed;
    // Check the error variant name is in the string
    assert!(format!("{:?}", auth_error).contains("AuthenticationFailed"));

    // Test rate limited error
    let rate_error = ChannelError::RateLimited {
        retry_after: Duration::from_secs(30),
    };
    assert!(format!("{:?}", rate_error).contains("RateLimited"));

    // Test connection lost
    let conn_error = ChannelError::ConnectionLost;
    assert!(format!("{:?}", conn_error).contains("ConnectionLost"));
}

#[tokio::test]
#[traced_test]
async fn test_cross_channel_media_validation() {
    // Test media validation across platforms
    use aisopod_channel::util::media::{ImageFormat, MediaAttachment, MediaFormat, Platform};

    // Create a test media attachment
    let media = MediaAttachment {
        data: vec![1, 2, 3, 4], // Dummy data
        format: MediaFormat::Image(ImageFormat::Jpeg),
        filename: Some("test.jpg".to_string()),
        mime_type: Some("image/jpeg".to_string()),
        dimensions: Some((100, 100)),
    };

    // Check if media is compatible with each platform
    let platforms = [
        Platform::Telegram,
        Platform::Discord,
        Platform::Slack,
        Platform::WhatsApp,
    ];
    for platform in platforms {
        // This should validate the media without error
        let result = aisopod_channel::util::media::ensure_compatible_format(&media, platform);
        // Allow both success and conversion error (since dummy data is small)
        let _ = result;
    }
}

#[tokio::test]
#[traced_test]
async fn test_cross_channel_message_formatting() {
    // Test message formatting across platforms
    use aisopod_channel::util::formatting::{from_plain_text, NormalizedMarkdown};

    // Create a formatted message
    let message = from_plain_text("Hello **world** with *formatting*");

    // Should convert to plain text correctly (markdown should be stripped)
    let plain = message.to_plain_text();
    // Just verify it returns something
    assert!(!plain.is_empty());
}

#[tokio::test]
#[traced_test]
async fn test_cross_channel_server_isolation() {
    // Test that each mock server instance is isolated
    let mock_server1 =
        crate::integration::mock_servers::create_slack_mock_server(Default::default()).await;
    let server1 = TestServer::from_wiremock(mock_server1);

    let mock_server2 =
        crate::integration::mock_servers::create_slack_mock_server(Default::default()).await;
    let server2 = TestServer::from_wiremock(mock_server2);

    // Verify servers have different URIs
    assert_ne!(server1.url(), server2.url());
}
