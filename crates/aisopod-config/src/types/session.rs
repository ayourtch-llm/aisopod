use serde::{Deserialize, Serialize};

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Message handling settings
    #[serde(default)]
    pub messages: MessageConfig,
    /// Session compaction settings
    #[serde(default)]
    pub compaction: CompactionConfig,
}

/// Message handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageConfig {
    /// Maximum messages in session
    #[serde(default)]
    pub max_messages: usize,
    /// Message retention policy
    #[serde(default)]
    pub retention: String,
    /// Message formatting
    #[serde(default)]
    pub format: String,
}

/// Session compaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Enabled flag
    #[serde(default)]
    pub enabled: bool,
    /// Minimum messages before compaction
    #[serde(default)]
    pub min_messages: usize,
    /// Compaction interval in seconds
    #[serde(default = "default_interval")]
    pub interval: u64,
}

fn default_interval() -> u64 {
    3600
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            messages: MessageConfig::default(),
            compaction: CompactionConfig::default(),
        }
    }
}

impl Default for MessageConfig {
    fn default() -> Self {
        Self {
            max_messages: 1000,
            retention: String::from("unlimited"),
            format: String::from("standard"),
        }
    }
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_messages: 100,
            interval: default_interval(),
        }
    }
}
