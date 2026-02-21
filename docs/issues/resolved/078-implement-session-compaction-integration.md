# Issue 078: Implement Session Compaction Integration

## Summary
Integrate session compaction into the session store so that after an agent run completes, the session can be compacted according to a configurable strategy. Track compaction history and preserve compaction summaries within the session.

## Location
- Crate: `aisopod-session`
- File: `crates/aisopod-session/src/compaction.rs`

## Current Behavior
Session compaction strategies are defined in the agent engine (Issue 068), but the session store has no hooks to trigger compaction or persist its results.

## Expected Behavior
The session store exposes a `compact` method that applies a compaction strategy to a session's message history, replaces old messages with a summary, updates compaction metadata, and tracks compaction history (count and last compaction time).

## Impact
Without compaction integration, long-running sessions will accumulate unbounded message history, leading to excessive token usage and degraded performance when constructing prompts.

## Suggested Implementation
1. Create `crates/aisopod-session/src/compaction.rs`.
2. Define a `CompactionRecord` struct with fields:
   - `compaction_count: u32` — how many times this session has been compacted.
   - `last_compacted_at: Option<chrono::DateTime<Utc>>` — when compaction last occurred.
   - `summary: Option<String>` — the compaction summary text.
3. Define a `CompactionStrategy` enum (or trait) that the caller provides:
   - `None` — do not compact.
   - `SlidingWindow { max_messages: u32 }` — keep only the last N messages.
   - `Summarize` — replace old messages with a summary (delegated to the agent engine).
4. Implement `SessionStore::compact(&self, key: &SessionKey, strategy: &CompactionStrategy, summary: Option<&str>) -> Result<CompactionRecord>`:
   - Look up the session by key.
   - Match on the strategy:
     - `None`: return the current compaction record unchanged.
     - `SlidingWindow { max_messages }`:
       - Count messages for the session.
       - If count > max_messages, DELETE the oldest `(count - max_messages)` messages.
       - Update `message_count` on the session.
     - `Summarize`:
       - Expect `summary` to be `Some`. If `None`, return an error.
       - Begin a transaction.
       - DELETE all messages for the session.
       - INSERT a single new message with `role = "system"` and `content` set to the summary text.
       - Update `message_count` to 1.
       - Commit.
   - In all cases, update the session's `metadata` to store `compaction_count` (incremented) and `last_compacted_at` (now).
   - Update `updated_at` on the session.
   - Return the new `CompactionRecord`.
5. Add a `SessionStore::get_compaction_record(&self, key: &SessionKey) -> Result<CompactionRecord>` method that reads compaction metadata from the session.
6. Export `CompactionRecord` and `CompactionStrategy` from `lib.rs`.

## Dependencies
- Issue 075 (implement SessionStore core CRUD operations)
- Issue 076 (implement message storage and history retrieval)
- Issue 068 (implement session compaction strategies in agent engine)

## Acceptance Criteria
- [x] `compact` with `SlidingWindow` removes the oldest messages beyond the window size
- [x] `compact` with `Summarize` replaces all messages with a single summary message
- [x] `compact` with `None` is a no-op
- [x] Compaction count is incremented and `last_compacted_at` is updated after each compaction
- [x] Compaction summary text is preserved in the session
- [x] `get_compaction_record` returns the current compaction state
- [x] `message_count` on the session is updated to reflect post-compaction state
- [x] `cargo check -p aisopod-session` succeeds

## Resolution
The implementation adds session compaction integration to the `aisopod-session` crate:

### Changes Made:
1. **`crates/aisopod-session/src/compaction.rs`**: 
   - Defined `CompactionRecord` struct with `compaction_count`, `last_compacted_at`, and `summary` fields
   - Defined `CompactionStrategy` enum with variants: `None`, `SlidingWindow`, and `Summarize`

2. **`crates/aisopod-session/src/store.rs`**:
   - Added `SessionStore::compact()` method that implements all three strategies:
     - `None`: Returns current compaction record without modification
     - `SlidingWindow`: Deletes oldest messages beyond the window size
     - `Summarize`: Replaces all messages with a single system role summary message
   - Added `SessionStore::get_compaction_record()` method to read compaction metadata
   - Proper scope validation using `verify_scope()`
   - Updates `message_count`, `compaction_count`, `last_compacted_at`, and `updated_at` fields

3. **`crates/aisopod-session/src/types.rs`**:
   - Added `id` field to `Session` struct for database row identification

4. **`crates/aisopod-session/src/db.rs`**:
   - Added migration to add compaction columns to sessions table (`compaction_count`, `last_compacted_at`, `last_compaction_summary`)

5. **`crates/aisopod-session/src/lib.rs`**:
   - Exported `CompactionRecord` and `CompactionStrategy` from the module

All acceptance criteria are met and verified by unit tests. Tests cover sliding window compaction, summarization, no-op strategy, scope validation, and initial state handling.

---
*Created: 2026-02-15*
*Resolved: 2026-02-21*
