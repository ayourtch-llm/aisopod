# Learning: Implementing Hook System for Lifecycle Events

## Issue
Issue 113: Implement a hook system that allows plugins to subscribe to lifecycle events throughout the application.

## Summary of Implementation

### 1. Hook Enum Extension
The original `Hook` enum had only 5 variants (SystemStart, SystemShutdown, ConfigChanged, PluginLoaded, PluginUnloaded). For Issue 113, it was expanded to 12 lifecycle event variants:

- `BeforeAgentRun` - Before the agent begins processing a request
- `AfterAgentRun` - After the agent completes processing a request
- `BeforeMessageSend` - Before a message is sent to the user
- `AfterMessageReceive` - After a message is received from the user
- `BeforeToolExecute` - Before a tool is executed
- `AfterToolExecute` - After a tool is executed
- `OnSessionCreate` - When a new session is created
- `OnSessionEnd` - When a session ends
- `OnGatewayStart` - When the gateway starts
- `OnGatewayShutdown` - When the gateway shuts down
- `OnClientConnect` - When a client connects
- `OnClientDisconnect` - When a client disconnects

### 2. HookContext Struct
Created `HookContext` struct to pass event data to handlers:
- Contains `hook` field for the hook type
- Contains `data` field (HashMap<String, serde_json::Value>) for arbitrary event data
- Provides helper methods: `new()`, `with_data()`, `with_data_entry()`

### 3. HookHandler Trait Update
Changed from sync to async:
- **Before**: `fn on_hook(&self, hook: Hook, data: Option<&serde_json::Value>) -> Result<...>`
- **After**: `async fn handle(&self, ctx: &HookContext) -> Result<...>`
- Uses `#[async_trait::async_trait]` attribute for async trait implementation

### 4. HookRegistry Implementation
Created `HookRegistry` struct with:
- `handlers: HashMap<Hook, Vec<(String, Arc<dyn HookHandler>)>>` - stores handlers per hook type
- Methods:
  - `new()` - creates empty registry
  - `register()` - registers a handler for a hook
  - `dispatch()` - dispatches event to all registered handlers
  - `handler_count()` - returns handler count for a hook
  - `total_hook_count()` - total handlers across all hooks
  - `hook_type_count()` - number of hook types with handlers
  - `transfer_from_api()` - transfers registrations from `PluginApi`

### 5. PluginHookHandler Update
Extended to include plugin_id:
- Added `plugin_id: String` field to track which plugin registered each handler
- Updated constructor: `PluginHookHandler::new(hook, plugin_id, handler)`

### 6. PluginApi Update
Modified `register_hook()` to accept plugin_id:
- Before: `register_hook(&mut self, hook: Hook, handler: Arc<dyn HookHandler>)`
- After: `register_hook(&mut self, hook: Hook, plugin_id: String, handler: Arc<dyn HookHandler>)`

### 7. PluginRegistry Integration
Added `HookRegistry` to `PluginRegistry`:
- Added `hook_registry: HookRegistry` field
- Added `hook_registry()` and `hook_registry_mut()` accessors
- Added `register_with_hooks()` method that:
  1. Registers the plugin
  2. Calls plugin's `register()` method with `PluginApi`
  3. Transfers hook registrations to `HookRegistry`

## Key Design Decisions

### Async vs Sync
The original `HookHandler` trait used sync `on_hook()`. The new async `handle()` method:
- Allows for non-blocking operations in hook handlers
- Better matches modern async-first Rust patterns
- Uses `#[async_trait]` macro for convenience

### Error Handling Strategy
Hook errors are logged but don't block other handlers:
```rust
match handler.handle(ctx).await {
    Ok(()) => debug!(...),
    Err(e) => debug!(...), // Log but continue
}
```
This ensures one failing handler doesn't prevent other hooks from running.

### Transfer Pattern
Hooks are registered through `PluginApi` during plugin registration, then transferred to `HookRegistry`:
1. Plugin's `register()` method uses `api.register_hook()`
2. `register_with_hooks()` transfers from API to registry
3. Registry handles dispatching during lifecycle events

### Type Safety
- `Hook` enum with `Copy + Clone + PartialEq + Eq + Hash` for efficient map keys
- Strong typing via `HookContext` instead of raw data
- `Arc<dyn HookHandler>` for thread-safe shared ownership

## Usage Example

```rust
use aisopod_plugin::{Hook, HookContext, HookHandler, HookRegistry};
use std::sync::Arc;
use async_trait::async_trait;

struct LoggingHandler;

#[async_trait]
impl HookHandler for LoggingHandler {
    async fn handle(&self, ctx: &HookContext) -> Result<(), Box<dyn std::error::Error>> {
        println!("Hook {} triggered with data: {:?}", ctx.hook, ctx.data);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let mut registry = HookRegistry::new();
    registry.register(
        Hook::BeforeAgentRun,
        "logging-plugin".to_string(),
        Arc::new(LoggingHandler),
    );
    
    let ctx = HookContext::new(Hook::BeforeAgentRun)
        .with_data_entry("request_id", serde_json::json!("123"));
    
    registry.dispatch(&ctx).await;
}
```

## Testing Strategy
Added comprehensive tests for:
- Hook enum properties (Debug, Clone, PartialEq, Eq, Hash)
- HookContext creation and data manipulation
- HookRegistry registration and dispatch
- Error handling (failing handlers don't block others)
- Handler count methods
- Total hook count

## Notes
- `HookRegistry` doesn't implement `Debug` because it contains `Arc<dyn HookHandler>` which doesn't implement `Debug`
- `PluginHookHandler` doesn't implement `Debug` for the same reason
- The `register_with_hooks()` method in `PluginRegistry` handles the integration pattern where plugins register hooks through `PluginApi` during their `register()` phase

## References
- Issue 113: Implement Hook System for Lifecycle Events
- Issue 108: PluginApi for capability registration
- Issue 110: PluginRegistry lifecycle management
