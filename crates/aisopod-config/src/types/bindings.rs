use serde::{Deserialize, Serialize};

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
}
