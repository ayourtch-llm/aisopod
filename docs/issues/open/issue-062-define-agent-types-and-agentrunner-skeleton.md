# Issue 062: Define Agent Types and AgentRunner Skeleton

## Summary
Define the core agent execution types (`AgentRunParams`, `AgentRunResult`, `AgentEvent` enum) and create the `AgentRunner` struct that holds shared references to config, provider registry, tool registry, and session store. Stub out the `run()`, `subscribe()`, and `abort()` methods.

## Location
- Crate: `aisopod-agent`
- Files: `crates/aisopod-agent/src/types.rs`, `crates/aisopod-agent/src/runner.rs`, `crates/aisopod-agent/src/lib.rs`

## Current Behavior
The `aisopod-agent` crate exists as an empty skeleton with only a doc comment in `lib.rs`. There are no agent execution types or runner logic.

## Expected Behavior
After this issue is completed:
- `AgentRunParams` struct exists with fields for messages, session key, and optional agent ID.
- `AgentRunResult` struct exists with fields for the final response, tool call results, and usage info.
- `AgentEvent` enum exists with variants: `TextDelta`, `ToolCallStart`, `ToolCallResult`, `ModelSwitch`, `Error`, `Complete`, `Usage`.
- `AgentRunner` struct holds `Arc<AisopodConfig>`, `Arc<ProviderRegistry>`, `Arc<ToolRegistry>`, and `Arc<SessionStore>`.
- `AgentRunner` has stubbed `run()`, `subscribe()`, and `abort()` methods that compile but return `todo!()` or placeholder values.

## Impact
These types form the public API surface of the agent engine. Every other agent subsystem (resolution, failover, compaction, streaming) depends on these foundational types. Defining them first establishes the contract that all downstream issues build against.

## Suggested Implementation
1. **Create `crates/aisopod-agent/src/types.rs`:**
   - Define `AgentRunParams` with fields: `session_key: String`, `messages: Vec<Message>`, `agent_id: Option<String>`.
   - Define `AgentRunResult` with fields: `response: String`, `tool_calls: Vec<ToolCallRecord>`, `usage: UsageReport`.
   - Define the `AgentEvent` enum:
     ```rust
     pub enum AgentEvent {
         TextDelta { text: String },
         ToolCallStart { tool_name: String, call_id: String },
         ToolCallResult { call_id: String, result: String, is_error: bool },
         ModelSwitch { from: String, to: String, reason: String },
         Error { message: String },
         Complete { result: AgentRunResult },
         Usage { usage: UsageReport },
     }
     ```
   - Define `UsageReport` with fields: `input_tokens: u64`, `output_tokens: u64`.
   - Derive `Clone`, `Debug`, `Serialize`, `Deserialize` on all types.

2. **Create `crates/aisopod-agent/src/runner.rs`:**
   - Define `AgentRunner`:
     ```rust
     pub struct AgentRunner {
         config: Arc<AisopodConfig>,
         providers: Arc<ProviderRegistry>,
         tools: Arc<ToolRegistry>,
         sessions: Arc<SessionStore>,
     }
     ```
   - Implement `AgentRunner::new(config, providers, tools, sessions)` constructor.
   - Stub `pub async fn run(&self, params: AgentRunParams) -> Result<AgentRunStream>` with `todo!()`.
   - Stub `pub fn subscribe(&self, session_key: &str) -> broadcast::Receiver<AgentEvent>` with `todo!()`.
   - Stub `pub async fn abort(&self, session_key: &str) -> Result<()>` with `todo!()`.

3. **Update `crates/aisopod-agent/src/lib.rs`:**
   - Add `pub mod types;` and `pub mod runner;`.
   - Re-export the key types from the crate root.

4. **Verify** — Run `cargo check -p aisopod-agent` to confirm everything compiles.

## Dependencies
- Issue 008 (Create aisopod-agent crate)
- Issue 016 (Core configuration types — provides `AisopodConfig`)
- Issue 039 (Provider registry — provides `ProviderRegistry`)
- Issue 050 (Tool registry — provides `ToolRegistry`)

## Acceptance Criteria
- [ ] `AgentRunParams`, `AgentRunResult`, and `AgentEvent` types are defined and derive standard traits.
- [ ] `AgentRunner` struct holds `Arc` references to config, providers, tools, and sessions.
- [ ] `run()`, `subscribe()`, and `abort()` methods exist as stubs and compile.
- [ ] `cargo check -p aisopod-agent` succeeds without errors.
- [ ] Types are re-exported from the crate root.

---
*Created: 2026-02-15*
