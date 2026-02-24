use anyhow::Result;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;

use crate::skills::{SkillContext, SkillMeta};

// Re-export Tool trait from aisopod-tools for convenience
use aisopod_tools::Tool;

/// The core trait that all skills must implement.
///
/// The `Skill` trait defines the interface that every skill
/// in the aisopod system must implement. Skills go through several stages:
///
/// 1. **Discovery**: The skill's `id()` and `meta()` are queried to identify
///    and describe the skill.
/// 2. **System Prompt Integration**: `system_prompt_fragment()` provides
///    additional context for the AI model.
/// 3. **Tool Registration**: `tools()` returns the set of tools the skill
///    provides for function calling.
/// 4. **Initialization**: After all skills are instantiated, `init()` is
///    called with runtime context to perform any async setup.
///
/// # Lifetime and Ownership
///
/// Skills must implement `Send + Sync` to support both compiled-in skills
/// and dynamically loaded skills. The trait is object-safe and can be used
/// as `dyn Skill`.
///
/// # Example
///
/// This example shows the basic structure of a skill:
///
/// ```ignore
/// use aisopod_plugin::{Skill, SkillMeta, SkillContext, SkillCategory};
/// use aisopod_tools::ToolResult;
/// use async_trait::async_trait;
/// use std::sync::Arc;
/// use serde_json::json;
///
/// struct MySkill {
///     meta: SkillMeta,
/// }
///
/// impl MySkill {
///     pub fn new() -> Self {
///         Self {
///             meta: SkillMeta::new(
///                 "my-skill",
///                 "1.0.0",
///                 "A sample skill",
///                 SkillCategory::Productivity,
///                 vec![],
///                 vec![],
///                 None,
///             ),
///         }
///     }
/// }
///
/// #[async_trait]
/// impl Skill for MySkill {
///     fn id(&self) -> &str {
///         "my-skill"
///     }
///
///     fn meta(&self) -> &SkillMeta {
///         &self.meta
///     }
///
///     fn system_prompt_fragment(&self) -> Option<String> {
///         Some("You have access to additional productivity tools.".to_string())
///     }
///
///     fn tools(&self) -> Vec<Arc<dyn Tool>> {
///         vec![Arc::new(MyTool::new())]
///     }
///
///     async fn init(&self, _ctx: &SkillContext) -> Result<()> {
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Skill: Send + Sync + std::fmt::Debug {
    /// Returns the unique identifier for this skill.
    ///
    /// This ID should be stable across versions and unique among all skills.
    /// It is typically used for configuration lookup and skill management.
    fn id(&self) -> &str;

    /// Returns metadata describing this skill.
    fn meta(&self) -> &SkillMeta;

    /// Returns a system prompt fragment to be merged into the agent's system prompt.
    ///
    /// This optional string provides additional context or instructions that should
    /// be incorporated into the agent's system prompt when this skill is enabled.
    /// The fragment can include information about what the skill does, when to use
    /// its tools, and any other relevant guidance for the AI model.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn system_prompt_fragment(&self) -> Option<String> {
    ///     Some("
    /// You have access to a file manipulation skill that allows you to:
    /// - Read files from the workspace
    /// - Write files to the workspace
    /// - List directory contents
    /// Use these tools when the user asks you to work with files.
    /// " .trim()
    ///         .to_string())
    /// }
    /// ```
    fn system_prompt_fragment(&self) -> Option<String>;

    /// Returns the set of tools this skill provides.
    ///
    /// This method returns a vector of tool implementations that the skill
    /// makes available for use by AI models through function calling.
    /// Each tool is wrapped in an `Arc` for shared ownership and thread-safe
    /// access across multiple agents.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn tools(&self) -> Vec<Arc<dyn Tool>> {
    ///     vec![
    ///         Arc::new(CalculatorTool::new()),
    ///         Arc::new(FormatterTool::new()),
    ///     ]
    /// }
    /// ```
    fn tools(&self) -> Vec<Arc<dyn Tool>>;

    /// Called during skill initialization with runtime context.
    ///
    /// This async method is where skills should perform any async setup
    /// such as connecting to databases, starting background tasks, loading
    /// cached data, or validating configuration.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The [`SkillContext`] containing runtime information including
    ///   configuration and the data directory
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails. The skill will be
    /// considered failed and may be excluded from the active skill set.
    /// The error message should provide sufficient detail for debugging.
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn init(&self, ctx: &SkillContext) -> Result<(), Box<dyn Error>> {
    ///     // Load configuration
    ///     let config: MyConfig = ctx.config_as()?;
    ///
    ///     // Create data directory if it doesn't exist
    ///     std::fs::create_dir_all(&ctx.data_dir)?;
    ///
    ///     // Connect to external service
    ///     self.connect(&config.endpoint).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    async fn init(&self, ctx: &SkillContext) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_meta_debug() {
        let meta = SkillMeta::new(
            "test",
            "1.0.0",
            "test",
            crate::skills::SkillCategory::Utility,
            vec![],
            vec![],
            None,
        );
        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("SkillMeta"));
    }

    #[test]
    fn test_skill_category_clone() {
        assert_eq!(crate::skills::SkillCategory::Messaging.clone(), crate::skills::SkillCategory::Messaging);
    }
}
