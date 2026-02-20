//! Message tool tests

use std::sync::Arc;

use aisopod_tools::{MessageSender, MessageTool, NoOpMessageSender, Tool, ToolContext, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

// Custom mock sender for testing
#[derive(Clone)]
struct MockMessageSender {
    sent_messages: Arc<std::sync::RwLock<Vec<String>>>,
}

impl MockMessageSender {
    fn new() -> Self {
        Self {
            sent_messages: Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }

    fn get_messages(&self) -> Vec<String> {
        self.sent_messages.read().unwrap().clone()
    }
}

#[async_trait]
impl MessageSender for MockMessageSender {
    async fn send_message(
        &self,
        channel: &str,
        content: &str,
        _account: Option<&str>,
        _peer: Option<&str>,
    ) -> Result<()> {
        self.sent_messages
            .write()
            .unwrap()
            .push(format!("{}: {}", channel, content));
        Ok(())
    }
}

#[tokio::test]
async fn test_message_tool_name() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    assert_eq!(tool.name(), "message");
}

#[tokio::test]
async fn test_message_tool_description() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    assert_eq!(tool.description(), "Send a message to a channel");
}

#[tokio::test]
async fn test_message_tool_schema() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let schema = tool.parameters_schema();

    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["channel"].is_object());
    assert!(schema["properties"]["content"].is_object());
    assert!(schema["properties"]["account"].is_object());
    assert!(schema["properties"]["peer"].is_object());

    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("channel")));
    assert!(required.contains(&json!("content")));
}

#[tokio::test]
async fn test_message_tool_execute_success() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "channel": "general",
                "content": "Hello, world!"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("Message sent to channel 'general'"));
}

#[tokio::test]
async fn test_message_tool_with_all_params() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "channel": "direct",
                "content": "Private message",
                "account": "slack-workspace",
                "peer": "user-123"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("Message sent to channel 'direct'"));
}

#[tokio::test]
async fn test_message_tool_missing_channel() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "content": "Hello"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Missing required parameter 'channel'"));
}

#[tokio::test]
async fn test_message_tool_missing_content() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "channel": "general"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Missing required parameter 'content'"));
}

#[tokio::test]
async fn test_message_tool_with_mock_sender() {
    let mock_sender = MockMessageSender::new();
    let tool = MessageTool::new(Arc::new(mock_sender.clone()));

    let result = tool
        .execute(
            json!({
                "channel": "test-channel",
                "content": "Test message"
            }),
            &ToolContext::new("test_agent", "test_session"),
        )
        .await;

    assert!(result.is_ok());
    
    // Check the mock sender's messages directly
    let messages = mock_sender.get_messages();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("test-channel"));
    assert!(messages[0].contains("Test message"));
}

#[tokio::test]
async fn test_message_tool_empty_channel() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "channel": "",
                "content": "Hello"
            }),
            &ctx,
        )
        .await;

    // Empty string is still a valid channel name
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_message_tool_empty_content() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "channel": "general",
                "content": ""
            }),
            &ctx,
        )
        .await;

    // Empty content is valid but may not make sense
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_message_tool_special_characters() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "channel": "test-channel-123",
                "content": "Special chars: !@#$%^&*()"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_message_tool_unicode_content() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "channel": "general",
                "content": "Hello 世界 🌍"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_noop_message_sender() {
    let sender = NoOpMessageSender::default();

    // Should succeed without errors
    sender
        .send_message("channel", "content", None, None)
        .await
        .unwrap();

    // With account and peer
    sender
        .send_message("channel", "content", Some("account"), Some("peer"))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_message_tool_with_account_and_peer() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "channel": "dm-user",
                "content": "Direct message",
                "account": "slack-workspace",
                "peer": "user-123"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_message_tool_multiline_content() {
    let tool = MessageTool::new(Arc::new(NoOpMessageSender::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "channel": "general",
                "content": "Line 1\nLine 2\nLine 3"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}
