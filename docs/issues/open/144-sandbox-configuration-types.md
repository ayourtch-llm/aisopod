# Issue 144: Define Sandbox Configuration Types

## Summary
Define the `SandboxConfig` struct and supporting enums (`SandboxRuntime`, `WorkspaceAccess`) that describe per-agent sandbox isolation settings. Extend the existing configuration crate and expose these types from `aisopod-tools` as well.

## Location
- Crate: `aisopod-config` (extend), `aisopod-tools`
- File: `crates/aisopod-config/src/types.rs` (extend), `crates/aisopod-tools/src/sandbox/config.rs` (new)

## Current Behavior
No sandbox-related configuration types exist. Agents have no way to declare container isolation, resource limits, or workspace access settings.

## Expected Behavior
After this issue is completed:
- A `SandboxConfig` struct is defined with fields: `enabled` (bool), `runtime` (SandboxRuntime), `image` (String), `workspace_access` (WorkspaceAccess), `network_access` (bool), `memory_limit` (Option<String>), `cpu_limit` (Option<f64>), `timeout` (Duration).
- A `SandboxRuntime` enum with variants `Docker` and `Podman`.
- A `WorkspaceAccess` enum with variants `None`, `ReadOnly`, and `ReadWrite`.
- All types derive `Serialize`, `Deserialize`, `Debug`, `Clone` and implement `Default`.
- `SandboxConfig` integrates into `AgentBinding` or `ToolsConfig` so each agent can opt into sandbox execution.

## Impact
Every other sandbox feature (container execution, workspace controls, resource limits) depends on these types. They are the foundation of the entire sandbox subsystem.

## Suggested Implementation

1. **Add the enums** in `crates/aisopod-config/src/types.rs` (or a new `sandbox.rs` module):
   ```rust
   use serde::{Deserialize, Serialize};
   use std::time::Duration;

   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
   #[serde(rename_all = "lowercase")]
   pub enum SandboxRuntime {
       Docker,
       Podman,
   }

   impl Default for SandboxRuntime {
       fn default() -> Self {
           Self::Docker
       }
   }

   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
   #[serde(rename_all = "lowercase")]
   pub enum WorkspaceAccess {
       None,
       ReadOnly,
       ReadWrite,
   }

   impl Default for WorkspaceAccess {
       fn default() -> Self {
           Self::ReadOnly
       }
   }
   ```

2. **Add the `SandboxConfig` struct:**
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct SandboxConfig {
       #[serde(default)]
       pub enabled: bool,

       #[serde(default)]
       pub runtime: SandboxRuntime,

       #[serde(default = "default_image")]
       pub image: String,

       #[serde(default)]
       pub workspace_access: WorkspaceAccess,

       #[serde(default = "default_true")]
       pub network_access: bool,

       pub memory_limit: Option<String>,

       pub cpu_limit: Option<f64>,

       #[serde(
           default = "default_timeout",
           with = "humantime_serde"
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
   ```

3. **Wire into the config hierarchy** by adding an `Option<SandboxConfig>` field to `AgentBinding` or `ToolsConfig`:
   ```rust
   // In AgentBinding or ToolsConfig
   #[serde(default)]
   pub sandbox: Option<SandboxConfig>,
   ```

4. **Re-export from `aisopod-tools`** so the tools crate can use these types directly:
   ```rust
   // crates/aisopod-tools/src/sandbox/mod.rs
   pub mod config;
   pub use aisopod_config::{SandboxConfig, SandboxRuntime, WorkspaceAccess};
   ```

5. **Add unit tests:**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_sandbox_config_default() {
           let config = SandboxConfig::default();
           assert!(!config.enabled);
           assert_eq!(config.runtime, SandboxRuntime::Docker);
           assert_eq!(config.workspace_access, WorkspaceAccess::ReadOnly);
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
   }
   ```

## Dependencies
- Issue 016 (core configuration types)

## Acceptance Criteria
- [ ] `SandboxConfig`, `SandboxRuntime`, and `WorkspaceAccess` types compile and derive all required traits
- [ ] `SandboxConfig` implements `Default` with sensible values (disabled, Docker, ReadOnly, 5-minute timeout)
- [ ] Types serialize/deserialize correctly with TOML
- [ ] `SandboxConfig` integrates into the existing config hierarchy (`AgentBinding` or `ToolsConfig`)
- [ ] Types are re-exported from `aisopod-tools` for use by the sandbox executor
- [ ] Unit tests cover default construction and deserialization

---
*Created: 2026-02-15*
