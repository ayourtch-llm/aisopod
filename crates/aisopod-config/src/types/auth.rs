use crate::sensitive::Sensitive;
use serde::{Deserialize, Serialize};

/// Authentication mode for gateway
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    /// Token-based authentication (Bearer token)
    Token,
    /// Password-based authentication (HTTP Basic)
    Password,
    /// No authentication required
    #[default]
    None,
}

/// Authentication configuration for the gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// API keys for external services
    #[serde(default)]
    pub api_keys: Vec<String>,
    /// Authentication profiles
    #[serde(default)]
    pub profiles: Vec<AuthProfile>,
    /// Gateway authentication mode
    #[serde(default)]
    pub gateway_mode: AuthMode,
    /// Token-based credentials (key -> role + scopes)
    #[serde(default)]
    pub tokens: Vec<TokenCredential>,
    /// Password-based credentials (username -> role + scopes)
    #[serde(default)]
    pub passwords: Vec<PasswordCredential>,
}

/// Token credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCredential {
    /// The token value
    pub token: String,
    /// Role associated with this token
    pub role: String,
    /// Scopes granted to this token
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// Password credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordCredential {
    /// Username
    pub username: String,
    /// Password
    pub password: Sensitive<String>,
    /// Role associated with this user
    pub role: String,
    /// Scopes granted to this user
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// Authentication profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfile {
    /// Profile name
    pub name: String,
    /// API key reference
    #[serde(default)]
    pub api_key: Sensitive<String>,
    /// Provider type
    #[serde(default)]
    pub provider: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_keys: Vec::new(),
            profiles: Vec::new(),
            gateway_mode: AuthMode::None,
            tokens: Vec::new(),
            passwords: Vec::new(),
        }
    }
}

impl Default for TokenCredential {
    fn default() -> Self {
        Self {
            token: String::new(),
            role: String::new(),
            scopes: Vec::new(),
        }
    }
}

impl Default for PasswordCredential {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: Sensitive::default(),
            role: String::new(),
            scopes: Vec::new(),
        }
    }
}

impl Default for AuthProfile {
    fn default() -> Self {
        Self {
            name: String::new(),
            api_key: Sensitive::default(),
            provider: String::new(),
        }
    }
}
