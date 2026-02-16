# Issue 059: Implement Approval Workflow for Dangerous Operations

## Summary
Implement an approval workflow system for dangerous tool operations (e.g., bash commands). This includes an `ApprovalRequest` type, an async approval waiting mechanism with timeout, auto-approve rules for known-safe commands, approval state tracking, and integration with the bash tool.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/approval.rs`

## Current Behavior
No approval mechanism exists. All tool operations execute immediately without human review.

## Expected Behavior
After this issue is completed:
- An `ApprovalRequest` struct is defined with `id`, `agent_id`, `operation` (description of what is being approved), `risk_level`, and `timeout`.
- An `ApprovalHandler` trait is defined with an async `request_approval()` method that returns `Approved` or `Denied`.
- Auto-approve rules allow known-safe commands (e.g., `ls`, `cat`, `git status`) to bypass the approval flow.
- Approval state is tracked so pending, approved, and denied requests can be queried.
- The bash tool (Issue 052) integrates with this system by checking `ctx.approval_handler` before executing dangerous commands.

## Impact
The approval workflow is a critical safety mechanism. Without it, agents could execute destructive commands (e.g., `rm -rf /`, database modifications) without human oversight. This is especially important in production environments.

## Suggested Implementation
1. **Create `approval.rs`** — Add `crates/aisopod-tools/src/approval.rs`.

2. **Define `ApprovalRequest`**:
   ```rust
   pub struct ApprovalRequest {
       pub id: String,
       pub agent_id: String,
       pub operation: String,
       pub risk_level: RiskLevel,
       pub timeout: Duration,
   }

   pub enum RiskLevel {
       Low,
       Medium,
       High,
       Critical,
   }
   ```

3. **Define `ApprovalResponse`**:
   ```rust
   pub enum ApprovalResponse {
       Approved,
       Denied { reason: String },
       TimedOut,
   }
   ```

4. **Define the `ApprovalHandler` trait**:
   ```rust
   #[async_trait]
   pub trait ApprovalHandler: Send + Sync {
       async fn request_approval(&self, request: &ApprovalRequest) -> Result<ApprovalResponse>;
   }
   ```

5. **Implement auto-approve logic** — Create a function `is_auto_approved(command: &str) -> bool` that checks the command against a configurable list of safe patterns (e.g., commands starting with `ls`, `cat`, `echo`, `git status`, `cargo check`). This list should be configurable.

6. **Implement `ApprovalStateTracker`** — A struct that tracks pending, approved, and denied requests:
   ```rust
   pub struct ApprovalStateTracker {
       requests: RwLock<HashMap<String, (ApprovalRequest, ApprovalStatus)>>,
   }

   pub enum ApprovalStatus {
       Pending,
       Approved,
       Denied { reason: String },
       TimedOut,
   }
   ```

7. **Integrate with the bash tool** — In the `BashTool::execute()` method (Issue 052), before running a command:
   1. Check if the command is auto-approved. If yes, proceed.
   2. If not, check if `ctx.approval_handler` is `Some`.
   3. If yes, create an `ApprovalRequest` and call `request_approval()`.
   4. If the response is `Approved`, proceed. Otherwise, return an error `ToolResult`.

8. **Re-export approval types from `lib.rs`**.

9. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 052 (Bash/shell tool)
- Issue 034 (Event broadcasting system — for delivering approval requests to clients)

## Acceptance Criteria
- [ ] `ApprovalRequest` type is defined with `id`, `agent_id`, `operation`, `risk_level`, and `timeout`.
- [ ] `ApprovalHandler` trait provides an async approval mechanism.
- [ ] Auto-approve rules correctly identify known-safe commands.
- [ ] Approval state tracking records pending, approved, denied, and timed-out requests.
- [ ] The approval flow integrates with the bash tool for dangerous commands.
- [ ] Approval requests time out correctly if not resolved.
- [ ] `cargo check -p aisopod-tools` compiles without errors.

---
*Created: 2026-02-15*
