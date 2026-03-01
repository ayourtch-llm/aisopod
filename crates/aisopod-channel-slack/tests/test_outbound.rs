//! Tests for OutboundAdapter implementation

use aisopod_channel::adapters::OutboundAdapter;
use aisopod_channel::message::{Media, MessageTarget, PeerInfo, PeerKind};
use aisopod_channel::plugin::ChannelPlugin;
use aisopod_channel::types::MediaType;
use aisopod_channel_slack::{SlackAccountConfig, SlackChannel};
use anyhow::Result;

#[tokio::test]
async fn test_outbound_adapter_send_text() -> Result<()> {
    let config = SlackAccountConfig {
        bot_token: "xoxb-test".to_string(),
        ..Default::default()
    };

    let mut channel = SlackChannel::new(config, "test-account").await?;

    let target = MessageTarget {
        channel: "slack-test".to_string(),
        account_id: "test-account".to_string(),
        peer: PeerInfo {
            id: "C123456".to_string(),
            kind: PeerKind::Channel,
            title: None,
        },
        thread_id: None,
    };

    // Note: This will fail without a real Slack connection, but we can verify the structure
    // The actual API call will be mocked by the test
    let result = channel.send_text(&target, "Hello, world!").await;

    // We expect this to fail because there's no real connection
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_outbound_adapter_send_media() -> Result<()> {
    let config = SlackAccountConfig {
        bot_token: "xoxb-test".to_string(),
        ..Default::default()
    };

    let mut channel = SlackChannel::new(config, "test-account").await?;

    let target = MessageTarget {
        channel: "slack-test".to_string(),
        account_id: "test-account".to_string(),
        peer: PeerInfo {
            id: "C123456".to_string(),
            kind: PeerKind::Channel,
            title: None,
        },
        thread_id: None,
    };

    let media = Media {
        media_type: MediaType::Image,
        url: None,
        data: Some(vec![1, 2, 3, 4]),
        filename: Some("test.png".to_string()),
        mime_type: Some("image/png".to_string()),
        size_bytes: Some(4),
    };

    // Note: This will fail without a real Slack connection
    let result = channel.send_media(&target, &media).await;

    // We expect this to fail because there's no real connection
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_slack_config_adapter_list_accounts() -> Result<()> {
    let config = SlackAccountConfig {
        bot_token: "xoxb-test".to_string(),
        ..Default::default()
    };

    let channel = SlackChannel::new(config, "test-account").await?;
    let config_adapter = channel.config();

    let accounts = config_adapter.list_accounts()?;
    assert!(accounts.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_slack_config_adapter_resolve_account() -> Result<()> {
    let config = SlackAccountConfig {
        bot_token: "xoxb-test".to_string(),
        ..Default::default()
    };

    let channel = SlackChannel::new(config, "test-account").await?;
    let config_adapter = channel.config();

    // Try to resolve a non-existent account
    let result = config_adapter.resolve_account("non-existent");
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_slack_config_adapter_delete_account() -> Result<()> {
    let config = SlackAccountConfig {
        bot_token: "xoxb-test".to_string(),
        ..Default::default()
    };

    let channel = SlackChannel::new(config, "test-account").await?;
    let config_adapter = channel.config();

    // Try to delete a non-existent account
    let result = config_adapter.delete_account("non-existent");
    assert!(result.is_err());

    Ok(())
}
