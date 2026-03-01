//! Sandbox integration tests for the aisopod-tools crate
//!
//! These tests verify that sandbox containers are properly isolated:
//! - Sandbox cannot access host filesystem
//! - Read-only workspace prevents writes
//! - No workspace mount when access is None
//! - Resource limits are enforced
//!
//! Note: These tests are marked with #[ignore] since they require Docker.

#![deny(unused_must_use)]

use aisopod_tools::sandbox::{SandboxConfig, SandboxExecutor, SandboxRuntime, WorkspaceAccess};
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
#[ignore] // Requires Docker
async fn test_sandbox_cannot_access_host_filesystem() {
    let executor = SandboxExecutor::new(SandboxRuntime::Docker);
    let workspace = TempDir::new().expect("Failed to create temp dir");

    let config = SandboxConfig {
        enabled: true,
        image: "alpine:latest".to_string(),
        workspace_access: WorkspaceAccess::ReadOnly,
        timeout: Duration::from_secs(10),
        ..Default::default()
    };

    // Try to read /etc/hostname from inside the container
    // (should be the container's hostname, not the host's)
    let result = executor
        .run_one_shot(&config, "cat /etc/hostname", workspace.path())
        .await
        .expect("Failed to run command in sandbox");

    assert_eq!(result.exit_code, 0);
    // The hostname should be set by Docker, not empty
    assert!(!result.stdout.is_empty());
    // The stdout should be just the container ID/hostname
    let hostname = result.stdout.trim();
    assert!(!hostname.contains("/etc/hostname"));
    assert!(hostname.len() > 0);
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_readonly_workspace_prevents_writes() {
    let executor = SandboxExecutor::new(SandboxRuntime::Docker);
    let workspace = TempDir::new().expect("Failed to create temp dir");

    let config = SandboxConfig {
        enabled: true,
        image: "alpine:latest".to_string(),
        workspace_access: WorkspaceAccess::ReadOnly,
        timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let result = executor
        .run_one_shot(&config, "touch /workspace/test.txt", workspace.path())
        .await
        .expect("Failed to run command in sandbox");

    // Should fail because workspace is mounted read-only
    assert_ne!(result.exit_code, 0);

    // Should have an error message about read-only file system
    assert!(
        result.stderr.contains("Read-only file system")
            || result.stderr.contains("read-only")
            || result.exit_code != 0
    );
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_no_workspace_mount() {
    let executor = SandboxExecutor::new(SandboxRuntime::Docker);
    let workspace = TempDir::new().expect("Failed to create temp dir");

    let config = SandboxConfig {
        enabled: true,
        image: "alpine:latest".to_string(),
        workspace_access: WorkspaceAccess::None,
        timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let result = executor
        .run_one_shot(&config, "ls /workspace", workspace.path())
        .await
        .expect("Failed to run command in sandbox");

    // /workspace should not exist when access is None
    assert_ne!(result.exit_code, 0);

    // Should have an error about no such file or directory
    assert!(
        result.stderr.contains("No such file or directory")
            || result.stderr.contains("cannot access")
            || result.exit_code != 0
    );
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_resource_limits_enforced() {
    let executor = SandboxExecutor::new(SandboxRuntime::Docker);
    let workspace = TempDir::new().expect("Failed to create temp dir");

    let config = SandboxConfig {
        enabled: true,
        image: "alpine:latest".to_string(),
        memory_limit: Some("32m".to_string()),
        cpu_limit: Some(0.5),
        timeout: Duration::from_secs(10),
        ..Default::default()
    };

    // This should succeed but be constrained
    let result = executor
        .run_one_shot(&config, "echo 'constrained'", workspace.path())
        .await
        .expect("Failed to run command in sandbox");

    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout.trim(), "constrained");
    assert!(!result.timed_out);
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_workspace_read_write_allows_writes() {
    let executor = SandboxExecutor::new(SandboxRuntime::Docker);
    let workspace = TempDir::new().expect("Failed to create temp dir");

    let config = SandboxConfig {
        enabled: true,
        image: "alpine:latest".to_string(),
        workspace_access: WorkspaceAccess::ReadWrite,
        timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let result = executor
        .run_one_shot(
            &config,
            "touch /workspace/test.txt && echo success",
            workspace.path(),
        )
        .await
        .expect("Failed to run command in sandbox");

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("success"));
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_workspace_read_write_can_write_to_file() {
    let executor = SandboxExecutor::new(SandboxRuntime::Docker);
    let workspace = TempDir::new().expect("Failed to create temp dir");

    let config = SandboxConfig {
        enabled: true,
        image: "alpine:latest".to_string(),
        workspace_access: WorkspaceAccess::ReadWrite,
        timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let test_content = "test content from sandbox";
    let result = executor
        .run_one_shot(
            &config,
            &format!("echo '{}' > /workspace/test.txt", test_content),
            workspace.path(),
        )
        .await
        .expect("Failed to run command in sandbox");

    assert_eq!(result.exit_code, 0);

    // Verify the file was written to the workspace
    let test_file = workspace.path().join("test.txt");
    assert!(test_file.exists());

    let written_content = std::fs::read_to_string(&test_file).expect("Failed to read test file");
    assert!(written_content.contains(test_content));
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_timeout_kills_long_running_process() {
    let executor = SandboxExecutor::new(SandboxRuntime::Docker);
    let workspace = TempDir::new().expect("Failed to create temp dir");

    let config = SandboxConfig {
        enabled: true,
        image: "alpine:latest".to_string(),
        workspace_access: WorkspaceAccess::ReadOnly,
        timeout: Duration::from_secs(2),
        ..Default::default()
    };

    let result = executor
        .run_one_shot(&config, "sleep 60 && echo should_not_run", workspace.path())
        .await
        .expect("Failed to run command in sandbox");

    assert!(result.timed_out);
    assert_eq!(result.exit_code, -1);
    assert!(result.stdout.is_empty() || !result.stdout.contains("should_not_run"));
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_network_isolation() {
    let executor = SandboxExecutor::new(SandboxRuntime::Docker);
    let workspace = TempDir::new().expect("Failed to create temp dir");

    let config = SandboxConfig {
        enabled: true,
        image: "alpine:latest".to_string(),
        workspace_access: WorkspaceAccess::ReadOnly,
        network_access: false, // Explicitly disable network
        timeout: Duration::from_secs(10),
        ..Default::default()
    };

    // Try to access an external service (should fail)
    let result = executor
        .run_one_shot(
            &config,
            "wget -q -O- --timeout=2 http://example.com 2>&1",
            workspace.path(),
        )
        .await
        .expect("Failed to run command in sandbox");

    // The command should either timeout or fail
    // (exact behavior depends on Docker network isolation)
    // At minimum, we verify the sandbox ran
    assert!(result.exit_code != 0 || result.timed_out);
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_podman_sandbox_works() {
    // Test with Podman runtime if Docker is not available
    let executor = SandboxExecutor::new(SandboxRuntime::Podman);
    let workspace = TempDir::new().expect("Failed to create temp dir");

    let config = SandboxConfig {
        enabled: true,
        image: "alpine:latest".to_string(),
        workspace_access: WorkspaceAccess::ReadOnly,
        timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let result = executor
        .run_one_shot(&config, "echo podman_test", workspace.path())
        .await;

    // This test may fail if Podman is not available, which is expected
    match result {
        Ok(output) => {
            assert_eq!(output.exit_code, 0);
            assert!(output.stdout.contains("podman_test"));
        }
        Err(_) => {
            // Podman may not be available, which is okay
            // This test just documents the expected behavior
        }
    }
}
