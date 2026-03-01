use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use super::SkillCategory;

/// Error types for manifest parsing and validation.
#[derive(Debug, Error)]
pub enum ManifestError {
    /// Failed to read the manifest file
    #[error("Failed to read manifest file: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse TOML content
    #[error("Failed to parse manifest TOML: {0}")]
    Parse(#[from] toml::de::Error),

    /// Required field is missing
    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    /// Invalid value for a field
    #[error("Invalid field value: {0}")]
    InvalidField(&'static str),
}

/// Manifest definition for a skill, parsed from `skill.toml`.
///
/// This struct represents the metadata and requirements for a skill
/// as defined in its manifest file. It includes identification information,
/// descriptive data, and runtime requirements that must be satisfied
/// for the skill to operate correctly.
#[derive(Debug, Deserialize, Clone)]
pub struct SkillManifest {
    /// The unique identifier for this skill.
    ///
    /// This ID should be stable across versions and unique among all skills.
    #[serde(rename = "id")]
    pub id: String,

    /// The human-readable name of the skill.
    pub name: String,

    /// A brief description of what this skill does.
    pub description: String,

    /// The version of this skill following semantic versioning.
    pub version: String,

    /// The category this skill belongs to.
    pub category: SkillCategory,

    /// Environment variables that must be set at runtime.
    ///
    /// This list contains the names of environment variables that the skill
    /// requires to be present in the process environment. If any of these
    /// are missing, the skill may fail to initialize or operate correctly.
    #[serde(default)]
    pub required_env_vars: Vec<String>,

    /// Executables that must be available on the system PATH.
    ///
    /// This list contains the names of external binaries that the skill
    /// depends on. The skill will fail to initialize if any of these
    /// executables are not found on the system PATH.
    #[serde(default)]
    pub required_binaries: Vec<String>,

    /// Optional platform constraint for this skill.
    ///
    /// When set, constrains this skill to run only on the specified
    /// operating system. Valid values include "linux", "macos", and "windows".
    /// When None, the skill is platform-agnostic.
    #[serde(default)]
    pub platform: Option<String>,
}

impl SkillManifest {
    /// Creates a new `SkillManifest` with the given fields.
    ///
    /// This is a convenience constructor for programmatic creation
    /// of manifest instances.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for this skill
    /// * `name` - The human-readable name
    /// * `description` - A brief description of the skill
    /// * `version` - The version following semantic versioning
    /// * `category` - The category this skill belongs to
    /// * `required_env_vars` - Environment variables required at runtime
    /// * `required_binaries` - External executables required on PATH
    /// * `platform` - Optional OS constraint ("linux", "macos", "windows")
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        version: impl Into<String>,
        category: SkillCategory,
        required_env_vars: Vec<String>,
        required_binaries: Vec<String>,
        platform: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            version: version.into(),
            category,
            required_env_vars,
            required_binaries,
            platform,
        }
    }

    /// Validates the manifest has all required fields and valid values.
    ///
    /// Returns `Ok(())` if the manifest is valid, or an error describing
    /// the validation failure.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.id.trim().is_empty() {
            return Err(ManifestError::MissingField("id"));
        }

        if self.name.trim().is_empty() {
            return Err(ManifestError::MissingField("name"));
        }

        if self.version.trim().is_empty() {
            return Err(ManifestError::MissingField("version"));
        }

        // Validate platform if provided
        if let Some(ref platform) = self.platform {
            match platform.as_str() {
                "linux" | "macos" | "windows" => {}
                _ => {
                    return Err(ManifestError::InvalidField(
                        "platform must be 'linux', 'macos', or 'windows'",
                    ))
                }
            }
        }

        Ok(())
    }
}

/// Parses a skill manifest from a `skill.toml` file.
///
/// Reads the manifest file at the given path and deserializes it into
/// a `SkillManifest` struct using TOML parsing.
///
/// # Arguments
///
/// * `path` - The path to the `skill.toml` file
///
/// # Returns
///
/// * `Ok(SkillManifest)` - The parsed manifest if successful
/// * `Err(ManifestError)` - An error describing the failure
///
/// # Errors
///
/// This function returns an error if:
/// - The file cannot be read (I/O error)
/// - The TOML content cannot be parsed
/// - Required fields are missing
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::skills::{SkillManifest, parse_manifest};
/// use std::path::PathBuf;
///
/// let manifest = parse_manifest(&PathBuf::from("my-skill/skill.toml"))?;
/// println!("Loaded skill: {}", manifest.name);
/// ```
pub fn parse_manifest(path: &Path) -> Result<SkillManifest, ManifestError> {
    let content = std::fs::read_to_string(path)?;
    let manifest: SkillManifest = toml::from_str(&content)?;
    manifest.validate()?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_skill_manifest_new() {
        let manifest = SkillManifest::new(
            "test-skill",
            "Test Skill",
            "A test skill",
            "1.0.0",
            SkillCategory::Productivity,
            vec!["API_KEY".to_string()],
            vec!["curl".to_string()],
            Some("linux".to_string()),
        );

        assert_eq!(manifest.id, "test-skill");
        assert_eq!(manifest.name, "Test Skill");
        assert_eq!(manifest.description, "A test skill");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.category, SkillCategory::Productivity);
        assert_eq!(manifest.required_env_vars, vec!["API_KEY"]);
        assert_eq!(manifest.required_binaries, vec!["curl"]);
        assert_eq!(manifest.platform, Some("linux".to_string()));
    }

    #[test]
    fn test_skill_manifest_default_values() {
        let manifest = SkillManifest::new(
            "test-skill",
            "Test Skill",
            "A test skill",
            "1.0.0",
            SkillCategory::Utility,
            vec![],
            vec![],
            None,
        );

        assert!(manifest.required_env_vars.is_empty());
        assert!(manifest.required_binaries.is_empty());
        assert!(manifest.platform.is_none());
    }

    #[test]
    fn test_skill_manifest_validation_missing_id() {
        let mut manifest = SkillManifest::default_for_test();
        manifest.id = "".to_string();

        let result = manifest.validate();
        assert!(matches!(result, Err(ManifestError::MissingField("id"))));
    }

    #[test]
    fn test_skill_manifest_validation_missing_name() {
        let mut manifest = SkillManifest::default_for_test();
        manifest.name = "".to_string();

        let result = manifest.validate();
        assert!(matches!(result, Err(ManifestError::MissingField("name"))));
    }

    #[test]
    fn test_skill_manifest_validation_missing_version() {
        let mut manifest = SkillManifest::default_for_test();
        manifest.version = "".to_string();

        let result = manifest.validate();
        assert!(matches!(
            result,
            Err(ManifestError::MissingField("version"))
        ));
    }

    #[test]
    fn test_skill_manifest_validation_invalid_platform() {
        let mut manifest = SkillManifest::default_for_test();
        manifest.platform = Some("invalid".to_string());

        let result = manifest.validate();
        assert!(matches!(result, Err(ManifestError::InvalidField(_))));
    }

    #[test]
    fn test_skill_manifest_validation_success() {
        let manifest = SkillManifest::new(
            "test-skill",
            "Test Skill",
            "A test skill",
            "1.0.0",
            SkillCategory::Productivity,
            vec![],
            vec![],
            Some("linux".to_string()),
        );

        let result = manifest.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_skill_manifest_validation_whitespace_id() {
        let mut manifest = SkillManifest::default_for_test();
        manifest.id = "   ".to_string();

        let result = manifest.validate();
        assert!(matches!(result, Err(ManifestError::MissingField("id"))));
    }

    #[test]
    fn test_skill_manifest_platform_valid_values() {
        for platform in ["linux", "macos", "windows"] {
            let mut manifest = SkillManifest::default_for_test();
            manifest.platform = Some(platform.to_string());

            let result = manifest.validate();
            assert!(result.is_ok(), "Platform {} should be valid", platform);
        }
    }

    #[test]
    fn test_parse_manifest_with_all_fields() {
        use std::path::PathBuf;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("skill.toml");

        let content = r#"
id = "test-skill"
name = "Test Skill"
description = "A test skill"
version = "1.0.0"
category = "Productivity"
required_env_vars = ["API_KEY", "SECRET"]
required_binaries = ["curl", "jq"]
platform = "linux"
"#;
        fs::write(&manifest_path, content).unwrap();

        let manifest = parse_manifest(&manifest_path).unwrap();

        assert_eq!(manifest.id, "test-skill");
        assert_eq!(manifest.name, "Test Skill");
        assert_eq!(manifest.description, "A test skill");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.category, SkillCategory::Productivity);
        assert_eq!(manifest.required_env_vars, vec!["API_KEY", "SECRET"]);
        assert_eq!(manifest.required_binaries, vec!["curl", "jq"]);
        assert_eq!(manifest.platform, Some("linux".to_string()));
    }

    #[test]
    fn test_parse_manifest_default_optional_fields() {
        use std::path::PathBuf;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("skill.toml");

        let content = r#"
id = "minimal-skill"
name = "Minimal Skill"
description = "Minimal"
version = "0.1.0"
category = "Utility"
"#;
        fs::write(&manifest_path, content).unwrap();

        let manifest = parse_manifest(&manifest_path).unwrap();

        assert!(manifest.required_env_vars.is_empty());
        assert!(manifest.required_binaries.is_empty());
        assert!(manifest.platform.is_none());
    }

    #[test]
    fn test_skill_manifest_validation_whitespace_name() {
        let mut manifest = SkillManifest::default_for_test();
        manifest.name = "   ".to_string();

        let result = manifest.validate();
        assert!(matches!(result, Err(ManifestError::MissingField("name"))));
    }

    #[test]
    fn test_skill_manifest_validation_whitespace_version() {
        let mut manifest = SkillManifest::default_for_test();
        manifest.version = "   ".to_string();

        let result = manifest.validate();
        assert!(matches!(
            result,
            Err(ManifestError::MissingField("version"))
        ));
    }

    #[test]
    fn test_skill_manifest_validation_platform_uppercase() {
        let mut manifest = SkillManifest::default_for_test();
        manifest.platform = Some("LINUX".to_string());

        // Note: platform validation is case-sensitive (as per the spec in validate())
        // but we normalize in validate_requirements() for runtime checks
        let result = manifest.validate();
        assert!(
            result.is_err(),
            "Validation should reject uppercase platform"
        );
    }

    #[test]
    fn test_skill_manifest_debug() {
        let manifest = SkillManifest::new(
            "test",
            "Test",
            "Test",
            "1.0.0",
            SkillCategory::Utility,
            vec![],
            vec![],
            None,
        );
        let debug_str = format!("{:?}", manifest);
        assert!(debug_str.contains("SkillManifest"));
    }

    #[test]
    fn test_skill_manifest_clone() {
        let manifest = SkillManifest::new(
            "test",
            "Test",
            "Test",
            "1.0.0",
            SkillCategory::Utility,
            vec![],
            vec![],
            None,
        );
        let cloned = manifest.clone();
        assert_eq!(manifest.id, cloned.id);
        assert_eq!(manifest.name, cloned.name);
    }

    // Helper for tests
    impl SkillManifest {
        fn default_for_test() -> Self {
            SkillManifest::new(
                "test-skill",
                "Test Skill",
                "A test skill",
                "1.0.0",
                SkillCategory::Utility,
                vec![],
                vec![],
                None,
            )
        }
    }
}
