# Issue 024: Implement Config File Watcher for Hot Reload

## Summary
Add the `notify` crate as a dependency and implement a config file watcher that monitors the configuration file for changes. On detecting a change, it reloads the config, validates it, and emits a notification via a `tokio` channel so that other components can react to configuration updates at runtime.

## Location
- Crate: `aisopod-config`
- File: `crates/aisopod-config/src/watcher.rs`, `crates/aisopod-config/Cargo.toml`

## Current Behavior
Configuration is loaded once at startup. Any changes to the config file require a full application restart to take effect. There is no file watching or hot reload mechanism.

## Expected Behavior
- `ConfigWatcher` struct watches a config file path for modifications
- On file change: reload the file, parse it, expand env vars, process includes, validate
- If the new config is valid, send it via a `tokio::sync::watch` channel
- If the new config is invalid, log the error but keep the previous valid config
- Diff detection identifies which top-level sections changed
- Debounce rapid changes (e.g., editors that write multiple times)
- Clean shutdown via a `stop()` method or drop

## Impact
Hot reload enables zero-downtime configuration updates, which is essential for long-running services. Operators can adjust settings (e.g., add agents, change model parameters) without restarting the gateway.

## Suggested Implementation
1. Add dependencies to `crates/aisopod-config/Cargo.toml`:
   ```toml
   [dependencies]
   notify = "6"
   tokio = { workspace = true, features = ["sync", "time"] }
   tracing = { workspace = true }
   ```
2. Create `crates/aisopod-config/src/watcher.rs`:
   ```rust
   use std::path::{Path, PathBuf};
   use std::time::Duration;
   use anyhow::Result;
   use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event};
   use tokio::sync::watch;
   use tracing::{info, warn, error};
   use crate::loader::load_config;
   use crate::types::AisopodConfig;

   pub struct ConfigWatcher {
       _watcher: RecommendedWatcher,
       pub receiver: watch::Receiver<AisopodConfig>,
   }

   impl ConfigWatcher {
       /// Start watching a configuration file for changes.
       pub fn new(config_path: &Path) -> Result<Self> {
           let initial_config = load_config(config_path)?;
           let (tx, rx) = watch::channel(initial_config);

           let path = config_path.to_path_buf();
           let mut watcher = notify::recommended_watcher(
               move |res: Result<Event, notify::Error>| {
                   match res {
                       Ok(_event) => {
                           info!("Config file changed, reloading: {}", path.display());
                           match load_config(&path) {
                               Ok(new_config) => {
                                   if tx.send(new_config).is_err() {
                                       warn!("Config receiver dropped");
                                   }
                               }
                               Err(e) => {
                                   error!(
                                       "Failed to reload config: {}. Keeping previous config.",
                                       e
                                   );
                               }
                           }
                       }
                       Err(e) => {
                           error!("File watch error: {}", e);
                       }
                   }
               },
           )?;

           watcher.watch(config_path, RecursiveMode::NonRecursive)?;

           Ok(Self {
               _watcher: watcher,
               receiver: rx,
           })
       }
   }
   ```
3. Declare the module in `lib.rs`:
   ```rust
   pub mod watcher;
   pub use watcher::ConfigWatcher;
   ```
4. Add a section-level diff detection utility:
   ```rust
   /// Identify which top-level sections changed between two configs.
   pub fn diff_sections(old: &AisopodConfig, new: &AisopodConfig) -> Vec<String> {
       let mut changed = Vec::new();
       let old_val = serde_json::to_value(old).unwrap_or_default();
       let new_val = serde_json::to_value(new).unwrap_or_default();

       if let (serde_json::Value::Object(o), serde_json::Value::Object(n)) = (&old_val, &new_val) {
           for (key, old_v) in o {
               if n.get(key) != Some(old_v) {
                   changed.push(key.clone());
               }
           }
           for key in n.keys() {
               if !o.contains_key(key) {
                   changed.push(key.clone());
               }
           }
       }
       changed
   }
   ```
5. Add a debounce mechanism using `tokio::time::sleep` to avoid reloading on every rapid write event from text editors.
6. Add tests:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use std::io::Write;
       use tempfile::NamedTempFile;

       #[tokio::test]
       async fn test_watcher_detects_change() {
           // Create a temp config file
           let mut file = NamedTempFile::with_suffix(".json5").unwrap();
           writeln!(file, "{{ meta: {{ version: \"1.0\" }} }}").unwrap();

           let watcher = ConfigWatcher::new(file.path()).unwrap();
           let mut rx = watcher.receiver.clone();

           // Modify the file
           std::fs::write(
               file.path(),
               "{ meta: { version: \"2.0\" } }",
           ).unwrap();

           // Wait for the change to be detected
           tokio::time::sleep(Duration::from_millis(500)).await;
           rx.changed().await.unwrap();
           let config = rx.borrow();
           assert_eq!(config.meta.version, "2.0");
       }
   }
   ```
7. Run `cargo test -p aisopod-config` to verify all tests pass.

## Dependencies
017, 021

## Acceptance Criteria
- [ ] `notify` crate is added as a dependency
- [ ] `ConfigWatcher` watches a config file and reloads on changes
- [ ] Valid config changes are sent via a `tokio::sync::watch` channel
- [ ] Invalid configs are logged as errors and the previous config is retained
- [ ] Section-level diff detection identifies which sections changed
- [ ] Rapid changes are debounced to avoid excessive reloads
- [ ] Integration test verifies file change triggers reload and notification

---
*Created: 2026-02-15*
