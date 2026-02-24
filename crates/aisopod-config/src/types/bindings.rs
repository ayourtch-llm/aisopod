use serde::{Deserialize, Serialize};

use crate::types::SandboxConfig;

/// Agent binding for routing agents to channels
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentBinding {
    /// Agent ID
    pub agent_id: String,
    /// Channel IDs to bind to
    pub channels: Vec<String>,
    /// Priority for this binding
    #[serde(default)]
    pub priority: u32,
    /// Sandbox configuration for this agent's tool execution
    #[serde(default)]
    pub sandbox: Option<SandboxConfig>,
}
