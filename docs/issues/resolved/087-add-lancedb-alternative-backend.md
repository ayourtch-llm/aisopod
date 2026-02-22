# Issue 087: Add LanceDB Alternative Backend (Feature-Gated)

## Summary
Implement an optional LanceDB-based `MemoryStore` backend behind the `--features lancedb` feature flag, providing the same trait interface as the SQLite-Vec backend with a different underlying storage engine.

## Location
- Crate: `aisopod-memory`
- File: `crates/aisopod-memory/src/lancedb.rs`

## Current Behavior
The only `MemoryStore` implementation is `SqliteMemoryStore` (Issue 082). There is no alternative backend for users who want more advanced vector operations or different storage characteristics.

## Expected Behavior
A `LanceDbMemoryStore` struct implements the `MemoryStore` trait using LanceDB as the storage backend. The implementation is feature-gated behind `--features lancedb` so it does not add to compile times or dependencies for users who don't need it. It supports the same `store()`, `query()`, `delete()`, and `list()` operations as the SQLite backend.

## Impact
Provides users with an alternative vector storage backend that may offer better performance for large-scale deployments or additional vector index types. Demonstrates the extensibility of the `MemoryStore` trait.

## Suggested Implementation
1. Add the LanceDB dependency behind a feature flag in `crates/aisopod-memory/Cargo.toml`:
   ```toml
   [features]
   default = []
   lancedb = ["dep:lancedb", "arrow-schema", "arrow-array", "futures-util"]

   [dependencies]
   lancedb = { version = "0.15", optional = true }
   ```
2. Create `crates/aisopod-memory/src/lancedb.rs` and gate the entire module:
   ```rust
   #![cfg(feature = "lancedb")]
   ```
3. Define `LanceDbMemoryStore` struct with fields:
   - `db: lancedb::Connection`
   - `table: LanceDbTable`
   - `table_name: String` (default: `"memories"`)
   - `embedding_dim: usize`
4. Implement constructor `LanceDbMemoryStore::new(path: &str, embedding_dim: usize) -> Result<Self>`:
   - Connect to LanceDB at the given path.
   - Create or open the memories table with the appropriate schema (id, agent_id, content, source, session_key, tags, importance, metadata, created_at, updated_at, embedding vector).
5. Implement `MemoryStore::store()`:
   - Insert the memory entry as a record into the LanceDB table.
   - The embedding vector is stored natively by LanceDB.
6. Implement `MemoryStore::query()`:
   - Use LanceDB's built-in vector search API to find the nearest neighbors.
   - Apply filters for agent_id, tags, importance, etc., using LanceDB's filter syntax.
   - Return `Vec<MemoryMatch>` sorted by similarity.
7. Implement `MemoryStore::delete()`:
   - Delete the record by ID from the LanceDB table.
8. Implement `MemoryStore::list()`:
   - Query the table with filter predicates built from `MemoryFilter`.
   - Return `Vec<MemoryEntry>`.
9. Conditionally re-export `LanceDbMemoryStore` from `lib.rs`:
   ```rust
   #[cfg(feature = "lancedb")]
   pub mod lancedb;
   ```
10. Verify the feature-gated build:
    - `cargo check -p aisopod-memory` — should compile without LanceDB.
    - `cargo check -p aisopod-memory --features lancedb` — should compile with LanceDB.

## Dependencies
- Issue 081 (define memory types and MemoryStore trait)

## Acceptance Criteria
- [x] `LanceDbMemoryStore` implements the `MemoryStore` trait
- [x] The implementation is behind the `lancedb` feature flag and does not affect default compilation
- [x] `store()`, `query()`, `delete()`, and `list()` work with LanceDB as the backend
- [x] `cargo check -p aisopod-memory` compiles without the `lancedb` feature
- [x] `cargo check -p aisopod-memory --features lancedb` compiles with the `lancedb` feature

## Resolution

The `LanceDbMemoryStore` implementation was successfully added with all required functionality:

- **Module Structure**: Created `crates/aisopod-memory/src/lancedb.rs` with the entire module gated behind `#![cfg(feature = "lancedb")]`
- **Feature-Gated Dependencies**: Added `lancedb = { version = "0.15", optional = true }` to `Cargo.toml` with feature flags for `lancedb`, `arrow-schema`, `arrow-array`, and `futures-util`
- **MemoryStore Trait Implementation**: Implemented all required methods:
  - `store()`: Inserts memory entries as records into the LanceDB table with proper embedding storage
  - `query()`: Uses LanceDB's vector search API with filter support for agent_id, source, session_key, importance, created_at, and tags
  - `delete()`: Removes records by ID from the LanceDB table
  - `list()`: Queries the table with filter predicates and returns `Vec<MemoryEntry>`
- **Helper Structs**: Created `DbMemory` struct for serialization/deserialization with proper handling of nullable fields and JSON metadata
- **Conditional Exports**: Added `#[cfg(feature = "lancedb")] pub mod lancedb;` to `lib.rs` and re-exported `LanceDbMemoryStore`
- **Testing**: All tests pass including 3 new LanceDB-specific tests (`test_lancedb_store_new`, `test_lancedb_store_store_and_query`, `test_lancedb_store_delete`) and full test suite verification with `cargo test -p aisopod-memory --features lancedb`

---
*Created: 2026-02-15*
*Resolved: 2026-02-22*
