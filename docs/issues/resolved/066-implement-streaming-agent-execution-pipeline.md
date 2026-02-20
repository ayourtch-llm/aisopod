# Issue 066: Implement Streaming Agent Execution Pipeline

## Summary
Implement the core agent execution loop that ties together agent resolution, model selection, tool preparation, system prompt construction, transcript repair, model calling, tool call handling, and event streaming. This is the central pipeline that powers all agent interactions.

## Location
- Crate: `aisopod-agent`
- Files: `crates/aisopod-agent/src/pipeline.rs`, `crates/aisopod-agent/src/runner.rs`

## Current Behavior
`AgentRunner::run()` is a stub returning `todo!()`. No execution pipeline exists.

## Expected Behavior
After this issue is completed:
- `AgentRunner::run()` executes the full pipeline: resolve agent → select model → prepare tools → build system prompt → repair transcript → call model → handle tool calls → stream events → return result.
- Events are streamed to subscribers via `tokio::mpsc` channels.
- The tool call loop works: when the model requests a tool call, the tool is executed, the result is returned to the model, and the loop continues until the model produces a final response.
- `AgentRunStream` wraps the event receiver for consumers.

## Impact
This is the heart of the agent engine. Without it, no agent can execute. It integrates all previously built components (resolution, prompt, transcript, providers, tools) into a working execution loop.

## Suggested Implementation
1. **Create `crates/aisopod-agent/src/pipeline.rs`:**
   - Define `AgentPipeline` struct or implement the pipeline as methods on `AgentRunner`.
   - Implement the execution sequence:
     ```rust
     pub async fn execute(
         &self,
         params: AgentRunParams,
         event_tx: mpsc::Sender<AgentEvent>,
     ) -> Result<AgentRunResult> {
         // 1. Resolve agent ID
         let agent_id = resolve_session_agent_id(&self.config, &params.session_key)?;
         // 2. Resolve agent config
         let agent_config = resolve_agent_config(&self.config, &agent_id)?;
         // 3. Resolve model chain (primary + fallbacks)
         let model_chain = resolve_agent_model(&self.config, &agent_id)?;
         // 4. Prepare tool set (filtered by policy)
         let tools = self.tools.schemas_for_agent(&agent_id);
         // 5. Build system prompt
         let system_prompt = SystemPromptBuilder::new()
             .with_base_prompt(&agent_config.system_prompt)
             .with_dynamic_context()
             .with_tool_descriptions(&tools)
             .build();
         // 6. Repair message transcript
         let messages = repair_transcript(&params.messages, provider_kind);
         // 7. Call model in a loop
         loop {
             let response = provider.chat_stream(request).await?;
             // Stream TextDelta events
             // If tool calls requested, execute them and continue
             // If no tool calls, break with final result
         }
     }
     ```

2. **Tool call loop:**
   - When the model response includes tool calls, iterate through each one.
   - Send `AgentEvent::ToolCallStart` for each tool call.
   - Execute the tool via the tool registry.
   - Send `AgentEvent::ToolCallResult` with the result.
   - Append the tool results to the message history and call the model again.
   - Repeat until the model returns a text response without tool calls.

3. **Event streaming:**
   - Create a `tokio::mpsc::channel` at the start of `run()`.
   - Pass the sender to the pipeline.
   - Return the receiver wrapped in `AgentRunStream`.
   - Send `AgentEvent::TextDelta` as streaming text chunks arrive.
   - Send `AgentEvent::Complete` when the pipeline finishes.
   - Send `AgentEvent::Error` if the pipeline fails.

4. **Update `runner.rs`:**
   - Replace the `todo!()` in `run()` with the pipeline invocation.
   - Wire up the `subscribe()` method to use `tokio::broadcast` for multi-subscriber support.

5. **Update `crates/aisopod-agent/src/lib.rs`:**
   - Add `pub mod pipeline;`.

6. **Verify** — Run `cargo check -p aisopod-agent`.

## Dependencies
- Issue 062 (Agent types and AgentRunner skeleton)
- Issue 063 (Agent resolution and binding)
- Issue 064 (System prompt construction)
- Issue 065 (Message transcript repair)
- Issue 038 (ModelProvider trait — for calling models)
- Issue 050 (Tool registry — for executing tools)

## Acceptance Criteria
- [x] `AgentRunner::run()` executes the full pipeline from resolution to completion.
- [x] Events are streamed via `tokio::mpsc` to subscribers.
- [x] The tool call loop executes tools and returns results to the model.
- [x] `TextDelta`, `ToolCallStart`, `ToolCallResult`, `Complete`, and `Error` events are emitted at the correct points.
- [x] `AgentRunStream` provides an ergonomic receiver interface.
- [x] `cargo check -p aisopod-agent` succeeds without errors.

## Resolution

The streaming agent execution pipeline was implemented as specified:

### Changes Made:
1. **Created `crates/aisopod-agent/src/pipeline.rs`**:
   - `AgentPipeline` struct implementing the full execution sequence:
     - Agent resolution via `resolve_session_agent_id()`
     - Agent config retrieval via `resolve_agent_config()`
     - Model chain resolution via `resolve_agent_model()`
     - Tool set preparation via `self.tools.schemas_for_agent()`
     - System prompt construction using `SystemPromptBuilder`
     - Transcript repair using `repair_transcript()`
     - Model calling in a loop with tool call handling
   - Tool call loop implementation:
     - `AgentEvent::ToolCallStart` emitted for each tool
     - Tool execution via tool registry
     - `AgentEvent::ToolCallResult` with results
     - Continue until model returns final text response
   - Event streaming via `tokio::mpsc` channels
   - `AgentRunStream` wrapper for the event receiver

2. **Updated `crates/aisopod-agent/src/runner.rs`**:
   - Replaced `todo!()` in `AgentRunner::run()` with pipeline invocation
   - Wired up `subscribe()` for multi-subscriber support

3. **Updated `crates/aisopod-agent/src/lib.rs`**:
   - `pipeline` module already exposed

### Commits:
- `c6c0f39`: Initial implementation of streaming agent execution pipeline
- `05d4e82`: Fix pipeline to properly handle tool call loop

### Verification:
- `cargo test -p aisopod-agent`: 108 tests passed
- `cargo build` at top level: succeeded
- `cargo check -p aisopod-agent`: clean
- No `todo!()` macros in `aisopod-agent`

---
*Created: 2026-02-15*
*Resolved: 2026-02-20*
