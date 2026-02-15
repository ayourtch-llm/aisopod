# Issue 016: Define Core Configuration Types with Serde Derive

## Summary
Create Rust structs for `AisopodConfig` and all sub-config types (`MetaConfig`, `AuthConfig`, `EnvConfig`, `AgentsConfig`, `ModelsConfig`, `ChannelsConfig`, `ToolsConfig`, `SkillsConfig`, `PluginsConfig`, `SessionConfig`, `AgentBinding`, `MemoryConfig`, `GatewayConfig`). All structs derive `Serialize`, `Deserialize`, `Debug`, and `Clone`, and implement `Default` with sensible defaults.

## Location
- Crate: `aisopod-config`
- File: `crates/aisopod-config/src/types.rs`, `crates/aisopod-config/src/types/`

## Current Behavior
The `aisopod-config` crate exists as a scaffold (from Issue 003) with no configuration types defined. There are no Rust structs representing the application configuration.

## Expected Behavior
- A `types` module exists with `AisopodConfig` as the root configuration struct
- Each sub-config type is defined in its own sub-module under `types/`
- All structs derive `Serialize`, `Deserialize`, `Debug`, `Clone`
- All structs implement `Default` with sensible defaults
- The root struct composes all sub-config types as fields
- Types are re-exported from `crates/aisopod-config/src/lib.rs`

## Impact
These types form the foundation of the entire configuration system. Every other config-related feature (parsing, validation, env substitution, includes) depends on these structs being defined correctly. Other crates will use these types to access their configuration sections.

## Suggested Implementation
1. Create the directory `crates/aisopod-config/src/types/`.
2. Create `crates/aisopod-config/src/types/mod.rs` that declares and re-exports all sub-modules.
3. Create a sub-module file for each config type. For example, `crates/aisopod-config/src/types/meta.rs`:
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct MetaConfig {
       /// Configuration schema version
       #[serde(default = "default_version")]
       pub version: String,
   }

   fn default_version() -> String {
       "1.0".to_string()
   }

   impl Default for MetaConfig {
       fn default() -> Self {
           Self {
               version: default_version(),
           }
       }
   }
   ```
4. Create similar files for each sub-config type:
   - `auth.rs` — `AuthConfig` with fields for API keys, auth profiles
   - `env.rs` — `EnvConfig` with environment variable mappings
   - `agents.rs` — `AgentsConfig` with agent list and defaults (model, workspace, sandbox, subagents)
   - `models.rs` — `ModelsConfig` with model definitions, providers, fallbacks
   - `channels.rs` — `ChannelsConfig` with per-channel configuration
   - `tools.rs` — `ToolsConfig` with bash, exec, file system tool settings
   - `skills.rs` — `SkillsConfig` with skill modules
   - `plugins.rs` — `PluginsConfig` with plugin registry
   - `session.rs` — `SessionConfig` with message handling and compaction settings
   - `bindings.rs` — `AgentBinding` with agent-to-channel routing rules
   - `memory.rs` — `MemoryConfig` with QMD memory system settings
   - `gateway.rs` — `GatewayConfig` with HTTP server, bind address, port, TLS settings
5. Create the root config struct in `crates/aisopod-config/src/types/mod.rs`:
   ```rust
   use serde::{Deserialize, Serialize};

   mod meta;
   mod auth;
   mod env;
   mod agents;
   mod models;
   mod channels;
   mod tools;
   mod skills;
   mod plugins;
   mod session;
   mod bindings;
   mod memory;
   mod gateway;

   pub use meta::MetaConfig;
   pub use auth::AuthConfig;
   pub use env::EnvConfig;
   pub use agents::AgentsConfig;
   pub use models::ModelsConfig;
   pub use channels::ChannelsConfig;
   pub use tools::ToolsConfig;
   pub use skills::SkillsConfig;
   pub use plugins::PluginsConfig;
   pub use session::SessionConfig;
   pub use bindings::AgentBinding;
   pub use memory::MemoryConfig;
   pub use gateway::GatewayConfig;

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct AisopodConfig {
       #[serde(default)]
       pub meta: MetaConfig,
       #[serde(default)]
       pub auth: AuthConfig,
       #[serde(default)]
       pub env: EnvConfig,
       #[serde(default)]
       pub agents: AgentsConfig,
       #[serde(default)]
       pub models: ModelsConfig,
       #[serde(default)]
       pub channels: ChannelsConfig,
       #[serde(default)]
       pub tools: ToolsConfig,
       #[serde(default)]
       pub skills: SkillsConfig,
       #[serde(default)]
       pub plugins: PluginsConfig,
       #[serde(default)]
       pub session: SessionConfig,
       #[serde(default)]
       pub bindings: Vec<AgentBinding>,
       #[serde(default)]
       pub memory: MemoryConfig,
       #[serde(default)]
       pub gateway: GatewayConfig,
   }

   impl Default for AisopodConfig {
       fn default() -> Self {
           Self {
               meta: MetaConfig::default(),
               auth: AuthConfig::default(),
               env: EnvConfig::default(),
               agents: AgentsConfig::default(),
               models: ModelsConfig::default(),
               channels: ChannelsConfig::default(),
               tools: ToolsConfig::default(),
               skills: SkillsConfig::default(),
               plugins: PluginsConfig::default(),
               session: SessionConfig::default(),
               bindings: Vec::new(),
               memory: MemoryConfig::default(),
               gateway: GatewayConfig::default(),
           }
       }
   }
   ```
6. Update `crates/aisopod-config/src/lib.rs` to declare and re-export the types module:
   ```rust
   pub mod types;
   pub use types::AisopodConfig;
   ```
7. Run `cargo check -p aisopod-config` to verify everything compiles.
8. Run a quick test to verify `AisopodConfig::default()` produces a valid instance.

## Dependencies
003

## Acceptance Criteria
- [x] `AisopodConfig` and all 13 sub-config types are defined with `Serialize`, `Deserialize`, `Debug`, `Clone`
- [x] All types implement `Default` with sensible default values
- [x] Types are organized in sub-modules under `crates/aisopod-config/src/types/`
- [x] All types are re-exported from the crate root
- [x] `cargo check -p aisopod-config` succeeds without errors
- [x] `AisopodConfig::default()` produces a valid configuration instance

## Resolution
Implementation completed successfully:

- Created directory structure: `crates/aisopod-config/src/types/`
- Created 14 sub-module files:
  - `meta.rs` — `MetaConfig` with version field
  - `auth.rs` — `AuthConfig` with API keys and profiles
  - `env.rs` — `EnvConfig` with environment variable mappings
  - `agents.rs` — `AgentsConfig` with agent list and defaults
  - `models.rs` — `ModelsConfig` with model definitions and providers
  - `channels.rs` — `ChannelsConfig` with channel definitions
  - `tools.rs` — `ToolsConfig` with bash, exec, and filesystem settings
  - `skills.rs` — `SkillsConfig` with skill modules
  - `plugins.rs` — `PluginsConfig` with plugin registry
  - `session.rs` — `SessionConfig` with message and compaction settings
  - `bindings.rs` — `AgentBinding` with agent-to-channel routing
  - `memory.rs` — `MemoryConfig` with QMD memory settings
  - `gateway.rs` — `GatewayConfig` with HTTP server settings
  - `mod.rs` — Root `AisopodConfig` struct composing all sub-configs
- Updated `lib.rs` to re-export `types` module and `AisopodConfig`
- All structs derive `Serialize`, `Deserialize`, `Debug`, `Clone` and implement `Default`
- `cargo build` and `cargo test` passed successfully at top level

---
*Created: 2026-02-15*
*Resolved: 2026-02-16*
