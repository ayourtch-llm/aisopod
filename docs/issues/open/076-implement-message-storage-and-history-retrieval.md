# Issue 076: Implement Message Storage and History Retrieval

## Summary
Implement `append_messages` and `get_history` methods on `SessionStore` to store conversation messages as JSON blobs and retrieve them with pagination support (limit, offset, before/after timestamp filters).

## Location
- Crate: `aisopod-session`
- File: `crates/aisopod-session/src/store.rs`

## Current Behavior
The session store supports CRUD on sessions but has no way to persist or retrieve individual conversation messages within a session.

## Expected Behavior
Messages can be appended to a session in bulk, and the full history (or a paginated subset) can be retrieved efficiently. Messages are stored as JSON blobs in the `messages` table for maximum flexibility with varying message formats.

## Impact
Message history is critical for constructing agent prompts, supporting session compaction, and enabling users to review past conversations. Without message storage, sessions are empty shells.

## Suggested Implementation
1. Open `crates/aisopod-session/src/store.rs`.
2. Define a `HistoryQuery` struct with optional fields:
   - `limit: Option<u32>` — maximum number of messages to return.
   - `offset: Option<u32>` — number of messages to skip (for offset-based pagination).
   - `before: Option<chrono::DateTime<Utc>>` — only return messages created before this timestamp.
   - `after: Option<chrono::DateTime<Utc>>` — only return messages created after this timestamp.
3. Implement `append_messages(&self, key: &SessionKey, messages: &[StoredMessage]) -> Result<()>`:
   - Look up the session's `id` from the `sessions` table using the key fields.
   - If the session does not exist, return an error (callers should use `get_or_create` first).
   - Begin a transaction.
   - For each `StoredMessage`, INSERT a row into `messages` with:
     - `session_id` from the looked-up id.
     - `role` as a plain text string.
     - `content` serialized to a JSON string via `serde_json::to_string`.
     - `tool_calls` serialized to a JSON string if `Some`, otherwise NULL.
     - `created_at` as the message's timestamp in ISO 8601 format.
   - After inserting, UPDATE the session's `message_count` by adding the number of new messages and set `updated_at` to now.
   - Commit the transaction.
4. Implement `get_history(&self, key: &SessionKey, query: &HistoryQuery) -> Result<Vec<StoredMessage>>`:
   - Look up the session's `id` from the key fields.
   - Build a SELECT on `messages` WHERE `session_id` matches.
   - If `before` is set, add `AND created_at < ?`.
   - If `after` is set, add `AND created_at > ?`.
   - ORDER BY `created_at ASC` (oldest first).
   - If `offset` is set, add `OFFSET ?`.
   - If `limit` is set, add `LIMIT ?`; otherwise default to a sensible maximum (e.g., 1000).
   - Map each row back to a `StoredMessage`, deserializing `content` and `tool_calls` from JSON strings.
5. Export `HistoryQuery` from `lib.rs`.
6. Test with multiple messages, verifying ordering, limit, offset, and timestamp filters.

## Dependencies
- Issue 073 (define session types and SessionKey)
- Issue 074 (implement SQLite database schema and migrations)
- Issue 075 (implement SessionStore core CRUD operations)

## Acceptance Criteria
- [ ] `append_messages` inserts messages into the `messages` table within a transaction
- [ ] `append_messages` updates the session's `message_count` and `updated_at`
- [ ] `get_history` returns messages in chronological order
- [ ] `get_history` supports `limit` and `offset` pagination
- [ ] `get_history` supports `before` and `after` timestamp filters
- [ ] Messages are stored as JSON blobs, preserving arbitrary content structure
- [ ] An error is returned if appending to a non-existent session
- [ ] `cargo check -p aisopod-session` succeeds

---
*Created: 2026-02-15*
