# Issue 072: Add Agent Engine Unit and Integration Tests

## Summary
Create a comprehensive test suite for the agent execution engine, covering agent resolution, system prompt construction, the execution pipeline with mock providers, failover behavior, compaction strategies, subagent spawning with depth limits, usage tracking, and the abort mechanism.

## Location
- Crate: `aisopod-agent`
- Files: `crates/aisopod-agent/tests/`, per-module `#[cfg(test)]` submodules

## Current Behavior
No comprehensive test suite exists for the agent engine. Individual modules may have inline unit tests from their respective issues, but there are no integration tests that verify the full pipeline.

## Expected Behavior
After this issue is completed:
- Integration tests exercise the full execution pipeline from `AgentRunParams` to `AgentRunResult`.
- Mock providers simulate model responses, tool calls, errors, rate limits, and timeouts.
- All agent engine subsystems are tested both in isolation and as part of the integrated pipeline.
- Tests run reliably via `cargo test -p aisopod-agent`.

## Impact
Tests are essential for catching regressions, validating correctness across the complex agent engine, and giving contributors confidence when making changes. The agent engine is the most critical subsystem, so thorough testing is especially important.

## Suggested Implementation
1. **Create mock infrastructure** in `crates/aisopod-agent/tests/helpers/` or `src/test_utils.rs`:
   - `MockProvider` — implements the `ModelProvider` trait, returning configurable responses (text, tool calls, errors).
   - `MockToolRegistry` — returns configurable tool results.
   - `MockSessionStore` — in-memory session storage.
   - `test_config()` — helper to build a valid `AisopodConfig` for tests.

2. **Agent resolution tests** — `tests/resolution.rs`:
   - Test `resolve_session_agent_id()` with multiple bindings → first match wins.
   - Test default agent fallback when no binding matches.
   - Test `resolve_agent_config()` with a valid agent ID → returns config.
   - Test `resolve_agent_config()` with an invalid agent ID → returns error.
   - Test `resolve_agent_model()` returns primary model and fallback chain.
   - Test `list_agent_ids()` returns all configured agents.

3. **System prompt construction tests** — `tests/prompt.rs`:
   - Test prompt with only base prompt.
   - Test prompt with all components (base, dynamic context, tools, skills, memory).
   - Test tool descriptions are correctly formatted.
   - Test dynamic context includes a timestamp.

4. **Transcript repair tests** — `tests/transcript.rs`:
   - Test Anthropic repair with consecutive same-role messages.
   - Test OpenAI repair preserves system messages.
   - Test Gemini repair with turn violations.
   - Test valid sequences pass through unchanged.

5. **Execution pipeline integration tests** — `tests/pipeline.rs`:
   - Test a simple text-only response (no tool calls).
   - Test a response with one tool call → tool executed → final text response.
   - Test a response with multiple sequential tool calls.
   - Test that `TextDelta`, `ToolCallStart`, `ToolCallResult`, and `Complete` events are emitted in order.
   - Test error handling when the provider returns an error.

6. **Failover tests** — `tests/failover.rs`:
   - Test successful first attempt (no failover triggered).
   - Test failover on auth error → switches to next model.
   - Test failover on rate limit → waits or switches.
   - Test failover on context overflow → triggers compaction then retry.
   - Test all models exhausted → returns descriptive error.
   - Test `ModelSwitch` event emitted on failover.

7. **Compaction tests** — `tests/compaction.rs`:
   - Test `HardClear` keeps only the most recent N messages.
   - Test `ToolResultTruncation` truncates oversized tool results.
   - Test `Summary` replaces older messages with a summary placeholder.
   - Test `ContextWindowGuard` triggers compaction at the correct threshold.

8. **Subagent tests** — `tests/subagent.rs`:
   - Test successful subagent spawn.
   - Test depth limit enforcement → error at max depth.
   - Test model allowlist blocks disallowed models.
   - Test thread ID propagation.
   - Test resource budget enforcement.

9. **Usage tracking tests** — `tests/usage.rs`:
   - Test per-request usage recording.
   - Test per-session accumulation.
   - Test per-agent aggregation.
   - Test `UsageEvent` emission.
   - Test `reset_session()` behavior.

10. **Abort tests** — `tests/abort.rs`:
    - Test aborting a running agent stops execution.
    - Test abort event is received by subscribers.
    - Test aborting an inactive session.

11. **Run all tests** — `cargo test -p aisopod-agent`.

## Dependencies
- Issue 062 (Agent types and AgentRunner skeleton)
- Issue 063 (Agent resolution and binding)
- Issue 064 (System prompt construction)
- Issue 065 (Message transcript repair)
- Issue 066 (Streaming agent execution pipeline)
- Issue 067 (Model failover)
- Issue 068 (Session compaction strategies)
- Issue 069 (Subagent spawning)
- Issue 070 (Usage tracking)
- Issue 071 (Agent abort mechanism)

## Acceptance Criteria
- [ ] Mock provider, tool registry, and session store are implemented for testing.
- [ ] Agent resolution tests cover binding evaluation, config lookup, and model chain resolution.
- [ ] System prompt construction tests verify all prompt components.
- [ ] Transcript repair tests cover all provider strategies.
- [ ] Pipeline integration tests verify end-to-end execution with tool calls.
- [ ] Failover tests cover all error types and the model exhaustion case.
- [ ] Compaction tests verify all strategies and the context guard.
- [ ] Subagent tests verify depth limits, allowlists, and resource budgets.
- [ ] Usage tracking tests verify accumulation, aggregation, and reporting.
- [ ] Abort tests verify cancellation and subscriber notification.
- [ ] All tests pass: `cargo test -p aisopod-agent`.

---
*Created: 2026-02-15*
