//! Skill-agent integration functions.
//!
//! This module provides functions to integrate skills with agents:
//! - Resolving skill references from agent configuration
//! - Merging skill system prompt fragments into agent prompts
//! - Collecting tools from assigned skills

use std::sync::Arc;

/// Trait for skills that can be integrated with agents.
///
/// This trait is designed to be compatible with aisopod_plugin::skills::Skill
/// but without creating a circular dependency. It mirrors the key methods
/// needed by the agent.
pub trait Skill: Send + Sync + std::fmt::Debug {
    /// Returns the unique identifier for this skill.
    fn id(&self) -> &str;

    /// Returns metadata describing this skill.
    fn meta(&self) -> SkillMeta;

    /// Returns a system prompt fragment to be merged into the agent's system prompt.
    fn system_prompt_fragment(&self) -> Option<String>;

    /// Returns the set of tools this skill provides.
    fn tools(&self) -> Vec<Arc<dyn Tool>>;
}

/// Metadata describing a skill's identity and requirements.
#[derive(Debug, Clone)]
pub struct SkillMeta {
    /// The unique identifier for this skill.
    pub name: String,
    /// The version of this skill following semantic versioning.
    pub version: String,
    /// A brief description of what this skill does.
    pub description: String,
}

impl SkillMeta {
    /// Creates a new `SkillMeta` instance.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
        }
    }
}

/// Runtime context provided to skills during initialization.
pub struct SkillContext {
    /// The path to a dedicated data directory for this skill.
    pub data_dir: std::path::PathBuf,
    /// Optional identifier for the agent that will use this skill.
    pub agent_id: Option<String>,
}

impl SkillContext {
    /// Creates a new `SkillContext` instance.
    pub fn new(data_dir: std::path::PathBuf, agent_id: Option<String>) -> Self {
        Self { data_dir, agent_id }
    }
}

/// A minimal re-export of the Tool trait from aisopod-tools.
///
/// This is used by the Skill trait to avoid importing from aisopod-plugin.
use aisopod_tools::Tool;

/// Resolves skills for an agent from the skill registry.
///
/// Given an agent configuration and the skill registry, this function
/// retrieves all skill references that have been assigned to the agent.
///
/// # Arguments
///
/// * `agent_config` - The agent configuration containing the skills list
/// * `registry` - The skill registry containing registered skills
///
/// # Returns
///
/// Returns a vector of `Arc<dyn Skill>` references for all skills assigned
/// to the agent. Skills that are not registered in the registry are silently skipped.
pub fn resolve_agent_skills(
    agent_config: &aisopod_config::types::Agent,
    registry: &SkillRegistry,
) -> Vec<Arc<dyn Skill>> {
    agent_config
        .skills
        .iter()
        .filter_map(|id| registry.get(id))
        .collect()
}

/// Merges skill system prompt fragments into the agent's base system prompt.
///
/// This function collects all system prompt fragments from the assigned skills
/// and appends them to the agent's base system prompt.
///
/// # Arguments
///
/// * `base_prompt` - The agent's base system prompt
/// * `skills` - A slice of skill references
///
/// # Returns
///
/// Returns the merged system prompt with skill fragments appended.
/// Each fragment is separated by two newlines from the base prompt and from each other.
pub fn merge_skill_prompts(base_prompt: &str, skills: &[Arc<dyn Skill>]) -> String {
    let mut prompt = base_prompt.to_string();
    for skill in skills {
        if let Some(fragment) = skill.system_prompt_fragment() {
            prompt.push_str("\n\n");
            prompt.push_str(&fragment);
        }
    }
    prompt
}

/// Collects all tools from assigned skills.
///
/// This function gathers all tools provided by the assigned skills
/// into a single vector.
///
/// # Arguments
///
/// * `skills` - A slice of skill references
///
/// # Returns
///
/// Returns a vector of `Arc<dyn Tool>` references from all assigned skills.
pub fn collect_skill_tools(skills: &[Arc<dyn Skill>]) -> Vec<Arc<dyn aisopod_tools::Tool>> {
    skills.iter().flat_map(|skill| skill.tools()).collect()
}

/// A skill registry implementation for agent skill management.
///
/// This registry stores skills and allows them to be assigned to agents.
#[derive(Default)]
pub struct SkillRegistry {
    /// Map of skill ID to skill instance.
    skills: std::collections::HashMap<String, Arc<dyn Skill>>,
}

impl SkillRegistry {
    /// Creates a new empty `SkillRegistry`.
    pub fn new() -> Self {
        Self {
            skills: std::collections::HashMap::new(),
        }
    }

    /// Registers a skill with the registry.
    pub fn register(&mut self, skill: Arc<dyn Skill>) {
        let id = skill.id().to_string();
        self.skills.insert(id, skill);
    }

    /// Retrieves a skill by its ID.
    pub fn get(&self, id: &str) -> Option<Arc<dyn Skill>> {
        self.skills.get(id).cloned()
    }

    /// Returns a list of all registered skill IDs.
    pub fn list(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }
}
