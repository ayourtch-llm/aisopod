//! Session tool tests

use std::sync::Arc;

use aisopod_tools::{NoOpSessionManager, SessionManager, SessionTool, Tool, ToolContext, ToolResult};
use aisopod_tools::builtins::session::SessionInfo;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::collections::HashMap;

// Mock session manager for testing
#[derive(Clone)]
struct MockSessionManager {
    sessions: Arc<std::sync::Mutex<HashMap<String, SessionInfo>>>,
    messages: Arc<std::sync::Mutex<Vec<String>>>,
    history: Arc<std::sync::Mutex<HashMap<String, Vec<Value>>>>,
}

impl MockSessionManager {
    fn new() -> Self {
        Self {
            sessions: Arc::new(std::sync::Mutex::new(HashMap::new())),
            messages: Arc::new(std::sync::Mutex::new(Vec::new())),
            history: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn add_session(&self, session_id: &str, agent_id: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(
            session_id.to_string(),
            SessionInfo::new(
                session_id,
                agent_id,
                Utc::now(),
                Some(json!({"type": "test"})),
            ),
        );
    }

    fn get_messages(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }

    fn get_history(&self, session_id: &str) -> Vec<Value> {
        self.history
            .lock()
            .unwrap()
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }
}

#[async_trait]
impl SessionManager for MockSessionManager {
    async fn list_sessions(&self, _limit: Option<usize>) -> Result<Vec<SessionInfo>> {
        Ok(self.sessions.lock().unwrap().values().cloned().collect())
    }

    async fn send_to_session(&self, session_id: &str, message: &str) -> Result<()> {
        self.messages
            .lock()
            .unwrap()
            .push(format!("{}: {}", session_id, message));
        Ok(())
    }

    async fn patch_metadata(&self, session_id: &str, metadata: Value) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            let mut current_metadata = session.metadata.clone().unwrap_or(json!({}));
            
            // Merge metadata
            if let (Some(current_obj), Some(new_obj)) = (
                current_metadata.as_object_mut(),
                metadata.as_object(),
            ) {
                for (k, v) in new_obj {
                    let k: String = k.clone();
                    let v: Value = v.clone();
                    current_obj.insert(k, v);
                }
            }
            session.metadata = Some(current_metadata);
        }
        Ok(())
    }

    async fn get_history(&self, session_id: &str, _limit: Option<usize>) -> Result<Vec<Value>> {
        Ok(self
            .history
            .lock()
            .unwrap()
            .get(session_id)
            .cloned()
            .unwrap_or_default())
    }
}

#[tokio::test]
async fn test_session_tool_name() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
    assert_eq!(tool.name(), "session");
}

#[tokio::test]
async fn test_session_tool_description() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
    assert_eq!(tool.description(), "Manage and interact with agent sessions");
}

#[tokio::test]
async fn test_session_tool_schema() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
async fn test_session_tool_execute_list_with_mock() {
    let manager = MockSessionManager::new();
    manager.add_session("session-1", "agent-1");
    manager.add_session("session-2", "agent-2");
    
    let tool = SessionTool::new(Arc::new(manager));
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
    assert!(output.content.contains("session-1"));
    assert!(output.content.contains("session-2"));
}

#[tokio::test]
async fn test_session_tool_execute_send() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
async fn test_session_tool_execute_send_with_mock() {
    let manager = MockSessionManager::new();
    let tool = SessionTool::new(Arc::new(manager.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "send",
                "session_id": "session-456",
                "message": "Test message content"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    
    let messages = manager.get_messages();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("session-456"));
    assert!(messages[0].contains("Test message content"));
}

#[tokio::test]
async fn test_session_tool_execute_patch() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
async fn test_session_tool_execute_patch_with_mock() {
    let manager = MockSessionManager::new();
    let tool = SessionTool::new(Arc::new(manager.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    manager.add_session("session-789", "agent-3");

    let result = tool
        .execute(
            json!({
                "operation": "patch",
                "session_id": "session-789",
                "metadata": {"new_key": "new_value", "updated": true}
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    
    // Verify metadata was merged
    let sessions = manager.sessions.lock().unwrap();
    let session = sessions.get("session-789").unwrap();
    let metadata = session.metadata.as_ref().unwrap();
    
    assert_eq!(metadata["new_key"], "new_value");
    assert_eq!(metadata["updated"], true);
    assert_eq!(metadata["type"], "test"); // Original metadata preserved
}

#[tokio::test]
async fn test_session_tool_execute_history() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
async fn test_session_tool_execute_history_with_mock() {
    let manager = MockSessionManager::new();
    let tool = SessionTool::new(Arc::new(manager.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    // Add some history (scoped to release lock before calling tool.execute)
    {
        let mut history = manager.history.lock().unwrap();
        history.insert(
            "session-history".to_string(),
            vec![
                json!({"type": "user", "content": "Hello"}),
                json!({"type": "assistant", "content": "Hi there"}),
            ],
        );
    } // Lock is released here

    let result = tool
        .execute(
            json!({
                "operation": "history",
                "session_id": "session-history"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("Hello"));
    assert!(output.content.contains("Hi there"));
}

#[tokio::test]
async fn test_session_tool_execute_invalid_operation() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool.execute(json!({}), &ctx).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Missing required parameter 'operation'"));
}

#[tokio::test]
async fn test_session_tool_send_missing_session_id() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
async fn test_session_tool_patch_missing_session_id() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "patch",
                "metadata": {"key": "value"}
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Missing required parameter 'session_id'"));
}

#[tokio::test]
async fn test_session_tool_patch_missing_metadata() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
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
async fn test_session_tool_send_empty_message() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "send",
                "session_id": "session-123",
                "message": ""
            }),
            &ctx,
        )
        .await;

    // Empty message is valid
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_session_tool_with_complex_metadata() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "patch",
                "session_id": "session-123",
                "metadata": {
                    "nested": {
                        "deep": {
                            "value": "test"
                        }
                    },
                    "array": [1, 2, 3],
                    "boolean": true,
                    "number": 42.5
                }
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_session_tool_list_with_limit() {
    let manager = MockSessionManager::new();
    manager.add_session("session-1", "agent-1");
    manager.add_session("session-2", "agent-2");
    manager.add_session("session-3", "agent-3");
    
    let tool = SessionTool::new(Arc::new(manager));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "list",
                "limit": 2
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    // The limit is passed to the manager, but the output format may vary
}

#[tokio::test]
async fn test_session_tool_multiple_operations() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    // First send a message
    let result = tool
        .execute(
            json!({
                "operation": "send",
                "session_id": "session-1",
                "message": "First message"
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());

    // Then patch metadata
    let result = tool
        .execute(
            json!({
                "operation": "patch",
                "session_id": "session-1",
                "metadata": {"status": "active"}
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());

    // Then get history
    let result = tool
        .execute(
            json!({
                "operation": "history",
                "session_id": "session-1"
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_session_tool_noop_manager_list() {
    let manager = NoOpSessionManager::default();

    let sessions = manager.list_sessions(None).await.unwrap();
    assert!(sessions.is_empty());
}

#[tokio::test]
async fn test_session_tool_noop_manager_send() {
    let manager = NoOpSessionManager::default();

    // Should succeed without errors
    manager
        .send_to_session("session-123", "test message")
        .await
        .unwrap();

    // With account and peer
    manager
        .send_to_session("session-456", "another message")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_session_tool_noop_manager_patch() {
    let manager = NoOpSessionManager::default();

    manager
        .patch_metadata("session-123", json!({"key": "value"}))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_session_tool_noop_manager_history() {
    let manager = NoOpSessionManager::default();

    let history = manager.get_history("session-123", None).await.unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn test_session_tool_session_info_new() {
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

#[tokio::test]
async fn test_session_tool_special_characters_in_message() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "send",
                "session_id": "session-123",
                "message": "Special: !@#$%^&*()_+-=[]{}|;':\",./<>?"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_session_tool_multiline_message() {
    let tool = SessionTool::new(Arc::new(NoOpSessionManager::default()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "send",
                "session_id": "session-123",
                "message": "Line 1\nLine 2\nLine 3"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}
