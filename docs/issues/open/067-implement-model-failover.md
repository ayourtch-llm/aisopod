# Issue 067: Implement Model Failover

## Summary
Implement the multi-model failover system that automatically switches to fallback models when the primary model encounters errors such as authentication failures, rate limits, context overflow, or timeouts.

## Location
- Crate: `aisopod-agent`
- File: `crates/aisopod-agent/src/failover.rs`

## Current Behavior
No failover logic exists. If a model call fails, the agent execution fails entirely.

## Expected Behavior
After this issue is completed:
- The execution pipeline attempts the primary model first.
- On auth errors, it tries the next auth profile for the same model before failing over to the next model.
- On rate limit errors, it optionally waits (if a retry-after header is provided) or fails over.
- On context overflow errors, it triggers compaction and retries, or fails over if compaction is insufficient.
- On timeout errors, it fails over to the next model immediately.
- `FailoverState` tracks attempted models and the current position in the chain.
- A `ModelSwitch` event is emitted when failover occurs.
- If all models in the chain are exhausted, a clear error is returned to the user.

## Impact
Failover is essential for production reliability. Users should not experience failures when a single model is temporarily unavailable. The fallback chain ensures continuous service by transparently switching to alternative models.

## Suggested Implementation
1. **Create `crates/aisopod-agent/src/failover.rs`:**
   - Define `FailoverState`:
     ```rust
     pub struct FailoverState {
         pub attempted_models: Vec<ModelAttempt>,
         pub current_model_index: usize,
         pub max_attempts: usize,
     }

     pub struct ModelAttempt {
         pub model_id: String,
         pub error: Option<String>,
         pub duration: Duration,
     }
     ```
   - Implement `FailoverState::new(model_chain: &ModelChain)` to initialize from the resolved model chain.
   - Implement `FailoverState::current_model() -> &str` to return the current model ID.
   - Implement `FailoverState::advance() -> Option<&str>` to move to the next model. Returns `None` if all models exhausted.

2. **Define error classification:**
   - Create a `FailoverAction` enum:
     ```rust
     pub enum FailoverAction {
         RetryWithNextAuth,
         WaitAndRetry { delay: Duration },
         CompactAndRetry,
         FailoverToNext,
         Abort { error: String },
     }
     ```
   - Implement `classify_error(error: &ProviderError) -> FailoverAction` that maps provider errors to failover actions.

3. **Implement the failover loop** (called from the execution pipeline):
   ```rust
   pub async fn execute_with_failover(
       state: &mut FailoverState,
       call_model: impl Fn(&str) -> Future<Result<Response>>,
       event_tx: &mpsc::Sender<AgentEvent>,
   ) -> Result<Response>
   ```
   - Loop through model attempts.
   - On each failure, classify the error and take the appropriate action.
   - Emit `AgentEvent::ModelSwitch` when switching models.
   - Record each attempt in `FailoverState::attempted_models`.

4. **Integrate with pipeline:**
   - Update the execution pipeline (Issue 066) to use `execute_with_failover()` instead of a direct model call.

5. **Update `crates/aisopod-agent/src/lib.rs`:**
   - Add `pub mod failover;`.

6. **Add unit tests:**
   - Test successful first attempt (no failover).
   - Test failover on auth error (try next auth, then next model).
   - Test failover on rate limit with wait.
   - Test failover on timeout (immediate switch).
   - Test all models exhausted returns a clear error.
   - Test `ModelSwitch` event is emitted on failover.

7. **Verify** — Run `cargo test -p aisopod-agent`.

## Dependencies
- Issue 066 (Streaming agent execution pipeline)
- Issue 040 (Auth profile management — for trying alternative auth profiles)

## Acceptance Criteria
- [ ] `FailoverState` tracks model attempts and current position in the chain.
- [ ] Auth errors trigger next auth profile before model failover.
- [ ] Rate limit errors wait or fail over appropriately.
- [ ] Context overflow triggers compaction then retry or failover.
- [ ] Timeout errors trigger immediate failover.
- [ ] `ModelSwitch` event is emitted when failover occurs.
- [ ] All models exhausted returns a descriptive error.
- [ ] Unit tests cover all failover scenarios.
- [ ] `cargo check -p aisopod-agent` succeeds without errors.

---
*Created: 2026-02-15*
