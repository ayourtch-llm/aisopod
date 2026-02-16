//! Configuration types module
//!
//! This module defines all configuration types for the aisopod application.
//! Each sub-module defines a specific configuration area.

use serde::{Deserialize, Serialize};

mod agents;
mod auth;
mod bindings;
mod channels;
mod env;
mod gateway;
mod meta;
mod memory;
mod models;
mod plugins;
mod session;
mod skills;
mod tools;

pub use agents::Agent;
pub use agents::AgentsConfig;
pub use auth::AuthConfig;
pub use bindings::AgentBinding;
pub use channels::ChannelsConfig;
pub use env::EnvConfig;
pub use gateway::GatewayConfig;
pub use meta::MetaConfig;
pub use memory::MemoryConfig;
pub use models::ModelsConfig;
pub use plugins::PluginsConfig;
pub use session::SessionConfig;
pub use skills::SkillsConfig;
pub use tools::ToolsConfig;

/// Root configuration struct that composes all configuration types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AisopodConfig {
    /// Metadata configuration
    #[serde(default)]
    pub meta: MetaConfig,
    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,
    /// Environment configuration
    #[serde(default)]
    pub env: EnvConfig,
    /// Agents configuration
    #[serde(default)]
    pub agents: AgentsConfig,
    /// Models configuration
    #[serde(default)]
    pub models: ModelsConfig,
    /// Channels configuration
    #[serde(default)]
    pub channels: ChannelsConfig,
    /// Tools configuration
    #[serde(default)]
    pub tools: ToolsConfig,
    /// Skills configuration
    #[serde(default)]
    pub skills: SkillsConfig,
    /// Plugins configuration
    #[serde(default)]
    pub plugins: PluginsConfig,
    /// Session configuration
    #[serde(default)]
    pub session: SessionConfig,
    /// Agent bindings (routing rules)
    #[serde(default)]
    pub bindings: Vec<AgentBinding>,
    /// Memory configuration
    #[serde(default)]
    pub memory: MemoryConfig,
    /// Gateway configuration
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
