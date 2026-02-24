use humantime_serde::deserialize as humantime_deserialize;
use humantime_serde::serialize as humantime_serialize;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Sandbox runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxRuntime {
    /// Docker container runtime
    Docker,
    /// Podman container runtime
    Podman,
}

impl Default for SandboxRuntime {
    fn default() -> Self {
        Self::Docker
    }
}

/// Workspace access configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceAccess {
    /// No workspace access
    None,
    /// Read-only workspace access
    ReadOnly,
    /// Read-write workspace access
    ReadWrite,
}

impl Default for WorkspaceAccess {
    fn default() -> Self {
        Self::ReadOnly
    }
}

/// Sandbox configuration for agent tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Whether sandbox execution is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Container runtime to use (Docker or Podman)
    #[serde(default)]
    pub runtime: SandboxRuntime,

    /// Container image to use for execution
    #[serde(default = "default_image")]
    pub image: String,

    /// Workspace access permissions
    #[serde(default)]
    pub workspace_access: WorkspaceAccess,

    /// Whether network access is allowed
    #[serde(default = "default_true")]
    pub network_access: bool,

    /// Memory limit (e.g., "256m", "1g")
    pub memory_limit: Option<String>,

    /// CPU limit as a fraction (e.g., 0.5 for 50% of one CPU core)
    pub cpu_limit: Option<f64>,

    /// Execution timeout
    #[serde(
        default = "default_timeout",
        serialize_with = "humantime_serialize",
        deserialize_with = "humantime_deserialize"
    )]
    pub timeout: Duration,
}

fn default_image() -> String {
    "ubuntu:latest".to_string()
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> Duration {
    Duration::from_secs(300)
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            runtime: SandboxRuntime::default(),
            image: default_image(),
            workspace_access: WorkspaceAccess::default(),
            network_access: true,
            memory_limit: None,
            cpu_limit: None,
            timeout: default_timeout(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.runtime, SandboxRuntime::Docker);
        assert_eq!(config.workspace_access, WorkspaceAccess::ReadOnly);
        assert_eq!(config.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_sandbox_config_deserialize() {
        let toml = r#"
            enabled = true
            runtime = "podman"
            image = "alpine:latest"
            workspace_access = "none"
            network_access = false
            memory_limit = "256m"
            cpu_limit = 0.5
            timeout = "60s"
        "#;
        let config: SandboxConfig = toml::from_str(toml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.runtime, SandboxRuntime::Podman);
        assert_eq!(config.workspace_access, WorkspaceAccess::None);
    }

    #[test]
    fn test_sandbox_config_serialize() {
        let config = SandboxConfig {
            enabled: true,
            runtime: SandboxRuntime::Podman,
            image: "alpine:latest".to_string(),
            workspace_access: WorkspaceAccess::None,
            network_access: false,
            memory_limit: Some("256m".to_string()),
            cpu_limit: Some(0.5),
            timeout: Duration::from_secs(60),
        };
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("enabled = true"));
        assert!(toml_str.contains("runtime = \"podman\""));
    }
}
