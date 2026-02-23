# PluginRegistry Lifecycle Management Implementation

## Summary

This document captures key learnings from implementing the `PluginRegistry` struct with full lifecycle management for the aisopod plugin system.

## Implementation Details

### Registry Structure

The `PluginRegistry` uses two key data structures:

```rust
pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn Plugin>>,
    load_order: Vec<String>,
}
```

- **`plugins: HashMap<String, Arc<dyn Plugin>>`**: Stores plugin instances keyed by their unique ID
- **`load_order: Vec<String>`**: Maintains registration order for ordered initialization and reverse-order shutdown

The `Arc<dyn Plugin>` allows:
- Thread-safe sharing of plugin instances
- Object-safe trait objects for dynamic dispatch
- Efficient cloning without duplicating plugin state

### Lifecycle Methods

#### `register()`
- Checks for duplicate plugin IDs before registration
- Adds plugin to both `plugins` HashMap and `load_order` Vec
- Logs registration event via `tracing::info!`

#### `init_all()`
- Initializes plugins in registration order (FIFO)
- Stops on first initialization failure
- Returns `RegistryError::InitFailed` with plugin ID and error message

#### `shutdown_all()`
- Shuts down plugins in reverse registration order (LIFO)
- Continues shutdown for all plugins even if individual failures occur
- Logs shutdown failures at `warn` level but does not propagate errors

### Error Handling

The `RegistryError` enum provides type-safe error handling:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    DuplicatePlugin(String),
    InitFailed(String, String),
    NotFound(String),
}
```

This design:
- Prevents runtime string comparisons for error types
- Provides structured error information for debugging
- Integrates with `thiserror` for automatic `Display` implementation

### Thread Safety

The registry is designed for multi-threaded usage:
- All plugins are `Arc<dyn Plugin>` ensuring thread-safe sharing
- The registry itself is `Send + Sync` through its components
- Registration order is protected by the `&mut self` requirement on `register()`

## Design Decisions

### Why Two Data Structures?

Using both a `HashMap` and a `Vec` enables:
- O(1) lookup by ID via HashMap
- O(1) iteration in registration order via Vec
- No need to sort or re-index on operations

### Why Reverse Order for Shutdown?

The LIFO shutdown order ensures:
- Plugins that depend on other plugins are shut down first
- Dependencies can still function during dependent plugin shutdown
- Cleaner resource cleanup chain

### Why Continue on Shutdown Failure?

Shutdown failures:
- May be transient (network issues, temporary resource locks)
- The system is already shutting down
- Stopping early could leave other plugins in inconsistent states

Logging at warn level allows administrators to see which plugins had issues without blocking the full shutdown sequence.

## Testing Strategy

Unit tests cover:
- Registry creation and initial state
- Plugin registration (success and duplicate cases)
- Plugin retrieval by ID
- Plugin listing in registration order
- Initialization of all plugins
- Shutdown of all plugins
- Load order preservation

### Testing Pattern

Tests use a `TestPlugin` mock struct that:
- Implements the `Plugin` trait
- Stores plugin metadata
- Returns `Ok(())` for all async operations (real plugins would do actual work)

This pattern:
- Isolates registry logic from plugin implementation details
- Allows testing without external dependencies
- Can be extended to track method calls for more complex scenarios

## Common Pitfalls Avoided

1. **Shadowing `self` in async methods**: The `&self` receiver works correctly with `async fn` because `async fn` returns a `impl Future` that captures the reference properly.

2. **HashMap vs BTreeMap**: Using `HashMap` for O(1) lookups since order is tracked separately in `Vec`.

3. **Arc ownership**: Plugins are stored as `Arc<dyn Plugin>` to allow:
   - Multiple references to the same plugin
   - Thread-safe sharing
   - No copying of plugin state

4. **Error propagation**: `init_all()` stops on first failure (fail-fast), while `shutdown_all()` continues (best-effort) because shutdown should complete even if individual plugins fail.

## Future Enhancements

Potential improvements:
1. **Configurable shutdown timeout**: Allow plugins to have shutdown deadlines
2. **Shutdown hooks**: Add callbacks before/after shutdown
3. **Health checking**: Add health status tracking per plugin
4. **Lazy initialization**: Support on-demand initialization of plugins
5. **Graceful degradation**: Allow registry to continue with failed plugins during init

## References

- Issue: #110 - Implement PluginRegistry Lifecycle Management
- Related: #107 - Define Plugin Trait and PluginMeta Types
- Related: #108 - Implement Plugin API for Capability Registration
- Related: #111 - Implement Compiled-in Plugin Loading
- Related: #112 - Implement Dynamic Shared Library Plugin Loading
