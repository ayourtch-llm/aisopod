# Issue 077: Implement Session Key Generation and Routing

## Summary
Implement the logic that generates composite `SessionKey` values from channel information and agent bindings, with routing rules that collapse DM sessions to an agent's "main" session and isolate group sessions by peer ID.

## Location
- Crate: `aisopod-session`
- File: `crates/aisopod-session/src/routing.rs`

## Current Behavior
`SessionKey` is defined as a struct but there is no logic to construct it from runtime channel and agent context. Callers would have to manually assemble keys, leading to inconsistencies.

## Expected Behavior
A `SessionKeyBuilder` or `resolve_session_key` function takes channel context (channel type, account ID, peer kind, peer ID) and agent binding information (agent ID) and produces a normalized `SessionKey`. DM conversations with a given user always map to the same session per agent, while group conversations are isolated by group ID.

## Impact
Correct session key generation is essential for routing messages to the right conversation history. Incorrect keys would cause messages to land in the wrong session or create duplicate sessions for the same conversation.

## Suggested Implementation
1. Create `crates/aisopod-session/src/routing.rs`.
2. Define a `ChannelContext` struct (or re-use one from `aisopod-channel` if available) with fields:
   - `channel: String` — channel type identifier (e.g., `"discord"`, `"slack"`, `"cli"`).
   - `account_id: String` — the bot account on this channel.
   - `peer_kind: PeerKind` — enum with `Dm` and `Group` variants.
   - `peer_id: String` — the user ID (for DMs) or group/channel ID (for groups).
3. Define a `resolve_session_key(agent_id: &str, ctx: &ChannelContext) -> SessionKey` function:
   - Normalize `agent_id` by trimming and lowercasing.
   - Normalize `channel` and `account_id` similarly.
   - For `PeerKind::Dm`:
     - Set `peer_kind` to `"dm"`.
     - Set `peer_id` to the normalized user ID.
     - This means each user gets one session per agent, regardless of which DM channel instance is used.
   - For `PeerKind::Group`:
     - Set `peer_kind` to `"group"`.
     - Set `peer_id` to the normalized group ID.
     - This isolates each group conversation into its own session.
   - Return the assembled `SessionKey`.
4. Add a `SessionKey::canonical_string(&self) -> String` method that produces a deterministic string representation (e.g., `"agent_id:channel:account_id:peer_kind:peer_id"`) for logging and debugging.
5. Ensure normalization is consistent — define a `normalize(s: &str) -> String` helper that trims whitespace and converts to lowercase.
6. Export `resolve_session_key`, `ChannelContext`, and `PeerKind` from `lib.rs`.

## Dependencies
- Issue 073 (define session types and SessionKey)
- Issue 063 (implement agent resolution and binding)

## Acceptance Criteria
- [ ] `resolve_session_key` produces a valid `SessionKey` from agent ID and channel context
- [ ] DM sessions for the same user and agent always produce the same key
- [ ] Group sessions for different groups produce different keys
- [ ] All key components are normalized (trimmed, lowercased)
- [ ] `SessionKey::canonical_string` returns a deterministic, human-readable representation
- [ ] `PeerKind` enum distinguishes DM from group conversations
- [ ] `cargo check -p aisopod-session` succeeds

---
*Created: 2026-02-15*
