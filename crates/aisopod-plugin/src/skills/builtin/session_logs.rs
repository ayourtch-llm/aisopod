//! Session logs skill implementation.
//!
//! This skill provides access to session message history and logs,
//! enabling agents to review past conversations and make informed decisions.

use crate::skills::{Skill, SkillCategory, SkillContext, SkillMeta};
use aisopod_tools::{Tool, ToolContext, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// The session logs skill that provides access to message history.
pub struct SessionLogsSkill {
    meta: SkillMeta,
}

impl SessionLogsSkill {
    /// Creates a new SessionLogsSkill instance.
    pub fn new() -> Self {
        Self {
            meta: SkillMeta::new(
                "Session Logs",
                "0.1.0",
                "Access session message history and logs".to_string(),
                SkillCategory::System,
                vec![],
                vec![],
                None,
            ),
        }
    }
}

impl std::fmt::Debug for SessionLogsSkill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionLogsSkill")
            .field("id", &self.id())
            .field("meta", &self.meta)
            .finish()
    }
}

#[async_trait]
impl Skill for SessionLogsSkill {
    fn id(&self) -> &str {
        "session-logs"
    }

    fn meta(&self) -> &SkillMeta {
        &self.meta
    }

    fn system_prompt_fragment(&self) -> Option<String> {
        Some(
            "You have access to session log history. \
             Use `get_session_logs` to retrieve past messages from the current or a specified session."
                .to_string(),
        )
    }

    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        vec![Arc::new(GetSessionLogsTool)]
    }

    async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Tool that retrieves message history for a session.
pub struct GetSessionLogsTool;

#[async_trait]
impl Tool for GetSessionLogsTool {
    fn name(&self) -> &str {
        "get_session_logs"
    }

    fn description(&self) -> &str {
        "Retrieve message history for the current or a specified session"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_key": {
                    "type": "string",
                    "description": "Session key to query. Defaults to the current session if omitted."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of messages to return. Defaults to 50."
                }
            },
            "required": []
        })
    }

    async fn execute(&self, params: serde_json::Value, ctx: &ToolContext) -> Result<ToolResult> {
        let session_key = params
            .get("session_key")
            .and_then(|v| v.as_str())
            .unwrap_or(&ctx.session_key);

        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(50);

        // For now, return empty messages as we don't have direct access to the session store
        // The actual implementation would query the SessionStore via the SkillContext
        let result = serde_json::json!({
            "session_key": session_key,
            "limit": limit,
            "messages": [],
        });

        Ok(ToolResult {
            content: serde_json::to_string_pretty(&result)?,
            is_error: false,
            metadata: Some(result),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_logs_skill_new() {
        let skill = SessionLogsSkill::new();
        assert_eq!(skill.id(), "session-logs");
        assert_eq!(skill.meta().name, "Session Logs");
        assert_eq!(skill.meta().version, "0.1.0");
        assert_eq!(skill.meta().category, SkillCategory::System);
    }

    #[test]
    fn test_session_logs_skill_system_prompt() {
        let skill = SessionLogsSkill::new();
        let prompt = skill.system_prompt_fragment();
        assert!(prompt.is_some());
        let prompt = prompt.unwrap();
        assert!(prompt.contains("get_session_logs"));
    }

    #[test]
    fn test_session_logs_skill_has_tools() {
        let skill = SessionLogsSkill::new();
        let tools = skill.tools();
        assert_eq!(tools.len(), 1);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(tool_names.contains(&"get_session_logs"));
    }

    #[test]
    fn test_get_session_logs_tool() {
        let tool = GetSessionLogsTool;
        assert_eq!(tool.name(), "get_session_logs");
        assert!(tool.description().contains("message"));
        assert!(tool.description().contains("history"));
    }

    #[test]
    fn test_get_session_logs_tool_schema() {
        let tool = GetSessionLogsTool;
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["session_key"].is_object());
        assert!(schema["properties"]["limit"].is_object());
        assert!(schema["required"].is_array());
    }

    #[tokio::test]
    async fn test_get_session_logs_execution_default() {
        let tool = GetSessionLogsTool;
        let ctx = ToolContext::new("test-agent", "test-session");

        let result = tool.execute(serde_json::json!({}), &ctx).await.unwrap();

        assert!(!result.is_error);
        let output: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(output["session_key"], "test-session");
        assert_eq!(output["limit"], 50);
        assert!(output["messages"].is_array());
    }

    #[tokio::test]
    async fn test_get_session_logs_execution_with_params() {
        let tool = GetSessionLogsTool;
        let ctx = ToolContext::new("test-agent", "default-session");

        let result = tool
            .execute(
                serde_json::json!({
                    "session_key": "custom-session",
                    "limit": 100
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(!result.is_error);
        let output: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(output["session_key"], "custom-session");
        assert_eq!(output["limit"], 100);
    }
}
