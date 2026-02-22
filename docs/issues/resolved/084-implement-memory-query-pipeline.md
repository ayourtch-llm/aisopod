# Issue 084: Implement Memory Query Pipeline

## Summary
Implement the end-to-end memory query pipeline that takes a natural-language query, generates an embedding, performs vector similarity search, applies filters, re-ranks results, and formats them for context injection into an agent's prompt.

## Location
- Crate: `aisopod-memory`
- File: `crates/aisopod-memory/src/pipeline.rs`

## Current Behavior
The `SqliteMemoryStore` (Issue 082) can perform raw vector queries and the `EmbeddingProvider` (Issue 083) can generate embeddings, but there is no orchestration layer that ties them together into a coherent query flow.

## Expected Behavior
A `MemoryQueryPipeline` struct orchestrates the full query flow:
1. Accept a natural-language query string and options.
2. Generate a query embedding via the `EmbeddingProvider`.
3. Execute vector similarity search via the `MemoryStore`.
4. Apply post-retrieval filters (agent_id, tags, importance threshold).
5. Re-rank results by combining similarity score with importance and recency.
6. Format the top results as a context string suitable for injection into a system prompt.

## Impact
This is the primary interface that the agent engine uses to retrieve relevant memories. Without this pipeline, agents cannot benefit from stored memories during execution.

## Suggested Implementation
1. Create `crates/aisopod-memory/src/pipeline.rs`.
2. Define `MemoryQueryPipeline` struct with fields:
   - `store: Arc<dyn MemoryStore>` — the underlying memory store.
   - `embedder: Arc<dyn EmbeddingProvider>` — the embedding provider.
3. Implement `MemoryQueryPipeline::new(store, embedder) -> Self`.
4. Implement `pub async fn query(&self, query: &str, opts: MemoryQueryOptions) -> Result<Vec<MemoryMatch>>`:
   - Call `self.embedder.embed(query)` to get the query vector.
   - Call `self.store.query(query, opts)` passing the embedding to the store (you may need to adjust the store's query method signature or pass the embedding via the options/a separate parameter).
   - Apply any post-retrieval filtering that the store doesn't handle (e.g., complex tag intersection logic).
   - Re-rank results using a combined score: `final_score = similarity_weight * score + importance_weight * importance + recency_weight * recency_factor`. Use configurable weights with sensible defaults (e.g., 0.7, 0.2, 0.1).
   - Sort by `final_score` descending and truncate to `top_k`.
   - Return the re-ranked `Vec<MemoryMatch>`.
5. Implement `pub fn format_context(&self, matches: &[MemoryMatch]) -> String`:
   - Format each matched memory as a bullet point: `- [score: 0.85] {content}`.
   - Join all bullets with newlines.
   - Wrap in a section header: `## Relevant Memories\n{bullets}`.
   - Return the formatted string.
6. Implement a convenience method `pub async fn query_and_format(&self, query: &str, opts: MemoryQueryOptions) -> Result<String>` that calls `query()` then `format_context()`.
7. Add a helper function to compute recency factor: `fn recency_factor(created_at: DateTime<Utc>) -> f32` that returns a value between 0.0 and 1.0 based on how recent the memory is (e.g., exponential decay over days).
8. Re-export `MemoryQueryPipeline` from `lib.rs`.
9. Run `cargo check -p aisopod-memory` to verify compilation.

## Dependencies
- Issue 082 (implement SQLite-Vec vector storage backend)
- Issue 083 (define embedding provider trait and OpenAI implementation)

## Resolution

The implementation was completed as follows:

1. Created `crates/aisopod-memory/src/pipeline.rs` with:
   - `MemoryQueryPipeline` struct with `store` and `embedder` fields
   - `new()` constructor that takes `Arc<dyn MemoryStore>` and `Arc<dyn EmbeddingProvider>`
   - `query()` async method that:
     - Generates query embedding via `embedder.embed()`
     - Executes vector similarity search via `store.query()`
     - Applies post-retrieval filtering
     - Re-ranks results using combined score: `score * 0.7 + importance * 0.2 + recency * 0.1`
     - Sorts by final score descending and truncates to `top_k`
   - `format_context()` method that formats results as bullet points with scores
   - `query_and_format()` convenience method that combines query and format
   - `recency_factor()` helper with exponential decay (halves every 7 days)

2. Updated `crates/aisopod-memory/src/lib.rs` to re-export `MemoryQueryPipeline`

3. Added `tempfile = "3.10"` to dev-dependencies for testing

4. Updated `crates/aisopod-memory/src/sqlite.rs` tests to use `tokio::test` and async/await

### Verification

All acceptance criteria verified:
- ✅ `MemoryQueryPipeline` accepts query string and returns ranked `MemoryMatch` results
- ✅ Query embedding is generated via `EmbeddingProvider`
- ✅ Vector similarity search is performed via `MemoryStore`
- ✅ Results are re-ranked by combined score of similarity, importance, and recency
- ✅ `format_context()` produces readable string suitable for prompt injection
- ✅ `query_and_format()` convenience method works end-to-end
- ✅ `cargo check -p aisopod-memory` compiles without errors
- ✅ All tests pass (`cargo test -p aisopod-memory`): 49 unit tests + 15 integration tests + 15 management tests + 7 search tests + 9 storage tests = 95 tests total

---
*Created: 2026-02-15*
*Resolved: 2026-02-22*
