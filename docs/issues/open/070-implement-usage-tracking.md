# Issue 070: Implement Usage Tracking

## Summary
Implement token usage tracking at per-request, per-session, and per-agent levels. Report usage via `AgentEvent::Usage` events so that consumers can monitor and enforce token budgets.

## Location
- Crate: `aisopod-agent`
- File: `crates/aisopod-agent/src/usage.rs`

## Current Behavior
No usage tracking exists. Token consumption from model calls is not recorded or reported.

## Expected Behavior
After this issue is completed:
- Each model request records input and output token counts.
- Per-session usage accumulates across all requests in a session.
- Per-agent usage aggregates across all sessions for a given agent.
- `AgentEvent::Usage` events are emitted after each model call with the token counts.
- A `UsageTracker` provides methods to query cumulative usage.

## Impact
Usage tracking is essential for cost monitoring, budget enforcement, and observability. Without it, operators cannot understand or control the token consumption of their agents.

## Suggested Implementation
1. **Create `crates/aisopod-agent/src/usage.rs`:**
   - Define `UsageTracker`:
     ```rust
     pub struct UsageTracker {
         /// Per-session usage, keyed by session_key
         session_usage: DashMap<String, UsageReport>,
         /// Per-agent usage, keyed by agent_id
         agent_usage: DashMap<String, UsageReport>,
     }
     ```
   - Use `DashMap` (or `RwLock<HashMap>`) for concurrent access.
   - Define `UsageReport` (if not already in types.rs):
     ```rust
     #[derive(Clone, Debug, Default)]
     pub struct UsageReport {
         pub input_tokens: u64,
         pub output_tokens: u64,
         pub total_tokens: u64,
         pub request_count: u64,
     }
     ```

2. **Implement tracking methods:**
   - `UsageTracker::new() -> Self` — create an empty tracker.
   - `record_request(session_key: &str, agent_id: &str, input_tokens: u64, output_tokens: u64)`:
     - Add to the session's cumulative usage.
     - Add to the agent's cumulative usage.
     - Increment `request_count` for both.
   - `get_session_usage(session_key: &str) -> Option<UsageReport>` — return cumulative usage for a session.
   - `get_agent_usage(agent_id: &str) -> Option<UsageReport>` — return cumulative usage for an agent.
   - `reset_session(session_key: &str)` — clear usage for a session (e.g., on session end).

3. **Integration with pipeline:**
   - After each model call in the execution pipeline, call `usage_tracker.record_request()`.
   - Emit `AgentEvent::Usage { usage }` with the per-request token counts.
   - Add `Arc<UsageTracker>` to `AgentRunner`.

4. **Update `crates/aisopod-agent/src/lib.rs`:**
   - Add `pub mod usage;`.

5. **Add unit tests:**
   - Test recording a single request and retrieving session usage.
   - Test accumulation across multiple requests in the same session.
   - Test per-agent aggregation across different sessions.
   - Test `reset_session()` clears session usage but not agent usage.
   - Test concurrent access (spawn multiple tasks recording simultaneously).

6. **Verify** — Run `cargo test -p aisopod-agent`.

## Dependencies
- Issue 066 (Streaming agent execution pipeline — integration point for recording usage)

## Acceptance Criteria
- [ ] `UsageTracker` records per-request, per-session, and per-agent token usage.
- [ ] `AgentEvent::Usage` events are emitted after each model call.
- [ ] Session and agent usage can be queried via `get_session_usage()` and `get_agent_usage()`.
- [ ] Concurrent access is safe (no data races).
- [ ] Unit tests verify accumulation, aggregation, and reset behavior.
- [ ] `cargo check -p aisopod-agent` succeeds without errors.

---
*Created: 2026-02-15*
