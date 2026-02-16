# Issue 107: Define Plugin Trait and PluginMeta Types

## Summary
Define the core `Plugin` trait, `PluginMeta` struct, and `PluginContext` struct in the `aisopod-plugin` crate. These types form the foundation of the entire plugin system, establishing the interface that every plugin must implement and the metadata that describes each plugin's identity and capabilities.

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

## Acceptance Criteria
- [x] `Plugin` trait is defined with `id()`, `meta()`, `register()`, `init()`, and `shutdown()` methods
- [x] `PluginMeta` struct includes name, version, description, author, supported_channels, and supported_providers fields
- [x] `PluginContext` struct provides runtime context including config and data directory
- [x] All public types and methods have documentation comments
- [x] `cargo build -p aisopod-plugin` compiles without errors
- [x] `cargo doc -p aisopod-plugin` generates documentation without warnings

## Resolution

Issue 107 was implemented by the implementation manager in commit 29179614a8b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6.

### Implementation Details:
1. **Created `meta.rs`** with `PluginMeta` struct:
   - Fields: name, version, description, author, supported_channels, supported_providers
   - Derives: Debug, Clone, Serialize, Deserialize

2. **Created `context.rs`** with `PluginContext` struct:
   - Fields: config (Arc<serde_json::Value>), data_dir (PathBuf)
   - Provides runtime context to plugins during initialization

3. **Created `trait.rs`** with `Plugin` trait:
   - Methods: id(), meta(), register(), init(), shutdown()
   - Uses async_trait for async methods
   - Implements Send + Sync bounds for compatibility with compiled-in and dynamic plugins

4. **Updated `lib.rs`**:
   - Declares modules: `trait`, `meta`, `context`
   - Re-exports: `Plugin`, `PluginMeta`, `PluginContext`
   - Added `PluginApi` trait (minimal placeholder for future expansion)

5. **Added `async-trait` dependency** to `Cargo.toml`

### Files Created:
- `crates/aisopod-plugin/src/context.rs`
- `crates/aisopod-plugin/src/meta.rs`
- `crates/aisopod-plugin/src/trait.rs`

### Verification:
- `cargo build -p aisopod-plugin` compiles successfully
- `cargo doc -p aisopod-plugin` generates documentation without warnings
- All public types have comprehensive documentation

---
*Created: 2026-02-15*
*Resolved: 2026-02-16*
