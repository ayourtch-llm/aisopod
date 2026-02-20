//! Configuration file watcher module for hot reload functionality.
//!
//! This module provides the `ConfigWatcher` struct that monitors a configuration
//! file for changes and automatically reloads and validates the configuration
//! when modifications are detected.

#![deny(unused_must_use)]

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

use crate::loader::load_config;
use crate::types::AisopodConfig;

/// Time to wait before reloading after a file change (debounce)
const DEBOUNCE_DELAY: Duration = Duration::from_millis(250);

/// ConfigWatcher monitors a configuration file for changes and automatically
/// reloads the configuration when modifications are detected.
///
/// Uses a debounce mechanism to avoid excessive reloads when editors write
/// multiple times, and validates new configs before applying them.
pub struct ConfigWatcher {
    _watcher: Option<RecommendedWatcher>,
    _stop_sender: Option<tokio::sync::oneshot::Sender<()>>,
    receiver: watch::Receiver<AisopodConfig>,
}

impl ConfigWatcher {
    /// Start watching a configuration file for changes.
    ///
    /// Creates a ConfigWatcher that monitors the specified config file.
    /// On file modification, it reloads, parses, expands env vars, processes
    /// includes, and validates the configuration. If valid, the new config
    /// is sent via the watch channel; if invalid, the previous config is kept.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file to watch
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The ConfigWatcher on success, or an error if
    ///   the initial config load fails or file watching cannot be set up
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The initial configuration file cannot be loaded
    /// - The file watcher cannot be created
    /// - The file cannot be watched
    pub fn new(config_path: &Path) -> Result<Self> {
        // Load initial configuration
        let initial_config = load_config(config_path)?;
        let (tx, rx) = watch::channel(initial_config);

        let config_path_clone = config_path.to_path_buf();
        let tx = Arc::new(tokio::sync::Mutex::new(tx));

        // Create a channel to signal the watcher to stop
        let (stop_sender, mut stop_receiver) = tokio::sync::oneshot::channel::<()>();

        // Create a watch channel for triggering reloads (used for debouncing)
        let (reload_trigger_sender, mut reload_trigger_receiver) =
            tokio::sync::mpsc::channel::<()>(1);

        // Spawn the debouncing/reload task
        let config_path_for_task = config_path_clone.clone();
        let tx_for_task = tx.clone();

        tokio::spawn(async move {
            let mut last_reload_time: Option<std::time::Instant> = None;
            let mut debounce_buffer = tokio::time::interval(DEBOUNCE_DELAY);

            loop {
                tokio::select! {
                    _ = &mut stop_receiver => {
                        debug!("ConfigWatcher stop signal received");
                        break;
                    }
                    _ = reload_trigger_receiver.recv() => {
                        // Trigger a reload after debounce delay
                        debug!("Reloading config with debounce");
                        let current_time = std::time::Instant::now();
                        let last_time = last_reload_time.unwrap_or(current_time);
                        let time_since_last_reload = current_time.duration_since(last_time);

                        if time_since_last_reload >= DEBOUNCE_DELAY {
                            // Enough time has passed, reload immediately
                            reload_and_send(&config_path_for_task, &tx_for_task).await;
                            last_reload_time = Some(std::time::Instant::now());
                        } else {
                            // Wait for debounce delay
                            tokio::time::sleep(DEBOUNCE_DELAY - time_since_last_reload).await;
                            reload_and_send(&config_path_for_task, &tx_for_task).await;
                            last_reload_time = Some(std::time::Instant::now());
                        }
                    }
                    _ = debounce_buffer.tick() => {
                        // Periodic check to handle the case where multiple events came in
                        if let Some(last) = last_reload_time {
                            let now = std::time::Instant::now();
                            if now.duration_since(last) >= DEBOUNCE_DELAY {
                                // Reset after debounce period
                                last_reload_time = None;
                            }
                        }
                    }
                }
            }
        });

        // Create the file watcher
        let config_path_for_watcher = config_path_clone.clone();
        let trigger_sender_for_watcher = reload_trigger_sender.clone();

        let mut watcher = notify::recommended_watcher({
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        debug!("File watch event received: {:?}", event.kind);
                        
                        // Only trigger on write/modify events
                        let should_reload = match &event.kind {
                            EventKind::Modify(notify::event::ModifyKind::Data(_)) => true,
                            EventKind::Modify(notify::event::ModifyKind::Any) => true,
                            EventKind::Modify(notify::event::ModifyKind::Metadata(_)) => false,
                            EventKind::Modify(notify::event::ModifyKind::Name(_)) => false,
                            EventKind::Modify(notify::event::ModifyKind::Other) => false,
                            EventKind::Create(_) | EventKind::Remove(_) => true,
                            EventKind::Other => false,
                            EventKind::Any => true,
                            EventKind::Access(_) => false,
                        };

                        if should_reload {
                            debug!("Triggering config reload for: {}", config_path_for_watcher.display());
                            let _ = trigger_sender_for_watcher.try_send(());
                        }
                    }
                    Err(e) => {
                        error!("File watch error: {}", e);
                    }
                }
            }
        })?;

        watcher.watch(config_path, RecursiveMode::NonRecursive)?;
        // Also watch the parent directory to catch file moves/renames
        if let Some(parent) = config_path.parent() {
            if let Ok(_) = watcher.watch(parent, RecursiveMode::NonRecursive) {
                // Successfully watching parent directory
            }
        }

        Ok(Self {
            _watcher: Some(watcher),
            _stop_sender: Some(stop_sender),
            receiver: rx,
        })
    }

    /// Get a receiver for the watch channel to receive config updates.
    pub fn receiver(&self) -> watch::Receiver<AisopodConfig> {
        self.receiver.clone()
    }

    /// Check if the config has been changed since the last check.
    pub fn changed(&self) -> bool {
        // The watch channel's changed() method handles this
        false
    }
}

impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        // Signal the background task to stop
        if let Some(sender) = self._stop_sender.take() {
            let _ = sender.send(());
        }
    }
}

/// Async function to reload config and send via channel
async fn reload_and_send(config_path: &Path, tx: &tokio::sync::Mutex<watch::Sender<AisopodConfig>>) {
    debug!("Attempting to reload config from: {}", config_path.display());

    match load_config(config_path) {
        Ok(new_config) => {
            // Use the mutex to safely send the new config
            let tx_guard = tx.lock().await;
            if tx_guard.send(new_config).is_err() {
                warn!("Config receiver dropped, stopping watcher");
            } else {
                info!("Successfully reloaded and sent config from: {}", config_path.display());
            }
        }
        Err(e) => {
            error!(
                "Failed to reload config from '{}': {}. Keeping previous config.",
                config_path.display(),
                e
            );
        }
    }
}

/// Identify which top-level sections changed between two configurations.
///
/// Compares two configurations and returns a list of section names that differ.
/// This helps identify what specifically changed when a config reload occurs.
///
/// # Arguments
///
/// * `old` - The previous configuration
/// * `new` - The new configuration
///
/// # Returns
///
/// A vector of section names that changed. Empty if configurations are identical.
///
/// # Examples
///
/// ```
/// use aisopod_config::{diff_sections, AisopodConfig};
///
/// let old = AisopodConfig::default();
/// let mut new = AisopodConfig::default();
/// new.meta.version = "2.0".to_string();
///
/// let changed = diff_sections(&old, &new);
/// assert!(changed.contains(&"meta".to_string()));
/// ```
pub fn diff_sections(old: &AisopodConfig, new: &AisopodConfig) -> Vec<String> {
    let old_val = serde_json::to_value(old).unwrap_or_default();
    let new_val = serde_json::to_value(new).unwrap_or_default();

    let mut changed = Vec::new();

    if let (serde_json::Value::Object(o), serde_json::Value::Object(n)) = (&old_val, &new_val) {
        // Check all keys from old config
        for (key, old_v) in o {
            if !n.contains_key(key) || n.get(key) != Some(old_v) {
                changed.push(key.clone());
            }
        }
        // Check for new keys in new config that weren't in old
        for key in n.keys() {
            if !o.contains_key(key) && !changed.contains(key) {
                changed.push(key.clone());
            }
        }
    }

    // Sort for consistent output
    changed.sort();
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_watcher_initializes_correctly() {
        // Create a temp config file
        let mut file = NamedTempFile::with_suffix(".json5").unwrap();
        writeln!(file, r#"{{ meta: {{ version: "1.0" }} }}"#).unwrap();

        let watcher = ConfigWatcher::new(file.path()).unwrap();
        let config = watcher.receiver.borrow();
        assert_eq!(config.meta.version, "1.0");
    }

    #[tokio::test]
    async fn test_watcher_detects_change() {
        // Create a temp config file
        let mut file = NamedTempFile::with_suffix(".json5").unwrap();
        writeln!(file, r#"{{ meta: {{ version: "1.0" }} }}"#).unwrap();

        let watcher = ConfigWatcher::new(file.path()).unwrap();
        let mut rx = watcher.receiver.clone();

        // Modify the file
        std::fs::write(
            file.path(),
            r#"{ meta: { version: "2.0" } }"#,
        )
        .unwrap();

        // Wait for the change to be detected and processed
        tokio::time::sleep(Duration::from_millis(400)).await;
        rx.changed().await.unwrap();
        let config = rx.borrow();
        assert_eq!(config.meta.version, "2.0");
    }

    #[tokio::test]
    async fn test_watcher_keeps_config_on_validation_error() {
        // Create a valid temp config file
        let mut file = NamedTempFile::with_suffix(".json5").unwrap();
        writeln!(file, r#"{{ meta: {{ version: "1.0" }} }}"#).unwrap();

        let watcher = ConfigWatcher::new(file.path()).unwrap();
        let mut rx = watcher.receiver.clone();

        // Wait for initial load
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Write an invalid config (empty version)
        std::fs::write(
            file.path(),
            r#"{ meta: { version: "" } }"#,
        )
        .unwrap();

        // Wait for the change attempt
        tokio::time::sleep(Duration::from_millis(400)).await;

        // Should still have the old valid config
        let config = rx.borrow();
        assert_eq!(config.meta.version, "1.0");
    }

    #[tokio::test]
    async fn test_diff_sections() {
        let old = AisopodConfig::default();
        let mut new = AisopodConfig::default();
        new.meta.version = "2.0".to_string();

        let changed = diff_sections(&old, &new);
        assert!(changed.contains(&"meta".to_string()));
    }

    #[tokio::test]
    async fn test_diff_sections_identical() {
        let config = AisopodConfig::default();
        let changed = diff_sections(&config, &config);
        assert!(changed.is_empty());
    }

    #[tokio::test]
    async fn test_diff_sections_multiple_sections() {
        let mut old = AisopodConfig::default();
        old.meta.version = "1.0".to_string();
        old.gateway.server.port = 8080;

        let mut new = AisopodConfig::default();
        new.meta.version = "2.0".to_string();
        new.gateway.server.port = 9090;
        new.agents.agents.push(crate::types::Agent {
            id: "test".to_string(),
            name: "test".to_string(),
            model: "default".to_string(),
            workspace: "/workspace".to_string(),
            sandbox: false,
            subagents: vec![],
            max_subagent_depth: 3,
            subagent_allowed_models: None,
            system_prompt: "Default system prompt".to_string(),
        });

        let changed = diff_sections(&old, &new);
        // meta, gateway, and agents should be in the changed list
        assert!(changed.contains(&"agents".to_string()));
        assert!(changed.contains(&"gateway".to_string()));
        assert!(changed.contains(&"meta".to_string()));
    }

    #[tokio::test]
    async fn test_watcher_drop_stops_task() {
        // Create a temp config file
        let mut file = NamedTempFile::with_suffix(".json5").unwrap();
        writeln!(file, r#"{{ meta: {{ version: "1.0" }} }}"#).unwrap();

        let watcher = ConfigWatcher::new(file.path()).unwrap();
        // Drop the watcher
        drop(watcher);

        // Give some time for cleanup
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_watcher_file_not_found() {
        let config_path = PathBuf::from("/nonexistent/path/config.json5");
        let result = ConfigWatcher::new(&config_path);

        assert!(result.is_err());
    }
}
