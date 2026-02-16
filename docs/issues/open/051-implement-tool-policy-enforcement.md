# Issue 051: Implement Tool Policy Enforcement

## Summary
Implement the tool allow/deny policy system that controls which tools an agent is permitted to use. This includes a `ToolPolicy` struct with optional allow and deny lists, a `ToolPolicyEngine` that evaluates policies at both global and per-agent levels, and clear denial messages when a tool is blocked.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/policy.rs`

## Current Behavior
No tool policy mechanism exists. Any tool could be invoked without restriction.

## Expected Behavior
After this issue is completed:
- A `ToolPolicy` struct defines optional `allow` and `deny` lists of tool names.
- A `ToolPolicyEngine` holds a global policy and a map of per-agent policies.
- `is_allowed(agent_id, tool_name)` evaluates the combined policy and returns whether the tool is permitted.
- When a tool is denied, the engine returns a clear message explaining why (e.g., "Tool 'bash' is denied by agent policy for agent 'agent-1'").

## Impact
Policy enforcement is a security-critical feature. Without it, agents could use any tool without restriction, which is unsafe in multi-agent or user-facing environments.

## Suggested Implementation
1. **Create `policy.rs`** — Add `crates/aisopod-tools/src/policy.rs`.

2. **Define `ToolPolicy`**:
   ```rust
   pub struct ToolPolicy {
       pub allow: Option<Vec<String>>,
       pub deny: Option<Vec<String>>,
   }
   ```
   Document the semantics: if `allow` is `Some`, only listed tools are permitted (whitelist mode). If `deny` is `Some`, listed tools are blocked (blacklist mode). If both are set, `deny` takes precedence.

3. **Define `ToolPolicyEngine`**:
   ```rust
   pub struct ToolPolicyEngine {
       global_policy: ToolPolicy,
       agent_policies: HashMap<String, ToolPolicy>,
   }
   ```

4. **Implement `is_allowed()`**:
   - First check the global `deny` list. If the tool is in it, deny with a message.
   - Then check the per-agent `deny` list. If the tool is in it, deny with a message.
   - If the per-agent policy has an `allow` list, the tool must be in it.
   - If the global policy has an `allow` list, the tool must be in it.
   - Return a `Result<(), String>` where the `Err` variant contains the denial reason.

5. **Implement `set_global_policy()`** and **`set_agent_policy(agent_id, policy)`** methods.

6. **Implement `remove_agent_policy(agent_id)`** for cleanup when an agent is destroyed.

7. **Re-export `ToolPolicy` and `ToolPolicyEngine` from `lib.rs`**.

8. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 050 (Tool registry)
- Issue 016 (Define core configuration types)

## Acceptance Criteria
- [ ] `ToolPolicy` struct is defined with `allow` and `deny` fields.
- [ ] `ToolPolicyEngine` supports global and per-agent policies.
- [ ] `is_allowed()` correctly evaluates combined policies.
- [ ] Denied tools produce clear, descriptive denial messages.
- [ ] `cargo check -p aisopod-tools` compiles without errors.

---
*Created: 2026-02-15*
