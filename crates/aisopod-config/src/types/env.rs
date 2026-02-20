use serde::{Deserialize, Serialize};

/// Environment configuration for variable mappings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvConfig {
    /// Environment variable mappings
    #[serde(default)]
    pub mappings: Vec<EnvMapping>,
}

/// Environment variable mapping
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvMapping {
    /// Environment variable name
    pub name: String,
    /// Default value if not set
    #[serde(default)]
    pub default: Option<String>,
    /// Whether the variable is required
    #[serde(default)]
    pub required: bool,
}
