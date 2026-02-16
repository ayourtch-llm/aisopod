# Issue 091: Define Message Types

## Summary
Define the core message data types used throughout the channel abstraction layer: `IncomingMessage`, `OutgoingMessage`, `MessageContent`, `MessageTarget`, `SenderInfo`, `PeerInfo`, `PeerKind`, `Media`, and `MediaType`. These types represent all messages flowing between channels and the agent engine.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/src/message.rs`

## Current Behavior
The `aisopod-channel` crate has no message type definitions. There is no standardized format for representing incoming or outgoing messages.

## Expected Behavior
The crate exports well-documented message types with serde serialization support:

- `IncomingMessage` — struct with fields: `id: String`, `channel: String`, `account_id: String`, `sender: SenderInfo`, `peer: PeerInfo`, `content: MessageContent`, `reply_to: Option<String>`, `timestamp: DateTime<Utc>`, `metadata: serde_json::Value`.
- `OutgoingMessage` — struct with fields: `target: MessageTarget`, `content: MessageContent`, `reply_to: Option<String>`.
- `MessageContent` — enum with variants: `Text(String)`, `Media(Media)`, `Mixed(Vec<MessagePart>)`.
- `MessagePart` — enum with variants: `Text(String)`, `Media(Media)`.
- `MessageTarget` — struct with fields: `channel: String`, `account_id: String`, `peer: PeerInfo`, `thread_id: Option<String>`.
- `SenderInfo` — struct with fields: `id: String`, `display_name: Option<String>`, `username: Option<String>`, `is_bot: bool`.
- `PeerInfo` — struct with fields: `id: String`, `kind: PeerKind`, `title: Option<String>`.
- `PeerKind` — enum with variants: `User`, `Group`, `Channel`, `Thread`.
- `Media` — struct with fields: `media_type: MediaType`, `url: Option<String>`, `data: Option<Vec<u8>>`, `filename: Option<String>`, `mime_type: Option<String>`, `size_bytes: Option<u64>`.

## Impact
These types are used by the message routing pipeline (Issue 093), all adapter traits (Issue 090), and the agent engine integration. A well-defined message model ensures consistent data flow across the entire system.

## Suggested Implementation
1. Open `crates/aisopod-channel/src/message.rs`.
2. Add imports for `chrono::{DateTime, Utc}`, `serde::{Serialize, Deserialize}`, and `MediaType` from `types.rs`.
3. Define `PeerKind` as a `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` enum with variants `User`, `Group`, `Channel`, `Thread`.
4. Define `PeerInfo` struct with fields `id: String`, `kind: PeerKind`, `title: Option<String>`. Derive `Debug, Clone, Serialize, Deserialize`.
5. Define `SenderInfo` struct with fields `id: String`, `display_name: Option<String>`, `username: Option<String>`, `is_bot: bool`. Derive `Debug, Clone, Serialize, Deserialize`.
6. Define `Media` struct with fields `media_type: MediaType`, `url: Option<String>`, `data: Option<Vec<u8>>`, `filename: Option<String>`, `mime_type: Option<String>`, `size_bytes: Option<u64>`. Derive `Debug, Clone, Serialize, Deserialize`.
7. Define `MessagePart` enum with variants `Text(String)` and `Media(Media)`. Derive `Debug, Clone, Serialize, Deserialize`.
8. Define `MessageContent` enum with variants `Text(String)`, `Media(Media)`, `Mixed(Vec<MessagePart>)`. Derive `Debug, Clone, Serialize, Deserialize`.
9. Define `MessageTarget` struct with fields `channel: String`, `account_id: String`, `peer: PeerInfo`, `thread_id: Option<String>`. Derive `Debug, Clone, Serialize, Deserialize`.
10. Define `IncomingMessage` struct with fields `id: String`, `channel: String`, `account_id: String`, `sender: SenderInfo`, `peer: PeerInfo`, `content: MessageContent`, `reply_to: Option<String>`, `timestamp: DateTime<Utc>`, `metadata: serde_json::Value`. Derive `Debug, Clone, Serialize, Deserialize`.
11. Define `OutgoingMessage` struct with fields `target: MessageTarget`, `content: MessageContent`, `reply_to: Option<String>`. Derive `Debug, Clone, Serialize, Deserialize`.
12. Add doc-comments (`///`) to every type, field, and variant explaining its purpose.
13. Re-export all message types from `crates/aisopod-channel/src/lib.rs`.
14. Run `cargo check -p aisopod-channel` to verify everything compiles.

## Dependencies
- Issue 089 (define ChannelPlugin trait and channel metadata types — provides `MediaType`)

## Acceptance Criteria
- [ ] `IncomingMessage`, `OutgoingMessage`, `MessageContent`, `MessageTarget`, `SenderInfo`, `PeerInfo`, `PeerKind`, `Media`, and `MessagePart` are defined and exported
- [ ] All types derive `Debug`, `Clone`, `Serialize`, and `Deserialize`
- [ ] `MessageContent` correctly represents text, media, and mixed content
- [ ] Every public type, field, and variant has a doc-comment
- [ ] `cargo check -p aisopod-channel` compiles without errors

---
*Created: 2026-02-15*
