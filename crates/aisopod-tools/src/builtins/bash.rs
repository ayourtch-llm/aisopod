//! Built-in bash/shell tool for executing shell commands.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use crate::{Tool, ToolContext, ToolResult};

/// A built-in tool that executes shell commands.
///
/// This tool provides a safe way to execute shell commands from within
/// the aisopod system. It supports timeout, working directory, and
/// environment variable configuration.
///
/// # Configuration
///
/// - `default_timeout`: Maximum duration for command execution.
/// - `default_working_dir`: Optional default working directory for commands.
///
/// # Parameters
///
/// The tool accepts the following parameters:
///
/// - `command`: The shell command to execute (required).
/// - `timeout`: Optional timeout in seconds (overrides default).
/// - `working_dir`: Optional working directory (overrides default).
/// - `env`: Optional environment variables as key-value pairs.
///
/// # Example
///
/// ```json
/// {
///   "command": "ls -la",
///   "timeout": 30,
///   "working_dir": "/tmp",
///   "env": {
///     "MY_VAR": "value"
///   }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BashTool {
    /// Default timeout for command execution.
    pub default_timeout: Duration,
    /// Default working directory for commands.
    pub default_working_dir: Option<PathBuf>,
}

impl Default for BashTool {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            default_working_dir: None,
        }
    }
}

impl BashTool {
    /// Creates a new BashTool with the given configuration.
    pub fn new(default_timeout: Duration, default_working_dir: Option<PathBuf>) -> Self {
        Self {
            default_timeout,
            default_working_dir,
        }
    }

    /// Creates a new BashTool with default configuration.
    pub fn with_defaults() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a shell command"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "timeout": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Optional timeout in seconds (overrides default)"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Optional working directory (overrides default)"
                },
                "env": {
                    "type": "object",
                    "additionalProperties": { "type": "string" },
                    "description": "Optional environment variables as key-value pairs"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        // Extract command parameter (required)
        let command_str = params
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter 'command'"))?;

        // Check for empty command
        if command_str.trim().is_empty() {
            return Ok(ToolResult::error("Command cannot be empty"));
        }

        // Extract optional timeout (in seconds)
        let timeout = params
            .get("timeout")
            .and_then(|v| v.as_u64())
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        // Extract optional working directory
        let working_dir = params
            .get("working_dir")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .or(self.default_working_dir.clone());

        // Extract environment variables
        let env_vars: HashMap<String, String> = params
            .get("env")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        // Build the command
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command_str);

        // Set working directory if specified
        if let Some(ref wd) = working_dir {
            cmd.current_dir(wd);
        }

        // Set environment variables
        for (key, value) in &env_vars {
            cmd.env(key, value);
        }

        // Also inherit workspace path from context if no working_dir specified
        if working_dir.is_none() {
            if let Some(ref ws_path) = ctx.workspace_path {
                cmd.current_dir(ws_path);
            }
        }

        // Execute the command
        let output = tokio::time::timeout(timeout, cmd.output())
            .await
            .map_err(|_| anyhow::anyhow!("Command timed out after {} seconds", timeout.as_secs()))?;

        let output = output.map_err(|e| anyhow::anyhow!("Command execution failed: {}", e))?;

        // Parse stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check exit status
        let exit_code = output.status.code();

        match exit_code {
            Some(0) => {
                // Success - combine stdout and stderr for output
                let mut content = stdout.to_string();
                if !stderr.is_empty() {
                    if !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str(&stderr);
                }
                Ok(ToolResult::success(content))
            }
            Some(code) => {
                // Non-zero exit code - mark as error
                let mut error_msg = format!("Command failed with exit code {}", code);
                if !stderr.is_empty() {
                    error_msg.push_str(&format!("\n\nstderr:\n{}", stderr));
                }
                if !stdout.is_empty() {
                    if !stderr.is_empty() {
                        error_msg.push_str(&format!("\n\nstdout:\n{}", stdout));
                    } else {
                        error_msg.push_str(&format!("\nstdout:\n{}", stdout));
                    }
                }
                Ok(ToolResult::error(error_msg))
            }
            None => {
                // Process was terminated by a signal
                Ok(ToolResult::error(
                    "Command was terminated by a signal".to_string(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

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
        // Note: sh -c doesn't pass args by default, need shift
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

    #[test]
    fn test_bash_tool_name() {
        let tool = BashTool::default();
        assert_eq!(tool.name(), "bash");
    }

    #[test]
    fn test_bash_tool_description() {
        let tool = BashTool::default();
        assert_eq!(tool.description(), "Execute a shell command");
    }

    #[test]
    fn test_bash_tool_schema() {
        let tool = BashTool::default();
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["command"].is_object());
        assert!(schema["properties"]["command"]["type"].as_str().unwrap().contains("string"));
        assert!(schema["required"].as_array().unwrap().contains(&json!("command")));
    }
}
