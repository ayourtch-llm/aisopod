# Issue 073: Define Session Types and SessionKey

## Summary
Define the core data types for session management in the `aisopod-session` crate, including the composite `SessionKey`, the `Session` struct, and all supporting types needed to model session lifecycle, filtering, and stored messages.

## Location
- Crate: `aisopod-session`
- File: `crates/aisopod-session/src/types.rs`

## Current Behavior
The `aisopod-session` crate exists as a skeleton with no session-related types defined.

## Expected Behavior
The crate exports a complete set of types that represent sessions, session keys, messages, metadata, status, filters, and patches. These types form the foundation for all session storage and retrieval operations.

## Impact
Every other session management issue depends on these types. Without them, the session store, message history, key generation, and multi-agent isolation cannot be implemented.

## Suggested Implementation
1. Open or create `crates/aisopod-session/src/types.rs`.
2. Define `SessionKey` as a struct with fields:
   - `agent_id: String` — which agent owns the session.
   - `channel: String` — the channel type (e.g., `"discord"`, `"slack"`).
   - `account_id: String` — the bot/account identifier on the channel.
   - `peer_kind: String` — `"dm"` or `"group"`.
   - `peer_id: String` — the remote user or group identifier.
3. Derive `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize` on `SessionKey`.
4. Define `SessionStatus` as an enum with variants: `Active`, `Idle`, `Compacted`, `Archived`.
5. Define `SessionMetadata` as a struct with an inner `HashMap<String, serde_json::Value>` or similar flexible map.
6. Define `Session` with fields:
   - `key: SessionKey`
   - `created_at: chrono::DateTime<Utc>`
   - `updated_at: chrono::DateTime<Utc>`
   - `message_count: u64`
   - `token_usage: u64`
   - `metadata: SessionMetadata`
   - `status: SessionStatus`
7. Define `SessionSummary` as a lighter view of `Session` for list endpoints (key, status, message_count, updated_at).
8. Define `SessionFilter` with optional fields for filtering by agent_id, channel, status, and date ranges.
9. Define `SessionPatch` with optional fields for updating metadata, status, and token_usage.
10. Define `StoredMessage` with fields:
    - `id: i64` — auto-increment row id.
    - `role: String` — `"user"`, `"assistant"`, `"system"`, `"tool"`.
    - `content: serde_json::Value` — message content as a JSON blob.
    - `tool_calls: Option<serde_json::Value>` — optional tool call data.
    - `created_at: chrono::DateTime<Utc>`
11. Re-export all types from `crates/aisopod-session/src/lib.rs`.
12. Run `cargo check -p aisopod-session` to confirm everything compiles.

## Dependencies
- Issue 006 (create aisopod-session crate)
- Issue 016 (define core configuration types)

## Acceptance Criteria
- [ ] `SessionKey` struct is defined with all five fields
- [ ] `Session` struct is defined with key, timestamps, message_count, token_usage, metadata, status
- [ ] `SessionMetadata` supports arbitrary key-value metadata
- [ ] `SessionStatus` enum has `Active`, `Idle`, `Compacted`, `Archived` variants
- [ ] `SessionSummary` provides a lightweight session view
- [ ] `SessionFilter` and `SessionPatch` structs are defined with optional fields
- [ ] `StoredMessage` struct is defined with role, content, tool_calls, timestamps
- [ ] All types derive `Clone`, `Debug`, and `Serialize`/`Deserialize` where appropriate
- [ ] `cargo check -p aisopod-session` succeeds

---
*Created: 2026-02-15*
