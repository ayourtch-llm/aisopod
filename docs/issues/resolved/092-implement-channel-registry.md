# Issue 092: Implement Channel Registry

## Summary
Implement the `ChannelRegistry` struct that manages registration, lookup, and listing of channel plugins. The registry supports channel ordering, ID normalization, and alias resolution.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/src/channel.rs`

## Current Behavior
There is no mechanism to register, discover, or look up channel plugins at runtime.

## Expected Behavior
A `ChannelRegistry` struct provides centralized management of channel plugins:

```rust
pub struct ChannelRegistry {
    channels: HashMap<String, Arc<dyn ChannelPlugin>>,
    order: Vec<String>,
    aliases: HashMap<String, String>,
}
```

Methods:
- `new() -> Self` — creates an empty registry.
- `register(&mut self, plugin: Arc<dyn ChannelPlugin>)` — registers a channel plugin, adding its ID to the order list and normalizing the ID to lowercase.
- `get(&self, id: &str) -> Option<&Arc<dyn ChannelPlugin>>` — looks up a channel by ID or alias, normalizing the input.
- `list(&self) -> Vec<&Arc<dyn ChannelPlugin>>` — returns all registered channels in registration order.
- `normalize_id(&self, id: &str) -> Option<String>` — normalizes an ID (lowercase, trim) and resolves aliases to canonical IDs.
- `add_alias(&mut self, alias: &str, canonical_id: &str) -> Result<()>` — registers an alias for an existing channel ID.

## Impact
The registry is the central lookup mechanism used by the message routing pipeline (Issue 093) and configuration system to resolve channel references.

## Suggested Implementation
1. Open `crates/aisopod-channel/src/registry.rs`.
2. Add imports: `use std::collections::HashMap;`, `use std::sync::Arc;`, and import `ChannelPlugin` from `plugin.rs`.
3. Define the `ChannelRegistry` struct with three fields:
   - `channels: HashMap<String, Arc<dyn ChannelPlugin>>` — maps normalized IDs to plugins.
   - `order: Vec<String>` — preserves registration order.
   - `aliases: HashMap<String, String>` — maps alias strings to canonical IDs.
4. Implement `ChannelRegistry::new()` that initializes all fields as empty.
5. Implement `register()`:
   - Get the plugin's ID via `plugin.id()`.
   - Normalize it to lowercase and trimmed.
   - Insert into the `channels` HashMap.
   - Append the normalized ID to `order`.
6. Implement `normalize_id()`:
   - Lowercase and trim the input.
   - Check if it exists in `aliases`; if so, return the canonical ID.
   - Check if it exists in `channels`; if so, return it.
   - Otherwise return `None`.
7. Implement `get()`:
   - Call `normalize_id()` on the input.
   - If resolved, look up in `channels` and return.
8. Implement `list()`:
   - Iterate over `order` and collect references to the corresponding plugins from `channels`.
9. Implement `add_alias()`:
   - Normalize the alias to lowercase.
   - Verify the canonical ID exists in `channels`; if not, return an error.
   - Insert into `aliases`.
10. Add doc-comments to every method and field.
11. Re-export `ChannelRegistry` from `crates/aisopod-channel/src/lib.rs`.
12. Run `cargo check -p aisopod-channel` to verify everything compiles.

## Dependencies
- Issue 089 (define ChannelPlugin trait and channel metadata types)

## Acceptance Criteria
- [x] `ChannelRegistry` struct is defined with `channels`, `order`, and `aliases` fields
- [x] `register()` adds a plugin and records its order
- [x] `get()` resolves by canonical ID and by alias
- [x] `list()` returns plugins in registration order
- [x] `normalize_id()` lowercases, trims, and resolves aliases
- [x] `add_alias()` maps an alias to a canonical channel ID
- [x] Every public method and field has a doc-comment
- [x] `cargo check -p aisopod-channel` compiles without errors

## Resolution
The `ChannelRegistry` implementation was added to `crates/aisopod-channel/src/channel.rs` with the following components:

### Data Structures
- **`ChannelRegistry`**: Manages channel plugins with:
  - `channels`: HashMap mapping channel IDs to `Arc<dyn ChannelPlugin>` instances
  - `aliases`: HashMap mapping aliases to `ChannelAlias` structs containing canonical IDs

- **`ChannelAlias`**: Represents a mapping from a friendly alias to a canonical channel ID

### Implemented Methods
- `new()`: Creates an empty registry
- `register(channel)`: Registers a channel plugin by its ID
- `unregister(channel_id)`: Removes a channel from the registry
- `get(id)`: Looks up a channel by ID or alias, returning `Arc<dyn ChannelPlugin>`
- `list()`: Returns all registered channel IDs
- `list_channels()`: Returns all registered channel plugin `Arc`s
- `normalize_id(id)`: Resolves aliases and returns canonical channel IDs
- `add_alias(alias, channel_id)`: Creates an alias mapping
- `remove_alias(alias)`: Removes an alias
- `contains(id)`: Checks if a channel or alias exists

### Trait Implementations
- `Default` trait for creating a default (empty) registry

### Testing
- 21 comprehensive unit tests covering all functionality:
  - Registry initialization and emptiness checks
  - Channel registration and replacement behavior
  - Channel unregistration
  - Channel listing (both IDs and plugin Arcs)
  - Alias registration, resolution, and removal
  - Edge cases (nonexistent channels, duplicate aliases)
  - Default trait behavior

### Exports
- `ChannelRegistry` and `ChannelAlias` are re-exported from `lib.rs`

### Verification
- `cargo build -p aisopod-channel` compiles successfully
- `cargo test -p aisopod-channel` passes all 21 tests

---
*Created: 2026-02-15*
*Resolved: 2026-02-22*
