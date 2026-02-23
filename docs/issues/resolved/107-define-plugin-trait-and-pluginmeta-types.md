# Issue 107: Define Plugin Trait and PluginMeta Types

## Summary
Define the core `Plugin` trait, `PluginMeta` struct, and `PluginContext` struct in the `aisopod-plugin` crate. These types form the foundation of the entire plugin system, establishing the interface that every plugin must implement and the metadata that describes each plugin's identity and capabilities.

## Resolution

Issue 107 was completed by first implementing Issue 108 (PluginApi struct) as a dependency, then defining the complete plugin system API:

1. **Implemented Issue 108 first** - Created the `PluginApi` struct as a concrete struct (not a trait) with registration methods:
   - `register_channel()` - for registering channel implementations
   - `register_tool()` - for registering tool implementations
   - `register_command()` - for registering CLI commands
   - `register_provider()` - for registering model provider implementations
   - `register_hook()` - for registering lifecycle event handlers

2. **Defined PluginCommand struct** - For representing CLI commands with name, description, and handler functions.

3. **Defined Hook enum and HookHandler trait** - For handling lifecycle events like startup and shutdown.

4. **Updated Plugin trait** - Changed to use `PluginApi` as a concrete struct parameter in the `register()` method.

5. **Added dependencies** - The crate now depends on:
   - `aisopod-channel` crate
   - `aisopod-provider` crate
   - `aisopod-tools` crate

6. **Exported all types** - All public types are properly exported from `lib.rs`:
   - `Plugin` trait
   - `PluginMeta` struct
   - `PluginContext` struct
   - `PluginApi` struct
   - `PluginCommand` struct
   - `Hook` enum
   - `HookHandler` trait

7. **Added documentation** - All public items have comprehensive documentation comments explaining their role in the plugin lifecycle.

The implementation supports both compiled-in and dynamically loaded plugins through the plugin trait interface.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/trait.rs`, `crates/aisopod-plugin/src/meta.rs`, `crates/aisopod-plugin/src/context.rs`

## Current Behavior
The `aisopod-plugin` crate scaffold exists (Issue 010) but contains no trait definitions or types for the plugin system.

## Expected Behavior
The crate exports a `Plugin` trait with methods `id()`, `meta()`, `register()`, `init()`, and `shutdown()`. A `PluginMeta` struct describes each plugin's name, version, description, author, supported channels, and supported providers. A `PluginContext` struct provides runtime context to plugins during initialization, including access to configuration and logging.

## Impact
Every other plugin system issue depends on this trait definition. It is the single most critical type in the plugin architecture and must be designed carefully to support both compiled-in and dynamically loaded plugins.

## Suggested Implementation
1. **Define `PluginMeta` in `meta.rs`:**
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct PluginMeta {
       pub name: String,
       pub version: String,
       pub description: String,
       pub author: String,
       pub supported_channels: Vec<String>,
       pub supported_providers: Vec<String>,
   }
   ```
2. **Define `PluginContext` in `context.rs`:**
   ```rust
   use std::sync::Arc;

   /// Runtime context provided to plugins during initialization.
   pub struct PluginContext {
       pub config: Arc<serde_json::Value>,
       pub data_dir: std::path::PathBuf,
   }
   ```
3. **Define the `Plugin` trait in `trait.rs`:**
   ```rust
   use async_trait::async_trait;
   use crate::{PluginMeta, PluginContext, PluginApi};

   #[async_trait]
   pub trait Plugin: Send + Sync {
       /// Returns the unique identifier for this plugin.
       fn id(&self) -> &str;

       /// Returns metadata describing this plugin.
       fn meta(&self) -> &PluginMeta;

       /// Called during plugin loading to register capabilities.
       fn register(&self, api: &mut PluginApi) -> Result<(), Box<dyn std::error::Error>>;

       /// Called after all plugins are registered to perform initialization.
       async fn init(&self, ctx: &PluginContext) -> Result<(), Box<dyn std::error::Error>>;

       /// Called during graceful shutdown.
       async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
   }
   ```
4. **Re-export all types from `lib.rs`:**
   ```rust
   mod r#trait;
   mod meta;
   mod context;

   pub use r#trait::Plugin;
   pub use meta::PluginMeta;
   pub use context::PluginContext;
   ```
5. **Add documentation comments** to every public type and method explaining its role in the plugin lifecycle.

## Dependencies
- Issue 010 (create aisopod-plugin crate scaffold)
- Issue 049 (Tool trait) - for PluginApi to register tools
- Issue 089 (ChannelPlugin trait) - for PluginApi to register channels
- Issue 038 (ModelProvider trait) - for PluginApi to register providers
- Issue 108 (PluginApi struct) - MUST be resolved before this issue

## Acceptance Criteria
- [ ] `Plugin` trait is defined with `id()`, `meta()`, `register()`, `init()`, and `shutdown()` methods
- [ ] `PluginMeta` struct includes name, version, description, author, supported_channels, and supported_providers fields
- [ ] `PluginContext` struct provides runtime context including config and data directory
- [ ] All public types and methods have documentation comments
- [ ] `PluginApi` is a **struct** (not a trait) with registration methods
- [ ] `cargo build -p aisopod-plugin` compiles without errors
- [ ] `cargo doc -p aisopod-plugin` generates documentation without warnings

## Important Notes
This issue **MUST NOT** be resolved until Issue 108 (PluginApi) is fully implemented. The `PluginApi` type must be a concrete struct with methods like `register_channel()`, `register_tool()`, `register_provider()`, and `register_command()` before this issue can be considered complete.

## Resolved
2026-02-23
