use serde::{Deserialize, Serialize};

/// Tools configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    /// Bash tool settings
    #[serde(default)]
    pub bash: BashToolConfig,
    /// Exec tool settings
    #[serde(default)]
    pub exec: ExecToolConfig,
    /// File system tool settings
    #[serde(default)]
    pub filesystem: FileSystemToolConfig,
}

/// Bash tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashToolConfig {
    /// Enabled flag
    #[serde(default)]
    pub enabled: bool,
    /// Working directory
    #[serde(default)]
    pub working_dir: String,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

impl Default for BashToolConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            working_dir: String::new(),
            timeout: default_timeout(),
        }
    }
}

/// Exec tool configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecToolConfig {
    /// Enabled flag
    #[serde(default)]
    pub enabled: bool,
    /// Allowed commands
    #[serde(default)]
    pub allowed_commands: Vec<String>,
}

/// File system tool configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileSystemToolConfig {
    /// Enabled flag
    #[serde(default)]
    pub enabled: bool,
    /// Root directory
    #[serde(default)]
    pub root: String,
    /// Allowed operations
    #[serde(default)]
    pub operations: Vec<String>,
}

fn default_timeout() -> u64 {
    300
}
