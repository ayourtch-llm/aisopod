use serde::{Deserialize, Serialize};

/// Models configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    /// Model definitions
    #[serde(default)]
    pub models: Vec<Model>,
    /// Provider configurations
    #[serde(default)]
    pub providers: Vec<ModelProvider>,
    /// Model fallback configuration
    #[serde(default)]
    pub fallbacks: Vec<ModelFallback>,
}

/// Model definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Model ID
    pub id: String,
    /// Model name
    #[serde(default)]
    pub name: String,
    /// Provider name
    #[serde(default)]
    pub provider: String,
    /// Model capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Model provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    /// Provider name
    pub name: String,
    /// API endpoint
    #[serde(default)]
    pub endpoint: String,
    /// API key reference
    #[serde(default)]
    pub api_key: String,
}

/// Model fallback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFallback {
    /// Primary model
    pub primary: String,
    /// Fallback models in order
    #[serde(default)]
    pub fallbacks: Vec<String>,
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self {
            models: Vec::new(),
            providers: Vec::new(),
            fallbacks: Vec::new(),
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            provider: String::new(),
            capabilities: Vec::new(),
        }
    }
}

impl Default for ModelProvider {
    fn default() -> Self {
        Self {
            name: String::new(),
            endpoint: String::new(),
            api_key: String::new(),
        }
    }
}

impl Default for ModelFallback {
    fn default() -> Self {
        Self {
            primary: String::new(),
            fallbacks: Vec::new(),
        }
    }
}
