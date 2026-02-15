use serde::{Deserialize, Serialize};

/// Tools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Exec tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecToolConfig {
    /// Enabled flag
    #[serde(default)]
    pub enabled: bool,
    /// Allowed commands
    #[serde(default)]
    pub allowed_commands: Vec<String>,
}

/// File system tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            bash: BashToolConfig::default(),
            exec: ExecToolConfig::default(),
            filesystem: FileSystemToolConfig::default(),
        }
    }
}

impl Default for BashToolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            working_dir: String::new(),
            timeout: default_timeout(),
        }
    }
}

impl Default for ExecToolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_commands: Vec::new(),
        }
    }
}

impl Default for FileSystemToolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            root: String::new(),
            operations: Vec::new(),
        }
    }
}
