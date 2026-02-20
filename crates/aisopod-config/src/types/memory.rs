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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

fn default_ttl() -> u64 {
    86400
}
