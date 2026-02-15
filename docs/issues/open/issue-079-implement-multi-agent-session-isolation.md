# Issue 079: Implement Multi-Agent Session Isolation

## Summary
Ensure that sessions are strictly scoped by `agent_id` so that one agent cannot access or modify another agent's sessions. Provide a separate admin-level listing that can query across agents for monitoring purposes.

## Location
- Crate: `aisopod-session`
- File: `crates/aisopod-session/src/store.rs`

## Current Behavior
The session store CRUD operations accept a `SessionKey` that includes `agent_id`, but there is no enforcement layer to prevent one agent from querying or mutating another agent's sessions if a key is constructed with a different `agent_id`.

## Expected Behavior
All session store operations validate that the caller's agent ID matches the session's `agent_id`. A separate `list_all_sessions` method is available for admin views that need cross-agent visibility, clearly distinguished from the agent-scoped operations.

## Impact
Without isolation, a misconfigured or malicious agent could read or corrupt another agent's conversation history. This is a correctness and security concern in multi-agent deployments.

## Suggested Implementation
1. Open `crates/aisopod-session/src/store.rs`.
2. Add an `AgentScope` struct or use a simple `agent_id: &str` parameter to represent the calling agent's identity.
3. Modify `get_or_create`, `list`, `patch`, `delete`, `append_messages`, `get_history`, and `compact` to accept an `agent_id` scope parameter:
   - Before performing any database operation, assert that the `SessionKey.agent_id` matches the scope's `agent_id`.
   - If they do not match, return an `Err` with a clear message like `"access denied: agent 'X' cannot access sessions owned by agent 'Y'"`.
4. For `list`, ensure the query always includes `WHERE agent_id = ?` using the scope, regardless of what the `SessionFilter` contains. If the filter specifies a different `agent_id`, return an error or override it with the scoped value.
5. Add a new method `list_all_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>>`:
   - This method does NOT require an agent scope.
   - It queries across all agents, using only the filter criteria.
   - This is intended for admin/monitoring tools, not for agent runtime use.
6. Consider marking `list_all_sessions` with a doc comment warning that it bypasses agent isolation and should only be used by trusted admin code.
7. Add helper method `SessionStore::verify_scope(agent_id: &str, key: &SessionKey) -> Result<()>` to centralize the check.

## Dependencies
- Issue 075 (implement SessionStore core CRUD operations)

## Acceptance Criteria
- [ ] All agent-facing session operations require an agent scope parameter
- [ ] Operations return an error if the session key's `agent_id` does not match the scope
- [ ] `list` always filters by the scoped `agent_id`
- [ ] `list_all_sessions` can query sessions across all agents without a scope
- [ ] `list_all_sessions` is clearly documented as an admin-only method
- [ ] Agent A cannot read, write, or delete agent B's sessions through any agent-scoped method
- [ ] `cargo check -p aisopod-session` succeeds

---
*Created: 2026-02-15*
