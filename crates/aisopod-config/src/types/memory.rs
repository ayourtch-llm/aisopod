use serde::{Deserialize, Serialize};

/// Memory configuration for QMD memory system
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    /// Memory backend configuration
    #[serde(default)]
    pub backend: MemoryBackend,
    /// Memory settings
    #[serde(default)]
    pub settings: MemorySettings,
}

/// Memory backend configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryBackend {
    /// Backend type
    #[serde(default)]
    pub r#type: String,
    /// Connection string
    #[serde(default)]
    pub connection: String,
    /// Database name
    #[serde(default)]
    pub database: String,
}

/// Memory settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySettings {
    /// Default memory limit in MB
    #[serde(default)]
    pub memory_limit: usize,
    /// Eviction policy
    #[serde(default)]
    pub eviction_policy: String,
    /// TTL in seconds
    #[serde(default = "default_ttl")]
    pub ttl: u64,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            memory_limit: 0,
            eviction_policy: String::new(),
            ttl: default_ttl(),
        }
    }
}

fn default_ttl() -> u64 {
    86400
}
