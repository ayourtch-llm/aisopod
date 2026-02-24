use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use crate::skills::{parse_manifest, Skill, SkillContext, SkillMeta, SkillRegistry, SkillStatus};
use thiserror::Error;

use super::manifest::ManifestError;

/// Error types for skill discovery and loading.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// Failed to parse a manifest file
    #[error("Failed to parse manifest: {0}")]
    Manifest(#[from] ManifestError),

    /// Failed to read directory
    #[error("Failed to read directory: {0}")]
    Io(#[from] std::io::Error),

    /// Discovery was skipped for a skill
    #[error("Discovery skipped for skill '{skill_id}': {reason}")]
    Skipped {
        skill_id: String,
        reason: String,
    },
}

/// Result type for skill discovery operations.
pub type DiscoveryResult<T> = Result<T, DiscoveryError>;

/// Holds information about a discovered skill directory.
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    /// The path to the skill directory
    pub path: PathBuf,
    /// The parsed manifest
    pub manifest: super::SkillManifest,
}

/// Discovers skill directories in the given base directories.
///
/// Scans each base directory for subdirectories containing a `skill.toml` file.
/// Returns a vector of paths to directories that contain valid skill manifests.
///
/// # Arguments
///
/// * `base_dirs` - A slice of base directory paths to scan
///
/// # Returns
///
/// A vector of `PathBuf` paths to directories containing skill manifests.
/// Empty directories or directories without `skill.toml` are skipped.
///
/// # Example
///
/// ```ignore
/// use std::path::PathBuf;
/// use aisopod_plugin::skills::discover_skill_dirs;
///
/// let base_dirs = vec![
///     PathBuf::from("~/.aisopod/skills"),
///     PathBuf::from("built-in/skills"),
/// ];
///
/// let skill_dirs = discover_skill_dirs(&base_dirs);
/// for dir in skill_dirs {
///     println!("Found skill directory: {:?}", dir);
/// }
/// ```
pub fn discover_skill_dirs(base_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut skill_dirs = Vec::new();

    for base in base_dirs {
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Only process directories that contain a skill.toml file
                if path.is_dir() && path.join("skill.toml").exists() {
                    skill_dirs.push(path);
                }
            }
        }
    }

    skill_dirs
}

/// Validates that a skill's requirements are met.
///
/// Checks that:
/// - All required environment variables are set
/// - All required binaries are available on PATH
/// - The platform constraint (if any) matches the current OS
///
/// # Arguments
///
/// * `manifest` - The skill manifest to validate
///
/// # Returns
///
/// * `Ok(())` - If all requirements are satisfied
/// * `Err(Vec<String>)` - If any requirements are missing, with error messages
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::skills::{validate_requirements, SkillManifest, SkillCategory};
///
/// let manifest = SkillManifest::new(
///     "test-skill",
///     "Test Skill",
///     "A test skill",
///     "1.0.0",
///     SkillCategory::Productivity,
///     vec!["API_KEY".to_string()],
///     vec!["curl".to_string()],
///     None,
/// );
///
/// match validate_requirements(&manifest) {
///     Ok(()) => println!("Requirements satisfied"),
///     Err(errors) => {
///         for error in errors {
///             eprintln!("Requirement error: {}", error);
///         }
///     }
/// }
/// ```
pub fn validate_requirements(manifest: &super::SkillManifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Check environment variables
    for var in &manifest.required_env_vars {
        if env::var(var).is_err() {
            errors.push(format!("Missing environment variable: {}", var));
        }
    }

    // Check required binaries
    for bin in &manifest.required_binaries {
        if is_binary_available(bin) {
            // Binary is available
        } else {
            errors.push(format!("Missing required binary: {}", bin));
        }
    }

    // Check platform constraint
    if let Some(ref platform) = manifest.platform {
        let current_os = env::consts::OS;
        let platform_normalized = platform.to_lowercase();

        match platform_normalized.as_str() {
            "linux" if current_os == "linux" => {}
            "macos" if current_os == "macos" => {}
            "windows" if current_os == "windows" => {}
            _ => {
                errors.push(format!(
                    "Platform mismatch: requires '{}', running '{}'",
                    platform, current_os
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Checks if a binary is available on the system PATH.
///
/// Uses platform-appropriate methods to check binary availability:
/// - Unix: uses `which` command
/// - Windows: uses `where` command
///
/// # Arguments
///
/// * `binary_name` - The name of the binary to check
///
/// # Returns
///
/// `true` if the binary is found, `false` otherwise.
fn is_binary_available(binary_name: &str) -> bool {
    let output = if cfg!(unix) {
        Command::new("which").arg(binary_name).output()
    } else if cfg!(windows) {
        Command::new("where").arg(binary_name).output()
    } else {
        // For other platforms, try `which` first, then `where`
        Command::new("which").arg(binary_name).output()
    };

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Loads and registers skills from discovered directories.
///
/// This is the main orchestration function that:
/// 1. Discovers skill directories using `discover_skill_dirs`
/// 2. Parses each manifest with `parse_manifest`
/// 3. Validates requirements with `validate_requirements`
/// 4. Registers skills into the registry, marking them as `Degraded`
///    if requirements aren't met (instead of failing)
///
/// # Arguments
///
/// * `registry` - The skill registry to register skills into
/// * `base_dirs` - Base directories to scan for skills
/// * `skill_context` - Runtime context for skill initialization
///
/// # Returns
///
/// * `Ok(HashMap<String, SkillStatus>)` - Maps skill IDs to their final status
/// * `Err(DiscoveryError)` - If discovery fails catastrophically
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::skills::{SkillRegistry, SkillContext, load_skills};
/// use std::path::PathBuf;
///
/// let mut registry = SkillRegistry::new();
/// let context = SkillContext::new();
/// let base_dirs = vec![PathBuf::from("~/.aisopod/skills")];
///
/// match load_skills(&mut registry, &base_dirs, &context).await {
///     Ok(statuses) => {
///         for (skill_id, status) in statuses {
///             println!("Skill {} status: {:?}", skill_id, status);
///         }
///     }
///     Err(e) => eprintln!("Discovery failed: {}", e),
/// }
/// ```
pub async fn load_skills(
    registry: &mut SkillRegistry,
    base_dirs: &[PathBuf],
    skill_context: &SkillContext,
) -> DiscoveryResult<HashMap<String, SkillStatus>> {
    let mut skill_statuses = HashMap::new();

    // Discover all skill directories
    let dirs = discover_skill_dirs(base_dirs);

    if dirs.is_empty() {
        tracing::debug!("No skill directories found in {:?}", base_dirs);
        return Ok(skill_statuses);
    }

    tracing::info!("Found {} skill directory(ies)", dirs.len());

    // Process each discovered directory
    for skill_dir in dirs {
        let manifest_path = skill_dir.join("skill.toml");
        let skill_id = skill_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        tracing::info!("Processing skill directory: {:?}", skill_dir);

        // Parse the manifest
        let manifest = match parse_manifest(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(
                    "Failed to parse manifest for skill '{}': {}",
                    skill_id,
                    e
                );
                continue;
            }
        };

        // Validate requirements
        let skill_id_str = manifest.id.clone();
        match validate_requirements(&manifest) {
            Ok(()) => {
                // Requirements met - register as Ready
                match load_and_register_skill(registry, &manifest, skill_context).await {
                    Ok(()) => {
                        skill_statuses.insert(skill_id_str.clone(), SkillStatus::Ready);
                        tracing::info!("Registered skill '{}' as Ready", skill_id_str);
                    }
                    Err(e) => {
                        skill_statuses.insert(
                            skill_id_str.clone(),
                            SkillStatus::Failed {
                                error: e.to_string(),
                            },
                        );
                        tracing::warn!(
                            "Failed to initialize skill '{}': {}",
                            skill_id_str,
                            e
                        );
                    }
                }
            }
            Err(reasons) => {
                // Requirements not met - register as Degraded
                let reason = reasons.join("; ");
                skill_statuses.insert(skill_id_str.clone(), SkillStatus::Degraded { reason });
                tracing::info!(
                    "Registered skill '{}' as Degraded: {:?}",
                    manifest.id,
                    skill_statuses.get(&skill_id_str)
                );
            }
        }
    }

    Ok(skill_statuses)
}

/// Loads and registers a single skill instance.
///
/// This helper function creates a `SkillMeta` from the manifest and
/// registers a placeholder skill with the appropriate status.
///
/// # Arguments
///
/// * `registry` - The skill registry
/// * `manifest` - The parsed skill manifest
/// * `skill_context` - Runtime context for initialization
///
/// # Returns
///
/// * `Ok(())` - If registration succeeded
/// * `Err(Box<dyn Error>)` - If registration failed
async fn load_and_register_skill(
    registry: &mut SkillRegistry,
    manifest: &super::SkillManifest,
    skill_context: &SkillContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a placeholder skill for registration
    // In a full implementation, this would dynamically load the skill
    // from a shared library or compile-time registration
    let meta = SkillMeta::new(
        &manifest.id,
        &manifest.version,
        &manifest.description,
        manifest.category.clone(),
        manifest.required_env_vars.clone(),
        manifest.required_binaries.clone(),
        manifest.platform.clone(),
    );

    // For now, we just register a minimal placeholder
    // The actual skill implementation would be loaded from a dynamic library
    // or registered via build-time feature flags
    let skill: Arc<dyn Skill> = Arc::new(ArcPlaceholderSkill::new(meta));
    registry.register(skill);

    Ok(())
}

/// A placeholder skill implementation for the discovery pipeline.
///
/// This is used during the discovery phase when we only have a manifest
/// but haven't loaded the actual skill implementation yet.
struct ArcPlaceholderSkill {
    meta: SkillMeta,
}

impl ArcPlaceholderSkill {
    fn new(meta: SkillMeta) -> Self {
        Self { meta }
    }
}

impl std::fmt::Debug for ArcPlaceholderSkill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArcPlaceholderSkill").finish()
    }
}

impl std::fmt::Display for ArcPlaceholderSkill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Placeholder for skill: {}", self.meta.name)
    }
}

#[async_trait::async_trait]
impl Skill for ArcPlaceholderSkill {
    fn id(&self) -> &str {
        &self.meta.name
    }

    fn meta(&self) -> &SkillMeta {
        &self.meta
    }

    fn system_prompt_fragment(&self) -> Option<String> {
        None
    }

    fn tools(&self) -> Vec<Arc<dyn aisopod_tools::Tool>> {
        Vec::new()
    }

    async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use std::sync::Arc;
    use serde_json::json;

    #[test]
    fn test_discover_skill_dirs_empty() {
        let temp_dir = TempDir::new().unwrap();
        let base_dirs = vec![temp_dir.path().to_path_buf()];

        let dirs = discover_skill_dirs(&base_dirs);
        assert!(dirs.is_empty());
    }

    #[test]
    fn test_discover_skill_dirs_with_valid_skill() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        // Create a valid skill.toml
        let manifest_content = r#"
id = "test-skill"
name = "Test Skill"
description = "A test skill"
version = "1.0.0"
category = "Productivity"
"#;
        fs::write(skill_dir.join("skill.toml"), manifest_content).unwrap();

        let base_dirs = vec![temp_dir.path().to_path_buf()];
        let dirs = discover_skill_dirs(&base_dirs);

        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0], skill_dir);
    }

    #[test]
    fn test_discover_skill_dirs_with_multiple_skills() {
        let temp_dir = TempDir::new().unwrap();

        // Create two skill directories
        let skill1_dir = temp_dir.path().join("skill-1");
        let skill2_dir = temp_dir.path().join("skill-2");

        fs::create_dir_all(&skill1_dir).unwrap();
        fs::create_dir_all(&skill2_dir).unwrap();

        // Write manifests
        fs::write(skill1_dir.join("skill.toml"), r#"id="skill-1";name="Skill 1";description="S1";version="1.0.0";category="Utility""#).unwrap();
        fs::write(skill2_dir.join("skill.toml"), r#"id="skill-2";name="Skill 2";description="S2";version="1.0.0";category="Utility""#).unwrap();

        let base_dirs = vec![temp_dir.path().to_path_buf()];
        let dirs = discover_skill_dirs(&base_dirs);

        assert_eq!(dirs.len(), 2);
    }

    #[test]
    fn test_discover_skill_dirs_skips_invalid() {
        let temp_dir = TempDir::new().unwrap();

        // Create a directory without skill.toml
        let invalid_dir = temp_dir.path().join("invalid-skill");
        fs::create_dir_all(&invalid_dir).unwrap();
        fs::write(invalid_dir.join("other.toml"), "data").unwrap();

        let base_dirs = vec![temp_dir.path().to_path_buf()];
        let dirs = discover_skill_dirs(&base_dirs);

        assert!(dirs.is_empty());
    }

    #[test]
    fn test_validate_requirements_success() {
        use crate::skills::SkillCategory;
        use crate::skills::SkillManifest;

        let manifest = SkillManifest::new(
            "test-skill",
            "Test Skill",
            "A test skill",
            "1.0.0",
            SkillCategory::Utility,
            vec![],                                      // No env vars required
            vec!["echo".to_string()],                   // echo should be available
            None,
        );

        let result = validate_requirements(&manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_requirements_missing_binary() {
        use crate::skills::SkillCategory;
        use crate::skills::SkillManifest;

        let manifest = SkillManifest::new(
            "test-skill",
            "Test Skill",
            "A test skill",
            "1.0.0",
            SkillCategory::Utility,
            vec![],
            vec!["nonexistent_binary_xyz123".to_string()],
            None,
        );

        let result = validate_requirements(&manifest);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("nonexistent_binary_xyz123")));
    }

    #[test]
    fn test_validate_requirements_platform_mismatch() {
        use crate::skills::SkillCategory;
        use crate::skills::SkillManifest;

        let current_os = env::consts::OS;
        let opposite_os = if current_os == "linux" {
            "windows"
        } else {
            "linux"
        };

        let manifest = SkillManifest::new(
            "test-skill",
            "Test Skill",
            "A test skill",
            "1.0.0",
            SkillCategory::Utility,
            vec![],
            vec![],
            Some(opposite_os.to_string()),
        );

        let result = validate_requirements(&manifest);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Platform mismatch")));
    }

    #[test]
    fn test_is_binary_available() {
        // echo should be available on all platforms
        assert!(is_binary_available("echo"));
    }

    #[test]
    fn test_is_binary_not_available() {
        // A truly nonexistent binary
        assert!(!is_binary_available("this_binary_does_not_exist_12345_xyz"));
    }

    #[test]
    fn test_load_skills_empty() {
        use crate::skills::SkillContext;

        let mut registry = SkillRegistry::new();
        let base_dirs = vec![PathBuf::from("/nonexistent/path")];
        let context = SkillContext::new(
            Arc::new(json!({})),
            std::path::PathBuf::from("/tmp"),
            None,
        );

        let result = futures::executor::block_on(load_skills(&mut registry, &base_dirs, &context));

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_discover_skill_dirs_multiple_base_dirs() {
        let temp1 = TempDir::new().unwrap();
        let temp2 = TempDir::new().unwrap();

        // Create skills in both directories
        let skill1 = temp1.path().join("skill-1");
        fs::create_dir_all(&skill1).unwrap();
        fs::write(skill1.join("skill.toml"), r#"id="skill-1";name="Skill 1";description="S1";version="1.0.0";category="Utility""#).unwrap();

        let skill2 = temp2.path().join("skill-2");
        fs::create_dir_all(&skill2).unwrap();
        fs::write(skill2.join("skill.toml"), r#"id="skill-2";name="Skill 2";description="S2";version="1.0.0";category="Utility""#).unwrap();

        let base_dirs = vec![temp1.path().to_path_buf(), temp2.path().to_path_buf()];
        let dirs = discover_skill_dirs(&base_dirs);

        assert_eq!(dirs.len(), 2);
    }
}
