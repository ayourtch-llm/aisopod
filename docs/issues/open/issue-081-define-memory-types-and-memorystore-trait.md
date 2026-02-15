# Issue 081: Define Memory Types and MemoryStore Trait

## Summary
Define the core memory data types (`MemoryEntry`, `MemoryMetadata`, `MemoryMatch`, `MemoryFilter`, `MemoryQueryOptions`) and the `MemoryStore` trait with `store()`, `query()`, `delete()`, and `list()` methods. These form the foundation of the aisopod memory system.

## Location
- Crate: `aisopod-memory`
- File: `crates/aisopod-memory/src/types.rs` and `crates/aisopod-memory/src/store.rs`

## Current Behavior
The `aisopod-memory` crate exists as a placeholder with no memory types or trait definitions.

## Expected Behavior
The crate exports well-documented types and a trait that all memory backends (SQLite-Vec, LanceDB) implement:

- `MemoryEntry` — a stored memory with `id`, `agent_id`, `content`, `embedding: Vec<f32>`, `metadata: MemoryMetadata`, `created_at`, and `updated_at`.
- `MemoryMetadata` — metadata including `source: MemorySource`, `session_key: Option<String>`, `tags: Vec<String>`, `importance: f32` (0.0–1.0), and `custom: serde_json::Value`.
- `MemorySource` — enum with variants `Agent`, `User`, `System`.
- `MemoryMatch` — a query result wrapping a `MemoryEntry` with a `score: f32` representing similarity.
- `MemoryFilter` — filter criteria including optional `agent_id`, `tags`, `source`, `importance_min`, `session_key`, `created_after`, and `created_before`.
- `MemoryQueryOptions` — query options including `top_k: usize`, `filter: MemoryFilter`, and `min_score: Option<f32>`.
- `MemoryStore` async trait with methods: `store(entry) -> Result<String>`, `query(query_str, opts) -> Result<Vec<MemoryMatch>>`, `delete(id) -> Result<()>`, `list(filter) -> Result<Vec<MemoryEntry>>`.

## Impact
Every other memory issue depends on these types and the trait. Getting the API right here ensures a clean, extensible memory system.

## Suggested Implementation
1. Open `crates/aisopod-memory/src/types.rs`. Define `MemorySource` as a `#[derive(Debug, Clone, Serialize, Deserialize)]` enum with variants `Agent`, `User`, `System`.
2. Define `MemoryMetadata` struct with fields `source: MemorySource`, `session_key: Option<String>`, `tags: Vec<String>`, `importance: f32`, and `custom: serde_json::Value`. Add `#[derive(Debug, Clone, Serialize, Deserialize)]`.
3. Define `MemoryEntry` struct with fields `id: String`, `agent_id: String`, `content: String`, `embedding: Vec<f32>`, `metadata: MemoryMetadata`, `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>`. Derive the same traits.
4. Define `MemoryMatch` struct with fields `entry: MemoryEntry` and `score: f32`. Derive `Debug, Clone`.
5. Define `MemoryFilter` struct with optional filter fields: `agent_id: Option<String>`, `tags: Option<Vec<String>>`, `source: Option<MemorySource>`, `importance_min: Option<f32>`, `session_key: Option<String>`, `created_after: Option<DateTime<Utc>>`, `created_before: Option<DateTime<Utc>>`. Derive `Debug, Clone, Default`.
6. Define `MemoryQueryOptions` struct with fields `top_k: usize`, `filter: MemoryFilter`, `min_score: Option<f32>`. Derive `Debug, Clone`.
7. Open `crates/aisopod-memory/src/store.rs`. Define `#[async_trait] pub trait MemoryStore: Send + Sync` with four methods:
   - `async fn store(&self, entry: MemoryEntry) -> Result<String>` — stores entry, returns its ID.
   - `async fn query(&self, query: &str, opts: MemoryQueryOptions) -> Result<Vec<MemoryMatch>>` — semantic search.
   - `async fn delete(&self, id: &str) -> Result<()>` — deletes by ID.
   - `async fn list(&self, filter: MemoryFilter) -> Result<Vec<MemoryEntry>>` — lists with filter.
8. Add doc-comments (`///`) to every type, field, and method explaining its purpose.
9. Re-export all public types from `crates/aisopod-memory/src/lib.rs`.
10. Run `cargo check -p aisopod-memory` to verify everything compiles.

## Dependencies
- Issue 007 (create aisopod-memory crate)
- Issue 016 (define core configuration types)

## Acceptance Criteria
- [ ] `MemoryEntry`, `MemoryMetadata`, `MemorySource`, `MemoryMatch`, `MemoryFilter`, and `MemoryQueryOptions` are defined and exported
- [ ] `MemoryStore` async trait is defined with `store()`, `query()`, `delete()`, and `list()` methods
- [ ] All types derive appropriate traits (`Debug`, `Clone`, `Serialize`/`Deserialize` where needed)
- [ ] Every public type, field, and method has a doc-comment
- [ ] `cargo check -p aisopod-memory` compiles without errors

---
*Created: 2026-02-15*
