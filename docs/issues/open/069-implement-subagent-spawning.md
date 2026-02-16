# Issue 069: Implement Subagent Spawning

## Summary
Implement the ability to spawn child agents within a parent agent's session, with depth limit enforcement, model allowlist control, thread ID propagation for context sharing, and resource budget inheritance.

## Location
- Crate: `aisopod-agent`
- File: `crates/aisopod-agent/src/subagent.rs`

## Current Behavior
No subagent spawning logic exists in the agent engine. The subagent tool (Issue 055) defines the tool interface, but the engine-side orchestration is missing.

## Expected Behavior
After this issue is completed:
- A parent agent can spawn a child agent that runs within the parent's session context.
- Depth limits are enforced: a configurable `max_depth` prevents infinite recursion (e.g., agent A spawns agent B which spawns agent A).
- A model allowlist restricts which models subagents can use.
- Thread IDs propagate from parent to child for context sharing.
- Resource budgets (token limits) are inherited and decremented by child usage.
- The parent agent receives the child's final result as a tool call response.

## Impact
Subagent spawning enables complex multi-agent workflows where specialized agents handle subtasks. Without depth limits and resource controls, the system risks infinite recursion and unbounded resource consumption.

## Suggested Implementation
1. **Create `crates/aisopod-agent/src/subagent.rs`:**
   - Define `SubagentSpawnParams`:
     ```rust
     pub struct SubagentSpawnParams {
         pub agent_id: String,
         pub messages: Vec<Message>,
         pub parent_session_key: String,
         pub parent_depth: usize,
         pub thread_id: Option<String>,
         pub resource_budget: Option<ResourceBudget>,
     }
     ```
   - Define `ResourceBudget`:
     ```rust
     pub struct ResourceBudget {
         pub max_tokens: usize,
         pub remaining_tokens: usize,
     }
     ```
   - Implement `spawn_subagent(runner: &AgentRunner, params: SubagentSpawnParams) -> Result<AgentRunResult>`:
     - Check `params.parent_depth + 1 <= max_depth` from config. Return an error if exceeded.
     - Validate the requested agent's model against the subagent allowlist.
     - Create a child `AgentRunParams` with the incremented depth and inherited thread ID.
     - Call `runner.run()` with the child params.
     - Deduct child usage from the parent's resource budget.
     - Return the child's result.

2. **Depth limit enforcement:**
   - Read `max_subagent_depth` from the agent configuration (default: 3).
   - Track current depth in `AgentRunParams` (add a `depth: usize` field if not already present).
   - When spawning, check `depth + 1 <= max_subagent_depth`.

3. **Model allowlist:**
   - Read `subagent_allowed_models` from the agent configuration.
   - Before spawning, check the child agent's primary model against the allowlist.
   - If the allowlist is empty or `None`, allow all models.

4. **Thread ID propagation:**
   - If the parent has a `thread_id`, pass it to the child's `AgentRunParams`.
   - This allows the child to access shared context if needed.

5. **Resource budget inheritance:**
   - If the parent has a resource budget, create a child budget from the remaining tokens.
   - After the child completes, subtract the child's token usage from the parent's remaining budget.
   - If remaining budget is insufficient, return an error instead of spawning.

6. **Update `crates/aisopod-agent/src/lib.rs`:**
   - Add `pub mod subagent;`.

7. **Add unit tests:**
   - Test successful subagent spawn at depth 1 (within limit).
   - Test depth limit enforcement at max depth → returns error.
   - Test model allowlist blocks a disallowed model.
   - Test model allowlist allows an approved model.
   - Test thread ID propagation from parent to child.
   - Test resource budget deduction after child execution.
   - Test resource budget exhaustion prevents spawning.

8. **Verify** — Run `cargo test -p aisopod-agent`.

## Dependencies
- Issue 066 (Streaming agent execution pipeline — provides `AgentRunner::run()`)
- Issue 055 (Subagent spawning tool — tool-side interface)

## Acceptance Criteria
- [ ] Subagents can be spawned within a parent session.
- [ ] Depth limit is enforced and configurable.
- [ ] Model allowlist restricts which models subagents can use.
- [ ] Thread ID propagates from parent to child.
- [ ] Resource budgets are inherited and decremented.
- [ ] Unit tests cover depth limits, allowlist, thread propagation, and budget enforcement.
- [ ] `cargo check -p aisopod-agent` succeeds without errors.

---
*Created: 2026-02-15*
