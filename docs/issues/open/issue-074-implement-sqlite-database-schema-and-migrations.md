# Issue 074: Implement SQLite Database Schema and Migrations

## Summary
Add the `rusqlite` dependency to `aisopod-session` and implement the SQLite database schema for sessions and messages, including indexes for efficient querying and a migration runner that creates the schema on first run.

## Location
- Crate: `aisopod-session`
- File: `crates/aisopod-session/src/db.rs`

## Current Behavior
The `aisopod-session` crate has no database layer or persistence mechanism.

## Expected Behavior
On first startup, the session store automatically creates a SQLite database with `sessions` and `messages` tables, appropriate indexes, and a `schema_version` tracking mechanism. Subsequent startups detect the existing schema and skip creation.

## Impact
All session persistence depends on the database layer. Without it, sessions and messages cannot be stored or retrieved across restarts.

## Suggested Implementation
1. Add `rusqlite` (with the `bundled` feature) to `crates/aisopod-session/Cargo.toml`:
   ```toml
   [dependencies]
   rusqlite = { version = "0.31", features = ["bundled"] }
   ```
2. Create `crates/aisopod-session/src/db.rs`.
3. Define a `sessions` table with columns:
   - `id` INTEGER PRIMARY KEY AUTOINCREMENT
   - `agent_id` TEXT NOT NULL
   - `channel` TEXT NOT NULL
   - `account_id` TEXT NOT NULL
   - `peer_kind` TEXT NOT NULL
   - `peer_id` TEXT NOT NULL
   - `created_at` TEXT NOT NULL (ISO 8601)
   - `updated_at` TEXT NOT NULL (ISO 8601)
   - `message_count` INTEGER NOT NULL DEFAULT 0
   - `token_usage` INTEGER NOT NULL DEFAULT 0
   - `metadata` TEXT NOT NULL DEFAULT '{}'
   - `status` TEXT NOT NULL DEFAULT 'active'
   - Add a UNIQUE constraint on `(agent_id, channel, account_id, peer_kind, peer_id)`.
4. Define a `messages` table with columns:
   - `id` INTEGER PRIMARY KEY AUTOINCREMENT
   - `session_id` INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE
   - `role` TEXT NOT NULL
   - `content` TEXT NOT NULL
   - `tool_calls` TEXT
   - `created_at` TEXT NOT NULL (ISO 8601)
5. Add indexes:
   - `idx_sessions_agent_id` on `sessions(agent_id)`
   - `idx_sessions_channel` on `sessions(channel, account_id)`
   - `idx_sessions_status` on `sessions(status)`
   - `idx_sessions_updated_at` on `sessions(updated_at)`
   - `idx_messages_session_id` on `messages(session_id)`
   - `idx_messages_created_at` on `messages(session_id, created_at)`
6. Implement a `run_migrations(conn: &Connection) -> Result<()>` function that:
   - Creates a `schema_version` table if it does not exist.
   - Checks the current version number.
   - Runs any unapplied migration SQL statements in order.
   - Updates the version number after each migration.
7. Implement an `open_database(path: &Path) -> Result<Connection>` function that opens or creates the database file, enables WAL mode and foreign keys, and calls `run_migrations`.
8. Add unit tests that verify the schema is created correctly on a fresh in-memory database.

## Dependencies
- Issue 073 (define session types and SessionKey)

## Acceptance Criteria
- [ ] `rusqlite` is added as a dependency with the `bundled` feature
- [ ] `sessions` table is created with all required columns and a unique composite key
- [ ] `messages` table is created with a foreign key to `sessions` and ON DELETE CASCADE
- [ ] Indexes are created for agent_id, channel, status, updated_at, and message timestamps
- [ ] `run_migrations` creates the schema on first run and is idempotent on subsequent runs
- [ ] `open_database` returns a configured connection with WAL mode and foreign keys enabled
- [ ] Unit tests verify schema creation on an in-memory database
- [ ] `cargo check -p aisopod-session` succeeds

---
*Created: 2026-02-15*
