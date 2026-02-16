# Issue 088: Add Memory System Unit Tests

## Summary
Add comprehensive unit tests for all memory system functionality: vector storage and retrieval, similarity search ranking, memory management (expiration, consolidation, quota enforcement), and agent integration.

## Location
- Crate: `aisopod-memory`
- File: `crates/aisopod-memory/tests/` and/or `crates/aisopod-memory/src/*.rs` (inline tests)

## Current Behavior
The memory system crate has implementation code but no automated tests to verify correctness.

## Expected Behavior
A full test suite exercises every public API in the `aisopod-memory` crate, covering happy paths, edge cases, and error conditions. Tests run against in-memory SQLite databases for speed and isolation. A mock `EmbeddingProvider` is used to avoid real API calls in tests.

## Impact
Without tests, regressions can be introduced silently. Tests provide confidence that the memory system behaves correctly and serve as living documentation for expected behavior.

## Suggested Implementation
1. Create a test helper `MockEmbeddingProvider` that returns deterministic embeddings (e.g., a hash of the input text normalized to a unit vector). This avoids needing real API keys in tests.
2. Create a test helper function `fn test_store() -> SqliteMemoryStore` that opens an in-memory SQLite database (`:memory:`) and initializes the schema. Use this in every test for isolation.
3. **Vector storage and retrieval tests** (`test_storage.rs` or inline in `sqlite.rs`):
   - `test_store_and_retrieve` — store a memory entry, list it back, verify all fields match.
   - `test_store_generates_id` — store an entry with an empty ID, verify a UUID is generated.
   - `test_delete_entry` — store an entry, delete it, verify it no longer appears in list.
   - `test_delete_nonexistent` — delete an ID that doesn't exist, verify no error (idempotent).
   - `test_list_empty` — list from an empty store, verify an empty vector is returned.
   - `test_list_with_agent_filter` — store entries for agents A and B, list with agent_id filter, verify only matching entries are returned.
   - `test_list_with_tag_filter` — store entries with different tags, filter by tag, verify correct results.
   - `test_list_with_importance_filter` — store entries with varying importance, filter by minimum importance, verify threshold is applied.
4. **Similarity search tests** (`test_search.rs` or inline in `sqlite.rs`):
   - `test_similarity_search_returns_closest` — store entries with known embeddings, query with a vector close to one of them, verify it ranks highest.
   - `test_similarity_search_top_k` — store 20 entries, query with top_k=5, verify exactly 5 results are returned.
   - `test_similarity_search_min_score` — store entries, query with a high min_score, verify only entries above the threshold are returned.
   - `test_similarity_search_agent_scoped` — store entries for agents A and B, query scoped to agent A, verify no agent B results appear.
5. **Memory management tests** (`test_management.rs` or inline in `management.rs`):
   - `test_expire_old_entries` — store entries with old timestamps, run expiration, verify they are deleted.
   - `test_expire_preserves_important` — store an old entry with high importance, run expiration, verify it is preserved.
   - `test_consolidate_similar` — store two entries with very similar embeddings, run consolidation, verify they are merged into one.
   - `test_consolidate_preserves_different` — store two entries with dissimilar embeddings, run consolidation, verify both remain.
   - `test_enforce_quota` — store more entries than the quota allows, run quota enforcement, verify the count is reduced and lowest-importance entries were evicted.
   - `test_maintain_runs_all` — run `maintain()` and verify expiration, consolidation, and quota enforcement all execute.
6. **Agent integration tests** (`test_integration.rs`):
   - `test_build_memory_context` — store memories, call `build_memory_context()` with a conversation, verify a formatted context string is returned.
   - `test_memory_tool_store` — invoke the memory tool with a store action, verify the memory is persisted.
   - `test_memory_tool_query` — store memories, invoke the memory tool with a query action, verify results are returned.
   - `test_memory_tool_delete` — store a memory, invoke the memory tool with a delete action, verify it is removed.
   - `test_no_memory_configured` — run an agent without memory configured, verify it completes successfully without errors.
7. Run all tests with `cargo test -p aisopod-memory` and verify they pass.

## Dependencies
- Issue 081 (define memory types and MemoryStore trait)
- Issue 082 (implement SQLite-Vec vector storage backend)
- Issue 083 (define embedding provider trait and OpenAI implementation)
- Issue 084 (implement memory query pipeline)
- Issue 085 (implement automatic memory management)
- Issue 086 (implement memory integration with agent engine)
- Issue 087 (add LanceDB alternative backend)

## Acceptance Criteria
- [ ] A `MockEmbeddingProvider` generates deterministic embeddings without API calls
- [ ] Vector storage tests cover store, retrieve, delete, and list with various filters
- [ ] Similarity search tests verify ranking, top-K limits, min-score thresholds, and agent scoping
- [ ] Memory management tests verify expiration, consolidation, and quota enforcement
- [ ] Agent integration tests verify pre-run context injection, memory tool operations, and post-run extraction
- [ ] All tests use in-memory SQLite for speed and isolation
- [ ] `cargo test -p aisopod-memory` passes with all tests green

---
*Created: 2026-02-15*
