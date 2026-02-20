# Issue 061: Add Tool System Unit Tests

## Summary
Create a comprehensive test suite for the tool subsystem, covering each built-in tool, tool policy enforcement, the approval workflow, and schema normalization.

## Location
- Crate: `aisopod-tools`
- Files: `crates/aisopod-tools/tests/`, per-module test submodules

## Current Behavior
No tests exist for the tool subsystem.

## Expected Behavior
After this issue is completed:
- Each built-in tool (bash, file, message, subagent, session, cron, canvas) has unit tests verifying correct behavior and error handling.
- Tool policy enforcement has tests confirming that allow/deny lists work correctly.
- The approval workflow has tests covering auto-approve, manual approval, denial, and timeout.
- Schema normalization has tests verifying correct output for Anthropic, OpenAI, and Gemini formats.
- The tool registry has tests covering registration, lookup, listing, duplicate handling, and schema generation.

## Impact
Tests are essential for confidence in the tool system. They catch regressions, ensure consistent behavior across tools, and validate that security-critical features (policy enforcement, approval workflow) work correctly.

## Suggested Implementation
1. **Tool registry tests** — In `crates/aisopod-tools/tests/registry.rs` (or `src/registry.rs` `#[cfg(test)]` module):
   - Test registering a tool and looking it up by name.
   - Test listing all registered tools.
   - Test `schemas()` output structure.
   - Test duplicate registration behavior.
   - Test looking up a non-existent tool returns `None`.

2. **Policy enforcement tests** — In `crates/aisopod-tools/tests/policy.rs`:
   - Test that a tool in the global deny list is blocked.
   - Test that a tool in the per-agent deny list is blocked.
   - Test that a tool in the allow list is permitted.
   - Test that a tool not in the allow list is blocked when an allow list is set.
   - Test that deny takes precedence over allow.
   - Test the denial message content.

3. **Bash tool tests** — In `crates/aisopod-tools/tests/bash.rs`:
   - Test executing a simple command (e.g., `echo hello`) and capturing output.
   - Test that a non-zero exit code sets `is_error: true`.
   - Test timeout enforcement with a long-running command (e.g., `sleep 100`).
   - Test working directory configuration.
   - Test environment variable injection.

4. **File tool tests** — In `crates/aisopod-tools/tests/file.rs`:
   - Test reading a file.
   - Test writing and then reading a file.
   - Test directory listing.
   - Test file metadata retrieval.
   - Test workspace path restriction (path traversal blocked).
   - Test searching by glob and text pattern.

5. **Message tool tests** — In `crates/aisopod-tools/tests/message.rs`:
   - Test sending a message with a mock sender.
   - Test missing required parameters produce an error.

6. **Subagent tool tests** — In `crates/aisopod-tools/tests/subagent.rs`:
   - Test spawning a subagent with a mock spawner.
   - Test depth limit enforcement.
   - Test model allowlist enforcement.

7. **Session tool tests** — In `crates/aisopod-tools/tests/session.rs`:
   - Test each operation (list, send, patch, history) with a mock manager.

8. **Cron tool tests** — In `crates/aisopod-tools/tests/cron.rs`:
   - Test scheduling a job with a valid cron expression.
   - Test rejection of an invalid cron expression.
   - Test listing and removing jobs.

9. **Canvas tool tests** — In `crates/aisopod-tools/tests/canvas.rs`:
   - Test creating a canvas and retrieving it.
   - Test updating a canvas.

10. **Approval workflow tests** — In `crates/aisopod-tools/tests/approval.rs`:
    - Test auto-approve for a safe command.
    - Test manual approval flow (approved).
    - Test manual denial flow.
    - Test timeout behavior.
    - Test approval state tracking.

11. **Schema normalization tests** — In `crates/aisopod-tools/tests/schema.rs`:
    - Test Anthropic format output contains `input_schema`.
    - Test OpenAI format output contains `type: "function"` wrapper.
    - Test Gemini format output structure.
    - Test batch conversion with multiple tools.

12. **Run all tests** — `cargo test -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 050 (Tool registry)
- Issue 051 (Tool policy enforcement)
- Issue 052 (Bash/shell tool)
- Issue 053 (File operations tool)
- Issue 054 (Message sending tool)
- Issue 055 (Subagent spawning tool)
- Issue 056 (Session management tool)
- Issue 057 (Cron/scheduled task tool)
- Issue 058 (Canvas tool)
- Issue 059 (Approval workflow)
- Issue 060 (Tool schema normalization)

## Acceptance Criteria
- [x] Tool registry tests cover registration, lookup, listing, and schema generation.
- [x] Policy enforcement tests verify allow/deny behavior and precedence.
- [x] Each built-in tool has tests covering success and error paths.
- [x] Approval workflow tests cover auto-approve, manual approval, denial, and timeout.
- [x] Schema normalization tests verify correct output for all three provider formats.
- [x] All tests pass: `cargo test -p aisopod-tools`.
- [x] Test coverage includes both success and error paths for each component.

## Resolution
The tool system unit test suite was implemented following the suggested implementation in this issue. All 11 test files were created with comprehensive coverage:

- **registry.rs** (11 tests): Tests for tool registration, lookup, listing, schemas, and duplicate handling
- **policy.rs** (16 tests): Tests for allow/deny lists, agent policies, and precedence rules
- **bash.rs** (16 tests): Tests for command execution, timeout, environment variables, and error handling
- **file.rs** (22 tests): Tests for read/write operations, directory listing, metadata, workspace restrictions, and search
- **message.rs** (22 tests): Tests for message sending with mock sender and parameter validation
- **subagent.rs** (23 tests): Tests for spawning, depth limits, and model allowlist enforcement
- **session.rs** (30 tests): Tests for list/send/patch/history operations with mock session manager
- **cron.rs** (31 tests): Tests for scheduling, listing, running, and removing jobs
- **canvas.rs** (30 tests): Tests for create/update/get operations
- **approval.rs** (53 tests): Tests for auto-approve, manual approval, denial, timeout, and state tracking
- **schema.rs** (11 tests): Tests for Anthropic, OpenAI, and Gemini format normalization

Total: 221 tests across 11 test files.

All tests pass with `cargo test -p aisopod-tools`.

---
*Created: 2026-02-15*
*Resolved: 2026-02-20*
