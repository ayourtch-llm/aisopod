use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;

/// Runtime context provided to skills during initialization.
///
/// This struct contains runtime information and resources that skills
/// may need during their `init()` phase, such as configuration data,
/// access to a dedicated data directory, and information about the agent
/// that will use the skill.
///
/// # Lifecycle
///
/// 1. The skill system creates a `SkillContext` with the agent's configuration
///    and a dedicated data directory
/// 2. During `Skill::init()`, the skill receives this context
/// 3. The skill can read configuration values and use the data directory
///    for persistent storage
pub struct SkillContext {
    /// The skill's configuration as a JSON value.
    ///
    /// This contains the deserialized configuration from the skill's
    /// configuration section in the main configuration file.
    /// Skills can use serde_json::from_value to deserialize this into
    /// their specific configuration struct.
    pub config: Arc<Value>,
    /// The path to a dedicated data directory for this skill.
    ///
    /// Skills may use this directory to store persistent data,
    /// caches, or other runtime files. The directory is guaranteed
    /// to exist and be writable by the skill.
    pub data_dir: PathBuf,
    /// Optional identifier for the agent that will use this skill.
    ///
    /// When a skill is being initialized for a specific agent, this
    /// field contains the agent's unique identifier. This can be used
    /// for agent-specific configuration or data storage.
    pub agent_id: Option<String>,
}

impl SkillContext {
    /// Creates a new `SkillContext` instance.
    ///
    /// # Arguments
    ///
    /// * `config` - The skill's configuration as a JSON value wrapped in Arc
    /// * `data_dir` - The path to a dedicated data directory
    /// * `agent_id` - Optional agent identifier for agent-specific initialization
    pub fn new(config: Arc<Value>, data_dir: PathBuf, agent_id: Option<String>) -> Self {
        Self {
            config,
            data_dir,
            agent_id,
        }
    }

    /// Returns the configuration as a typed value.
    ///
    /// This helper method deserializes the configuration into the
    /// specified type T.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails due to type mismatch
    /// or invalid JSON structure.
    pub fn config_as<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.config.as_ref().clone())
    }
}

impl std::fmt::Debug for SkillContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillContext")
            .field("data_dir", &self.data_dir)
            .field("agent_id", &self.agent_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_skill_context_new() {
        let config = Arc::new(json!({"key": "value"}));
        let data_dir = PathBuf::from("/tmp/test-skill");
        let agent_id = Some("agent-123".to_string());

        let ctx = SkillContext::new(config.clone(), data_dir.clone(), agent_id.clone());

        assert_eq!(ctx.config, config);
        assert_eq!(ctx.data_dir, data_dir);
        assert_eq!(ctx.agent_id, agent_id);
    }

    #[test]
    fn test_skill_context_with_none_agent_id() {
        let config = Arc::new(json!({}));
        let data_dir = PathBuf::from("/tmp/test");

        let ctx = SkillContext::new(config, data_dir, None);

        assert!(ctx.agent_id.is_none());
    }

    #[test]
    fn test_skill_context_config_as() {
        let config = Arc::new(json!({"timeout": 30, "enabled": true}));
        let data_dir = PathBuf::from("/tmp/test");

        let ctx = SkillContext::new(config, data_dir, None);

        let result: serde_json::Value = ctx.config_as().unwrap();
        assert_eq!(result["timeout"], 30);
        assert_eq!(result["enabled"], true);
    }

    #[test]
    fn test_skill_context_config_as_error() {
        let config = Arc::new(json!({"key": "value"}));
        let data_dir = PathBuf::from("/tmp/test");

        let ctx = SkillContext::new(config, data_dir, None);

        // Try to deserialize into an incompatible type
        let result: Result<i32, _> = ctx.config_as();
        assert!(result.is_err());
    }

    #[test]
    fn test_skill_context_debug() {
        let config = Arc::new(json!({}));
        let data_dir = PathBuf::from("/tmp/test");
        let ctx = SkillContext::new(config, data_dir, Some("agent-1".to_string()));

        let debug_str = format!("{:?}", ctx);
        assert!(debug_str.contains("SkillContext"));
        assert!(debug_str.contains("agent-1"));
    }
}
