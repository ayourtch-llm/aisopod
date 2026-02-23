use serde::{Deserialize, Serialize};

/// Category classification for skills.
///
/// This enum provides a way to categorize skills for discovery,
/// filtering, and organizational purposes. Each variant represents
/// a broad category of functionality that skills can provide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillCategory {
    /// Skills related to messaging and communication.
    ///
    /// Examples: email, chat, social media integration.
    Messaging,
    /// Skills related to productivity and task management.
    ///
    /// Examples: calendar, file management, note-taking.
    Productivity,
    /// Skills related to AI and machine learning operations.
    ///
    /// Examples: model orchestration, data preprocessing, training pipelines.
    AiMl,
    /// Skills related to system integration and external API calls.
    ///
    /// Examples: HTTP clients, webhook handlers, service connectors.
    Integration,
    /// General utility skills that don't fit into other categories.
    ///
    /// Examples: logging, formatting, validation helpers.
    Utility,
    /// Core system skills for platform functionality.
    ///
    /// Examples: configuration management, security, diagnostics.
    System,
}

/// Metadata describing a skill's identity and requirements.
///
/// This struct contains information about a skill that can be used
/// for discovery, registration, and compatibility checking. It includes
/// basic identification, descriptive information, and runtime requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    /// The unique identifier for this skill.
    ///
    /// This ID should be stable across versions and unique among all skills.
    /// It is typically used for configuration lookup and skill management.
    pub name: String,
    /// The version of this skill following semantic versioning.
    pub version: String,
    /// A brief description of what this skill does.
    pub description: String,
    /// The category this skill belongs to.
    pub category: SkillCategory,
    /// Environment variables that must be set at runtime.
    ///
    /// This list contains the names of environment variables that the skill
    /// requires to be present in the process environment. If any of these
    /// are missing, the skill may fail to initialize or operate correctly.
    pub required_env_vars: Vec<String>,
    /// Executables that must be available on the system PATH.
    ///
    /// This list contains the names of external binaries that the skill
    /// depends on. The skill will fail to initialize if any of these
    /// executables are not found on the system PATH.
    pub required_binaries: Vec<String>,
    /// Optional platform constraint for this skill.
    ///
    /// When set, constrains this skill to run only on the specified
    /// operating system. Valid values include "linux", "macos", and "windows".
    /// When None, the skill is platform-agnostic.
    pub platform: Option<String>,
}

impl SkillMeta {
    /// Creates a new `SkillMeta` instance with the given fields.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique identifier for this skill
    /// * `version` - The version following semantic versioning
    /// * `description` - A brief description of the skill's functionality
    /// * `category` - The category this skill belongs to
    /// * `required_env_vars` - Environment variables required at runtime
    /// * `required_binaries` - External executables required on PATH
    /// * `platform` - Optional OS constraint ("linux", "macos", "windows")
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        category: SkillCategory,
        required_env_vars: Vec<String>,
        required_binaries: Vec<String>,
        platform: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            category,
            required_env_vars,
            required_binaries,
            platform,
        }
    }
}

impl Default for SkillMeta {
    /// Creates a default `SkillMeta` with empty values.
    ///
    /// This is useful for serialization/deserialization scenarios
    /// where the struct needs to be created before all fields are known.
    fn default() -> Self {
        Self {
            name: String::new(),
            version: String::new(),
            description: String::new(),
            category: SkillCategory::Utility,
            required_env_vars: Vec::new(),
            required_binaries: Vec::new(),
            platform: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_category_clone() {
        let category = SkillCategory::Messaging;
        let cloned = category.clone();
        assert_eq!(category, cloned);
    }

    #[test]
    fn test_skill_meta_new() {
        let meta = SkillMeta::new(
            "test-skill",
            "1.0.0",
            "A test skill",
            SkillCategory::Productivity,
            vec!["API_KEY".to_string()],
            vec!["curl".to_string()],
            Some("linux".to_string()),
        );

        assert_eq!(meta.name, "test-skill");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.description, "A test skill");
        assert_eq!(meta.category, SkillCategory::Productivity);
        assert_eq!(meta.required_env_vars, vec!["API_KEY"]);
        assert_eq!(meta.required_binaries, vec!["curl"]);
        assert_eq!(meta.platform, Some("linux".to_string()));
    }

    #[test]
    fn test_skill_meta_default() {
        let meta = SkillMeta::default();
        assert!(meta.name.is_empty());
        assert!(meta.version.is_empty());
        assert!(meta.description.is_empty());
        assert_eq!(meta.category, SkillCategory::Utility);
        assert!(meta.required_env_vars.is_empty());
        assert!(meta.required_binaries.is_empty());
        assert!(meta.platform.is_none());
    }

    #[test]
    fn test_skill_meta_debug() {
        let meta = SkillMeta::default();
        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("SkillMeta"));
    }
}
