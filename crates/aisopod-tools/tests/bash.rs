//! Bash tool tests

use std::time::Duration;

use aisopod_tools::{BashTool, Tool, ToolContext, ToolResult};
use serde_json::json;

#[tokio::test]
async fn test_bash_tool_name() {
    let tool = BashTool::default();
    assert_eq!(tool.name(), "bash");
}

#[tokio::test]
async fn test_bash_tool_description() {
    let tool = BashTool::default();
    assert_eq!(tool.description(), "Execute a shell command");
}

#[tokio::test]
async fn test_bash_tool_schema() {
    let tool = BashTool::default();
    let schema = tool.parameters_schema();

    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["command"].is_object());
    assert_eq!(schema["properties"]["command"]["type"], "string");
    assert!(schema["required"]
        .as_array()
        .unwrap()
        .contains(&json!("command")));
}

#[tokio::test]
async fn test_bash_tool_success() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "command": "echo hello"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("hello"));
}

#[tokio::test]
async fn test_bash_tool_with_args() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "command": "echo arg1=$1 arg2=$2",
                "env": {}
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_bash_tool_with_timeout() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    // Test with short timeout
    let result = tool
        .execute(
            json!({
                "command": "sleep 2",
                "timeout": 1
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("timed out"));
}

#[tokio::test]
async fn test_bash_tool_nonzero_exit() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "command": "exit 1"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_error);
    assert!(output.content.contains("exit code 1"));
}

#[tokio::test]
async fn test_bash_tool_empty_command() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "command": ""
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_error);
    assert!(output.content.contains("empty"));
}

#[tokio::test]
async fn test_bash_tool_missing_command() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "env": {}
            }),
            &ctx,
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_bash_tool_with_env() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "command": "echo $MY_VAR",
                "env": {
                    "MY_VAR": "test_value"
                }
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("test_value"));
}

#[tokio::test]
async fn test_bash_tool_stderr() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "command": "echo stdout; echo stderr >&2"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("stdout"));
    assert!(output.content.contains("stderr"));
}

#[tokio::test]
async fn test_bash_tool_auto_approved_commands() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    // Test echo (auto-approved)
    let result = tool
        .execute(
            json!({
                "command": "echo hello"
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());

    // Test ls (auto-approved)
    let result = tool
        .execute(
            json!({
                "command": "ls -la"
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());

    // Test pwd (auto-approved)
    let result = tool
        .execute(
            json!({
                "command": "pwd"
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());

    // Test whoami (auto-approved)
    let result = tool
        .execute(
            json!({
                "command": "whoami"
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_bash_tool_with_workspace() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session").with_workspace_path("/tmp");

    let result = tool
        .execute(
            json!({
                "command": "pwd"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("/tmp") || output.content.contains("/tmp/"));
}

#[tokio::test]
async fn test_bash_tool_command_timeout() {
    let tool = BashTool::new(Duration::from_secs(30), None);
    let ctx = ToolContext::new("test_agent", "test_session");

    // Command that should complete quickly
    let result = tool
        .execute(
            json!({
                "command": "echo quick"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}

#[tokio::test]
async fn test_bash_tool_with_working_dir() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "command": "pwd",
                "working_dir": "/tmp"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("/tmp"));
}

#[tokio::test]
async fn test_bash_tool_multiple_env_vars() {
    let tool = BashTool::default();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "command": "echo $VAR1 $VAR2",
                "env": {
                    "VAR1": "hello",
                    "VAR2": "world"
                }
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("hello"));
    assert!(output.content.contains("world"));
}

#[tokio::test]
async fn test_bash_tool_with_timeout_override() {
    let tool = BashTool::new(Duration::from_secs(1), None);
    let ctx = ToolContext::new("test_agent", "test_session");

    // Command should complete quickly (no timeout needed)
    let result = tool
        .execute(
            json!({
                "command": "echo fast"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
}
