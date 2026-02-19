//! Built-in cron/scheduled task tool for agents.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error};

use crate::{Tool, ToolContext, ToolResult};

/// Represents a scheduled job in the cron system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    /// Unique identifier for the job.
    pub id: String,
    /// Cron expression defining when the job runs.
    pub cron_expression: String,
    /// Command to execute when the job runs.
    pub command: String,
    /// Next scheduled run time.
    pub next_run: Option<DateTime<Utc>>,
    /// Last run time.
    pub last_run: Option<DateTime<Utc>>,
}

/// Trait for managing scheduled jobs.
#[async_trait]
pub trait JobScheduler: Send + Sync {
    /// Schedule a new job with the given cron expression and command.
    async fn schedule(
        &self,
        id: &str,
        cron_expression: &str,
        command: &str,
    ) -> Result<ScheduledJob>;

    /// List all scheduled jobs.
    async fn list(&self) -> Result<Vec<ScheduledJob>>;

    /// Run a job immediately (by ID).
    async fn run_now(&self, id: &str) -> Result<String>;

    /// Remove a scheduled job by ID.
    async fn remove(&self, id: &str) -> Result<bool>;
}

/// A no-op implementation of JobScheduler for testing.
#[derive(Debug, Default, Clone)]
pub struct NoOpJobScheduler {
    jobs: Arc<std::sync::Mutex<HashMap<String, ScheduledJob>>>,
}

impl NoOpJobScheduler {
    /// Creates a new NoOpJobScheduler.
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl JobScheduler for NoOpJobScheduler {
    async fn schedule(
        &self,
        id: &str,
        cron_expression: &str,
        command: &str,
    ) -> Result<ScheduledJob> {
        let schedule = Schedule::from_str(cron_expression)
            .map_err(|e| anyhow!("Invalid cron expression '{}': {}", cron_expression, e))?;

        let next_run = schedule.upcoming(Utc).next();

        let job = ScheduledJob {
            id: id.to_string(),
            cron_expression: cron_expression.to_string(),
            command: command.to_string(),
            next_run,
            last_run: None,
        };

        self.jobs.lock().unwrap().insert(id.to_string(), job.clone());
        Ok(job)
    }

    async fn list(&self) -> Result<Vec<ScheduledJob>> {
        Ok(self.jobs.lock().unwrap().values().cloned().collect())
    }

    async fn run_now(&self, id: &str) -> Result<String> {
        let jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get(id) {
            debug!("Running job {} immediately: {}", id, job.command);
            Ok(format!("Job '{}' would execute: {}", id, job.command))
        } else {
            Err(anyhow!("Job '{}' not found", id))
        }
    }

    async fn remove(&self, id: &str) -> Result<bool> {
        Ok(self.jobs.lock().unwrap().remove(id).is_some())
    }
}

/// A built-in tool for scheduling and managing cron-like recurring tasks.
#[derive(Clone)]
pub struct CronTool {
    /// The job scheduler implementation.
    scheduler: Arc<dyn JobScheduler>,
}

impl std::fmt::Debug for CronTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CronTool").finish()
    }
}

impl CronTool {
    /// Creates a new CronTool with the given scheduler.
    pub fn new(scheduler: Arc<dyn JobScheduler>) -> Self {
        Self { scheduler }
    }

    /// Creates a new CronTool with a no-op scheduler for testing.
    pub fn with_noop_scheduler() -> Self {
        Self::new(Arc::new(NoOpJobScheduler::new()))
    }

    /// Validates a cron expression.
    pub fn validate_cron_expression(&self, expression: &str) -> Result<()> {
        Schedule::from_str(expression)
            .map_err(|e| anyhow!("Invalid cron expression '{}': {}", expression, e))?;
        Ok(())
    }
}

impl Default for CronTool {
    fn default() -> Self {
        Self::with_noop_scheduler()
    }
}

#[async_trait]
impl Tool for CronTool {
    fn name(&self) -> &str {
        "cron"
    }

    fn description(&self) -> &str {
        "Schedule, list, run, and remove recurring tasks"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["schedule", "list", "run", "remove"],
                    "description": "The operation to perform: schedule (add a new job), list (show all jobs), run (execute a job immediately), or remove (delete a job)"
                },
                "cron_expression": {
                    "type": "string",
                    "description": "Cron expression defining when the job runs (required for 'schedule' operation)"
                },
                "command": {
                    "type": "string",
                    "description": "Command to execute when the job runs (required for 'schedule' operation)"
                },
                "job_id": {
                    "type": "string",
                    "description": "Unique identifier for the job (required for 'run' and 'remove' operations)"
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        // Extract operation
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter 'operation'"))?;

        match operation {
            "schedule" => self.execute_schedule(params).await,
            "list" => self.execute_list().await,
            "run" => self.execute_run(params).await,
            "remove" => self.execute_remove(params).await,
            _ => Err(anyhow!("Invalid operation '{}'. Must be one of: schedule, list, run, remove", operation)),
        }
    }
}

impl CronTool {
    async fn execute_schedule(&self, params: Value) -> Result<ToolResult> {
        let cron_expression = params
            .get("cron_expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter 'cron_expression' for 'schedule' operation"))?;

        let command = params
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter 'command' for 'schedule' operation"))?;

        // Validate cron expression
        self.validate_cron_expression(cron_expression)?;

        // Generate a job ID (could be improved with UUID)
        let job_id = format!(
            "job_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );

        debug!("Scheduling job '{}' with expression '{}'", job_id, cron_expression);

        let job = self
            .scheduler
            .schedule(&job_id, cron_expression, command)
            .await?;

        Ok(ToolResult::success(format!(
            "Job '{}' scheduled successfully.\nCron: {}\nCommand: {}",
            job.id, job.cron_expression, job.command
        )))
    }

    async fn execute_list(&self) -> Result<ToolResult> {
        debug!("Listing all scheduled jobs");

        let jobs = self.scheduler.list().await?;

        if jobs.is_empty() {
            return Ok(ToolResult::success("No scheduled jobs found."));
        }

        let mut output = String::from("Scheduled jobs:\n\n");
        for job in &jobs {
            output.push_str(&format!(
                "- ID: {}\n  Cron: {}\n  Command: {}\n  Next Run: {}\n  Last Run: {}\n\n",
                job.id,
                job.cron_expression,
                job.command,
                job.next_run
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_else(|| "N/A".to_string()),
                job.last_run
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_else(|| "N/A".to_string())
            ));
        }

        Ok(ToolResult::success(output))
    }

    async fn execute_run(&self, params: Value) -> Result<ToolResult> {
        let job_id = params
            .get("job_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter 'job_id' for 'run' operation"))?;

        debug!("Running job '{}'", job_id);

        let result = self.scheduler.run_now(job_id).await?;

        Ok(ToolResult::success(result))
    }

    async fn execute_remove(&self, params: Value) -> Result<ToolResult> {
        let job_id = params
            .get("job_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter 'job_id' for 'remove' operation"))?;

        debug!("Removing job '{}'", job_id);

        let removed = self.scheduler.remove(job_id).await?;

        if removed {
            Ok(ToolResult::success(format!(
                "Job '{}' removed successfully.",
                job_id
            )))
        } else {
            Ok(ToolResult::error(format!(
                "Job '{}' not found.",
                job_id
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToolContext;

    #[tokio::test]
    async fn test_cron_tool_name() {
        let tool = CronTool::with_noop_scheduler();
        assert_eq!(tool.name(), "cron");
    }

    #[tokio::test]
    async fn test_cron_tool_description() {
        let tool = CronTool::with_noop_scheduler();
        assert_eq!(tool.description(), "Schedule, list, run, and remove recurring tasks");
    }

    #[tokio::test]
    async fn test_cron_tool_schema() {
        let tool = CronTool::with_noop_scheduler();
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["operation"]["type"], "string");
        assert_eq!(
            schema["properties"]["operation"]["enum"],
            json!(["schedule", "list", "run", "remove"])
        );
        assert!(schema["required"].as_array().unwrap().contains(&json!("operation")));
    }

    #[tokio::test]
    async fn test_schedule_job() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "schedule",
                    "cron_expression": "0 * * * * * *",
                    "command": "echo hello"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("scheduled successfully"));
        assert!(output.content.contains("echo hello"));
    }

    #[tokio::test]
    async fn test_schedule_job_invalid_cron() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "schedule",
                    "cron_expression": "invalid cron",
                    "command": "echo hello"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid cron expression"));
    }

    #[tokio::test]
    async fn test_list_jobs() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        // First, schedule a job
        tool.execute(
            json!({
                "operation": "schedule",
                "cron_expression": "0 * * * * * *",
                "command": "echo test"
            }),
            &ctx,
        )
        .await
        .unwrap();

        // Then list jobs
        let result = tool
            .execute(json!({ "operation": "list" }), &ctx)
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("Scheduled jobs"));
        assert!(output.content.contains("echo test"));
    }

    #[tokio::test]
    async fn test_run_job() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        // First, schedule a job
        let schedule_result = tool
            .execute(
                json!({
                    "operation": "schedule",
                    "cron_expression": "0 * * * * * *",
                    "command": "echo run_test"
                }),
                &ctx,
            )
            .await;

        assert!(schedule_result.is_ok());

        // Extract job_id from the schedule response
        let output = schedule_result.unwrap().content;
        let job_id = output.split("Job '").nth(1).unwrap_or("").split("'").next().unwrap_or("").to_string();

        // Now run the job with the actual job_id
        let result = tool
            .execute(
                json!({
                    "operation": "run",
                    "job_id": job_id
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
    }

    #[tokio::test]
    async fn test_remove_job() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        // First, schedule a job
        let schedule_result = tool
            .execute(
                json!({
                    "operation": "schedule",
                    "cron_expression": "0 * * * * * *",
                    "command": "echo remove_test"
                }),
                &ctx,
            )
            .await;

        assert!(schedule_result.is_ok());

        // Extract job_id from the response (simple approach - in practice might need better parsing)
        let output = schedule_result.unwrap().content;
        let job_id = output
            .lines()
            .find(|line| line.starts_with("- ID:"))
            .map(|line| line.trim_start_matches("- ID:").trim().to_string())
            .unwrap_or_else(|| output.split("Job '").nth(1).unwrap_or("").split("'").next().unwrap_or("").to_string());

        // Now remove the job
        let result = tool
            .execute(
                json!({
                    "operation": "remove",
                    "job_id": job_id
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains("removed successfully"));
    }

    #[tokio::test]
    async fn test_remove_nonexistent_job() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "remove",
                    "job_id": "nonexistent_job"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_error);
        assert!(output.content.contains("not found"));
    }

    #[tokio::test]
    async fn test_missing_operation() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(json!({}), &ctx)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'operation'"));
    }

    #[tokio::test]
    async fn test_invalid_operation() {
        let tool = CronTool::with_noop_scheduler();
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
    async fn test_validate_cron_expression_valid() {
        let tool = CronTool::with_noop_scheduler();
        
        assert!(tool.validate_cron_expression("0 * * * * * *").is_ok());
        assert!(tool.validate_cron_expression("*/5 * * * * * *").is_ok());
        assert!(tool.validate_cron_expression("0 0 * * * * *").is_ok());
    }

    #[tokio::test]
    async fn test_validate_cron_expression_invalid() {
        let tool = CronTool::with_noop_scheduler();
        
        assert!(tool.validate_cron_expression("invalid").is_err());
        // A 7-field expression with too many fields is invalid
        assert!(tool.validate_cron_expression("* * * * * * * *").is_err());
    }

    #[tokio::test]
    async fn test_schedule_missing_cron_expression() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "schedule",
                    "command": "echo test"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'cron_expression'"));
    }

    #[tokio::test]
    async fn test_schedule_missing_command() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "schedule",
                    "cron_expression": "0 * * * * * *"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'command'"));
    }

    #[tokio::test]
    async fn test_run_missing_job_id() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "run"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'job_id'"));
    }

    #[tokio::test]
    async fn test_remove_missing_job_id() {
        let tool = CronTool::with_noop_scheduler();
        let ctx = ToolContext::new("test_agent", "test_session");

        let result = tool
            .execute(
                json!({
                    "operation": "remove"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing required parameter 'job_id'"));
    }
}
