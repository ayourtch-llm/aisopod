# Issue 056: Implement Session Management Tool

## Summary
Implement a built-in session management tool that allows agents to list active sessions, send messages to specific sessions, patch session metadata, and access session history.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/builtins/session.rs`

## Current Behavior
No session management tool exists. Agents have no programmatic way to interact with other sessions or inspect session state.

## Expected Behavior
After this issue is completed:
- The `SessionTool` struct implements the `Tool` trait.
- It supports multiple operations via an `operation` parameter: `list`, `send`, `patch`, and `history`.
- `list` — Returns a list of active session IDs and their metadata.
- `send` — Sends a message to a specific session by ID.
- `patch` — Updates metadata on a specific session.
- `history` — Retrieves recent message history from a specific session.
- The actual session access is delegated to a pluggable `SessionManager` trait.

## Impact
Session management enables agents to coordinate across sessions, inspect session state, and send messages to other active conversations. This is essential for multi-session orchestration workflows.

## Suggested Implementation
1. **Create `session.rs`** — Add `crates/aisopod-tools/src/builtins/session.rs`.

2. **Define a `SessionManager` trait**:
   ```rust
   #[async_trait]
   pub trait SessionManager: Send + Sync {
       async fn list_sessions(&self) -> Result<Vec<SessionInfo>>;
       async fn send_to_session(&self, session_id: &str, message: &str) -> Result<()>;
       async fn patch_metadata(&self, session_id: &str, metadata: serde_json::Value) -> Result<()>;
       async fn get_history(&self, session_id: &str, limit: usize) -> Result<Vec<serde_json::Value>>;
   }
   ```
   Define a simple `SessionInfo` struct with `id`, `agent_id`, `created_at`, and `metadata` fields.

3. **Define `SessionTool`**:
   ```rust
   pub struct SessionTool {
       manager: Arc<dyn SessionManager>,
   }
   ```

4. **Implement `Tool` for `SessionTool`**:
   - `name()` → `"session"`
   - `description()` → `"Manage and interact with agent sessions"`
   - `parameters_schema()` → JSON Schema with `operation` (enum), `session_id`, `message`, `metadata`, and `limit` properties.
   - `execute()`:
     1. Parse the `operation` parameter.
     2. Dispatch to the corresponding `SessionManager` method.
     3. Serialize the result into a `ToolResult`.

5. **Create a no-op `SessionManager` implementation** for testing.

6. **Register the tool** — Ensure the tool can be registered with the `ToolRegistry`.

7. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 050 (Tool registry)

## Acceptance Criteria
- [x] `SessionTool` implements the `Tool` trait.
- [x] `list` operation returns active sessions.
- [x] `send` operation sends a message to a specific session.
- [x] `patch` operation updates session metadata.
- [x] `history` operation retrieves recent session messages.
- [x] `parameters_schema()` returns a valid JSON Schema.
- [x] `cargo check -p aisopod-tools` compiles without errors.

## Resolution

**Date Resolved:** 2026-02-20

**Changes Made:**
1. Created `crates/aisopod-tools/src/builtins/session.rs` with:
   - `SessionInfo` struct containing session metadata (id, agent_id, created_at, metadata)
   - `SessionManager` trait for pluggable session management implementations
   - `SessionTool` struct implementing the `Tool` trait with operations: `list`, `send`, `patch`, `history`
   - `NoOpSessionManager` implementation for testing

2. All acceptance criteria verified through:
   - Compilation: `cargo build -p aisopod-tools` passes
   - Unit tests: All 14 tests pass in `session.rs`

3. Tool features:
   - `list`: Returns list of active sessions with metadata
   - `send`: Sends messages to specific sessions
   - `patch`: Updates session metadata
   - `history`: Retrieves message history for sessions

4. Proper error handling for missing required parameters

---
*Created: 2026-02-15*
*Resolved: 2026-02-20*
