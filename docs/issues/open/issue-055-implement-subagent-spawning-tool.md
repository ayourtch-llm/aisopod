# Issue 055: Implement Subagent Spawning Tool

## Summary
Implement a built-in subagent spawning tool that allows an agent to spawn child agents within the current session context, passing constraints and enforcing depth limits to prevent infinite recursion.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/builtins/subagent.rs`

## Current Behavior
No subagent spawning tool exists. Agents cannot delegate work to child agents.

## Expected Behavior
After this issue is completed:
- The `SubagentTool` struct implements the `Tool` trait.
- It accepts parameters: `agent_name` (string, required), `prompt` (string, required), optional `model` (string), and optional `max_depth` override.
- The tool checks the current agent depth against the configured maximum depth and rejects the spawn if the limit would be exceeded.
- An allowlist of permitted subagent model names is enforced.
- The actual agent spawning will be delegated to a pluggable `AgentSpawner` trait (to be wired to the agent engine later).
- The tool returns the subagent's response or an error if spawning fails.

## Impact
Subagent spawning enables complex multi-step workflows where a primary agent delegates subtasks to specialized child agents. Depth limits are essential to prevent infinite recursion that could exhaust resources.

## Suggested Implementation
1. **Create `subagent.rs`** — Add `crates/aisopod-tools/src/builtins/subagent.rs`.

2. **Define an `AgentSpawner` trait**:
   ```rust
   #[async_trait]
   pub trait AgentSpawner: Send + Sync {
       async fn spawn(
           &self,
           agent_name: &str,
           prompt: &str,
           model: Option<&str>,
           parent_context: &ToolContext,
       ) -> Result<String>;
   }
   ```

3. **Define `SubagentTool`**:
   ```rust
   pub struct SubagentTool {
       spawner: Arc<dyn AgentSpawner>,
       max_depth: u32,
       model_allowlist: Option<Vec<String>>,
   }
   ```

4. **Implement `Tool` for `SubagentTool`**:
   - `name()` → `"subagent"`
   - `description()` → `"Spawn a child agent to handle a subtask"`
   - `parameters_schema()` → JSON Schema with `agent_name`, `prompt`, `model` properties.
   - `execute()`:
     1. Parse the required `agent_name` and `prompt` parameters.
     2. Check the current depth (tracked via `ToolContext` metadata or a dedicated field). If it exceeds `max_depth`, return an error `ToolResult` with a message like "Maximum agent depth (N) exceeded".
     3. If a `model` is specified and an allowlist is configured, verify the model is in the allowlist.
     4. Call `self.spawner.spawn(...)`.
     5. Return the spawned agent's response as a `ToolResult`.

5. **Create a no-op `AgentSpawner` implementation** for testing.

6. **Register the tool** — Ensure the tool can be registered with the `ToolRegistry`.

7. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 050 (Tool registry)

## Acceptance Criteria
- [ ] `SubagentTool` implements the `Tool` trait.
- [ ] Subagents can be spawned with a name and prompt.
- [ ] Depth limit enforcement prevents spawning beyond the configured maximum.
- [ ] Model allowlist restricts which models subagents can use.
- [ ] `parameters_schema()` returns a valid JSON Schema.
- [ ] `cargo check -p aisopod-tools` compiles without errors.

---
*Created: 2026-02-15*
