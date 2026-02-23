# Issue 107: Define Plugin Trait and PluginMeta Types

## Summary

This issue defined the core plugin system types for the aisopod project. The issue was actually about implementing two related issues in the correct order:
1. **Issue 108**: Implement `PluginApi` struct for capability registration (MUST be done first)
2. **Issue 107**: Define `Plugin` trait and `PluginMeta` types

## Key Implementation Details

### Ordering Requirement

Issue 108 **MUST** be resolved before Issue 107 because:
- Issue 107's `Plugin` trait references `PluginApi` in its `register()` method
- Issue 108's `PluginApi` struct is the concrete implementation needed for registration

This dependency order is critical and must be followed to avoid circular reference errors.

### PluginApi as a Struct

The original design had `PluginApi` as a trait, but this was incorrect. The acceptance criteria specified:
- **`PluginApi` must be a struct** (not a trait)
- It should have registration methods: `register_channel()`, `register_tool()`, `register_command()`, `register_provider()`, `register_hook()`

### Type Imports

The `PluginApi` struct imports traits from other crates:
- `aisopod_channel::plugin::ChannelPlugin`
- `aisopod_provider::ModelProvider`
- `aisopod_tools::Tool`

These dependencies must be resolved before implementing `PluginApi`.

### Arc Usage for Trait Objects

Since the plugin system uses dynamic dispatch with trait objects, `Arc` is used for:
- `Arc<dyn ChannelPlugin>` - channel implementations
- `Arc<dyn Tool>` - tool implementations
- `Arc<dyn ModelProvider>` - provider implementations
- `Arc<dyn HookHandler>` - hook handlers
- `Arc<dyn Fn()>` - command handlers

This ensures proper memory management and thread safety.

### Debug Implementation

Types with trait objects cannot derive `Debug` directly. The solution was to:
1. Remove `#[derive(Debug)]` from structs containing `Arc<dyn Trait>`
2. Manually implement `Debug` only for fields that support it (counters instead of trait objects)

Example:
```rust
impl std::fmt::Debug for PluginApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginApi")
            .field("channel_count", &self.channels.len())  // Use count instead of the trait object
            .field("tool_count", &self.tools.len())
            .field("command_count", &self.commands.len())
            .field("provider_count", &self.providers.len())
            .field("hook_count", &self.hooks.len())
            .finish()
    }
}
```

### Hook Types

Created minimal types for hook handling:
- `Hook` enum - lifecycle event types (SystemStart, SystemShutdown, ConfigChanged, etc.)
- `HookHandler` trait - interface for hook callbacks
- `PluginHookHandler` struct - wrapper combining hook type and handler

## Files Created/Modified

### New Files
1. **`crates/aisopod-plugin/src/hook.rs`** - Hook types and HookHandler trait
2. **`crates/aisopod-plugin/src/command.rs`** - PluginCommand struct for CLI commands
3. **`crates/aisopod-plugin/src/api.rs`** - PluginApi struct with registration methods

### Modified Files
1. **`crates/aisopod-plugin/src/lib.rs`** - Exported new modules and types
2. **`crates/aisopod-plugin/src/trait.rs`** - Updated to use PluginApi as struct
3. **`crates/aisopod-plugin/Cargo.toml`** - Added dependencies for channel/provider/tools crates

## Lessons Learned

### 1. Dependency Ordering
Always check issue dependencies before starting work. The issue files should clearly state which other issues must be resolved first.

### 2. Struct vs Trait
When designing an API, ask:
- Does it need dynamic dispatch? Use traits.
- Is it a concrete implementation that collects and manages data? Use structs.

In this case, `PluginApi` is a concrete implementation that collects plugin capabilities, so it must be a struct.

### 3. Trait Objects and Debug
Types with `Arc<dyn Trait>` cannot derive `Debug`. Options:
- Implement `Debug` manually for supported fields
- Don't implement `Debug` at all
- Use wrapper types that support `Debug`

### 4. Test Design
When writing tests for traits with complex dependencies, consider:
- Whether test implementations match the actual trait signatures
- Whether test types can be easily mocked or stubbed
- Whether tests can be simplified to avoid deep dependency chains

## Acceptance Criteria Met

- [x] `Plugin` trait is defined with `id()`, `meta()`, `register()`, `init()`, and `shutdown()` methods
- [x] `PluginMeta` struct includes name, version, description, author, supported_channels, and supported_providers fields
- [x] `PluginContext` struct provides runtime context including config and data directory
- [x] All public types and methods have documentation comments
- [x] `PluginApi` is a **struct** (not a trait) with registration methods
- [x] `cargo build -p aisopod-plugin` compiles without errors
- [x] `cargo test -p aisopod-plugin` passes

## Next Steps

After this issue is resolved, the following can be addressed:
- Implementing actual plugin loading mechanisms
- Implementing the plugin registry
- Creating example plugins
- Documenting the plugin development process
