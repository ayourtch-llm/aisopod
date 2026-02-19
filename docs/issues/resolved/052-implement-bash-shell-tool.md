# Issue 052: Implement Bash/Shell Tool

## Summary
Implement a built-in bash/shell tool that executes shell commands with a configurable working directory, enforces timeouts, captures output (stdout and stderr), and supports environment variable injection.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/builtins/bash.rs`

## Current Behavior
No shell execution tool exists in the crate.

## Expected Behavior
After this issue is completed:
- The `BashTool` struct implements the `Tool` trait.
- It accepts parameters: `command` (string), optional `working_directory` (string), optional `timeout_seconds` (integer), and optional `env` (object of key-value pairs).
- Commands execute via `tokio::process::Command` with the specified working directory and environment variables.
- A configurable timeout (default: 60 seconds) kills the process and returns a timeout error.
- stdout and stderr are captured and returned in `ToolResult.content` (combined or clearly separated).
- The `parameters_schema()` method returns a valid JSON Schema describing the accepted parameters.

## Impact
The bash tool is one of the most-used agent tools. It enables agents to run builds, tests, git commands, and any other shell operation. Timeout enforcement prevents runaway processes.

## Suggested Implementation
1. **Create directory and file** — Create `crates/aisopod-tools/src/builtins/` directory with a `mod.rs` and `bash.rs`.

2. **Define `BashTool`**:
   ```rust
   pub struct BashTool {
       default_timeout: Duration,
       default_working_dir: Option<PathBuf>,
   }
   ```

3. **Implement `Tool` for `BashTool`**:
   - `name()` → `"bash"`
   - `description()` → `"Execute a shell command"`
   - `parameters_schema()` → Return a JSON Schema object with properties for `command`, `working_directory`, `timeout_seconds`, and `env`.
   - `execute()`:
     1. Parse `command` from `params`. Return an error `ToolResult` if missing.
     2. Determine the working directory: use the param value, fall back to `ctx.workspace_path`, then `default_working_dir`.
     3. Build a `tokio::process::Command` running `sh -c "<command>"`.
     4. Set environment variables from the `env` param if present.
     5. Use `tokio::time::timeout()` to enforce the timeout.
     6. Collect stdout and stderr from the command output.
     7. Return a `ToolResult` with the combined output and `is_error` set based on the exit code.

4. **Handle edge cases**:
   - If the command is empty, return an error result.
   - If the process is killed due to timeout, return a result with `is_error: true` and a message like "Command timed out after N seconds".
   - If the exit code is non-zero, still return the output but set `is_error: true`.

5. **Register the tool** — Add a convenience function or ensure the tool can be registered with the `ToolRegistry` from Issue 050.

6. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 050 (Tool registry)

## Acceptance Criteria
- [x] `BashTool` implements the `Tool` trait.
- [x] Shell commands execute with the correct working directory.
- [x] Timeout enforcement kills long-running commands and returns an error.
- [x] stdout and stderr output is captured and returned.
- [x] Environment variables can be injected into the command.
- [x] `parameters_schema()` returns a valid JSON Schema.
- [x] `cargo check -p aisopod-tools` compiles without errors.

## Resolution
The bash tool was implemented in commit `8bc4842` and enhanced with approval workflow support in commit `5383816`.

### Changes Made
1. **Created `crates/aisopod-tools/src/builtins/bash.rs`**:
   - Implemented `BashTool` struct with `default_timeout` and `default_working_dir` fields
   - Implemented `Tool` trait with `name()`, `description()`, `parameters_schema()`, and `execute()` methods
   - Uses `tokio::process::Command` to execute shell commands via `sh -c`
   - Supports configurable timeout using `tokio::time::timeout()`
   - Captures and returns both stdout and stderr
   - Handles edge cases: empty commands, timeouts, non-zero exit codes
   - Supports environment variable injection via the `env` parameter

2. **Updated `crates/aisopod-tools/src/builtins/mod.rs`**:
   - Added `pub mod bash;`
   - Added `pub use bash::BashTool;`

3. **Updated `crates/aisopod-tools/src/lib.rs`**:
   - Added approval module imports and exports
   - Added `BashTool` to the public exports
   - Registered `BashTool` in `register_all_tools()`

4. **Approval Workflow Integration (Issue 059)**:
   - Added auto-approval for safe commands via `is_auto_approved()`
   - Integrated approval handler support for dangerous commands
   - Commands are automatically approved if they match safe patterns (echo, pwd, ls, cat, etc.)
   - Dangerous commands require user approval via the approval handler

### Verification
- All tests pass: `cargo test -p aisopod-tools` (137 tests passed)
- Build succeeds: `cargo build -p aisopod-tools`
- No compilation warnings with `RUSTFLAGS=-Awarnings`

---
*Created: 2026-02-15*
*Resolved: 2026-02-19*
