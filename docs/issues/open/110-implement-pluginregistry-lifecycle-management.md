# Issue 110: Implement PluginRegistry Lifecycle Management

## Summary
Implement the `PluginRegistry` struct that manages the full lifecycle of plugins — registration, retrieval, listing, ordered initialization, and reverse-order shutdown. The registry stores plugins as `Arc<dyn Plugin>` keyed by their unique ID and detects duplicate registrations.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/registry.rs`

## Current Behavior
The `Plugin` trait (Issue 107) and `PluginApi` (Issue 108) exist but there is no central registry to manage plugin instances and their lifecycle.

## Expected Behavior
A `PluginRegistry` struct provides `register()`, `get()`, and `list()` methods for managing plugin instances. It supports `init_all()` to initialize plugins in registration order and `shutdown_all()` to shut them down in reverse order. Attempting to register a plugin with a duplicate ID returns an error.

## Impact
The registry is the central coordinator of the plugin system. All plugin loading strategies (compiled-in, dynamic) funnel through it, and it orchestrates the startup and shutdown sequences.

## Suggested Implementation
1. **Define `PluginRegistry` in `registry.rs`:**
   ```rust
   use std::collections::HashMap;
   use std::sync::Arc;
   use crate::{Plugin, PluginContext};

   pub struct PluginRegistry {
       plugins: HashMap<String, Arc<dyn Plugin>>,
       load_order: Vec<String>,
   }

   impl PluginRegistry {
       pub fn new() -> Self {
           Self {
               plugins: HashMap::new(),
               load_order: Vec::new(),
           }
       }
   }
   ```
2. **Implement `register()`:**
   ```rust
   impl PluginRegistry {
       pub fn register(&mut self, plugin: Arc<dyn Plugin>) -> Result<(), RegistryError> {
           let id = plugin.id().to_string();
           if self.plugins.contains_key(&id) {
               return Err(RegistryError::DuplicatePlugin(id));
           }
           self.load_order.push(id.clone());
           self.plugins.insert(id, plugin);
           Ok(())
       }
   }
   ```
3. **Implement `get()` and `list()`:**
   ```rust
   impl PluginRegistry {
       pub fn get(&self, id: &str) -> Option<&Arc<dyn Plugin>> {
           self.plugins.get(id)
       }

       pub fn list(&self) -> Vec<&Arc<dyn Plugin>> {
           self.load_order
               .iter()
               .filter_map(|id| self.plugins.get(id))
               .collect()
       }
   }
   ```
4. **Implement `init_all()`:**
   ```rust
   impl PluginRegistry {
       pub async fn init_all(&self, ctx: &PluginContext) -> Result<(), RegistryError> {
           for id in &self.load_order {
               if let Some(plugin) = self.plugins.get(id) {
                   tracing::info!(plugin_id = %id, "Initializing plugin");
                   plugin.init(ctx).await.map_err(|e| {
                       RegistryError::InitFailed(id.clone(), e.to_string())
                   })?;
               }
           }
           Ok(())
       }
   }
   ```
5. **Implement `shutdown_all()`:**
   ```rust
   impl PluginRegistry {
       pub async fn shutdown_all(&self) -> Result<(), RegistryError> {
           for id in self.load_order.iter().rev() {
               if let Some(plugin) = self.plugins.get(id) {
                   tracing::info!(plugin_id = %id, "Shutting down plugin");
                   if let Err(e) = plugin.shutdown().await {
                       tracing::error!(plugin_id = %id, error = %e, "Plugin shutdown failed");
                   }
               }
           }
           Ok(())
       }
   }
   ```
6. **Define `RegistryError`:**
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum RegistryError {
       #[error("Duplicate plugin ID: {0}")]
       DuplicatePlugin(String),
       #[error("Plugin '{0}' initialization failed: {1}")]
       InitFailed(String, String),
       #[error("Plugin not found: {0}")]
       NotFound(String),
   }
   ```
7. **Add tracing** for lifecycle events — log each plugin as it initializes and shuts down.

## Dependencies
- Issue 107 (Plugin trait and PluginMeta types)
- Issue 108 (PluginApi for capability registration)

## Acceptance Criteria
- [ ] `PluginRegistry` struct stores plugins keyed by unique ID
- [ ] `register()` adds plugins and detects duplicate IDs
- [ ] `get()` retrieves a plugin by ID
- [ ] `list()` returns plugins in registration order
- [ ] `init_all()` initializes plugins in load order
- [ ] `shutdown_all()` shuts down plugins in reverse load order
- [ ] Shutdown continues even if individual plugins fail
- [ ] `RegistryError` provides descriptive error variants
- [ ] `cargo build -p aisopod-plugin` compiles without errors

---
*Created: 2026-02-15*
