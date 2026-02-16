# Issue 075: Implement SessionStore Core CRUD Operations

## Summary
Implement the `SessionStore` struct with core CRUD operations for sessions: `get_or_create`, `list`, `patch`, and `delete`. These operations form the primary interface for managing session lifecycle.

## Location
- Crate: `aisopod-session`
- File: `crates/aisopod-session/src/store.rs`

## Current Behavior
The `aisopod-session` crate has types and a database schema but no application-level operations for creating, reading, updating, or deleting sessions.

## Expected Behavior
A `SessionStore` struct wraps a SQLite connection and exposes methods to manage sessions. Callers can retrieve or create sessions by key, list sessions with optional filters, update session fields, and delete sessions (cascading to their messages).

## Impact
The session store is the central API consumed by the agent engine, channels, and tools. Without these CRUD operations, no part of the system can persist or query session state.

## Suggested Implementation
1. Create `crates/aisopod-session/src/store.rs`.
2. Define `SessionStore` with a field for the database connection (e.g., `conn: rusqlite::Connection` or a wrapper).
3. Implement `SessionStore::new(path: &Path) -> Result<Self>`:
   - Call `open_database(path)` from the `db` module.
   - Return the store wrapping the connection.
4. Implement `get_or_create(&self, key: &SessionKey) -> Result<Session>`:
   - Query the `sessions` table for a row matching all five key fields.
   - If found, map the row to a `Session` and return it.
   - If not found, INSERT a new row with default values (`message_count = 0`, `status = Active`, `created_at = now`, `updated_at = now`) and return the new `Session`.
   - Use `INSERT OR IGNORE` followed by `SELECT`, or `INSERT ... ON CONFLICT DO NOTHING` with a subsequent select, to handle race conditions.
5. Implement `list(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>>`:
   - Build a SELECT query dynamically based on which filter fields are `Some`.
   - Support filtering by `agent_id`, `channel`, `status`, and date ranges on `updated_at`.
   - Order by `updated_at DESC` by default.
   - Map rows to `SessionSummary` structs.
6. Implement `patch(&self, key: &SessionKey, patch: &SessionPatch) -> Result<Session>`:
   - Build an UPDATE statement that sets only the fields present in the patch.
   - Always update `updated_at` to the current time.
   - Return the updated `Session` by re-querying.
7. Implement `delete(&self, key: &SessionKey) -> Result<bool>`:
   - DELETE from `sessions` where all key fields match.
   - Due to `ON DELETE CASCADE`, associated messages are removed automatically.
   - Return `true` if a row was deleted, `false` otherwise.
8. Re-export `SessionStore` from `lib.rs`.

## Dependencies
- Issue 073 (define session types and SessionKey)
- Issue 074 (implement SQLite database schema and migrations)

## Acceptance Criteria
- [ ] `SessionStore::new` opens or creates the database and runs migrations
- [ ] `get_or_create` returns an existing session or creates a new one
- [ ] `list` returns sessions matching the provided filter criteria
- [ ] `patch` updates only the specified fields and refreshes `updated_at`
- [ ] `delete` removes the session and cascades to messages, returning a boolean
- [ ] All methods return meaningful errors on failure
- [ ] `cargo check -p aisopod-session` succeeds

---
*Created: 2026-02-15*
