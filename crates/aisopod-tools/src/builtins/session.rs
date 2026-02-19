//! Built-in session management tool for agents to manage sessions.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{Tool, ToolContext, ToolResult};

/// Information about a session.
///
/// Contains metadata about an agent session including its identifier,
/// creation time, and associated metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// The unique identifier of the session.
    pub id: String,
    /// The identifier of the agent that owns this session.
    pub agent_id: String,
    /// The timestamp when the session was created.
    pub created_at: DateTime<Utc>,
    /// Optional metadata associated with the session.
    pub metadata: Option<Value>,
}

impl SessionInfo {
    /// Creates a new SessionInfo with the given values.
    pub fn new(
        id: impl Into<String>,
        agent_id: impl Into<String>,
        created_at: DateTime<Utc>,
        metadata: Option<Value>,
    ) -> Self {
        Self {
            id: id.into(),
            agent_id: agent_id.into(),
            created_at,
            metadata,
        }
    }
}

/// Trait for session management implementations.
///
/// This trait defines the interface for session management backends.
/// Implementations can manage sessions in various ways (in-memory, database, etc.)
#[async_trait]
pub trait SessionManager: Send + Sync {
    /// Lists all active sessions.
    ///
    /// # Arguments
    ///
    /// * `limit` - Optional maximum number of sessions to return.
    ///
    /// # Returns
    ///
    /// Returns a list of session information.
    async fn list_sessions(&self, limit: Option<usize>) -> Result<Vec<SessionInfo>>;

    /// Sends a message to a specific session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The target session identifier.
    /// * `message` - The message content to send.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the message was sent successfully.
    async fn send_to_session(&self, session_id: &str, message: &str) -> Result<()>;

    /// Patches metadata for a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The target session identifier.
    /// * `metadata` - The metadata to patch (will be merged with existing metadata).
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if metadata was updated successfully.
    async fn patch_metadata(&self, session_id: &str, metadata: Value) -> Result<()>;

    /// Gets the message history for a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The target session identifier.
    /// * `limit` - Optional maximum number of messages to return.
    ///
    /// # Returns
    ///
    /// Returns the message history for the session.
    async fn get_history(&self, session_id: &str, limit: Option<usize>) -> Result<Vec<Value>>;
}

/// A built-in tool for managing agent sessions.
///
/// This tool allows agents to manage sessions through various operations:
/// - `list`: List all active sessions
/// - `send`: Send a message to a specific session
/// - `patch`: Patch metadata for a session
/// - `history`: Get message history for a session
///
/// # Parameters
///
/// The tool accepts the following parameters:
///
/// - `operation`: The operation to perform (required): "list", "send", "patch", or "history"
/// - `session_id`: The session identifier (required for send/patch/history operations)
/// - `message`: The message content (required for send operation)
/// - `metadata`: The metadata to patch (required for patch operation)
/// - `limit`: Optional limit on number of results to return
///
/// # Example
///
/// ```json
/// {
///   "operation": "list",
///   "limit": 10
/// }
/// ```
#[derive(Clone)]
pub struct SessionTool {
    /// The session manager implementation.
    manager: Arc<dyn SessionManager>,
}

impl SessionTool {
    /// Creates a new SessionTool with the given manager.
    pub fn new(manager: Arc<dyn SessionManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl Tool for SessionTool {
    fn name(&self) -> &str {
        "session"
    }

    fn description(&self) -> &str {
        "Manage and interact with agent sessions"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["list", "send", "patch", "history"],
                    "description": "The operation to perform: list, send, patch, or history"
                },
                "session_id": {
                    "type": "string",
                    "description": "The session identifier (required for send/patch/history operations)"
                },
                "message": {
                    "type": "string",
                    "description": "The message content to send (required for send operation)"
                },
                "metadata": {
                    "type": "object",
                    "description": "The metadata to patch (required for patch operation)"
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Optional limit on number of results to return"
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        // Extract operation parameter (required)
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'operation'"))?;

        match operation {
            "list" => {
                let limit = params.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
                let sessions = self.manager.list_sessions(limit).await?;
                
                let result = json!({
                    "sessions": sessions.iter().map(|s| {
                        json!({
                            "id": s.id,
                            "agent_id": s.agent_id,
                            "created_at": s.created_at.to_rfc3339(),
                            "metadata": s.metadata
                        })
                    }).collect::<Vec<_>>()
                });
                
                Ok(ToolResult::success(result.to_string()))
            }

            "send" => {
                let session_id = params
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'session_id' for send operation"))?;

                let message = params
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'message' for send operation"))?;

                self.manager.send_to_session(session_id, message).await?;

                Ok(ToolResult::success(format!(
                    "Message sent to session '{}'",
                    session_id
                )))
            }

            "patch" => {
                let session_id = params
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'session_id' for patch operation"))?;

                let metadata = params
                    .get("metadata")
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'metadata' for patch operation"))?
                    .clone();

                self.manager.patch_metadata(session_id, metadata).await?;

                Ok(ToolResult::success(format!(
                    "Metadata patched for session '{}'",
                    session_id
                )))
            }

            "history" => {
                let session_id = params
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'session_id' for history operation"))?;

                let limit = params.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
                let history = self.manager.get_history(session_id, limit).await?;

                let result = json!({
                    "history": history
                });

                Ok(ToolResult::success(result.to_string()))
            }

            _ => {
                Err(anyhow::anyhow!(
                    "Invalid operation '{}'. Must be one of: list, send, patch, history",
                    operation
                ))
            }
        }
    }
}

/// A no-op SessionManager implementation for testing.
///
/// This implementation does nothing and always succeeds. It's useful
/// for testing scenarios where actual session management is not needed.
#[derive(Clone, Default)]
pub struct NoOpSessionManager;

#[async_trait]
impl SessionManager for NoOpSessionManager {
    async fn list_sessions(&self, _limit: Option<usize>) -> Result<Vec<SessionInfo>> {
        Ok(Vec::new())
    }

    async fn send_to_session(&self, _session_id: &str, _message: &str) -> Result<()> {
        // No-op: silently succeeds
        Ok(())
    }

    async fn patch_metadata(&self, _session_id: &str, _metadata: Value) -> Result<()> {
        // No-op: silently succeeds
        Ok(())
    }

    async fn get_history(&self, _session_id: &str, _limit: Option<usize>) -> Result<Vec<Value>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_info_new() {
        let info = SessionInfo::new(
            "session-123",
            "agent-456",
            Utc::now(),
            Some(json!({"key": "value"})),
        );

        assert_eq!(info.id, "session-123");
        assert_eq!(info.agent_id, "agent-456");
        assert!(info.metadata.is_some());
    }

    #[test]
    fn test_session_tool_name() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        assert_eq!(tool.name(), "session");
    }

    #[test]
    fn test_session_tool_description() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        assert_eq!(tool.description(), "Manage and interact with agent sessions");
    }

    #[test]
    fn test_session_tool_schema() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["operation"].is_object());
        assert_eq!(
            schema["properties"]["operation"]["enum"],
            json!(["list", "send", "patch", "history"])
        );
        assert!(schema["properties"]["session_id"].is_object());
        assert!(schema["properties"]["message"].is_object());
        assert!(schema["properties"]["metadata"].is_object());
        assert!(schema["properties"]["limit"].is_object());

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("operation")));
    }

    #[tokio::test]
    async fn test_session_tool_execute_list() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "list"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("\"sessions\":[]"));
    }

    #[tokio::test]
    async fn test_session_tool_execute_list_with_limit() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "list",
                    "limit": 10
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_session_tool_execute_send() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "send",
                    "session_id": "session-123",
                    "message": "Hello, session!"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("Message sent to session"));
    }

    #[tokio::test]
    async fn test_session_tool_execute_patch() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "patch",
                    "session_id": "session-123",
                    "metadata": {"key": "value"}
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("Metadata patched"));
    }

    #[tokio::test]
    async fn test_session_tool_execute_history() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "history",
                    "session_id": "session-123"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("\"history\":[]"));
    }

    #[tokio::test]
    async fn test_session_tool_execute_invalid_operation() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "invalid"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid operation"));
    }

    #[tokio::test]
    async fn test_session_tool_missing_operation() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool.execute(json!({}), &ctx).await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'operation'"));
    }

    #[tokio::test]
    async fn test_session_tool_send_missing_session_id() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "send",
                    "message": "Hello"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'session_id'"));
    }

    #[tokio::test]
    async fn test_session_tool_send_missing_message() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "send",
                    "session_id": "session-123"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'message'"));
    }

    #[tokio::test]
    async fn test_session_tool_patch_missing_metadata() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "patch",
                    "session_id": "session-123"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'metadata'"));
    }

    #[tokio::test]
    async fn test_session_tool_history_missing_session_id() {
        let tool = SessionTool::new(Arc::new(NoOpSessionManager));
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "history"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'session_id'"));
    }

    #[tokio::test]
    async fn test_noop_session_manager() {
        let manager = NoOpSessionManager::default();

        // List sessions should succeed
        let sessions = manager.list_sessions(None).await.unwrap();
        assert!(sessions.is_empty());

        // Send to session should succeed
        manager
            .send_to_session("session-123", "test message")
            .await
            .unwrap();

        // Patch metadata should succeed
        manager
            .patch_metadata("session-123", json!({"key": "value"}))
            .await
            .unwrap();

        // Get history should succeed
        let history = manager.get_history("session-123", None).await.unwrap();
        assert!(history.is_empty());
    }
}
