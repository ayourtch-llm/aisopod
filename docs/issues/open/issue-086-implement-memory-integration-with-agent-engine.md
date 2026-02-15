# Issue 086: Implement Memory Integration with Agent Engine

## Summary
Integrate the memory system with the agent execution engine so that relevant memories are automatically injected into agent context before each run, an agent tool enables explicit memory operations, and key facts are extracted after each run.

## Location
- Crate: `aisopod-memory` and `aisopod-agent`
- File: `crates/aisopod-memory/src/integration.rs` and `crates/aisopod-agent/src/memory.rs`

## Current Behavior
The memory system (store, query pipeline, management) operates independently. The agent engine has no awareness of memories and cannot retrieve or store them during execution.

## Expected Behavior
- **Pre-run:** Before an agent run, the system queries relevant memories using the current conversation context and injects them into the system prompt.
- **Agent tool:** An explicit `memory` tool is available to agents for storing, querying, and deleting memories during execution.
- **Post-run:** After an agent run completes, key facts are automatically extracted from the conversation and stored as new memories.

## Impact
This is the feature that makes the memory system useful to end users. Without integration, memories exist in isolation and agents cannot leverage persistent context across sessions.

## Suggested Implementation
1. **Pre-run memory injection** — create `crates/aisopod-memory/src/integration.rs`:
   - Define `pub async fn build_memory_context(pipeline: &MemoryQueryPipeline, agent_id: &str, conversation: &[Message], opts: MemoryQueryOptions) -> Result<String>`:
     - Extract the last N messages (e.g., last 5) from the conversation as the query context.
     - Concatenate their content into a single query string.
     - Call `pipeline.query_and_format(&query, opts)` to get the formatted memory context.
     - Return the formatted string.
   - In the agent engine (`crates/aisopod-agent/src/memory.rs`), before constructing the system prompt, call `build_memory_context()` and append the result to the system prompt.

2. **Agent memory tool** — define a tool that agents can invoke:
   - Create a `MemoryTool` struct implementing the `Tool` trait (from Issue 049).
   - The tool accepts JSON arguments with an `action` field: `"store"`, `"query"`, or `"delete"`.
   - For `"store"`: accept `content` and optional `tags`, generate an embedding, and call `store.store()`.
   - For `"query"`: accept `query` and optional `top_k`, call the query pipeline, and return formatted results.
   - For `"delete"`: accept `id` and call `store.delete()`.
   - Register the tool schema with the tool registry so models can invoke it.

3. **Post-run memory extraction** — in the agent engine, after a run completes:
   - Call `MemoryManager::extract_memories(agent_id, conversation)` to extract and store key facts from the conversation.
   - Optionally run `MemoryManager::maintain(agent_id)` to clean up.

4. **Wiring it together** — in the `AgentRunner` (from Issue 062):
   - Accept optional `MemoryQueryPipeline` and `MemoryManager` in the runner's configuration or constructor.
   - If memory is configured, run pre-query before execution and post-extraction after execution.
   - If memory is not configured, skip memory steps (memory is optional).

5. Re-export integration functions from `aisopod-memory/src/lib.rs`.
6. Run `cargo check -p aisopod-memory -p aisopod-agent` to verify compilation.

## Dependencies
- Issue 084 (implement memory query pipeline)
- Issue 062 (define agent types and AgentRunner skeleton)
- Issue 066 (implement streaming agent execution pipeline)

## Acceptance Criteria
- [ ] Relevant memories are queried and injected into the system prompt before each agent run
- [ ] The `MemoryTool` supports `store`, `query`, and `delete` actions and is registered with the tool system
- [ ] Post-run memory extraction stores key facts from the conversation
- [ ] Memory integration is optional — agents work correctly with or without memory configured
- [ ] `cargo check -p aisopod-memory -p aisopod-agent` compiles without errors

---
*Created: 2026-02-15*
