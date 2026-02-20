use serde::{Deserialize, Serialize};

/// Configuration schema version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConfig {
    #[serde(default = "default_version")]
    pub version: String,
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
        }
    }
}

fn default_version() -> String {
    "1.0".to_string()
}
