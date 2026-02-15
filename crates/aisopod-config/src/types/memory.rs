use serde::{Deserialize, Serialize};

/// Memory configuration for QMD memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Memory backend configuration
    #[serde(default)]
    pub backend: MemoryBackend,
    /// Memory settings
    #[serde(default)]
    pub settings: MemorySettings,
}

/// Memory backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn default_ttl() -> u64 {
    86400
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            backend: MemoryBackend::default(),
            settings: MemorySettings::default(),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self {
            r#type: String::from("memory"),
            connection: String::new(),
            database: String::new(),
        }
    }
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            memory_limit: 1024,
            eviction_policy: String::from("lru"),
            ttl: default_ttl(),
        }
    }
}
