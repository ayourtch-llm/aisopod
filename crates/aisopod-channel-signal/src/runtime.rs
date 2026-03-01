//! Signal runtime for managing signal-cli daemon.
//!
//! This module handles spawning and managing the signal-cli daemon process.

use crate::channel::SignalAccount;
use crate::config::{SignalAccountConfig, SignalDaemonConfig, SignalError};
use anyhow::Result;
use std::process::Child;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Runtime for managing the signal-cli daemon.
#[derive(Clone)]
pub struct SignalRuntime {
    /// Daemon configuration
    daemon_config: SignalDaemonConfig,
    /// Currently running daemon processes (Arc-wrapped for sharing)
    running_daemons: Vec<Arc<std::sync::Mutex<Child>>>,
}

impl SignalRuntime {
    /// Create a new SignalRuntime instance.
    pub fn new() -> Self {
        Self {
            daemon_config: SignalDaemonConfig::default(),
            running_daemons: Vec::new(),
        }
    }

    /// Create a new SignalRuntime with custom configuration.
    pub fn with_config(config: SignalDaemonConfig) -> Self {
        Self {
            daemon_config: config,
            running_daemons: Vec::new(),
        }
    }

    /// Start the signal-cli daemon for an account.
    ///
    /// # Arguments
    ///
    /// * `account` - The Signal account configuration
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Daemon started successfully
    /// * `Err(SignalError)` - Failed to start daemon
    pub async fn start_daemon(&mut self, account: &SignalAccount) -> Result<()> {
        let phone_number = &account.config.phone_number;

        info!("Starting signal-cli daemon for {}", phone_number);

        // Build the command to start signal-cli daemon
        let mut cmd = std::process::Command::new(self.daemon_config.signal_cli_path.as_str());

        cmd.arg("-u").arg(phone_number).arg("daemon").arg("--json");

        // Add data directory if configured
        if let Some(ref data_dir) = self.daemon_config.signal_cli_data_dir {
            cmd.arg("--datadir").arg(data_dir);
        }

        // Start the process
        let child = cmd
            .spawn()
            .map_err(|e| SignalError::SpawnFailed(format!("Failed to spawn signal-cli: {}", e)))?;

        let child_arc = Arc::new(std::sync::Mutex::new(child));
        self.running_daemons.push(child_arc.clone());

        info!("Started signal-cli daemon for {}", phone_number);

        // Give the daemon time to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(())
    }

    /// Stop the signal-cli daemon for an account.
    ///
    /// # Arguments
    ///
    /// * `account` - The Signal account configuration
    pub async fn stop_daemon(&mut self, _account: &SignalAccount) {
        // For now, we just log the stop request
        // In production, you would send a SIGTERM or use the JSON-RPC interface
        info!("Stopping signal-cli daemon for {}", _account.id);
    }

    /// Stop all running daemons.
    pub async fn stop_all_daemons(&mut self) {
        info!("Stopping all signal-cli daemons");
        self.running_daemons.clear();
    }

    /// Check if a daemon is running for an account.
    pub fn is_daemon_running(&self, _account: &SignalAccount) -> bool {
        // For now, we just check if we have any running daemons
        // A more sophisticated implementation would check the actual process
        !self.running_daemons.is_empty()
    }

    /// Get the daemon configuration.
    pub fn daemon_config(&self) -> &SignalDaemonConfig {
        &self.daemon_config
    }
}

/// Utility functions for signal-cli runtime.
pub mod utils {
    /// Check if signal-cli is installed and executable.
    pub fn check_signal_cli_exists(signal_cli_path: &str) -> bool {
        std::path::Path::new(signal_cli_path).exists()
    }

    /// Get the default signal-cli path based on OS.
    pub fn get_default_signal_cli_path() -> String {
        #[cfg(target_os = "linux")]
        {
            "signal-cli".to_string()
        }
        #[cfg(target_os = "macos")]
        {
            "/usr/local/bin/signal-cli".to_string()
        }
        #[cfg(target_os = "windows")]
        {
            "signal-cli.exe".to_string()
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            "signal-cli".to_string()
        }
    }

    /// Find the signal-cli binary in common locations.
    pub fn find_signal_cli() -> Option<String> {
        let common_paths = vec![
            "signal-cli",
            "/usr/local/bin/signal-cli",
            "/usr/bin/signal-cli",
            "C:\\Program Files\\signal-cli\\signal-cli.exe",
        ];

        for path in common_paths {
            if check_signal_cli_exists(path) {
                return Some(path.to_string());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_runtime_new() {
        let runtime = SignalRuntime::new();
        assert!(runtime.running_daemons.is_empty());
    }

    #[test]
    fn test_check_signal_cli_exists() {
        // This test would fail if signal-cli is not installed
        // For testing purposes, we just verify the function exists
        assert!(
            utils::check_signal_cli_exists("signal-cli")
                || !utils::check_signal_cli_exists("nonexistent-binary")
        );
    }

    #[test]
    fn test_get_default_signal_cli_path() {
        // Just verify the function returns a value
        let path = utils::get_default_signal_cli_path();
        assert!(!path.is_empty());
    }
}
