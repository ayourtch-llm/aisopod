//! Sandbox executor for container-based tool execution using Docker/Podman
//!
//! This module provides a `SandboxExecutor` that manages container lifecycle
//! for isolated tool execution. It creates containers from configured images,
//! mounts workspaces, executes commands, and ensures cleanup.

use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tokio::process::Command;

use crate::sandbox::config::{SandboxConfig, SandboxRuntime, WorkspaceAccess};
use crate::sandbox::{WorkspaceError, WorkspaceGuard};

/// A unique identifier for a container
#[derive(Debug, Clone)]
pub struct ContainerId(pub String);

/// Result of command execution inside a container
#[derive(Debug)]
pub struct ExecutionResult {
    /// Exit code of the command
    pub exit_code: i32,
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Whether the command timed out
    pub timed_out: bool,
}

/// Executor for sandboxed tool execution using Docker or Podman containers
#[derive(Debug, Clone)]
pub struct SandboxExecutor {
    /// Container runtime to use (Docker or Podman)
    runtime: SandboxRuntime,
}

impl SandboxExecutor {
    /// Creates a new SandboxExecutor with the specified runtime
    pub fn new(runtime: SandboxRuntime) -> Self {
        Self { runtime }
    }

    /// Returns the CLI command for the configured runtime
    fn runtime_command(&self) -> &str {
        match self.runtime {
            SandboxRuntime::Docker => "docker",
            SandboxRuntime::Podman => "podman",
        }
    }

    /// Creates a container from the configured image
    ///
    /// The container is created in a stopped state with resource limits
    /// and workspace mounts applied. Use `start_container()` to start it
    /// or `run_one_shot()` for automatic lifecycle management.
    pub async fn create_container(
        &self,
        config: &SandboxConfig,
    ) -> Result<ContainerId> {
        let mut cmd = Command::new(self.runtime_command());
        cmd.args(["create", "--rm"]);

        // Resource limits
        if let Some(ref mem) = config.memory_limit {
            cmd.args(["--memory", mem]);
        }
        if let Some(cpu) = config.cpu_limit {
            cmd.args(["--cpus", &cpu.to_string()]);
        }

        // Security: Non-root container execution
        // By default, run containers as user 1000:1000 (non-root)
        cmd.args(["--user", &config.user]);

        // Network access
        if !config.network_access {
            cmd.args(["--network", "none"]);
        }

        // Set working directory
        cmd.arg("-w").arg("/workspace");

        // Mount workspace based on access level
        match config.workspace_access {
            WorkspaceAccess::ReadOnly => {
                // We'll mount in execute() since we need the actual path
            }
            WorkspaceAccess::ReadWrite => {
                // We'll mount in execute() since we need the actual path
            }
            WorkspaceAccess::None => {
                // No workspace mount needed
            }
        }

        cmd.arg(&config.image);
        cmd.arg("sleep").arg("infinity");

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to create container: {}",
                stderr.trim()
            ));
        }

        let container_id = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        if container_id.is_empty() {
            return Err(anyhow!("Container creation returned empty ID"));
        }

        Ok(ContainerId(container_id))
    }

    /// Starts a previously created container
    pub async fn start_container(&self, id: &ContainerId) -> Result<()> {
        let output = Command::new(self.runtime_command())
            .args(["start", &id.0])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to start container {}: {}",
                id.0,
                stderr.trim()
            ));
        }

        Ok(())
    }

    /// Executes a command inside a running container
    ///
    /// This method mounts the workspace directory with the appropriate
    /// permissions and runs the specified command inside the container.
    pub async fn execute(
        &self,
        config: &SandboxConfig,
        container_id: &ContainerId,
        command: &str,
        working_dir: &Path,
    ) -> Result<ExecutionResult> {
        let guard =
            WorkspaceGuard::new(working_dir.to_path_buf(), config.workspace_access.clone())?;

        // Validate the working directory path
        let _ = guard.validate_path(working_dir)?;

        let mut cmd = Command::new(self.runtime_command());
        cmd.args(["exec"]);

        // Mount workspace based on guard settings
        if let Some(mount_args) = guard.mount_args() {
            cmd.args(&mount_args);
        }

        cmd.arg(&container_id.0);
        cmd.args(["sh", "-c", command]);

        let result = tokio::time::timeout(config.timeout, cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                let exit_code = output.status.code().unwrap_or(-1);
                Ok(ExecutionResult {
                    exit_code,
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    timed_out: false,
                })
            }
            Ok(Err(e)) => Err(anyhow!("Command execution failed: {}", e)),
            Err(_) => {
                // Kill the container on timeout
                let _ = self.kill_container(container_id).await;
                Ok(ExecutionResult {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: "Execution timed out".to_string(),
                    timed_out: true,
                })
            }
        }
    }

    /// Kills a running container
    async fn kill_container(&self, id: &ContainerId) -> Result<()> {
        let output = Command::new(self.runtime_command())
            .args(["kill", &id.0])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Don't return error if container is already stopped
            if !stderr.contains("is not running") && !stderr.contains("is not found") {
                return Err(anyhow!(
                    "Failed to kill container {}: {}",
                    id.0,
                    stderr.trim()
                ));
            }
        }

        Ok(())
    }

    /// Stops a running container
    pub async fn stop_container(&self, id: &ContainerId) -> Result<()> {
        let output = Command::new(self.runtime_command())
            .args(["stop", &id.0])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Don't return error if container is already stopped
            if !stderr.contains("is not running") && !stderr.contains("is not found") {
                return Err(anyhow!(
                    "Failed to stop container {}: {}",
                    id.0,
                    stderr.trim()
                ));
            }
        }

        Ok(())
    }

    /// Destroys (removes) a container
    pub async fn destroy_container(&self, id: &ContainerId) -> Result<()> {
        let output = Command::new(self.runtime_command())
            .args(["rm", "-f", &id.0])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Don't return error if container doesn't exist
            if !stderr.contains("No such container") {
                return Err(anyhow!(
                    "Failed to remove container {}: {}",
                    id.0,
                    stderr.trim()
                ));
            }
        }

        Ok(())
    }

    /// Runs a command in a container with automatic lifecycle management
    ///
    /// This convenience method creates a container, starts it, executes
    /// the command, and ensures cleanup even on error or timeout.
    pub async fn run_one_shot(
        &self,
        config: &SandboxConfig,
        command: &str,
        working_dir: &Path,
    ) -> Result<ExecutionResult> {
        // Validate workspace access before creating container
        let guard =
            WorkspaceGuard::new(working_dir.to_path_buf(), config.workspace_access.clone())?;
        let _ = guard.validate_path(working_dir)?;

        let container_id = self.create_container(config).await?;

        // Start the container
        if let Err(e) = self.start_container(&container_id).await {
            let _ = self.destroy_container(&container_id).await;
            return Err(e);
        }

        let result = self
            .execute(config, &container_id, command, working_dir)
            .await;

        // Always clean up
        let _ = self.destroy_container(&container_id).await;

        result
    }
}

impl Default for SandboxExecutor {
    fn default() -> Self {
        Self::new(SandboxRuntime::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    #[ignore] // Requires Docker/Podman
    async fn test_run_one_shot_echo() {
        let executor = SandboxExecutor::new(SandboxRuntime::Docker);
        let config = SandboxConfig {
            enabled: true,
            image: "alpine:latest".to_string(),
            timeout: Duration::from_secs(30),
            ..Default::default()
        };

        let result = executor
            .run_one_shot(&config, "echo hello", Path::new("/tmp"))
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout.trim(), "hello");
        assert!(!output.timed_out);
    }

    #[tokio::test]
    #[ignore] // Requires Docker/Podman
    async fn test_timeout_kills_container() {
        let executor = SandboxExecutor::new(SandboxRuntime::Docker);
        let config = SandboxConfig {
            enabled: true,
            image: "alpine:latest".to_string(),
            timeout: Duration::from_secs(2),
            ..Default::default()
        };

        let result = executor
            .run_one_shot(&config, "sleep 60", Path::new("/tmp"))
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.timed_out);
        assert_eq!(output.exit_code, -1);
    }

    #[tokio::test]
    #[ignore] // Requires Docker/Podman
    async fn test_workspace_read_write() {
        let executor = SandboxExecutor::new(SandboxRuntime::Docker);
        let config = SandboxConfig {
            enabled: true,
            image: "alpine:latest".to_string(),
            workspace_access: WorkspaceAccess::ReadWrite,
            timeout: Duration::from_secs(30),
            ..Default::default()
        };

        // Create a temp directory
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("test_aisopod.txt");

        let result = executor
            .run_one_shot(
                &config,
                &format!("echo test > {}", test_file.display()),
                &temp_dir,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().exit_code, 0);

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[tokio::test]
    #[ignore] // Requires Docker/Podman
    async fn test_container_id_is_unique() {
        let executor = SandboxExecutor::new(SandboxRuntime::Docker);
        let config = SandboxConfig {
            enabled: true,
            image: "alpine:latest".to_string(),
            timeout: Duration::from_secs(30),
            ..Default::default()
        };

        let id1 = executor.create_container(&config).await.unwrap();
        let id2 = executor.create_container(&config).await.unwrap();

        assert_ne!(id1.0, id2.0);

        let _ = executor.destroy_container(&id1).await;
        let _ = executor.destroy_container(&id2).await;
    }

    #[test]
    fn test_sandbox_config_default_user() {
        let config = SandboxConfig::default();
        assert_eq!(config.user, "1000:1000");
    }

    #[test]
    fn test_sandbox_config_custom_user() {
        let config = SandboxConfig {
            enabled: true,
            runtime: SandboxRuntime::Docker,
            image: "alpine:latest".to_string(),
            workspace_access: WorkspaceAccess::ReadOnly,
            network_access: true,
            memory_limit: None,
            cpu_limit: None,
            user: "1001:1001".to_string(),
            timeout: Duration::from_secs(60),
        };
        assert_eq!(config.user, "1001:1001");
    }
}
