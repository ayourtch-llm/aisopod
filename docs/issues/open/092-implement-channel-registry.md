# Issue 092: Implement Channel Registry

## Summary
Implement the `ChannelRegistry` struct that manages registration, lookup, and listing of channel plugins. The registry supports channel ordering, ID normalization, and alias resolution.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/src/registry.rs`

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
- [ ] `ChannelRegistry` struct is defined with `channels`, `order`, and `aliases` fields
- [ ] `register()` adds a plugin and records its order
- [ ] `get()` resolves by canonical ID and by alias
- [ ] `list()` returns plugins in registration order
- [ ] `normalize_id()` lowercases, trims, and resolves aliases
- [ ] `add_alias()` maps an alias to a canonical channel ID
- [ ] Every public method and field has a doc-comment
- [ ] `cargo check -p aisopod-channel` compiles without errors

---
*Created: 2026-02-15*
