# Issue 145: Implement Container Execution with Docker/Podman

## Summary
Implement a `SandboxExecutor` that creates, runs commands inside, and destroys Docker or Podman containers. It shells out to the container CLI to manage containers, mounts workspaces with appropriate permissions, captures stdout/stderr, and enforces timeout and resource limits.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/sandbox/executor.rs` (new)

## Current Behavior
Tool commands (e.g., bash) execute directly on the host with no isolation. There is no container-based execution path.

## Expected Behavior
After this issue is completed:
- A `SandboxExecutor` struct manages container lifecycle using the Docker or Podman CLI.
- `create_container()` creates a container from the configured image with resource limits and workspace mounts.
- `execute()` runs a command inside a running container and captures stdout/stderr.
- `destroy_container()` removes the container and cleans up.
- Resource limits (`--memory`, `--cpus`) are passed to the container runtime.
- Timeout is enforced; long-running commands are killed after the configured duration.
- Containers are always cleaned up, even on error or timeout.

## Impact
This is the core execution engine for sandbox isolation. Without it, agents cannot run tool commands in a secure, isolated environment. It directly enables per-agent sandboxing for production deployments.

## Suggested Implementation

1. **Define supporting types** in `crates/aisopod-tools/src/sandbox/executor.rs`:
   ```rust
   use std::path::Path;
   use std::time::Duration;
   use tokio::process::Command;

   use crate::sandbox::config::{SandboxConfig, SandboxRuntime, WorkspaceAccess};

   #[derive(Debug, Clone)]
   pub struct ContainerId(pub String);

   #[derive(Debug)]
   pub struct ExecutionResult {
       pub exit_code: i32,
       pub stdout: String,
       pub stderr: String,
       pub timed_out: bool,
   }
   ```

2. **Implement `SandboxExecutor`:**
   ```rust
   pub struct SandboxExecutor {
       runtime: SandboxRuntime,
   }

   impl SandboxExecutor {
       pub fn new(runtime: SandboxRuntime) -> Self {
           Self { runtime }
       }

       fn runtime_command(&self) -> &str {
           match self.runtime {
               SandboxRuntime::Docker => "docker",
               SandboxRuntime::Podman => "podman",
           }
       }

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

           // Network access
           if !config.network_access {
               cmd.args(["--network", "none"]);
           }

           cmd.arg(&config.image);
           cmd.arg("sleep").arg("infinity");

           let output = cmd.output().await?;
           let container_id = String::from_utf8_lossy(&output.stdout)
               .trim()
               .to_string();
           Ok(ContainerId(container_id))
       }

       pub async fn execute(
           &self,
           config: &SandboxConfig,
           container_id: &ContainerId,
           command: &str,
           working_dir: &Path,
       ) -> Result<ExecutionResult> {
           let mut cmd = Command::new(self.runtime_command());
           cmd.args(["exec"]);

           // Mount workspace
           match config.workspace_access {
               WorkspaceAccess::ReadOnly => {
                   let mount = format!("{}:/workspace:ro", working_dir.display());
                   cmd.args(["-v", &mount]);
               }
               WorkspaceAccess::ReadWrite => {
                   let mount = format!("{}:/workspace:rw", working_dir.display());
                   cmd.args(["-v", &mount]);
               }
               WorkspaceAccess::None => {}
           }

           cmd.arg(&container_id.0);
           cmd.args(["sh", "-c", command]);

           let result = tokio::time::timeout(
               config.timeout,
               cmd.output(),
           )
           .await;

           match result {
               Ok(Ok(output)) => Ok(ExecutionResult {
                   exit_code: output.status.code().unwrap_or(-1),
                   stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                   stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                   timed_out: false,
               }),
               Ok(Err(e)) => Err(e.into()),
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

       async fn kill_container(&self, id: &ContainerId) -> Result<()> {
           Command::new(self.runtime_command())
               .args(["kill", &id.0])
               .output()
               .await?;
           Ok(())
       }

       pub async fn destroy_container(&self, id: &ContainerId) -> Result<()> {
           Command::new(self.runtime_command())
               .args(["rm", "-f", &id.0])
               .output()
               .await?;
           Ok(())
       }
   }
   ```

3. **Add a convenience method** for one-shot execution (create → execute → destroy):
   ```rust
   impl SandboxExecutor {
       pub async fn run_one_shot(
           &self,
           config: &SandboxConfig,
           command: &str,
           working_dir: &Path,
       ) -> Result<ExecutionResult> {
           let container_id = self.create_container(config).await?;
           // Start the container
           Command::new(self.runtime_command())
               .args(["start", &container_id.0])
               .output()
               .await?;

           let result = self.execute(config, &container_id, command, working_dir).await;

           // Always clean up
           let _ = self.destroy_container(&container_id).await;
           result
       }
   }
   ```

4. **Register the module** in `crates/aisopod-tools/src/sandbox/mod.rs`:
   ```rust
   pub mod config;
   pub mod executor;
   ```

5. **Add integration tests** (require Docker/Podman available):
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[tokio::test]
       #[ignore] // Requires Docker
       async fn test_run_echo_in_container() {
           let executor = SandboxExecutor::new(SandboxRuntime::Docker);
           let config = SandboxConfig {
               enabled: true,
               image: "alpine:latest".to_string(),
               timeout: Duration::from_secs(30),
               ..Default::default()
           };

           let result = executor
               .run_one_shot(&config, "echo hello", Path::new("/tmp"))
               .await
               .unwrap();

           assert_eq!(result.exit_code, 0);
           assert_eq!(result.stdout.trim(), "hello");
           assert!(!result.timed_out);
       }

       #[tokio::test]
       #[ignore] // Requires Docker
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
               .await
               .unwrap();

           assert!(result.timed_out);
       }
   }
   ```

## Dependencies
- Issue 144 (sandbox configuration types)
- Issue 052 (bash tool — execution patterns and `ExecutionResult` conventions)

## Resolution

The `SandboxExecutor` was implemented in `crates/aisopod-tools/src/sandbox/executor.rs` with the following features:

### Implementation Details

1. **Container Management Types**:
   - `ContainerId`: A newtype wrapper around container ID strings
   - `ExecutionResult`: Struct containing exit code, stdout, stderr, and timeout status
   - `SandboxExecutor`: Main executor struct with runtime (Docker/Podman) configuration

2. **Core Methods**:
   - `create_container()`: Creates a container with resource limits (`--memory`, `--cpus`), network isolation (`--network none`), and workspace mounts
   - `start_container()`: Starts a previously created container
   - `execute()`: Runs commands inside containers with timeout enforcement
   - `stop_container()`: Gracefully stops running containers
   - `kill_container()`: Forcefully kills containers (used on timeout)
   - `destroy_container()`: Removes containers after use

3. **Convenience Method**:
   - `run_one_shot()`: Automatically manages the full container lifecycle (create → start → execute → destroy), ensuring cleanup even on error or timeout

4. **Workspace Mounting**:
   - Supports three access modes: `ReadOnly`, `ReadWrite`, and `None`
   - Mounts workspace directory as `/workspace` inside containers

5. **Timeout Handling**:
   - Uses `tokio::time::timeout` for execution timeouts
   - Automatically kills containers on timeout with appropriate error messages

### Files Modified/Created

- **Created**: `crates/aisopod-tools/src/sandbox/executor.rs`
- **Modified**: `crates/aisopod-tools/src/sandbox/mod.rs` - Added executor module export
- **Modified**: `crates/aisopod-tools/src/sandbox/config.rs` - Added `SandboxRuntime` and `WorkspaceAccess` re-exports
- **Modified**: `crates/aisopod-tools/src/lib.rs` - Added exports for new types

### Testing

- All existing tests continue to pass
- Integration tests marked with `#[ignore]` for Docker/Podman
- Unit tests for executor types (name, description, schema)

### Acceptance Criteria Met

- [x] `SandboxExecutor` creates containers using Docker or Podman CLI
- [x] Commands execute inside the container and stdout/stderr are captured
- [x] Workspace is mounted with correct permissions (read-only, read-write, or none)
- [x] Resource limits (`--memory`, `--cpus`) are passed to the container runtime
- [x] Timeout is enforced; long-running commands are killed
- [x] Containers are always cleaned up, even on error or timeout
- [x] Integration tests pass with Docker available (marked `#[ignore]` for CI without Docker)

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
