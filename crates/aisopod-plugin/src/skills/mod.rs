//! Core types for the skills system.
//!
//! This module provides the foundational types for defining and managing skills
//! in the aisopod system. Skills are reusable bundles of system prompt fragments
//! and tools that can be assigned to agents.
//!
//! # Core Types
//!
//! - [`Skill`]: The main async trait that all skills must implement.
//! - [`SkillMeta`]: Metadata describing a skill's identity and requirements.
//! - [`SkillCategory`]: Category classification for skills.
//! - [`SkillContext`]: Runtime context provided to skills during initialization.
//!
//! # Manifest Types
//!
//! - [`SkillManifest`]: Parsed skill manifest from `skill.toml` files.
//! - [`ManifestError`]: Error types for manifest parsing.
//! - [`parse_manifest`]: Function to parse a skill manifest from file.
//!
//! # Discovery Types
//!
//! - [`discover_skill_dirs`]: Function to scan directories for skills.
//! - [`validate_requirements`]: Function to check skill requirements.
//! - [`load_skills`]: Function to orchestrate the full discovery pipeline.
//!
//! # Registry Types
//!
//! - [`SkillRegistry`]: Central registry for skill discovery and lifecycle management.
//! - [`SkillStatus`]: Status indicating a skill's health and availability.
//!
//! # Example
//!
//! ```ignore
//! use aisopod_plugin::{Skill, SkillMeta, SkillCategory, SkillContext};
//! use aisopod_tools::ToolResult;
//! use async_trait::async_trait;
//! use std::sync::Arc;
//!
//! struct ExampleSkill {
//!     meta: SkillMeta,
//! }
//!
//! impl ExampleSkill {
//!     pub fn new() -> Self {
//!         Self {
//!             meta: SkillMeta::new(
//!                 "example-skill",
//!                 "1.0.0",
//!                 "An example skill",
//!                 SkillCategory::Productivity,
//!                 vec![],
//!                 vec![],
//!                 None,
//!             ),
//!         }
//!     }
//! }
//!
//! #[async_trait]
//! impl Skill for ExampleSkill {
//!     fn id(&self) -> &str {
//!         "example-skill"
//!     }
//!
//!     fn meta(&self) -> &SkillMeta {
//!         &self.meta
//!     }
//!
//!     fn system_prompt_fragment(&self) -> Option<String> {
//!         Some("You have access to an example skill.".to_string())
//!     }
//!
//!     fn tools(&self) -> Vec<Arc<dyn aisopod_tools::Tool>> {
//!         vec![]
//!     }
//!
//!     async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>> {
//!         Ok(())
//!     }
//! }
//! ```

mod builtin;
mod context;
mod discovery;
mod manifest;
mod meta;
mod registry;
mod scaffold;
mod r#trait;

use std::sync::Arc;

#[cfg(feature = "skill-healthcheck")]
pub use builtin::healthcheck;
#[cfg(feature = "skill-model-usage")]
pub use builtin::model_usage;
#[cfg(feature = "skill-session-logs")]
pub use builtin::session_logs;
pub use context::SkillContext;
pub use discovery::{
    discover_skill_dirs, load_skills, validate_requirements, DiscoveredSkill, DiscoveryError,
    DiscoveryResult,
};
pub use manifest::{parse_manifest, ManifestError, SkillManifest};
pub use meta::{SkillCategory, SkillMeta};
pub use r#trait::Skill;
pub use registry::{SkillRegistry, SkillStatus};
pub use scaffold::{scaffold_skill, to_pascal_case, ScaffoldOptions};

/// Merges skill system prompt fragments into a base prompt.
///
/// This function takes a base prompt and a collection of skills,
/// collecting all their system prompt fragments and appending them
/// to the base prompt with appropriate formatting.
///
/// # Arguments
///
/// * `base` - The base system prompt to start with
/// * `skills` - A slice of skill references wrapped in `Arc<dyn Skill>`
///
/// # Returns
///
/// A merged string containing the base prompt followed by all skill fragments.
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::skills::merge_skill_prompts;
/// use std::sync::Arc;
///
/// let base = "You are a helpful assistant.";
/// let skills: Vec<Arc<dyn Skill>> = vec![Arc::new(skill1), Arc::new(skill2)];
/// let merged = merge_skill_prompts(base, &skills);
/// ```
pub fn merge_skill_prompts(base: &str, skills: &[Arc<dyn Skill>]) -> String {
    let mut result = String::from(base);

    for skill in skills {
        if let Some(fragment) = skill.system_prompt_fragment() {
            if !fragment.is_empty() {
                // Add a blank line before each fragment for readability
                result.push('\n');
                result.push_str(&fragment);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::{SkillCategory, SkillMeta};
    use aisopod_tools::Tool;
    use async_trait::async_trait;
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestSkill {
        meta: SkillMeta,
        id: String,
        prompt_fragment: Option<String>,
    }

    impl TestSkill {
        fn new(id: &str, prompt_fragment: Option<&str>) -> Self {
            Self {
                meta: SkillMeta::new(
                    id,
                    "1.0.0",
                    format!("Test skill {}", id),
                    SkillCategory::Utility,
                    vec![],
                    vec![],
                    None,
                ),
                id: id.to_string(),
                prompt_fragment: prompt_fragment.map(|s| s.to_string()),
            }
        }
    }

    #[async_trait]
    impl Skill for TestSkill {
        fn id(&self) -> &str {
            &self.id
        }

        fn meta(&self) -> &SkillMeta {
            &self.meta
        }

        fn system_prompt_fragment(&self) -> Option<String> {
            self.prompt_fragment.clone()
        }

        fn tools(&self) -> Vec<Arc<dyn Tool>> {
            vec![]
        }

        async fn init(
            &self,
            _ctx: &crate::skills::SkillContext,
        ) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
    }

    #[test]
    fn test_merge_skill_prompts_empty_skills() {
        let base = "Base prompt.";
        let skills: Vec<Arc<dyn Skill>> = vec![];

        let result = merge_skill_prompts(base, &skills);

        assert_eq!(result, "Base prompt.");
    }

    #[test]
    fn test_merge_skill_prompts_with_one_skill() {
        let base = "Base prompt.";
        let skills: Vec<Arc<dyn Skill>> = vec![Arc::new(TestSkill::new(
            "skill1",
            Some("Skill 1 fragment."),
        ))];

        let result = merge_skill_prompts(base, &skills);

        assert!(result.contains("Base prompt."));
        assert!(result.contains("Skill 1 fragment."));
    }

    #[test]
    fn test_merge_skill_prompts_with_multiple_skills() {
        let base = "Base prompt.";
        let skills: Vec<Arc<dyn Skill>> = vec![
            Arc::new(TestSkill::new("skill1", Some("Skill 1 fragment."))),
            Arc::new(TestSkill::new("skill2", Some("Skill 2 fragment."))),
            Arc::new(TestSkill::new("skill3", Some("Skill 3 fragment."))),
        ];

        let result = merge_skill_prompts(base, &skills);

        assert!(result.contains("Base prompt."));
        assert!(result.contains("Skill 1 fragment."));
        assert!(result.contains("Skill 2 fragment."));
        assert!(result.contains("Skill 3 fragment."));
    }

    #[test]
    fn test_merge_skill_prompts_with_empty_fragments() {
        let base = "Base prompt.";
        let skills: Vec<Arc<dyn Skill>> = vec![
            Arc::new(TestSkill::new("skill1", None)),
            Arc::new(TestSkill::new("skill2", Some(""))), // Empty string - should be skipped
            Arc::new(TestSkill::new("skill3", Some("Real fragment."))),
        ];

        let result = merge_skill_prompts(base, &skills);

        assert!(result.contains("Base prompt."));
        // Check that skill1 (None fragment) and skill2 (empty fragment) are NOT in result
        assert!(!result.contains("skill1"));
        assert!(!result.contains("skill2"));
        assert!(result.contains("Real fragment."));
    }

    #[test]
    fn test_merge_skill_prompts_with_newlines() {
        let base = "Base prompt.";
        let skills: Vec<Arc<dyn Skill>> = vec![Arc::new(TestSkill::new(
            "skill1",
            Some("Skill 1\nMulti-line\nFragment."),
        ))];

        let result = merge_skill_prompts(base, &skills);

        assert!(result.contains("Base prompt."));
        assert!(result.contains("Skill 1"));
        assert!(result.contains("Multi-line"));
        assert!(result.contains("Fragment."));
    }

    #[test]
    fn test_merge_skill_prompts_preserves_order() {
        let base = "Base.";
        let skills: Vec<Arc<dyn Skill>> = vec![
            Arc::new(TestSkill::new("first", Some("First."))),
            Arc::new(TestSkill::new("second", Some("Second."))),
            Arc::new(TestSkill::new("third", Some("Third."))),
        ];

        let result = merge_skill_prompts(base, &skills);

        // Check order is preserved
        let first_pos = result.find("First").unwrap();
        let second_pos = result.find("Second").unwrap();
        let third_pos = result.find("Third").unwrap();

        assert!(first_pos < second_pos);
        assert!(second_pos < third_pos);
    }
}
