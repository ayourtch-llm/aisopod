# Issue 091: Define Message Types

## Summary
Define the core message data types used throughout the channel abstraction layer: `IncomingMessage`, `OutgoingMessage`, `MessageContent`, `MessageTarget`, `SenderInfo`, `PeerInfo`, `PeerKind`, `Media`, and `MediaType`. These types represent all messages flowing between channels and the agent engine.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/src/message.rs`

## Current Behavior
The `aisopod-channel` crate had no message type definitions. There was no standardized format for representing incoming or outgoing messages.

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

## Resolution

The message types were implemented in `crates/aisopod-channel/src/message.rs` with the following definitions:

- **PeerKind** - Enum with `User`, `Group`, `Channel`, `Thread` variants
- **PeerInfo** - Struct with `id`, `kind`, `title` fields
- **SenderInfo** - Struct with `id`, `display_name`, `username`, `is_bot` fields
- **Media** - Struct with `media_type`, `url`, `data`, `filename`, `mime_type`, `size_bytes` fields
- **MessagePart** - Enum with `Text(String)` and `Media(Media)` variants
- **MessageContent** - Enum with `Text(String)`, `Media(Media)`, `Mixed(Vec<MessagePart>)` variants
- **MessageTarget** - Struct with `channel`, `account_id`, `peer`, `thread_id` fields
- **IncomingMessage** - Struct with `id`, `channel`, `account_id`, `sender`, `peer`, `content`, `reply_to`, `timestamp`, `metadata` fields
- **OutgoingMessage** - Struct with `target`, `content`, `reply_to` fields

All types derive `Debug`, `Clone`, `Serialize`, and `Deserialize`, include comprehensive doc-comments, and are re-exported from `lib.rs`. The implementation passes `cargo build` and `cargo test`.

## Dependencies
- Issue 089 (define ChannelPlugin trait and channel metadata types — provides `MediaType`)

## Acceptance Criteria
- [x] `IncomingMessage`, `OutgoingMessage`, `MessageContent`, `MessageTarget`, `SenderInfo`, `PeerInfo`, `PeerKind`, `Media`, and `MessagePart` are defined and exported
- [x] All types derive `Debug`, `Clone`, `Serialize`, and `Deserialize`
- [x] `MessageContent` correctly represents text, media, and mixed content
- [x] Every public type, field, and variant has a doc-comment
- [x] `cargo check -p aisopod-channel` compiles without errors

---
*Created: 2026-02-15*
*Resolved: 2026-02-22*
