# Issue 089: Define ChannelPlugin Trait and Channel Metadata Types

## Summary
Define the core `ChannelPlugin` trait and its associated metadata types (`ChannelMeta`, `ChannelCapabilities`) in the `aisopod-channel` crate. These form the foundation for all channel integrations.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/src/plugin.rs` and `crates/aisopod-channel/src/types.rs`

## Current Behavior
The `aisopod-channel` crate exists as a placeholder with no trait or type definitions.

## Expected Behavior
The crate exports a well-documented `ChannelPlugin` trait and supporting types:

- `ChannelPlugin` — async trait with methods `id()`, `meta()`, `capabilities()`, `config()`.
- `ChannelMeta` — metadata struct with `label: String`, `docs_url: Option<String>`, and `ui_hints: serde_json::Value`.
- `ChannelCapabilities` — struct describing what a channel supports: `chat_types: Vec<ChatType>`, `supports_media: bool`, `supports_reactions: bool`, `supports_threads: bool`, `supports_typing: bool`, `supports_voice: bool`, `max_message_length: Option<usize>`, `supported_media_types: Vec<MediaType>`.
- `ChatType` — enum with variants `Dm`, `Group`, `Channel`, `Thread`.
- `MediaType` — enum with variants `Image`, `Audio`, `Video`, `Document`, `Other(String)`.

## Impact
Every other channel issue depends on this trait and these types. A clean, well-documented API here ensures all channel plugins and adapters share a consistent interface.

## Resolution

The following types were implemented in the `aisopod-channel` crate:

### Types (`crates/aisopod-channel/src/types.rs`)

- **`ChatType`** - Enum with variants `Dm`, `Group`, `Channel`, `Thread`. Derives `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`.
- **`MediaType`** - Enum with variants `Image`, `Audio`, `Video`, `Document`, `Other(String)`. Derives `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`.
- **`ChannelMeta`** - Struct with fields `label: String`, `docs_url: Option<String>`, `ui_hints: serde_json::Value`. Derives `Debug`, `Clone`, `Serialize`, `Deserialize`.
- **`ChannelCapabilities`** - Struct with fields `chat_types: Vec<ChatType>`, `supports_media: bool`, `supports_reactions: bool`, `supports_threads: bool`, `supports_typing: bool`, `supports_voice: bool`, `max_message_length: Option<usize>`, `supported_media_types: Vec<MediaType>`. Derives `Debug`, `Clone`, `Serialize`, `Deserialize`.

### Plugin Trait (`crates/aisopod-channel/src/plugin.rs`)

- **`ChannelPlugin`** - Async trait with methods:
  - `fn id(&self) -> &str` - Returns the unique channel plugin identifier
  - `fn meta(&self) -> &ChannelMeta` - Returns channel metadata
  - `fn capabilities(&self) -> &ChannelCapabilities` - Returns channel capabilities
  - `fn config(&self) -> &dyn ChannelConfigAdapter` - Returns configuration adapter

### Exports (`crates/aisopod-channel/src/lib.rs`)

All public types are re-exported from the crate root for easy access.

### Verification

- `cargo build -p aisopod-channel` compiles successfully
- `cargo test -p aisopod-channel` passes all 21 tests
- All public types have comprehensive doc-comments
- All types derive appropriate traits as required

## Dependencies
- Issue 009 (create aisopod-channel crate)
- Issue 016 (define core configuration types)

## Acceptance Criteria
- [ ] `ChannelPlugin` trait is defined with `id()`, `meta()`, `capabilities()`, and `config()` methods
- [ ] `ChannelMeta` struct is defined with `label`, `docs_url`, and `ui_hints` fields
- [ ] `ChannelCapabilities` struct is defined with all capability flags and media type list
- [ ] `ChatType` and `MediaType` enums are defined
- [ ] All types derive appropriate traits (`Debug`, `Clone`, `Serialize`/`Deserialize` where needed)
- [ ] Every public type, field, and method has a doc-comment
- [ ] `cargo check -p aisopod-channel` compiles without errors

---
*Created: 2026-02-15*
*Resolved: 2026-02-22*
