//! Built-in message sending tool for agents to send messages to channels.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{Tool, ToolContext, ToolResult};

/// Trait for message sending implementations.
///
/// This trait defines the interface for message sending backends.
/// Implementations can send messages to various channels (Slack, Discord, etc.)
#[async_trait]
pub trait MessageSender: Send + Sync {
    /// Sends a message to the specified channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - The target channel identifier
    /// * `content` - The message content to send
    /// * `account` - Optional account identifier for multi-account setups
    /// * `peer` - Optional peer identifier for direct messages
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the message was sent successfully.
    async fn send_message(
        &self,
        channel: &str,
        content: &str,
        account: Option<&str>,
        peer: Option<&str>,
    ) -> Result<()>;
}

/// A built-in tool for sending messages to channels.
///
/// This tool allows agents to send messages to various channel types
/// through a unified interface. The actual message delivery is handled
/// by an implementation of the `MessageSender` trait.
///
/// # Parameters
///
/// The tool accepts the following parameters:
///
/// - `channel`: The target channel identifier (required)
/// - `content`: The message content to send (required)
/// - `account`: Optional account identifier for multi-account setups
/// - `peer`: Optional peer identifier for direct messages
///
/// # Example
///
/// ```json
/// {
///   "channel": "general",
///   "content": "Hello, world!",
///   "account": "slack-workspace-1",
///   "peer": "user-123"
/// }
/// ```
#[derive(Clone)]
pub struct MessageTool {
    /// The message sender implementation.
    sender: Arc<dyn MessageSender>,
}

impl MessageTool {
    /// Creates a new MessageTool with the given sender.
    pub fn new(sender: Arc<dyn MessageSender>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl Tool for MessageTool {
    fn name(&self) -> &str {
        "message"
    }

    fn description(&self) -> &str {
        "Send a message to a channel"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "channel": {
                    "type": "string",
                    "description": "The target channel identifier"
                },
                "content": {
                    "type": "string",
                    "description": "The message content to send"
                },
                "account": {
                    "type": "string",
                    "description": "Optional account identifier for multi-account setups"
                },
                "peer": {
                    "type": "string",
                    "description": "Optional peer identifier for direct messages"
                }
            },
            "required": ["channel", "content"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        // Extract channel parameter (required)
        let channel = params
            .get("channel")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'channel'"))?;

        // Extract content parameter (required)
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'content'"))?;

        // Extract optional account parameter
        let account = params.get("account").and_then(|v| v.as_str());

        // Extract optional peer parameter
        let peer = params.get("peer").and_then(|v| v.as_str());

        // Send the message
        self.sender
            .send_message(channel, content, account, peer)
            .await?;

        Ok(ToolResult::success(format!(
            "Message sent to channel '{}'",
            channel
        )))
    }
}

/// A no-op MessageSender implementation for testing.
///
/// This implementation does nothing and always succeeds. It's useful
/// for testing scenarios where actual message delivery is not needed.
#[derive(Clone, Default)]
pub struct NoOpMessageSender;

#[async_trait]
impl MessageSender for NoOpMessageSender {
    async fn send_message(
        &self,
        _channel: &str,
        _content: &str,
        _account: Option<&str>,
        _peer: Option<&str>,
    ) -> Result<()> {
        // No-op: silently succeeds
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_tool_name() {
        let tool = MessageTool::new(Arc::new(NoOpMessageSender));
        assert_eq!(tool.name(), "message");
    }

    #[test]
    fn test_message_tool_description() {
        let tool = MessageTool::new(Arc::new(NoOpMessageSender));
        assert_eq!(tool.description(), "Send a message to a channel");
    }

    #[test]
    fn test_message_tool_schema() {
        let tool = MessageTool::new(Arc::new(NoOpMessageSender));
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
        let tool = MessageTool::new(Arc::new(NoOpMessageSender));
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
        let tool = MessageTool::new(Arc::new(NoOpMessageSender));
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
        let tool = MessageTool::new(Arc::new(NoOpMessageSender));
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
        let tool = MessageTool::new(Arc::new(NoOpMessageSender));
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
    async fn test_noop_sender() {
        let sender = NoOpMessageSender::default();

        // Should succeed without errors
        sender
            .send_message("channel", "content", None, None)
            .await
            .unwrap();
    }
}
