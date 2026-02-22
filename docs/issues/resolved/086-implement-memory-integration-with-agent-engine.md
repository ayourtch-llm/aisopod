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
- [x] Relevant memories are queried and injected into the system prompt before each agent run
- [x] The `MemoryTool` supports `store`, `query`, and `delete` actions and is registered with the tool system
- [x] Post-run memory extraction stores key facts from the conversation
- [x] Memory integration is optional — agents work correctly with or without memory configured
- [x] `cargo check -p aisopod-memory -p aisopod-agent` compiles without errors

## Resolution
The memory integration was implemented in commit c39b2e7 with the following changes:

### Files Modified
- `crates/aisopod-memory/src/integration.rs` - New file implementing:
  - `build_memory_context()` - Pre-run memory injection function
  - Helper functions for memory context building
  - Integration tests

- `crates/aisopod-agent/src/memory.rs` - New file implementing:
  - `MemoryConfig` struct for memory integration configuration
  - `inject_memory_context()` - Pre-run memory context injection into system prompts
  - `extract_memories_after_run()` - Post-run memory extraction from conversations
  - `MemoryTool` struct implementing the `Tool` trait with:
    - `handle_store()` - Store new memories with content and optional tags
    - `handle_query()` - Query memories and return formatted results
    - `handle_delete()` - Delete memories by ID
  - `create_memory_tool_schema()` - Creates the tool schema for registration

- `crates/aisopod-agent/src/runner.rs` - Updated:
  - Added `memory_pipeline` field for optional memory query pipeline
  - Added `memory_manager` field for optional memory manager
  - Added `memory_config` field for memory configuration
  - Updated `AgentRunner::new()` to accept optional memory components
  - Memory steps are skipped if not configured (memory is optional)

- `crates/aisopod-agent/src/pipeline.rs` - Updated to use memory integration
- `crates/aisopod-memory/src/lib.rs` - Re-exported `build_memory_context`
- `crates/aisopod-memory/src/management.rs` - Added `extract_memories()` method
- `crates/aisopod-memory/src/pipeline.rs` - Added `query_and_format()` method

### Follow-up Fixes
- Commit edf5b05: Fixed grammar in memory tool description and sqlite-vec integration
- Commit 64e0233: Fixed formatting issues and clippy warnings

### Testing
- All integration tests pass (`cargo test -p aisopod-memory`)
- All agent tests pass (`cargo test -p aisopod-agent`)
- Full test suite passes (`cargo test`)

---
*Created: 2026-02-15*
*Resolved: 2026-02-22*
