# Issue 082: Implement SQLite-Vec Vector Storage Backend

## Summary
Implement the primary `MemoryStore` backend using `rusqlite` with the `sqlite-vec` extension for vector storage and cosine similarity search.

## Location
- Crate: `aisopod-memory`
- File: `crates/aisopod-memory/src/sqlite.rs`

## Current Behavior
The `MemoryStore` trait is defined (Issue 081) but has no concrete implementation. There is no database schema or vector storage.

## Expected Behavior
A `SqliteMemoryStore` struct implements the `MemoryStore` trait backed by SQLite with the `sqlite-vec` extension. It creates a `memories` table for metadata and a `memory_embeddings` virtual table for vector operations. It supports storing, querying (with cosine similarity), deleting, and listing memory entries with configurable embedding dimensions.

## Impact
This is the default storage backend for the entire memory system. All higher-level features (query pipeline, memory management, agent integration) depend on a working vector store.

## Suggested Implementation
1. Add `rusqlite = { version = "0.33", features = ["bundled"] }` and `sqlite-vec` to the `[dependencies]` section of `crates/aisopod-memory/Cargo.toml`.
2. Create `crates/aisopod-memory/src/sqlite.rs`.
3. Define `SqliteMemoryStore` struct holding `db: Arc<Mutex<rusqlite::Connection>>` and `embedding_dim: usize`.
4. Implement a constructor `SqliteMemoryStore::new(path: &str, embedding_dim: usize) -> Result<Self>`:
   - Open the SQLite database at `path` (use `:memory:` for tests).
   - Load the `sqlite-vec` extension using `db.load_extension()` or the appropriate API.
   - Run the schema creation SQL:
     ```sql
     CREATE TABLE IF NOT EXISTS memories (
         id TEXT PRIMARY KEY,
         agent_id TEXT NOT NULL,
         content TEXT NOT NULL,
         source TEXT NOT NULL,
         session_key TEXT,
         tags TEXT DEFAULT '[]',
         importance REAL DEFAULT 0.5,
         metadata TEXT DEFAULT '{}',
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL
     );

     CREATE VIRTUAL TABLE IF NOT EXISTS memory_embeddings USING vec0(
         id TEXT PRIMARY KEY,
         embedding float[1536]
     );
     ```
   - Adjust the `float[1536]` dimension dynamically based on `embedding_dim`.
5. Implement `MemoryStore::store()`:
   - Generate a UUID if `entry.id` is empty.
   - Insert into `memories` table (serialize tags as JSON array, metadata as JSON object).
   - Insert into `memory_embeddings` virtual table (embedding as a float vector).
   - Return the entry ID.
6. Implement `MemoryStore::query()`:
   - Accept a query embedding (you'll receive the raw text here; the query pipeline in Issue 084 will handle embedding generation before calling this).
   - Execute a vector similarity search against `memory_embeddings` using `vec_distance_cosine()`.
   - Join with the `memories` table to fetch full entry data.
   - Apply `MemoryQueryOptions.top_k` as a LIMIT.
   - Apply `MemoryQueryOptions.min_score` as a threshold filter.
   - Return `Vec<MemoryMatch>` sorted by descending similarity score.
7. Implement `MemoryStore::delete()`:
   - Delete from both `memories` and `memory_embeddings` by ID.
8. Implement `MemoryStore::list()`:
   - Build a dynamic SQL query from `MemoryFilter` fields.
   - Apply WHERE clauses for `agent_id`, `source`, `session_key`, `importance >= importance_min`, `created_at >= created_after`, `created_at <= created_before`.
   - For `tags` filter, use `json_each()` to match entries containing any of the requested tags.
   - Return `Vec<MemoryEntry>`.
9. Add an index on `memories.agent_id` for fast scoped queries.
10. Run `cargo check -p aisopod-memory` and verify compilation.

## Dependencies
- Issue 081 (define memory types and MemoryStore trait)

## Acceptance Criteria
- [ ] `SqliteMemoryStore` implements the `MemoryStore` trait
- [ ] `rusqlite` with bundled features and `sqlite-vec` extension loads successfully
- [ ] `memories` table and `memory_embeddings` virtual table are created on initialization
- [ ] `store()` inserts entries into both tables and returns the entry ID
- [ ] `query()` performs cosine similarity search and returns results sorted by score
- [ ] `delete()` removes entries from both tables
- [ ] `list()` applies all `MemoryFilter` fields correctly
- [ ] Embedding dimensions are configurable (not hardcoded to 1536)
- [ ] `cargo check -p aisopod-memory` compiles without errors

---
*Created: 2026-02-15*
