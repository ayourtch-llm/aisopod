# Issue 063: Implement Agent Resolution and Binding

## Summary
Implement the agent resolution system that maps session keys to agent configurations via binding rules. This includes `resolve_session_agent_id()`, `resolve_agent_config()`, `resolve_agent_model()`, `list_agent_ids()`, and the `AgentBinding`/`BindingMatch` types with matching logic.

## Location
- Crate: `aisopod-agent`
- Files: `crates/aisopod-agent/src/resolution.rs`, `crates/aisopod-agent/src/binding.rs`

## Current Behavior
No agent resolution logic exists. There is no way to determine which agent configuration should handle a given session.

## Expected Behavior
After this issue is completed:
- `resolve_session_agent_id(session_key)` evaluates binding rules and returns the matching agent ID.
- `resolve_agent_config(agent_id)` retrieves the full agent configuration from the config.
- `resolve_agent_model(agent_id)` resolves the primary model with its fallback chain.
- `list_agent_ids()` returns all configured agent IDs.
- `AgentBinding` and `BindingMatch` types encode the matching rules (channel, account ID, peer, guild ID).
- Binding evaluation supports wildcard and exact matching.

## Impact
Agent resolution is the first step in the execution pipeline. Without it, the system cannot determine which agent, model, or configuration to use for a given session. Every execution request flows through resolution first.

## Suggested Implementation
1. **Create `crates/aisopod-agent/src/binding.rs`:**
   - Define `AgentBinding`:
     ```rust
     pub struct AgentBinding {
         pub agent_id: String,
         pub match_rule: BindingMatch,
     }
     ```
   - Define `BindingMatch`:
     ```rust
     pub struct BindingMatch {
         pub channel: Option<String>,
         pub account_id: Option<String>,
         pub peer: Option<PeerMatch>,
         pub guild_id: Option<String>,
     }
     ```
   - Define `PeerMatch` as a struct or enum for peer matching criteria.
   - Implement `BindingMatch::matches(session_metadata: &SessionMetadata) -> bool` that checks each field. A `None` field means "match any". All non-`None` fields must match.
   - Implement `AgentBinding::evaluate(session_metadata: &SessionMetadata) -> Option<&str>` that returns the `agent_id` if the binding matches.

2. **Create `crates/aisopod-agent/src/resolution.rs`:**
   - Implement `resolve_session_agent_id(config: &AisopodConfig, session_key: &str) -> Result<String>`:
     - Parse the session key to extract metadata (channel, account, etc.).
     - Iterate through configured bindings in order.
     - Return the first matching agent ID, or a default agent if none match.
   - Implement `resolve_agent_config(config: &AisopodConfig, agent_id: &str) -> Result<AgentConfig>`:
     - Look up the agent in the configuration's agent list.
     - Return an error if the agent ID is not found.
   - Implement `resolve_agent_model(config: &AisopodConfig, agent_id: &str) -> Result<ModelChain>`:
     - Get the agent config, extract the primary model ID.
     - Build the fallback chain from the agent's `fallback_models` list.
     - Return a `ModelChain` struct with primary and fallbacks.
   - Implement `list_agent_ids(config: &AisopodConfig) -> Vec<String>`:
     - Return all agent IDs from the configuration.

3. **Update `crates/aisopod-agent/src/lib.rs`:**
   - Add `pub mod resolution;` and `pub mod binding;`.

4. **Add unit tests** in each module's `#[cfg(test)]` block:
   - Test binding matching with various field combinations.
   - Test resolution with multiple bindings (first match wins).
   - Test default agent fallback when no binding matches.
   - Test `list_agent_ids()` returns all configured agents.

5. **Verify** — Run `cargo test -p aisopod-agent`.

## Dependencies
- Issue 062 (Agent types and AgentRunner skeleton)
- Issue 016 (Core configuration types — provides agent config structures)

## Acceptance Criteria
- [ ] `AgentBinding` and `BindingMatch` types are defined with matching logic.
- [ ] `resolve_session_agent_id()` maps session keys to agent IDs via binding evaluation.
- [ ] `resolve_agent_config()` retrieves agent configuration by ID.
- [ ] `resolve_agent_model()` returns the primary model and fallback chain.
- [ ] `list_agent_ids()` enumerates all configured agents.
- [ ] Unit tests verify correct binding evaluation and resolution behavior.
- [ ] `cargo check -p aisopod-agent` succeeds without errors.

---
*Created: 2026-02-15*
