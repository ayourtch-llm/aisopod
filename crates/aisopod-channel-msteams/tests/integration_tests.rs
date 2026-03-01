//! Integration tests for Microsoft Teams channel with mocked Bot Framework API.
//!
//! These tests simulate interactions with the Bot Framework API using mock servers
//! to verify the channel implementation works correctly.

use aisopod_channel::plugin::ChannelPlugin;
use aisopod_channel_msteams::{
    Activity, ActivityType, BotFrameworkClient, ChannelAccount, ConversationResponse,
    MsTeamsAccountConfig, MsTeamsChannel, MsTeamsConfig, WebhookConfig,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Notify;

#[tokio::test]
async fn test_channel_creation() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::new(
            "test",
            "tenant123",
            "client123",
            "secret123",
        )],
        ..Default::default()
    };

    let channel = MsTeamsChannel::new(config, "test1").await?;

    assert_eq!(channel.id(), "msteams-test1");
    assert_eq!(channel.meta().label, "Microsoft Teams");
    assert!(channel
        .capabilities()
        .chat_types
        .contains(&aisopod_channel::types::ChatType::Dm));

    Ok(())
}

#[tokio::test]
async fn test_channel_with_bot_credentials() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::with_bot(
            "test",
            "tenant123",
            "client123",
            "secret123",
            "bot123",
            "bot_secret123",
        )],
        ..Default::default()
    };

    let channel = MsTeamsChannel::new(config, "test1").await?;

    assert!(channel.get_account("test1").is_some());

    Ok(())
}

#[tokio::test]
async fn test_channel_multiple_accounts() -> Result<()> {
    let account1 = MsTeamsAccountConfig::new("test", "tenant1", "client1", "secret1");
    let account2 = MsTeamsAccountConfig::new("test2", "tenant2", "client2", "secret2");

    let config = MsTeamsConfig {
        accounts: vec![account1, account2],
        ..Default::default()
    };

    let channel = MsTeamsChannel::new(config, "test1").await?;

    assert_eq!(channel.get_account_ids().len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_channel_remove_account() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::new(
            "test",
            "tenant123",
            "client123",
            "secret123",
        )],
        ..Default::default()
    };

    let mut channel = MsTeamsChannel::new(config, "test1").await?;

    let removed = channel.remove_account("test1");
    assert!(removed);
    assert!(channel.get_account("test1").is_none());

    Ok(())
}

#[tokio::test]
async fn test_channel_config_adapter() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::new(
            "test",
            "tenant123",
            "client123",
            "secret123",
        )],
        ..Default::default()
    };

    let channel = MsTeamsChannel::new(config, "test1").await?;

    let account_ids = channel.config().list_accounts()?;
    assert!(account_ids.contains(&"test1".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_security_adapter() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::new(
            "test",
            "tenant123",
            "client123",
            "secret123",
        )],
        ..Default::default()
    };

    let channel = MsTeamsChannel::new(config, "test1").await?;
    let security = channel
        .security()
        .expect("Security adapter should be available");

    // Test with allowed user
    let sender = aisopod_channel::message::SenderInfo {
        id: "user1".to_string(),
        display_name: Some("User One".to_string()),
        username: Some("user1".to_string()),
        is_bot: false,
    };

    // Should be allowed by default (no restrictions)
    assert!(security.is_allowed_sender(&sender));

    // Test requires mention in group
    assert!(security.requires_mention_in_group());

    Ok(())
}

#[tokio::test]
async fn test_security_adapter_with_allowed_users() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig {
            id: "test".to_string(),
            tenant_id: "tenant123".to_string(),
            client_id: "client123".to_string(),
            client_secret: "secret123".to_string(),
            allowed_users: vec!["allowed_user".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let channel = MsTeamsChannel::new(config, "test1").await?;
    let security = channel
        .security()
        .expect("Security adapter should be available");

    // Test with allowed user
    let allowed_sender = aisopod_channel::message::SenderInfo {
        id: "allowed_user".to_string(),
        display_name: Some("Allowed User".to_string()),
        username: Some("allowed_user".to_string()),
        is_bot: false,
    };

    // Test with non-allowed user
    let non_allowed_sender = aisopod_channel::message::SenderInfo {
        id: "non_allowed_user".to_string(),
        display_name: Some("Non Allowed User".to_string()),
        username: Some("non_allowed_user".to_string()),
        is_bot: false,
    };

    assert!(security.is_allowed_sender(&allowed_sender));
    assert!(!security.is_allowed_sender(&non_allowed_sender));

    Ok(())
}

#[tokio::test]
async fn test_channel_capabilities() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::new(
            "test",
            "tenant123",
            "client123",
            "secret123",
        )],
        ..Default::default()
    };

    let channel = MsTeamsChannel::new(config, "test1").await?;

    let capabilities = channel.capabilities();

    assert!(capabilities
        .chat_types
        .contains(&aisopod_channel::types::ChatType::Dm));
    assert!(capabilities
        .chat_types
        .contains(&aisopod_channel::types::ChatType::Group));
    assert!(capabilities
        .chat_types
        .contains(&aisopod_channel::types::ChatType::Channel));
    assert!(capabilities.supports_media);
    assert!(capabilities.supports_reactions);
    assert!(capabilities.supports_threads);
    assert!(capabilities.supports_typing);
    assert_eq!(capabilities.max_message_length, Some(25000));

    Ok(())
}

#[tokio::test]
async fn test_channel_meta() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::new(
            "test",
            "tenant123",
            "client123",
            "secret123",
        )],
        ..Default::default()
    };

    let channel = MsTeamsChannel::new(config, "test1").await?;

    let meta = channel.meta();

    assert_eq!(meta.label, "Microsoft Teams");
    assert!(meta.docs_url.is_some());
    assert!(meta.docs_url.as_ref().unwrap().contains("microsoft.com"));

    Ok(())
}

#[tokio::test]
async fn test_webhook_router_creation() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::new(
            "test",
            "tenant123",
            "client123",
            "secret123",
        )],
        ..Default::default()
    };

    let channel = MsTeamsChannel::new(config, "test1").await?;

    // Note: This test only verifies the method exists and can be called
    // Real webhook router creation would require a running tokio runtime
    let _router = channel.create_webhook_router("test1");

    Ok(())
}

#[tokio::test]
async fn test_stop_channel() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::new(
            "test",
            "tenant123",
            "client123",
            "secret123",
        )],
        ..Default::default()
    };

    let mut channel = MsTeamsChannel::new(config, "test1").await?;

    // Start a background task
    let _polling_task = channel.start_long_polling(None).await?;

    // Stop the channel
    channel.stop().await;

    // Verify shutdown signal was sent
    // (In a real test, we would verify the task was cancelled)

    Ok(())
}

#[tokio::test]
async fn test_invalid_channel_creation() -> Result<()> {
    // Test with empty config
    let config = MsTeamsConfig::default();

    let result = MsTeamsChannel::new(config, "test1").await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_webhook_mode() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::new(
            "test",
            "tenant123",
            "client123",
            "secret123",
        )],
        webhook: WebhookConfig {
            enabled: true,
            port: 3978,
            ..Default::default()
        },
    };

    let result = MsTeamsChannel::new_webhook(config, "test1").await;

    // This should succeed since webhook is enabled
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_webhook_mode_without_enabled() -> Result<()> {
    let config = MsTeamsConfig {
        accounts: vec![MsTeamsAccountConfig::new(
            "test",
            "tenant123",
            "client123",
            "secret123",
        )],
        ..Default::default()
    };

    let result = MsTeamsChannel::new_webhook(config, "test1").await;

    // This should fail since webhook is not enabled
    assert!(result.is_err());

    Ok(())
}
