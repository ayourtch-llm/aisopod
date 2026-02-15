use serde::{Deserialize, Serialize};

/// Skills configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsConfig {
    /// Skill modules
    #[serde(default)]
    pub modules: Vec<SkillModule>,
    /// Skill settings
    #[serde(default)]
    pub settings: SkillSettings,
}

/// Skill module definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillModule {
    /// Module ID
    pub id: String,
    /// Module name
    #[serde(default)]
    pub name: String,
    /// Module path
    #[serde(default)]
    pub path: String,
    /// Enabled flag
    #[serde(default)]
    pub enabled: bool,
}

/// Skill settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSettings {
    /// Default timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Maximum execution count
    #[serde(default)]
    pub max_executions: u32,
}

fn default_timeout() -> u64 {
    60
}

impl Default for SkillsConfig {
    fn default() -> Self {
        Self {
            modules: Vec::new(),
            settings: SkillSettings::default(),
        }
    }
}

impl Default for SkillModule {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            path: String::new(),
            enabled: true,
        }
    }
}

impl Default for SkillSettings {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            max_executions: 100,
        }
    }
}
