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
- [ ] `BashTool` implements the `Tool` trait.
- [ ] Shell commands execute with the correct working directory.
- [ ] Timeout enforcement kills long-running commands and returns an error.
- [ ] stdout and stderr output is captured and returned.
- [ ] Environment variables can be injected into the command.
- [ ] `parameters_schema()` returns a valid JSON Schema.
- [ ] `cargo check -p aisopod-tools` compiles without errors.

---
*Created: 2026-02-15*
