# Issue 071: Implement Agent Abort Mechanism

## Summary
Implement the ability to cancel a running agent execution via the `abort()` method. Use a `tokio::CancellationToken` or similar mechanism to stop the execution loop, clean up in-flight tool executions, and notify subscribers of the abort.

## Location
- Crate: `aisopod-agent`
- File: `crates/aisopod-agent/src/abort.rs`, updates to `crates/aisopod-agent/src/runner.rs` and `crates/aisopod-agent/src/pipeline.rs`

## Current Behavior
`AgentRunner::abort()` is a stub returning `todo!()`. There is no way to cancel a running agent execution.

## Expected Behavior
After this issue is completed:
- Calling `AgentRunner::abort(session_key)` cancels the running agent execution for that session.
- A `tokio::CancellationToken` (or equivalent) is checked at key points in the execution loop.
- In-flight tool executions are cancelled or allowed to complete with a short grace period.
- Subscribers receive an `AgentEvent::Error` with an abort reason.
- The `AgentRunStream` terminates cleanly after abort.
- Subsequent calls to `abort()` on an already-aborted session are no-ops.

## Impact
Without abort, users cannot stop a misbehaving or long-running agent. This is critical for user experience and resource management — a stuck agent would consume resources indefinitely.

## Suggested Implementation
1. **Create `crates/aisopod-agent/src/abort.rs`:**
   - Define `AbortHandle`:
     ```rust
     pub struct AbortHandle {
         token: CancellationToken,
         session_key: String,
     }
     ```
   - Implement `AbortHandle::new(session_key: String) -> Self`.
   - Implement `AbortHandle::abort(&self)` — triggers the cancellation token.
   - Implement `AbortHandle::is_aborted(&self) -> bool` — checks if cancellation was requested.

2. **Track active sessions in `AgentRunner`:**
   - Add a `active_sessions: DashMap<String, AbortHandle>` field to `AgentRunner`.
   - When `run()` is called, create an `AbortHandle` and insert it into `active_sessions`.
   - When the execution completes (success or error), remove the entry.

3. **Check cancellation in the execution loop:**
   - At the start of each iteration of the tool call loop, check `abort_handle.is_aborted()`.
   - Before each model call, check for cancellation.
   - Use `tokio::select!` to race between the model call and the cancellation token:
     ```rust
     tokio::select! {
         result = provider.chat_stream(request) => { /* handle result */ }
         _ = abort_handle.token.cancelled() => {
             // Send abort event and return
         }
     }
     ```

4. **Clean up in-flight tool executions:**
   - If a tool is currently executing when abort is called, allow a short grace period (e.g., 5 seconds) for it to complete.
   - After the grace period, drop the tool execution future.

5. **Notify subscribers:**
   - Send `AgentEvent::Error { message: "Agent execution aborted".into() }` to the event channel.

6. **Update `runner.rs`:**
   - Replace the `todo!()` in `abort()`:
     ```rust
     pub async fn abort(&self, session_key: &str) -> Result<()> {
         if let Some(handle) = self.active_sessions.get(session_key) {
             handle.abort();
             Ok(())
         } else {
             Err(anyhow!("No active session: {}", session_key))
         }
     }
     ```

7. **Update `crates/aisopod-agent/src/lib.rs`:**
   - Add `pub mod abort;`.

8. **Add unit tests:**
   - Test aborting a running agent → `AgentEvent::Error` with abort message received.
   - Test aborting an already-completed session → error or no-op.
   - Test that the execution loop exits promptly after abort.
   - Test that the `AbortHandle` is removed from `active_sessions` after execution ends.
   - Test concurrent abort and execution completion (race condition safety).

9. **Verify** — Run `cargo test -p aisopod-agent`.

## Dependencies
- Issue 066 (Streaming agent execution pipeline — the loop that needs cancellation points)

## Acceptance Criteria
- [ ] `AgentRunner::abort()` cancels the running execution for a session.
- [ ] Cancellation is checked at key points in the execution loop.
- [ ] In-flight tool executions are handled gracefully on abort.
- [ ] Subscribers receive an error event indicating the abort.
- [ ] `AgentRunStream` terminates cleanly after abort.
- [ ] Aborting an inactive session returns an appropriate error or is a no-op.
- [ ] Unit tests verify abort behavior under various conditions.
- [ ] `cargo check -p aisopod-agent` succeeds without errors.

---
*Created: 2026-02-15*
