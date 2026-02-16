# Issue 057: Implement Cron/Scheduled Task Tool

## Summary
Implement a built-in cron/scheduled task tool that allows agents to schedule recurring tasks using cron expressions, list scheduled jobs, run a job immediately, and remove scheduled jobs.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/builtins/cron.rs`

## Current Behavior
No scheduled task tool exists. Agents have no way to schedule recurring or deferred work.

## Expected Behavior
After this issue is completed:
- The `CronTool` struct implements the `Tool` trait.
- It supports multiple operations via an `operation` parameter: `schedule`, `list`, `run`, and `remove`.
- `schedule` — Creates a new scheduled job with a cron expression and a command/prompt to execute.
- `list` — Returns all currently scheduled jobs with their IDs, cron expressions, and next run times.
- `run` — Triggers a scheduled job immediately, out of its normal schedule.
- `remove` — Removes a scheduled job by ID.
- Cron expression parsing is handled by the `cron` crate (or similar).

## Impact
Scheduled tasks enable agents to perform periodic work like health checks, reports, backups, or recurring notifications. This is critical for long-running autonomous agents.

## Suggested Implementation
1. **Create `cron.rs`** — Add `crates/aisopod-tools/src/builtins/cron.rs`.

2. **Add the `cron` crate dependency** — In `crates/aisopod-tools/Cargo.toml`, add a dependency on the `cron` crate for parsing cron expressions.

3. **Define a `JobScheduler` trait**:
   ```rust
   #[async_trait]
   pub trait JobScheduler: Send + Sync {
       async fn schedule(&self, cron_expr: &str, command: &str) -> Result<String>; // returns job ID
       async fn list(&self) -> Result<Vec<ScheduledJob>>;
       async fn run_now(&self, job_id: &str) -> Result<String>;
       async fn remove(&self, job_id: &str) -> Result<bool>;
   }
   ```
   Define a `ScheduledJob` struct with `id`, `cron_expression`, `command`, `next_run`, and `last_run` fields.

4. **Define `CronTool`**:
   ```rust
   pub struct CronTool {
       scheduler: Arc<dyn JobScheduler>,
   }
   ```

5. **Implement `Tool` for `CronTool`**:
   - `name()` → `"cron"`
   - `description()` → `"Schedule, list, run, and remove recurring tasks"`
   - `parameters_schema()` → JSON Schema with `operation` (enum), `cron_expression`, `command`, and `job_id` properties.
   - `execute()`:
     1. Parse the `operation` parameter.
     2. For `schedule`: validate the cron expression using the `cron` crate, then call `scheduler.schedule()`.
     3. For `list`: call `scheduler.list()` and format the results.
     4. For `run`: call `scheduler.run_now()` with the provided `job_id`.
     5. For `remove`: call `scheduler.remove()` with the provided `job_id`.

6. **Create a no-op `JobScheduler` implementation** for testing.

7. **Register the tool** — Ensure the tool can be registered with the `ToolRegistry`.

8. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 050 (Tool registry)

## Acceptance Criteria
- [ ] `CronTool` implements the `Tool` trait.
- [ ] `schedule` operation creates a job with a valid cron expression.
- [ ] `list` operation returns all scheduled jobs with next run times.
- [ ] `run` operation triggers a job immediately.
- [ ] `remove` operation deletes a scheduled job.
- [ ] Invalid cron expressions are rejected with a clear error message.
- [ ] `parameters_schema()` returns a valid JSON Schema.
- [ ] `cargo check -p aisopod-tools` compiles without errors.

---
*Created: 2026-02-15*
