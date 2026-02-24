//! Model usage skill implementation.
//!
//! This skill provides tools for tracking and reporting token consumption
//! and model usage statistics, enabling agents to understand their resource usage.

use crate::skills::{Skill, SkillCategory, SkillContext, SkillMeta};
use aisopod_tools::{Tool, ToolContext, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// The model usage skill that provides access to usage tracking.
pub struct ModelUsageSkill {
    meta: SkillMeta,
}

impl ModelUsageSkill {
    /// Creates a new ModelUsageSkill instance.
    pub fn new() -> Self {
        Self {
            meta: SkillMeta::new(
                "Model Usage",
                "0.1.0",
                "Track and report model usage and token consumption".to_string(),
                SkillCategory::System,
                vec![],
                vec![],
                None,
            ),
        }
    }
}

impl std::fmt::Debug for ModelUsageSkill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelUsageSkill")
            .field("id", &self.id())
            .field("meta", &self.meta)
            .finish()
    }
}

#[async_trait]
impl Skill for ModelUsageSkill {
    fn id(&self) -> &str {
        "model-usage"
    }

    fn meta(&self) -> &SkillMeta {
        &self.meta
    }

    fn system_prompt_fragment(&self) -> Option<String> {
        Some(
            "You have access to model usage tracking tools. \
             Use `get_usage_summary` for an overview of model usage across sessions. \
             Use `get_token_consumption` for detailed token consumption data."
                .to_string(),
        )
    }

    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        vec![
            Arc::new(GetUsageSummaryTool),
            Arc::new(GetTokenConsumptionTool),
        ]
    }

    async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Tool that gets a summary of model usage.
pub struct GetUsageSummaryTool;

#[async_trait]
impl Tool for GetUsageSummaryTool {
    fn name(&self) -> &str {
        "get_usage_summary"
    }

    fn description(&self) -> &str {
        "Get a summary of model usage including total requests and token counts"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "since": {
                    "type": "string",
                    "description": "ISO 8601 timestamp to filter usage from. Defaults to last 24 hours."
                }
            },
            "required": []
        })
    }

    async fn execute(
        &self,
        _params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult> {
        // For now, return empty usage data as we don't have direct access to the usage tracker
        // The actual implementation would query the UsageTracker via the SkillContext
        let summary = serde_json::json!({
            "total_requests": 0,
            "total_input_tokens": 0,
            "total_output_tokens": 0,
            "total_tokens": 0,
            "models": {},
        });

        Ok(ToolResult {
            content: serde_json::to_string_pretty(&summary)?,
            is_error: false,
            metadata: Some(summary),
        })
    }
}

/// Tool that gets detailed token consumption data.
pub struct GetTokenConsumptionTool;

#[async_trait]
impl Tool for GetTokenConsumptionTool {
    fn name(&self) -> &str {
        "get_token_consumption"
    }

    fn description(&self) -> &str {
        "Get detailed token consumption data broken down by model and session"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "model": {
                    "type": "string",
                    "description": "Filter by model name"
                },
                "session_key": {
                    "type": "string",
                    "description": "Filter by session key"
                }
            },
            "required": []
        })
    }

    async fn execute(
        &self,
        _params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult> {
        // For now, return empty consumption data as we don't have direct access to the usage tracker
        // The actual implementation would query the UsageTracker via the SkillContext
        let data = serde_json::json!({
            "consumption": [],
        });

        Ok(ToolResult {
            content: serde_json::to_string_pretty(&data)?,
            is_error: false,
            metadata: Some(data),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_usage_skill_new() {
        let skill = ModelUsageSkill::new();
        assert_eq!(skill.id(), "model-usage");
        assert_eq!(skill.meta().name, "Model Usage");
        assert_eq!(skill.meta().version, "0.1.0");
        assert_eq!(skill.meta().category, SkillCategory::System);
    }

    #[test]
    fn test_model_usage_skill_system_prompt() {
        let skill = ModelUsageSkill::new();
        let prompt = skill.system_prompt_fragment();
        assert!(prompt.is_some());
        let prompt = prompt.unwrap();
        assert!(prompt.contains("get_usage_summary"));
        assert!(prompt.contains("get_token_consumption"));
    }

    #[test]
    fn test_model_usage_skill_has_tools() {
        let skill = ModelUsageSkill::new();
        let tools = skill.tools();
        assert_eq!(tools.len(), 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(tool_names.contains(&"get_usage_summary"));
        assert!(tool_names.contains(&"get_token_consumption"));
    }

    #[test]
    fn test_get_usage_summary_tool() {
        let tool = GetUsageSummaryTool;
        assert_eq!(tool.name(), "get_usage_summary");
        assert!(tool.description().contains("usage"));
        assert!(tool.description().contains("summary"));
    }

    #[test]
    fn test_get_token_consumption_tool() {
        let tool = GetTokenConsumptionTool;
        assert_eq!(tool.name(), "get_token_consumption");
        assert!(tool.description().contains("token"));
        assert!(tool.description().contains("consumption"));
    }

    #[test]
    fn test_get_usage_summary_tool_schema() {
        let tool = GetUsageSummaryTool;
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["since"].is_object());
        assert!(schema["required"].is_array());
    }

    #[test]
    fn test_get_token_consumption_tool_schema() {
        let tool = GetTokenConsumptionTool;
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["model"].is_object());
        assert!(schema["properties"]["session_key"].is_object());
        assert!(schema["required"].is_array());
    }

    #[tokio::test]
    async fn test_get_usage_summary_execution() {
        let tool = GetUsageSummaryTool;
        let ctx = ToolContext::new("test-agent", "test-session");

        let result = tool.execute(serde_json::json!({}), &ctx).await.unwrap();

        assert!(!result.is_error);
        let output: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(output["total_requests"], 0);
        assert_eq!(output["total_input_tokens"], 0);
        assert_eq!(output["total_output_tokens"], 0);
        assert_eq!(output["total_tokens"], 0);
        assert!(output["models"].is_object());
    }

    #[tokio::test]
    async fn test_get_token_consumption_execution() {
        let tool = GetTokenConsumptionTool;
        let ctx = ToolContext::new("test-agent", "test-session");

        let result = tool.execute(serde_json::json!({}), &ctx).await.unwrap();

        assert!(!result.is_error);
        let output: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert!(output["consumption"].is_array());
    }

    #[tokio::test]
    async fn test_get_token_consumption_execution_with_filters() {
        let tool = GetTokenConsumptionTool;
        let ctx = ToolContext::new("test-agent", "test-session");

        let result = tool
            .execute(
                serde_json::json!({
                    "model": "gpt-4",
                    "session_key": "session-123"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(!result.is_error);
        let output: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert!(output["consumption"].is_array());
    }
}
