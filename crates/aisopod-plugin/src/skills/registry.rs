use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::skills::Skill;

/// Status of a skill indicating its health and availability.
///
/// This enum tracks the lifecycle state of skills in the registry:
/// - `Ready`: Skill is loaded and operational
/// - `Degraded`: Skill is loaded but missing requirements
/// - `Failed`: Skill failed to initialize
/// - `Unloaded`: Skill is not loaded
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillStatus {
    /// Skill is loaded and ready.
    Ready,
    /// Skill is loaded but missing requirements (env vars, binaries).
    Degraded { reason: String },
    /// Skill failed to initialize.
    Failed { error: String },
    /// Skill is not loaded.
    Unloaded,
}

/// Central registry for managing skills.
///
/// The `SkillRegistry` is the single access point for all skill lookup
/// and lifecycle operations. It stores registered skills as `Arc<dyn Skill>`
/// keyed by ID, manages per-agent skill assignments, and provides status
/// reporting for skill health and availability.
///
/// # Key Features
///
/// - **Skill Registration**: Register skills with `register()` and look them up by ID
/// - **Agent Assignments**: Assign skills to agents and retrieve per-agent skill lists
/// - **Status Management**: Track and update skill status for health monitoring
/// - **Discovery**: Enumerate all registered skills
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::{SkillRegistry, Skill, SkillMeta, SkillCategory};
/// use aisopod_tools::Tool;
/// use async_trait::async_trait;
/// use std::sync::Arc;
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
///                 SkillCategory::Utility,
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
///     fn id(&self) -> &str { "my-skill" }
///     fn meta(&self) -> &SkillMeta { &self.meta }
///     fn system_prompt_fragment(&self) -> Option<String> { None }
///     fn tools(&self) -> Vec<Arc<dyn Tool>> { vec![] }
///     async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn Error>> { Ok(()) }
/// }
///
/// let mut registry = SkillRegistry::new();
/// let skill = Arc::new(MySkill::new()) as Arc<dyn Skill>;
/// registry.register(skill);
/// ```
pub struct SkillRegistry {
    /// Map of skill ID to skill instance.
    skills: HashMap<String, Arc<dyn Skill>>,
    /// Map of agent ID to list of assigned skill IDs.
    agent_skills: HashMap<String, Vec<String>>,
    /// Map of skill ID to its current status.
    statuses: HashMap<String, SkillStatus>,
}

impl SkillRegistry {
    /// Creates a new empty `SkillRegistry`.
    ///
    /// Returns a registry with no registered skills, no agent assignments,
    /// and no status entries.
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            agent_skills: HashMap::new(),
            statuses: HashMap::new(),
        }
    }

    /// Registers a skill with the registry.
    ///
    /// Inserts the skill into the registry and sets its initial status
    /// to `SkillStatus::Ready`. If a skill with the same ID already exists,
    /// it will be overwritten with a warning logged.
    ///
    /// # Arguments
    ///
    /// * `skill` - The skill to register, wrapped in `Arc<dyn Skill>`
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut registry = SkillRegistry::new();
    /// let skill = Arc::new(MySkill::new()) as Arc<dyn Skill>;
    /// registry.register(skill);
    /// ```
    pub fn register(&mut self, skill: Arc<dyn Skill>) {
        let id = skill.id().to_string();
        self.statuses.insert(id.clone(), SkillStatus::Ready);
        self.skills.insert(id, skill);
    }

    /// Retrieves a skill by its ID.
    ///
    /// Returns `Some(Arc<dyn Skill>)` if the skill is registered,
    /// or `None` if no skill with the given ID exists.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the skill to retrieve
    ///
    /// # Example
    ///
    /// ```ignore
    /// let skill = registry.get("my-skill");
    /// if let Some(s) = skill {
    ///     // Use the skill
    /// }
    /// ```
    pub fn get(&self, id: &str) -> Option<Arc<dyn Skill>> {
        self.skills.get(id).cloned()
    }

    /// Returns a list of all registered skill IDs.
    ///
    /// Returns a `Vec<&str>` containing the IDs of all registered skills.
    /// The order is not guaranteed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let skill_ids = registry.list();
    /// for id in skill_ids {
    ///     println!("Registered skill: {}", id);
    /// }
    /// ```
    pub fn list(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Assigns skills to an agent.
    ///
    /// Stores a mapping from the agent ID to a list of skill IDs.
    /// This does not verify that the skills exist in the registry.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The unique identifier of the agent
    /// * `skill_ids` - List of skill IDs to assign to this agent
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.assign_to_agent("agent-1", vec!["skill-1".to_string(), "skill-2".to_string()]);
    /// ```
    pub fn assign_to_agent(&mut self, agent_id: &str, skill_ids: Vec<String>) {
        self.agent_skills.insert(agent_id.to_string(), skill_ids);
    }

    /// Retrieves all skills assigned to an agent.
    ///
    /// Returns a `Vec<Arc<dyn Skill>>` containing all skills that have been
    /// assigned to the given agent. Skills that are not registered in the
    /// registry are silently skipped.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The unique identifier of the agent
    ///
    /// # Example
    ///
    /// ```ignore
    /// let agent_skills = registry.skills_for_agent("agent-1");
    /// for skill in agent_skills {
    ///     println!("Agent has skill: {}", skill.id());
    /// }
    /// ```
    pub fn skills_for_agent(&self, agent_id: &str) -> Vec<Arc<dyn Skill>> {
        self.agent_skills
            .get(agent_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.skills.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the current status of a skill.
    ///
    /// Returns `Some(&SkillStatus)` if the skill is registered,
    /// or `None` if no skill with the given ID exists.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the skill
    ///
    /// # Example
    ///
    /// ```ignore
    /// let status = registry.status("my-skill");
    /// if let Some(SkillStatus::Ready) = status {
    ///     println!("Skill is ready");
    /// }
    /// ```
    pub fn status(&self, id: &str) -> Option<&SkillStatus> {
        self.statuses.get(id)
    }

    /// Updates the status of a skill.
    ///
    /// Sets or updates the status for the given skill ID.
    /// If the skill is not yet registered, its status is still recorded.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the skill
    /// * `status` - The new status to set
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.set_status("my-skill", SkillStatus::Degraded {
    ///     reason: "Missing API key".to_string()
    /// });
    /// ```
    pub fn set_status(&mut self, id: &str, status: SkillStatus) {
        self.statuses.insert(id.to_string(), status);
    }
}

impl Default for SkillRegistry {
    /// Creates a default (empty) `SkillRegistry`.
    ///
    /// This is equivalent to calling `SkillRegistry::new()`.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::{Skill, SkillCategory, SkillMeta, SkillContext};
    use aisopod_tools::Tool;
    use async_trait::async_trait;
    use std::sync::Arc;

    // Test skill implementation
    #[derive(Debug)]
    struct TestSkill {
        meta: SkillMeta,
        id: String,
    }

    impl TestSkill {
        fn new(id: &str) -> Self {
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
            None
        }

        fn tools(&self) -> Vec<Arc<dyn Tool>> {
            vec![]
        }

        async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
    }

    #[test]
    fn test_registry_new() {
        let registry = SkillRegistry::new();
        assert!(registry.skills.is_empty());
        assert!(registry.agent_skills.is_empty());
        assert!(registry.statuses.is_empty());
    }

    #[test]
    fn test_registry_default() {
        let registry = SkillRegistry::default();
        assert!(registry.skills.is_empty());
        assert!(registry.agent_skills.is_empty());
        assert!(registry.statuses.is_empty());
    }

    #[test]
    fn test_register_skill() {
        let mut registry = SkillRegistry::new();
        let skill = Arc::new(TestSkill::new("test-skill")) as Arc<dyn Skill>;
        
        registry.register(skill.clone());
        
        assert_eq!(registry.skills.len(), 1);
        assert_eq!(registry.statuses.len(), 1);
        assert!(matches!(
            registry.statuses.get("test-skill"),
            Some(SkillStatus::Ready)
        ));
        // Verify by ID since Arc<dyn Skill> doesn't implement PartialEq
        assert!(registry.get("test-skill").is_some());
        assert_eq!(registry.get("test-skill").unwrap().id(), "test-skill");
    }

    #[test]
    fn test_register_overwrites_existing() {
        let mut registry = SkillRegistry::new();
        
        // Register first skill with id "test-skill"
        let skill1 = Arc::new(TestSkill::new("test-skill")) as Arc<dyn Skill>;
        registry.register(skill1);
        
        // Verify it's registered
        assert_eq!(registry.skills.len(), 1);
        assert_eq!(registry.get("test-skill").unwrap().id(), "test-skill");
        
        // Register second skill with DIFFERENT id - this should add it
        let skill2 = Arc::new(TestSkill::new("test-skill-2")) as Arc<dyn Skill>;
        registry.register(skill2.clone());
        
        // Now we should have 2 skills
        assert_eq!(registry.skills.len(), 2);
        
        // Test actual overwrite: register a NEW skill with EXISTING id "test-skill"
        let skill3 = Arc::new(TestSkill::new("test-skill")) as Arc<dyn Skill>;
        registry.register(skill3.clone());
        
        // Should still be 2 (overwrite, not add)
        assert_eq!(registry.skills.len(), 2);
        // The original "test-skill" should be overwritten
        assert_eq!(registry.get("test-skill").unwrap().id(), "test-skill");
    }

    #[test]
    fn test_get_skill() {
        let mut registry = SkillRegistry::new();
        let skill = Arc::new(TestSkill::new("test-skill")) as Arc<dyn Skill>;
        
        registry.register(skill.clone());
        
        // Verify by ID since Arc<dyn Skill> doesn't implement PartialEq
        assert!(registry.get("test-skill").is_some());
        assert_eq!(registry.get("test-skill").unwrap().id(), "test-skill");
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_list_skills() {
        let mut registry = SkillRegistry::new();
        
        assert!(registry.list().is_empty());
        
        let skill1 = Arc::new(TestSkill::new("skill-1")) as Arc<dyn Skill>;
        let skill2 = Arc::new(TestSkill::new("skill-2")) as Arc<dyn Skill>;
        
        registry.register(skill1);
        registry.register(skill2);
        
        let ids = registry.list();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"skill-1"));
        assert!(ids.contains(&"skill-2"));
    }

    #[test]
    fn test_assign_to_agent() {
        let mut registry = SkillRegistry::new();
        
        registry.assign_to_agent("agent-1", vec!["skill-1".to_string(), "skill-2".to_string()]);
        
        assert_eq!(registry.agent_skills.len(), 1);
        assert_eq!(
            registry.agent_skills.get("agent-1"),
            Some(&vec!["skill-1".to_string(), "skill-2".to_string()])
        );
    }

    #[test]
    fn test_skills_for_agent() {
        let mut registry = SkillRegistry::new();
        
        let skill1 = Arc::new(TestSkill::new("skill-1")) as Arc<dyn Skill>;
        let skill2 = Arc::new(TestSkill::new("skill-2")) as Arc<dyn Skill>;
        
        registry.register(skill1.clone());
        registry.register(skill2.clone());
        
        registry.assign_to_agent("agent-1", vec!["skill-1".to_string(), "skill-2".to_string()]);
        
        let agent_skills = registry.skills_for_agent("agent-1");
        assert_eq!(agent_skills.len(), 2);
        // Verify by ID since Arc<dyn Skill> doesn't implement PartialEq
        let agent_ids: Vec<&str> = agent_skills.iter().map(|s| s.id()).collect();
        assert!(agent_ids.contains(&"skill-1"));
        assert!(agent_ids.contains(&"skill-2"));
    }

    #[test]
    fn test_skills_for_agent_empty() {
        let registry = SkillRegistry::new();
        
        assert!(registry.skills_for_agent("agent-1").is_empty());
    }

    #[test]
    fn test_skills_for_agent_with_unregistered() {
        let mut registry = SkillRegistry::new();
        
        let skill1 = Arc::new(TestSkill::new("skill-1")) as Arc<dyn Skill>;
        
        registry.register(skill1.clone());
        
        registry.assign_to_agent(
            "agent-1",
            vec!["skill-1".to_string(), "skill-2".to_string()],
        );
        
        let agent_skills = registry.skills_for_agent("agent-1");
        assert_eq!(agent_skills.len(), 1);
        // Verify by ID since Arc<dyn Skill> doesn't implement PartialEq
        assert_eq!(agent_skills[0].id(), "skill-1");
    }

    #[test]
    fn test_status() {
        let mut registry = SkillRegistry::new();
        
        let skill = Arc::new(TestSkill::new("test-skill")) as Arc<dyn Skill>;
        registry.register(skill);
        
        assert!(matches!(registry.status("test-skill"), Some(SkillStatus::Ready)));
        assert!(registry.status("nonexistent").is_none());
    }

    #[test]
    fn test_set_status() {
        let mut registry = SkillRegistry::new();
        
        let skill = Arc::new(TestSkill::new("test-skill")) as Arc<dyn Skill>;
        registry.register(skill);
        
        registry.set_status("test-skill", SkillStatus::Degraded {
            reason: "Missing config".to_string(),
        });
        
        assert!(matches!(
            registry.status("test-skill"),
            Some(SkillStatus::Degraded { .. })
        ));
        
        registry.set_status("test-skill", SkillStatus::Failed {
            error: "Initialization failed".to_string(),
        });
        
        assert!(matches!(
            registry.status("test-skill"),
            Some(SkillStatus::Failed { .. })
        ));
    }

    #[test]
    fn test_status_enum_serialization() {
        use serde_json;
        
        let ready = SkillStatus::Ready;
        let degraded = SkillStatus::Degraded { reason: "test".to_string() };
        let failed = SkillStatus::Failed { error: "error".to_string() };
        let unloaded = SkillStatus::Unloaded;
        
        let ready_json = serde_json::to_string(&ready).unwrap();
        let degraded_json = serde_json::to_string(&degraded).unwrap();
        let failed_json = serde_json::to_string(&failed).unwrap();
        let unloaded_json = serde_json::to_string(&unloaded).unwrap();
        
        assert_eq!(ready_json, "\"Ready\"");
        assert!(degraded_json.contains("Degraded"));
        assert!(failed_json.contains("Failed"));
        assert_eq!(unloaded_json, "\"Unloaded\"");
    }

    #[test]
    fn test_skills_for_agent_with_multiple_agents() {
        let mut registry = SkillRegistry::new();
        
        let skill1 = Arc::new(TestSkill::new("skill-1")) as Arc<dyn Skill>;
        let skill2 = Arc::new(TestSkill::new("skill-2")) as Arc<dyn Skill>;
        let skill3 = Arc::new(TestSkill::new("skill-3")) as Arc<dyn Skill>;
        
        registry.register(skill1);
        registry.register(skill2);
        registry.register(skill3);
        
        registry.assign_to_agent("agent-1", vec!["skill-1".to_string(), "skill-2".to_string()]);
        registry.assign_to_agent("agent-2", vec!["skill-2".to_string(), "skill-3".to_string()]);
        
        let agent1_skills = registry.skills_for_agent("agent-1");
        let agent2_skills = registry.skills_for_agent("agent-2");
        
        assert_eq!(agent1_skills.len(), 2);
        assert_eq!(agent2_skills.len(), 2);
        
        let agent1_ids: Vec<&str> = agent1_skills.iter().map(|s| s.id()).collect();
        let agent2_ids: Vec<&str> = agent2_skills.iter().map(|s| s.id()).collect();
        
        assert!(agent1_ids.contains(&"skill-1"));
        assert!(agent1_ids.contains(&"skill-2"));
        assert!(agent2_ids.contains(&"skill-2"));
        assert!(agent2_ids.contains(&"skill-3"));
    }

    #[test]
    fn test_skill_registry_with_all_status_types() {
        let mut registry = SkillRegistry::new();
        
        let skill1 = Arc::new(TestSkill::new("skill-ready")) as Arc<dyn Skill>;
        let skill2 = Arc::new(TestSkill::new("skill-degraded")) as Arc<dyn Skill>;
        let skill3 = Arc::new(TestSkill::new("skill-failed")) as Arc<dyn Skill>;
        
        registry.register(skill1);
        registry.register(skill2);
        registry.register(skill3);
        
        // Set different statuses
        registry.set_status("skill-degraded", SkillStatus::Degraded {
            reason: "Missing API key".to_string(),
        });
        registry.set_status("skill-failed", SkillStatus::Failed {
            error: "Initialization failed".to_string(),
        });
        
        // Verify all statuses
        assert!(matches!(registry.status("skill-ready"), Some(SkillStatus::Ready)));
        assert!(matches!(registry.status("skill-degraded"), Some(SkillStatus::Degraded { .. })));
        assert!(matches!(registry.status("skill-failed"), Some(SkillStatus::Failed { .. })));
    }
}
