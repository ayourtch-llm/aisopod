use serde::{Deserialize, Serialize};

/// Models configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelFallback {
    /// Primary model
    pub primary: String,
    /// Fallback models in order
    #[serde(default)]
    pub fallbacks: Vec<String>,
}
