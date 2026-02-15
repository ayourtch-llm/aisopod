use serde::{Deserialize, Serialize};

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// API keys for external services
    #[serde(default)]
    pub api_keys: Vec<String>,
    /// Authentication profiles
    #[serde(default)]
    pub profiles: Vec<AuthProfile>,
}

/// Authentication profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfile {
    /// Profile name
    pub name: String,
    /// API key reference
    #[serde(default)]
    pub api_key: String,
    /// Provider type
    #[serde(default)]
    pub provider: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_keys: Vec::new(),
            profiles: Vec::new(),
        }
    }
}

impl Default for AuthProfile {
    fn default() -> Self {
        Self {
            name: String::new(),
            api_key: String::new(),
            provider: String::new(),
        }
    }
}
