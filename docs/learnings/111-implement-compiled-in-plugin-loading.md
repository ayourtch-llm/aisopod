# Compiled-In Plugin Loading Implementation

## Summary

This document captures key learnings from implementing the compiled-in plugin loading feature for the aisopod plugin system (Issue #111).

## Implementation Overview

The compiled-in plugin loading feature allows built-in plugins to be selectively included at compile time via Cargo feature flags. Plugins not included via features produce zero runtime overhead.

## Architecture

### Core Components

1. **`aisopod-plugin` crate**: Contains the core `Plugin` trait and `PluginRegistry`
2. **Provider crates** (`aisopod-provider-openai`, `aisopod-provider-anthropic`): Implement provider traits from `aisopod-provider`
3. **Channel crates** (`aisopod-channel-telegram`, etc.): Implement channel traits from `aisopod-channel`
4. **`builtin.rs`**: Provides `register_builtin_plugins()` and `list_available_builtins()` functions

### Dependency Flow (Corrected)

```
 aisopod-plugin
   ├── optional: aisopod-channel-telegram
   ├── optional: aisopod-channel-discord
   ├── optional: aisopod-channel-slack
   ├── optional: aisopod-channel-whatsapp
   ├── optional: aisopod-provider-openai
   └── optional: aisopod-provider-anthropic

 aisopod-provider-openai
   └── depends on: aisopod-provider (NOT aisopod-plugin)

 aisopod-provider-anthropic
   └── depends on: aisopod-provider (NOT aisopod-plugin)

 aisopod-channel-* (all channels)
   └── depend on: aisopod-channel (NOT aisopod-plugin)
```

## Key Design Decisions

### 1. Wrapper Types for Providers

The provider crates (`aisopod-provider-openai`, `aisopod-provider-anthropic`) do NOT depend on `aisopod-plugin`. Instead:

- Provider crates implement provider-specific logic
- `builtin.rs` creates wrapper types that implement the `Plugin` trait
- Wrappers delegate to underlying provider instances

**Reason**: Prevents circular dependencies. If provider crates depended on `aisopod-plugin` AND `aisopod-plugin` had optional dependencies on provider crates, we'd have:
```
aisopod-plugin → aisopod-provider-openai → aisopod-plugin (cycle!)
```

### 2. Conditional Compilation Pattern

Each plugin is registered using `#[cfg(feature = "...")]`:

```rust
pub fn register_builtin_plugins(registry: &mut PluginRegistry) -> Result<(), RegistryError> {
    #[cfg(feature = "plugin-telegram")]
    {
        register_telegram_plugin(registry)?;
    }
    
    #[cfg(feature = "plugin-openai")]
    {
        register_openai_plugin(registry)?;
    }
    
    // ... more plugins
    Ok(())
}
```

**Benefits**:
- Disabled plugins produce zero runtime overhead (no code included)
- Each plugin is independent and can be enabled/disabled separately
- `all-plugins` meta-feature enables all built-in plugins

### 3. Wrapper Type Implementation

For provider plugins, wrapper types implement `Plugin` trait:

```rust
#[cfg(feature = "plugin-openai")]
#[derive(Debug)]
pub struct OpenAIPluginWrapper {
    provider: aisopod_provider_openai::OpenAIPlugin,
}

#[cfg(feature = "plugin-openai")]
impl Plugin for OpenAIPluginWrapper {
    fn id(&self) -> &str {
        "openai"
    }
    
    fn meta(&self) -> &crate::PluginMeta {
        // Static metadata
    }
    
    async fn init(&self, _ctx: &crate::PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing OpenAI plugin");
        Ok(())
    }
    
    // ... other methods
}
```

### 4. Channel Plugins Integration

Channel plugins implement the `ChannelPlugin` trait from `aisopod-channel`. The `builtin.rs` registers them directly:

```rust
#[cfg(feature = "plugin-telegram")]
fn register_telegram_plugin(registry: &mut PluginRegistry) -> Result<(), RegistryError> {
    use aisopod_channel_telegram::TelegramChannel;
    use aisopod_channel::ChannelPlugin;
    
    let channel = TelegramChannel::default();
    registry.register(Arc::new(channel));
    info!("Registered built-in plugin: telegram");
    Ok(())
}
```

## Feature Flag Structure

### Channel Plugins

```toml
plugin-telegram = ["dep:aisopod-channel-telegram"]
plugin-discord = ["dep:aisopod-channel-discord"]
plugin-slack = ["dep:aisopod-channel-slack"]
plugin-whatsapp = ["dep:aisopod-channel-whatsapp"]
```

### Provider Plugins

```toml
plugin-openai = ["dep:aisopod-provider-openai"]
plugin-anthropic = ["dep:aisopod-provider-anthropic"]
```

### Meta Feature

```toml
all-plugins = [
    "plugin-telegram",
    "plugin-discord",
    "plugin-slack",
    "plugin-whatsapp",
    "plugin-openai",
    "plugin-anthropic",
]
```

## Verification Steps

The following commands verify the implementation:

```bash
# Basic build (no features)
cargo build -p aisopod-plugin

# Build with all plugins enabled
cargo build -p aisopod-plugin --features all-plugins

# Build with minimal features
cargo build -p aisopod-plugin --no-default-features

# Run tests
cargo test -p aisopod-plugin
```

## Common Pitfalls and Solutions

### 1. Circular Dependency (RESOLVED)

**Problem**: Provider crates depended on `aisopod-plugin`, while `aisopod-plugin` had optional dependencies on provider crates.

**Solution**: Removed `aisopod-plugin` dependency from provider crates. Created wrapper types in `builtin.rs` that implement `Plugin` trait.

### 2. Debug Trait on Provider Structs

**Problem**: Some provider structs had `#[derive(Debug)]` but their dependencies didn't implement `Debug`.

**Solution**: Removed `#[derive(Debug)]` from provider plugin structs in `builtin.rs`.

### 3. Unused Imports

**Problem**: Removed provider implementations still imported types that no longer existed.

**Solution**: Updated provider implementations to only import types from `aisopod-provider`, not `aisopod-plugin`.

## Testing Strategy

### Unit Tests

1. **`test_list_available_builtins()`**: Verifies the function compiles and returns valid plugin IDs
2. **`test_register_builtin_plugins_empty()`**: Verifies the function works with no features enabled

### Feature-specific Testing

Build with specific features to verify:
- Individual plugins can be compiled
- Plugins don't interfere with each other
- `all-plugins` feature enables everything

## Usage Examples

### Build with specific plugins

```bash
# Only Telegram and OpenAI
cargo build --features plugin-telegram,plugin-openai

# Only Discord
cargo build --features plugin-discord

# All plugins
cargo build --features all-plugins
```

### Build minimal binary

```bash
# No plugins at all
cargo build --no-default-features
```

### Runtime plugin listing

```rust
use aisopod_plugin::list_available_builtins;

let builtins = list_available_builtins();
for plugin_id in builtins {
    println!("Available plugin: {}", plugin_id);
}
```

## Files Modified

1. **`crates/aisopod-plugin/Cargo.toml`**: Feature flags and optional dependencies
2. **`crates/aisopod-plugin/src/builtin.rs`**: Main implementation with conditional compilation
3. **`crates/aisopod-provider-openai/Cargo.toml`**: Removed `aisopod-plugin` dependency
4. **`crates/aisopod-provider-openai/src/lib.rs`**: Simplified to not use `aisopod-plugin`
5. **`crates/aisopod-provider-anthropic/Cargo.toml`**: Removed `aisopod-plugin` dependency
6. **`crates/aisopod-provider-anthropic/src/lib.rs`**: Simplified to not use `aisopod-plugin`

## Acceptance Criteria (Issue #111)

- [x] Cargo feature flags are defined for each built-in plugin
- [x] `register_builtin_plugins()` conditionally registers plugins based on enabled features
- [x] Disabled plugins produce zero runtime overhead (no code included)
- [x] `list_available_builtins()` reports which plugins are compiled in
- [x] Building with `--features all-plugins` includes all built-in plugins
- [x] Building with no features produces a minimal binary
- [x] `cargo build -p aisopod-plugin` compiles without errors (with and without features)

## Future Enhancements

1. **Channel-specific metadata**: Each channel plugin could expose its own metadata via `list_available_builtins()`
2. **Dynamic version reporting**: The `PluginMeta` in wrappers could be generated at compile time
3. **Plugin ordering**: Support for specifying registration order via feature attributes
4. **Plugin groups**: Meta-features for categories of plugins (e.g., `all-channels`, `all-providers`)
5. **Plugin dependencies**: Feature flags that auto-enable required dependencies

## References

- Issue: #111 - Implement Compiled-In Plugin Loading (Phase 1)
- Issue: #110 - Implement PluginRegistry Lifecycle Management
- Issue: #107 - Define Plugin Trait and PluginMeta Types
- Issue: #112 - Implement Dynamic Shared Library Plugin Loading (future)
