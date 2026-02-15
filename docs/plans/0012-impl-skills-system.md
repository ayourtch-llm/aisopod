# 0012 — Skills System

**Master Plan Reference:** Section 3.10 — Skills System  
**Phase:** 5 (Extensibility)  
**Dependencies:** 0005 (Tool System), 0006 (Agent Engine), 0011 (Plugin System)

---

## Objective

Implement the skills framework that allows AI agents to be extended with
domain-specific capabilities, matching OpenClaw's 43-skill ecosystem.

---

## Deliverables

### 1. Skill Interface

```rust
pub trait Skill: Send + Sync {
    /// Skill unique identifier
    fn id(&self) -> &str;

    /// Skill metadata
    fn meta(&self) -> &SkillMeta;

    /// System prompt additions for the agent
    fn system_prompt_fragment(&self) -> Option<&str>;

    /// Tools provided by this skill
    fn tools(&self) -> Vec<Arc<dyn Tool>>;

    /// Initialize the skill
    fn init(&self, ctx: &SkillContext) -> Result<()> { Ok(()) }
}

pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: SkillCategory,
    pub required_env_vars: Vec<String>,
    pub required_binaries: Vec<String>,
    pub platform: Option<Platform>,
}

pub enum SkillCategory {
    Messaging,
    Productivity,
    AiMl,
    Integration,
    Utility,
    System,
}
```

### 2. Skill Discovery & Loading

- Scan skill directories (`~/.aisopod/skills/`, built-in)
- Manifest-based skill definition (skill metadata + requirements)
- Feature-gated built-in skills
- Requirement validation (env vars, binaries, OS)

### 3. Skill Registry

```rust
pub struct SkillRegistry {
    skills: HashMap<String, Arc<dyn Skill>>,
    agent_skills: HashMap<String, Vec<String>>,  // agent_id -> skill_ids
}

impl SkillRegistry {
    pub fn register(&mut self, skill: Arc<dyn Skill>) -> Result<()>;
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Skill>>;
    pub fn list(&self) -> Vec<&Arc<dyn Skill>>;
    pub fn skills_for_agent(&self, agent_id: &str) -> Vec<&Arc<dyn Skill>>;
    pub fn status(&self) -> Vec<SkillStatus>;
}
```

### 4. Priority Skills to Port

**Tier 1 (essential):**
- `healthcheck` — System health monitoring
- `session-logs` — Session log access
- `model-usage` — Usage tracking and reporting
- `coding-agent` — Code generation/editing assistance
- `summarize` — Text summarization

**Tier 2 (popular integrations):**
- `github` — GitHub API integration
- `weather` — Weather information
- `openai-image-gen` — Image generation
- `openai-whisper` — Audio transcription

**Tier 3 (platform-specific):**
- Apple notes, Bear notes, Notion, Obsidian, Trello, etc.
- These may be implemented later or as community contributions

### 5. Skill-Agent Integration

- Per-agent skill assignment via config
- Skill system prompt fragments merged into agent prompt
- Skill tools added to agent tool set
- Skill-specific context injection

### 6. Skill Creator

- Tool/command to scaffold new skills
- Template generation with manifest and basic implementation
- Documentation generation

---

## Acceptance Criteria

- [ ] Skill trait is well-defined and implementable
- [ ] Skill registry manages discovery and lifecycle
- [ ] Tier 1 skills are implemented and functional
- [ ] Skill system prompt fragments merge correctly
- [ ] Skill tools appear in agent tool sets
- [ ] Per-agent skill assignment works via config
- [ ] Skill status reporting works
- [ ] Skill creator generates valid scaffolding
- [ ] Unit tests cover skill registration and integration
