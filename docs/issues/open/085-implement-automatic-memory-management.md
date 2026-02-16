# Issue 085: Implement Automatic Memory Management

## Summary
Implement automatic memory lifecycle management including extraction of key facts from conversations, importance scoring, memory consolidation (merging similar entries), expiration policies (time-based and importance-based), and per-agent storage quota enforcement.

## Location
- Crate: `aisopod-memory`
- File: `crates/aisopod-memory/src/management.rs`

## Current Behavior
Memories can be stored and queried (Issues 082–084), but there is no automation around what gets stored, when memories expire, or how storage limits are enforced. All memory operations must be performed manually.

## Expected Behavior
A `MemoryManager` struct provides automatic memory lifecycle management:
- Extracts key facts from conversation transcripts and stores them as memories.
- Scores importance of each memory based on frequency, recency, and explicit marking.
- Consolidates similar memories into a single entry to reduce redundancy.
- Expires old or low-importance memories according to configurable policies.
- Enforces per-agent storage quotas, evicting lowest-importance memories when limits are exceeded.

## Impact
Without automatic management, the memory store will grow unbounded and contain redundant entries, degrading query performance and relevance. This feature keeps the memory system clean and focused.

## Suggested Implementation
1. Create `crates/aisopod-memory/src/management.rs`.
2. Define `MemoryManagerConfig` struct with fields:
   - `max_memories_per_agent: usize` (default: 1000)
   - `expiration_days: Option<u64>` (default: 90 days)
   - `min_importance_threshold: f32` (default: 0.1)
   - `consolidation_similarity_threshold: f32` (default: 0.92)
3. Define `MemoryManager` struct with fields:
   - `store: Arc<dyn MemoryStore>`
   - `embedder: Arc<dyn EmbeddingProvider>`
   - `config: MemoryManagerConfig`
4. **Fact extraction** — implement `pub async fn extract_memories(&self, agent_id: &str, conversation: &[Message]) -> Result<Vec<MemoryEntry>>`:
   - Iterate through conversation messages.
   - Identify key facts, decisions, preferences, and instructions (use simple heuristics: look for sentences with named entities, assertions, user preferences indicated by "I like", "remember that", etc.).
   - For each extracted fact, generate an embedding and create a `MemoryEntry` with `importance` set based on heuristics (explicit "remember" = 0.9, general fact = 0.5).
   - Store each entry via the `MemoryStore`.
   - Return the list of newly created entries.
5. **Importance scoring** — implement `pub fn score_importance(&self, entry: &MemoryEntry, access_count: u32, last_accessed: DateTime<Utc>) -> f32`:
   - Compute a weighted score: `base_importance * 0.4 + frequency_factor * 0.3 + recency_factor * 0.3`.
   - `frequency_factor` = `min(1.0, access_count as f32 / 10.0)`.
   - `recency_factor` = exponential decay based on days since last access.
   - Clamp result to 0.0–1.0.
6. **Memory consolidation** — implement `pub async fn consolidate(&self, agent_id: &str) -> Result<u32>`:
   - List all memories for the agent.
   - For each pair of memories, compute cosine similarity of their embeddings.
   - If similarity exceeds `consolidation_similarity_threshold`, merge the two entries: combine content, keep the higher importance, update the embedding to the average, delete the duplicate.
   - Return the number of entries consolidated.
7. **Memory expiration** — implement `pub async fn expire(&self, agent_id: &str) -> Result<u32>`:
   - List all memories for the agent.
   - Delete entries where `created_at` is older than `expiration_days` AND `importance` is below `min_importance_threshold`.
   - Return the number of entries expired.
8. **Quota enforcement** — implement `pub async fn enforce_quota(&self, agent_id: &str) -> Result<u32>`:
   - Count the agent's memories. If count exceeds `max_memories_per_agent`, delete the lowest-importance entries until the count is within limits.
   - Return the number of entries evicted.
9. Implement a convenience method `pub async fn maintain(&self, agent_id: &str) -> Result<()>` that runs expiration, consolidation, and quota enforcement in sequence.
10. Re-export `MemoryManager` and `MemoryManagerConfig` from `lib.rs`.
11. Run `cargo check -p aisopod-memory` to verify compilation.

## Dependencies
- Issue 082 (implement SQLite-Vec vector storage backend)
- Issue 083 (define embedding provider trait and OpenAI implementation)
- Issue 084 (implement memory query pipeline)

## Acceptance Criteria
- [ ] `MemoryManager` extracts key facts from conversation transcripts and stores them
- [ ] Importance scoring combines base importance, frequency, and recency
- [ ] Memory consolidation merges entries above the similarity threshold
- [ ] Memory expiration deletes old, low-importance entries
- [ ] Per-agent storage quotas are enforced by evicting lowest-importance entries
- [ ] `maintain()` runs all management tasks in sequence
- [ ] All thresholds and limits are configurable via `MemoryManagerConfig`
- [ ] `cargo check -p aisopod-memory` compiles without errors

---
*Created: 2026-02-15*
