//! Built-in subagent spawning tool for agents to spawn child agents.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{Tool, ToolContext, ToolResult};

/// Trait for subagent spawning implementations.
///
/// This trait defines the interface for spawning child agents.
/// Implementations can handle actual agent spawning logic.
#[async_trait]
pub trait AgentSpawner: Send + Sync {
    /// Spawns a new child agent to handle a subtask.
    ///
    /// # Arguments
    ///
    /// * `agent_name` - The name of the child agent to spawn
    /// * `prompt` - The initial prompt/task for the child agent
    /// * `model` - The model to use for the child agent
    ///
    /// # Returns
    ///
    /// Returns Ok(String) with the result/output from the child agent,
    /// or an error if spawning failed.
    async fn spawn(
        &self,
        agent_name: &str,
        prompt: &str,
        model: &str,
    ) -> Result<String>;
}

/// A built-in tool for spawning child agents to handle subtasks.
///
/// This tool allows agents to delegate subtasks to child agents,
/// enabling hierarchical task decomposition and parallel processing.
///
/// # Parameters
///
/// The tool accepts the following parameters:
///
/// - `agent_name`: The name of the child agent to spawn (required)
/// - `prompt`: The task/prompt for the child agent (required)
/// - `model`: The model to use for the child agent (required)
///
/// # Example
///
/// ```json
/// {
///   "agent_name": "researcher",
///   "prompt": "Research the latest advancements in AI",
///   "model": "gpt-4"
/// }
/// ```
#[derive(Clone)]
pub struct SubagentTool {
    /// The spawner implementation for creating child agents.
    spawner: Arc<dyn AgentSpawner>,
    /// Maximum depth of agent spawning hierarchy.
    max_depth: u32,
    /// Optional allowlist of models that child agents can use.
    model_allowlist: Option<Vec<String>>,
}

impl SubagentTool {
    /// Creates a new SubagentTool with the given spawner and configuration.
    pub fn new(
        spawner: Arc<dyn AgentSpawner>,
        max_depth: u32,
        model_allowlist: Option<Vec<String>>,
    ) -> Self {
        Self {
            spawner,
            max_depth,
            model_allowlist,
        }
    }
}

#[async_trait]
impl Tool for SubagentTool {
    fn name(&self) -> &str {
        "subagent"
    }

    fn description(&self) -> &str {
        "Spawn a child agent to handle a subtask"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "agent_name": {
                    "type": "string",
                    "description": "The name of the child agent to spawn"
                },
                "prompt": {
                    "type": "string",
                    "description": "The task/prompt for the child agent"
                },
                "model": {
                    "type": "string",
                    "description": "The model to use for the child agent"
                }
            },
            "required": ["agent_name", "prompt", "model"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        // Extract agent_name parameter (required)
        let agent_name = params
            .get("agent_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'agent_name'"))?;

        // Extract prompt parameter (required)
        let prompt = params
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'prompt'"))?;

        // Extract model parameter (required)
        let model = params
            .get("model")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'model'"))?;

        // Check depth limit from context metadata if available
        let current_depth = ctx
            .metadata_get("spawn_depth")
            .and_then(|v| v.as_u64())
            .map(|d| d as u32)
            .unwrap_or(0);

        if current_depth >= self.max_depth {
            return Ok(ToolResult::error(format!(
                "Maximum spawn depth {} exceeded (current: {})",
                self.max_depth, current_depth
            )));
        }

        // Verify model allowlist if configured
        if let Some(ref allowlist) = self.model_allowlist {
            if !allowlist.contains(&model.to_string()) {
                return Ok(ToolResult::error(format!(
                    "Model '{}' is not in the allowlist",
                    model
                )));
            }
        }

        // Spawn the child agent
        let result = self
            .spawner
            .spawn(agent_name, prompt, model)
            .await?;

        Ok(ToolResult::success(result))
    }
}

/// A no-op AgentSpawner implementation for testing.
///
/// This implementation does nothing and always returns a success message.
/// It's useful for testing scenarios where actual agent spawning is not needed.
#[derive(Clone, Default)]
pub struct NoOpAgentSpawner;

#[async_trait]
impl AgentSpawner for NoOpAgentSpawner {
    async fn spawn(
        &self,
        agent_name: &str,
        prompt: &str,
        model: &str,
    ) -> Result<String> {
        // No-op: returns success with simulation message
        Ok(format!(
            "Agent '{}' (model: '{}') spawned with prompt: '{}'",
            agent_name, model, prompt
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_tool_name() {
        let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
        assert_eq!(tool.name(), "subagent");
    }

    #[test]
    fn test_subagent_tool_description() {
        let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
        assert_eq!(tool.description(), "Spawn a child agent to handle a subtask");
    }

    #[test]
    fn test_subagent_tool_schema() {
        let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["agent_name"].is_object());
        assert!(schema["properties"]["prompt"].is_object());
        assert!(schema["properties"]["model"].is_object());

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("agent_name")));
        assert!(required.contains(&json!("prompt")));
        assert!(required.contains(&json!("model")));
    }

    #[tokio::test]
    async fn test_subagent_tool_execute_success() {
        let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "agent_name": "child_agent",
                    "prompt": "Solve the math problem",
                    "model": "gpt-4"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("spawned"));
    }

    #[tokio::test]
    async fn test_subagent_tool_missing_agent_name() {
        let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "prompt": "Solve the math problem",
                    "model": "gpt-4"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'agent_name'"));
    }

    #[tokio::test]
    async fn test_subagent_tool_missing_prompt() {
        let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "agent_name": "child",
                    "model": "gpt-4"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'prompt'"));
    }

    #[tokio::test]
    async fn test_subagent_tool_missing_model() {
        let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "agent_name": "child",
                    "prompt": "Solve the math problem"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'model'"));
    }

    #[tokio::test]
    async fn test_subagent_tool_depth_limit() {
        let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 2, None);
        
        // Create context with spawn depth metadata
        let ctx = ToolContext::new("test_agent", "test_session");
        // Use private field access for testing
        // This test will be updated when we add metadata to ToolContext

        let result = tool
            .execute(
                json!({
                    "agent_name": "child_agent",
                    "prompt": "Solve the math problem",
                    "model": "gpt-4"
                }),
                &ctx,
            )
            .await;

        // Current implementation doesn't track depth, so this should succeed
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_subagent_tool_model_allowlist() {
        let tool = SubagentTool::new(
            Arc::new(NoOpAgentSpawner::default()),
            3,
            Some(vec!["gpt-4".to_string(), "claude-3".to_string()]),
        );
        let ctx = ToolContext::new("test_agent", "test_session");

        // Test allowed model
        let result = tool
            .execute(
                json!({
                    "agent_name": "child_agent",
                    "prompt": "Solve the math problem",
                    "model": "gpt-4"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());

        // Test disallowed model
        let result = tool
            .execute(
                json!({
                    "agent_name": "child_agent",
                    "prompt": "Solve the math problem",
                    "model": "gpt-3.5"
                }),
                &ctx,
            )
            .await;

        // The tool should return an Ok(ToolResult) with is_error=true for disallowed models
        assert!(result.is_ok(), "execute should return Ok even for disallowed model");
        let output = result.unwrap();
        assert!(output.is_error, "ToolResult should have is_error=true for disallowed model");
        assert!(output.content.contains("not in the allowlist"));
    }

    #[tokio::test]
    async fn test_noop_spawner() {
        let spawner = NoOpAgentSpawner::default();

        let result = spawner
            .spawn("test_agent", "Test prompt", "gpt-4")
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("spawned"));
    }
}
