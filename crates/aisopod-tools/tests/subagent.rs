//! Subagent tool tests

use std::sync::Arc;

use aisopod_tools::{NoOpAgentSpawner, SubagentTool, Tool, ToolContext, ToolResult};
use aisopod_tools::builtins::AgentSpawner;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

// Mock spawner that tracks spawn calls
#[derive(Clone)]
struct MockSpawner {
    spawn_count: std::sync::Arc<std::sync::Mutex<u32>>,
}

impl MockSpawner {
    fn new() -> Self {
        Self {
            spawn_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    fn get_count(&self) -> u32 {
        *self.spawn_count.lock().unwrap()
    }
}

#[async_trait]
impl AgentSpawner for MockSpawner {
    async fn spawn(
        &self,
        _agent_name: &str,
        _prompt: &str,
        _model: &str,
    ) -> Result<String> {
        *self.spawn_count.lock().unwrap() += 1;
        Ok("Task completed successfully".to_string())
    }
}

#[tokio::test]
async fn test_subagent_tool_name() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
    assert_eq!(tool.name(), "subagent");
}

#[tokio::test]
async fn test_subagent_tool_description() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
    assert_eq!(tool.description(), "Spawn a child agent to handle a subtask");
}

#[tokio::test]
async fn test_subagent_tool_schema() {
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
async fn test_subagent_tool_with_mock_spawner() {
    let spawner = MockSpawner::new();
    let tool = SubagentTool::new(Arc::new(spawner.clone()), 3, None);
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "agent_name": "child_agent",
                "prompt": "Do some work",
                "model": "gpt-4"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(spawner.get_count(), 1);
}

#[tokio::test]
async fn test_subagent_tool_depth_limit_enforcement() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 2, None);
    
    // Create context with spawn depth metadata set to 2
    let mut ctx = ToolContext::new("test_agent", "test_session");
    let metadata = json!({
        "spawn_depth": 2
    });
    ctx = ctx.with_metadata(metadata);

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
    assert!(output.is_error);
    assert!(output.content.contains("Maximum spawn depth"));
    assert!(output.content.contains("exceeded"));
}

#[tokio::test]
async fn test_subagent_tool_depth_limit_not_exceeded() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 5, None);
    
    // Create context with spawn depth metadata set to 2 (below limit)
    let mut ctx = ToolContext::new("test_agent", "test_session");
    let metadata = json!({
        "spawn_depth": 2
    });
    ctx = ctx.with_metadata(metadata);

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
}

#[tokio::test]
async fn test_subagent_tool_model_allowlist_enforcement() {
    let tool = SubagentTool::new(
        Arc::new(NoOpAgentSpawner::default()),
        3,
        Some(vec![
            "gpt-4".to_string(),
            "claude-3-opus".to_string(),
            "claude-3-sonnet".to_string(),
        ]),
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
    let output = result.unwrap();
    assert!(!output.is_error);

    // Test another allowed model
    let result = tool
        .execute(
            json!({
                "agent_name": "child_agent",
                "prompt": "Solve the math problem",
                "model": "claude-3-opus"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_subagent_tool_model_allowlist_rejection() {
    let tool = SubagentTool::new(
        Arc::new(NoOpAgentSpawner::default()),
        3,
        Some(vec![
            "gpt-4".to_string(),
            "claude-3-opus".to_string(),
        ]),
    );
    let ctx = ToolContext::new("test_agent", "test_session");

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

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_error);
    assert!(output.content.contains("not in the allowlist"));
    assert!(output.content.contains("gpt-3.5"));
}

#[tokio::test]
async fn test_subagent_tool_model_allowlist_multiple_rejected() {
    let tool = SubagentTool::new(
        Arc::new(NoOpAgentSpawner::default()),
        3,
        Some(vec!["gpt-4".to_string()]),
    );
    let ctx = ToolContext::new("test_agent", "test_session");

    // Test multiple disallowed models
    let disallowed_models = ["claude-3", "gemini-1.5", "llama-3", "gpt-3.5"];

    for model in disallowed_models {
        let result = tool
            .execute(
                json!({
                    "agent_name": "child_agent",
                    "prompt": "Solve the math problem",
                    "model": model
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_error);
        assert!(output.content.contains("not in the allowlist"));
        assert!(output.content.contains(model));
    }
}

#[tokio::test]
async fn test_subagent_tool_with_empty_model_allowlist() {
    // Empty allowlist means no models are allowed
    let tool = SubagentTool::new(
        Arc::new(NoOpAgentSpawner::default()),
        3,
        Some(vec![]),
    );
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
    assert!(output.is_error);
    assert!(output.content.contains("not in the allowlist"));
}

#[tokio::test]
async fn test_subagent_tool_no_allowlist_allows_all() {
    // None allowlist means all models are allowed
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "agent_name": "child_agent",
                "prompt": "Solve the math problem",
                "model": "any-model"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_subagent_tool_with_workspace() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
    let ctx = ToolContext::new("test_agent", "test_session")
        .with_workspace_path("/tmp/test_workspace");

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
}

#[tokio::test]
async fn test_subagent_tool_with_custom_agent_name() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "agent_name": "research_assistant",
                "prompt": "Research the latest advancements in AI",
                "model": "gpt-4"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_subagent_tool_with_complex_prompt() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "agent_name": "complex_agent",
                "prompt": "First, read the file at /tmp/data.json. Then analyze the data and return a summary.",
                "model": "gpt-4"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_subagent_tool_with_max_depth_one() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 1, None);
    let ctx = ToolContext::new("test_agent", "test_session");

    // First spawn should succeed (depth 0 -> 1, limit is 1)
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
}

#[tokio::test]
async fn test_subagent_tool_empty_agent_name() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "agent_name": "",
                "prompt": "Solve the math problem",
                "model": "gpt-4"
            }),
            &ctx,
        )
        .await;

    // Empty string is still a valid parameter value
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_subagent_tool_empty_prompt() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "agent_name": "child",
                "prompt": "",
                "model": "gpt-4"
            }),
            &ctx,
        )
        .await;

    // Empty prompt is valid but may result in empty output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_subagent_tool_empty_model() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "agent_name": "child",
                "prompt": "Solve the math problem",
                "model": ""
            }),
            &ctx,
        )
        .await;

    // Empty model is still a valid parameter value
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_subagent_tool_with_multiline_prompt() {
    let tool = SubagentTool::new(Arc::new(NoOpAgentSpawner::default()), 3, None);
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "agent_name": "multiline_agent",
                "prompt": "Step 1: Analyze the data\nStep 2: Generate report\nStep 3: Send to user",
                "model": "gpt-4"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}
