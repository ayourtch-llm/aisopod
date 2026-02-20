use serde::{Deserialize, Serialize};

/// Agents configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    /// List of agents
    #[serde(default)]
    pub agents: Vec<Agent>,
    /// Default agent configuration
    #[serde(default)]
    pub default: AgentDefaults,
}

/// Agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Agent ID
    pub id: String,
    /// Agent name
    #[serde(default)]
    pub name: String,
    /// Model to use
    #[serde(default)]
    pub model: String,
    /// Workspace path
    #[serde(default)]
    pub workspace: String,
    /// Sandbox configuration
    #[serde(default)]
    pub sandbox: bool,
    /// Subagents
    #[serde(default)]
    pub subagents: Vec<String>,
    /// System prompt for this agent
    #[serde(default)]
    pub system_prompt: String,
    /// Maximum depth for subagent spawning (default: 3)
    #[serde(default = "default_max_subagent_depth")]
    pub max_subagent_depth: usize,
    /// Optional allowlist of models that subagents can use
    #[serde(default)]
    pub subagent_allowed_models: Option<Vec<String>>,
}

/// Default maximum depth for subagent spawning
fn default_max_subagent_depth() -> usize {
    3
}

/// Default agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefaults {
    /// Default model
    #[serde(default)]
    pub model: String,
    /// Default workspace
    #[serde(default)]
    pub workspace: String,
    /// Default sandbox setting
    #[serde(default)]
    pub sandbox: bool,
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            agents: Vec::new(),
            default: AgentDefaults::default(),
        }
    }
}

impl Default for AgentDefaults {
    fn default() -> Self {
        Self {
            model: String::new(),
            workspace: String::new(),
            sandbox: false,
        }
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            model: String::new(),
            workspace: String::new(),
            sandbox: false,
            subagents: Vec::new(),
            system_prompt: String::new(),
            max_subagent_depth: default_max_subagent_depth(),
            subagent_allowed_models: None,
        }
    }
}
