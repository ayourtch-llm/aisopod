use serde::{Deserialize, Serialize};

/// Environment configuration for variable mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConfig {
    /// Environment variable mappings
    #[serde(default)]
    pub mappings: Vec<EnvMapping>,
}

/// Environment variable mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }
}

impl Default for EnvMapping {
    fn default() -> Self {
        Self {
            name: String::new(),
            default: None,
            required: false,
        }
    }
}
