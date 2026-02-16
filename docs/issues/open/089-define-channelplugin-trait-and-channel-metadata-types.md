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

## Suggested Implementation
1. Open `crates/aisopod-channel/src/types.rs`. Define `ChatType` as a `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` enum with variants `Dm`, `Group`, `Channel`, `Thread`.
2. Define `MediaType` as a `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` enum with variants `Image`, `Audio`, `Video`, `Document`, `Other(String)`.
3. Define `ChannelMeta` struct with fields `label: String`, `docs_url: Option<String>`, `ui_hints: serde_json::Value`. Derive `Debug, Clone, Serialize, Deserialize`.
4. Define `ChannelCapabilities` struct with fields `chat_types: Vec<ChatType>`, `supports_media: bool`, `supports_reactions: bool`, `supports_threads: bool`, `supports_typing: bool`, `supports_voice: bool`, `max_message_length: Option<usize>`, `supported_media_types: Vec<MediaType>`. Derive `Debug, Clone, Serialize, Deserialize`.
5. Open `crates/aisopod-channel/src/plugin.rs`. Import the types from `types.rs` and the `ChannelConfigAdapter` trait (defined in Issue 090). Define the `ChannelPlugin` trait using `#[async_trait]`:
   ```rust
   #[async_trait]
   pub trait ChannelPlugin: Send + Sync {
       fn id(&self) -> &str;
       fn meta(&self) -> &ChannelMeta;
       fn capabilities(&self) -> &ChannelCapabilities;
       fn config(&self) -> &dyn ChannelConfigAdapter;
   }
   ```
6. Add doc-comments (`///`) to every type, field, variant, and method explaining its purpose.
7. Re-export all public types from `crates/aisopod-channel/src/lib.rs`.
8. Run `cargo check -p aisopod-channel` to verify everything compiles.

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
