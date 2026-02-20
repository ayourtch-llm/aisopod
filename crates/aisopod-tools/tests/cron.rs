//! Cron tool tests

use std::sync::Arc;

use aisopod_tools::{
    CronTool, JobScheduler, NoOpJobScheduler, ScheduledJob, Tool, ToolContext, ToolResult,
};
use anyhow::Result;
use async_trait::async_trait;
use cron::Schedule;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;

// Mock job scheduler for testing
#[derive(Clone)]
struct MockJobScheduler {
    jobs: Arc<std::sync::Mutex<HashMap<String, ScheduledJob>>>,
}

impl MockJobScheduler {
    fn new() -> Self {
        Self {
            jobs: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn get_job_ids(&self) -> Vec<String> {
        self.jobs.lock().unwrap().keys().cloned().collect()
    }

    fn get_job(&self, id: &str) -> Option<ScheduledJob> {
        self.jobs.lock().unwrap().get(id).cloned()
    }
}

#[async_trait]
impl JobScheduler for MockJobScheduler {
    async fn schedule(
        &self,
        id: &str,
        cron_expression: &str,
        command: &str,
    ) -> Result<ScheduledJob> {
        // Use the cron crate to parse and validate
        let schedule = cron::Schedule::from_str(cron_expression)
            .map_err(|e| anyhow::anyhow!("Invalid cron expression '{}': {}", cron_expression, e))?;

        let next_run = schedule.upcoming(chrono::Utc).next();

        let job = ScheduledJob {
            id: id.to_string(),
            cron_expression: cron_expression.to_string(),
            command: command.to_string(),
            next_run,
            last_run: None,
        };

        self.jobs
            .lock()
            .unwrap()
            .insert(id.to_string(), job.clone());
        Ok(job)
    }

    async fn list(&self) -> Result<Vec<ScheduledJob>> {
        Ok(self.jobs.lock().unwrap().values().cloned().collect())
    }

    async fn run_now(&self, id: &str) -> Result<String> {
        let jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get(id) {
            Ok(format!("Job '{}' executed: {}", id, job.command))
        } else {
            Err(anyhow::anyhow!("Job '{}' not found", id))
        }
    }

    async fn remove(&self, id: &str) -> Result<bool> {
        Ok(self.jobs.lock().unwrap().remove(id).is_some())
    }
}

#[tokio::test]
async fn test_cron_tool_name() {
    let tool = CronTool::with_noop_scheduler();
    assert_eq!(tool.name(), "cron");
}

#[tokio::test]
async fn test_cron_tool_description() {
    let tool = CronTool::with_noop_scheduler();
    assert_eq!(
        tool.description(),
        "Schedule, list, run, and remove recurring tasks"
    );
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
    assert!(schema["properties"]["cron_expression"].is_object());
    assert!(schema["properties"]["command"].is_object());
    assert!(schema["properties"]["job_id"].is_object());

    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("operation")));
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
    let result = tool.execute(json!({ "operation": "list" }), &ctx).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("Scheduled jobs"));
    assert!(output.content.contains("echo test"));
}

#[tokio::test]
async fn test_list_jobs_empty() {
    let tool = CronTool::with_noop_scheduler();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool.execute(json!({ "operation": "list" }), &ctx).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("No scheduled jobs"));
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
    let job_id = output
        .lines()
        .find(|line| line.starts_with("- ID:"))
        .map(|line| line.trim_start_matches("- ID:").trim().to_string())
        .unwrap_or_else(|| {
            output
                .split("Job '")
                .nth(1)
                .unwrap_or("")
                .split("'")
                .next()
                .unwrap_or("")
                .to_string()
        });

    // Now run the job
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

    // Extract job_id from the response
    let output = schedule_result.unwrap().content;
    let job_id = output
        .lines()
        .find(|line| line.starts_with("- ID:"))
        .map(|line| line.trim_start_matches("- ID:").trim().to_string())
        .unwrap_or_else(|| {
            output
                .split("Job '")
                .nth(1)
                .unwrap_or("")
                .split("'")
                .next()
                .unwrap_or("")
                .to_string()
        });

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

    let result = tool.execute(json!({}), &ctx).await;

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
    assert!(tool.validate_cron_expression("0 0/30 * * * ? *").is_ok());
    assert!(tool
        .validate_cron_expression("0 0 12 ? * MON-FRI *")
        .is_ok());
}

#[tokio::test]
async fn test_validate_cron_expression_invalid() {
    let tool = CronTool::with_noop_scheduler();

    assert!(tool.validate_cron_expression("invalid").is_err());
    assert!(tool.validate_cron_expression("* * * * * * * *").is_err()); // Too many fields
    assert!(tool.validate_cron_expression("not a cron").is_err());
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

#[tokio::test]
async fn test_schedule_with_mock_scheduler() {
    let scheduler = MockJobScheduler::new();
    let tool = CronTool::new(Arc::new(scheduler.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "schedule",
                "cron_expression": "0 * * * * * *",
                "command": "echo mock_test"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(scheduler.get_job_ids().len(), 1);
}

#[tokio::test]
async fn test_list_with_mock_scheduler() {
    let scheduler = MockJobScheduler::new();
    scheduler
        .schedule("job-1", "0 * * * * * *", "echo job1")
        .await
        .unwrap();
    scheduler
        .schedule("job-2", "0 */5 * * * * *", "echo job2")
        .await
        .unwrap();

    let tool = CronTool::new(Arc::new(scheduler.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool.execute(json!({ "operation": "list" }), &ctx).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("job-1"));
    assert!(output.content.contains("job-2"));
}

#[tokio::test]
async fn test_run_with_mock_scheduler() {
    let scheduler = MockJobScheduler::new();
    scheduler
        .schedule("test-job", "0 * * * * * *", "echo test_command")
        .await
        .unwrap();

    let tool = CronTool::new(Arc::new(scheduler.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "run",
                "job_id": "test-job"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_error);
    assert!(output.content.contains("test_command"));
}

#[tokio::test]
async fn test_remove_with_mock_scheduler() {
    let scheduler = MockJobScheduler::new();
    scheduler
        .schedule("removable-job", "0 * * * * * *", "echo remove_me")
        .await
        .unwrap();
    assert_eq!(scheduler.get_job_ids().len(), 1);

    let tool = CronTool::new(Arc::new(scheduler.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "remove",
                "job_id": "removable-job"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(scheduler.get_job_ids().len(), 0);
}

#[tokio::test]
async fn test_multiple_operations_sequence() {
    let scheduler = MockJobScheduler::new();
    let tool = CronTool::new(Arc::new(scheduler.clone()));
    let ctx = ToolContext::new("test_agent", "test_session");

    // Schedule first job
    let result = tool
        .execute(
            json!({
                "operation": "schedule",
                "cron_expression": "0 * * * * * *",
                "command": "echo first"
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());
    assert_eq!(scheduler.get_job_ids().len(), 1);

    // Schedule second job
    let result = tool
        .execute(
            json!({
                "operation": "schedule",
                "cron_expression": "0 */10 * * * * *",
                "command": "echo second"
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());
    assert_eq!(scheduler.get_job_ids().len(), 2);

    // List jobs
    let result = tool.execute(json!({ "operation": "list" }), &ctx).await;
    assert!(result.is_ok());

    // Remove first job
    let job_ids = scheduler.get_job_ids();
    let first_job_id = job_ids[0].clone();
    let result = tool
        .execute(
            json!({
                "operation": "remove",
                "job_id": first_job_id
            }),
            &ctx,
        )
        .await;
    assert!(result.is_ok());
    assert_eq!(scheduler.get_job_ids().len(), 1);
}

#[tokio::test]
async fn test_cron_expression_various_formats() {
    let tool = CronTool::with_noop_scheduler();
    let ctx = ToolContext::new("test_agent", "test_session");

    let cron_expressions = vec![
        "0 0 12 * * ? *",       // Every day at 12:00 PM
        "0 0/15 * * * ? *",     // Every 15 minutes
        "0 0 12 ? * MON-FRI *", // Every weekday at 12:00 PM
        "0 0 0 1 * ? *",        // First day of every month at midnight
    ];

    for expr in cron_expressions {
        let result = tool
            .execute(
                json!({
                    "operation": "schedule",
                    "cron_expression": expr,
                    "command": "echo test"
                }),
                &ctx,
            )
            .await;

        assert!(
            result.is_ok(),
            "Expression '{}' should be valid: {:?}",
            expr,
            result
        );
    }
}

#[tokio::test]
async fn test_cron_tool_with_complex_command() {
    let tool = CronTool::with_noop_scheduler();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "schedule",
                "cron_expression": "0 * * * * * *",
                "command": "ls -la /tmp | grep test && echo done"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.content.contains("echo done"));
}

#[tokio::test]
async fn test_cron_tool_with_special_characters() {
    let tool = CronTool::with_noop_scheduler();
    let ctx = ToolContext::new("test_agent", "test_session");

    let result = tool
        .execute(
            json!({
                "operation": "schedule",
                "cron_expression": "0 * * * * * *",
                "command": "echo 'Special chars: !@#$%^&*()'"
            }),
            &ctx,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_noop_scheduler_schedule() {
    let scheduler = NoOpJobScheduler::new();

    let result = scheduler
        .schedule("test-job", "0 * * * * * *", "echo test")
        .await;

    assert!(result.is_ok());
    let job = result.unwrap();
    assert_eq!(job.id, "test-job");
    assert_eq!(job.cron_expression, "0 * * * * * *");
    assert_eq!(job.command, "echo test");
}

#[tokio::test]
async fn test_noop_scheduler_list() {
    let scheduler = NoOpJobScheduler::new();

    let result = scheduler.list().await;

    assert!(result.is_ok());
    let jobs = result.unwrap();
    assert!(jobs.is_empty());
}

#[tokio::test]
async fn test_noop_scheduler_run() {
    let scheduler = NoOpJobScheduler::new();

    let result = scheduler.run_now("nonexistent").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_noop_scheduler_remove() {
    let scheduler = NoOpJobScheduler::new();

    let result = scheduler.remove("nonexistent").await;

    assert!(result.is_ok());
    assert!(!result.unwrap());
}
