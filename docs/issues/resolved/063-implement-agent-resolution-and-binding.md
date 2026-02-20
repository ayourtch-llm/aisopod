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
- [x] `AgentBinding` and `BindingMatch` types are defined with matching logic.
- [x] `resolve_session_agent_id()` maps session keys to agent IDs via binding evaluation.
- [x] `resolve_agent_config()` retrieves agent configuration by ID.
- [x] `resolve_agent_model()` returns the primary model and fallback chain.
- [x] `list_agent_ids()` enumerates all configured agents.
- [x] Unit tests verify correct binding evaluation and resolution behavior.
- [x] `cargo check -p aisopod-agent` succeeds without errors.

## Resolution

The agent resolution and binding system was implemented as specified:

### Changes Made:
1. **Created `crates/aisopod-agent/src/binding.rs`**:
   - `AgentBinding` struct with `agent_id` and `match_rule` fields
   - `BindingMatch` struct with `channel`, `account_id`, `peer` (Option<PeerMatch>), and `guild_id` fields
   - `PeerMatch` enum with `Any`, `Id`, and `Pattern` variants (with Serialize/Deserialize)
   - `BindingMatch::matches()` method implementing matching logic with wildcard support
   - `AgentBinding::evaluate()` method returning agent_id on match

2. **Created `crates/aisopod-agent/src/resolution.rs`**:
   - `resolve_session_agent_id()` - parses session key, iterates bindings, returns matching agent_id or default
   - `resolve_agent_config()` - retrieves agent config by ID from configuration
   - `resolve_agent_model()` - returns ModelChain with primary model and fallback chain
   - `list_agent_ids()` - returns all configured agent IDs

3. **Updated `crates/aisopod-agent/src/lib.rs`**:
   - Added `pub mod resolution;` and `pub mod binding;`

4. **Unit tests** in each module:
   - Binding match tests for various field combinations
   - Resolution tests for multiple bindings (first match wins)
   - Default agent fallback when no binding matches
   - Model chain construction tests

### Commits:
- `27dcfec`: Initial implementation of issue 063
- `6fb2851`: Fix BindingMatch.peer to use PeerMatch type

### Verification:
- `cargo test -p aisopod-agent`: 108 tests passed
- `cargo build` at top level: succeeded
- `cargo check -p aisopod-agent`: clean

---
*Created: 2026-02-15*
*Resolved: 2026-02-20*
