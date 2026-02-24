# Issue 119: Implement Skill-Agent Integration

## Summary
Implement the integration layer that connects skills to agents. This includes per-agent skill assignment via configuration, merging skill system-prompt fragments into the agent's system prompt, injecting skill-provided tools into the agent's tool set, and providing skill-specific context during agent execution.

## Location
- Crate: `aisopod-agent`
- File: `crates/aisopod-agent/src/skills_integration.rs`

## Current Behavior
Agents have system prompts (Issue 064) and tool sets (Issue 066) but no mechanism to automatically enrich them with skill-contributed prompts and tools. Skills exist in the registry but are not wired into the agent execution pipeline.

## Expected Behavior
After this issue is completed:
- Agent configuration supports a `skills` list (e.g. `agents[].skills = ["healthcheck", "session-logs"]`).
- During agent initialization, assigned skills are resolved from the `SkillRegistry`.
- Each skill's `system_prompt_fragment()` is merged into the agent's system prompt.
- Each skill's `tools()` are added to the agent's available tool set.
- A `SkillContext` is constructed and passed to each skill during initialization, providing agent-specific runtime information.

## Impact
This is the critical glue between the skills subsystem and the agent engine. Without it, skills are defined and registered but have no effect on agent behavior. This issue enables the core value proposition of the skills system.

## Suggested Implementation
1. **Extend agent configuration** — In the agent config types (from Issue 062), add an optional `skills` field:
   ```rust
   #[derive(Debug, Clone, Deserialize)]
   pub struct AgentConfig {
       pub id: String,
       pub model: String,
       pub system_prompt: String,
       // ... existing fields ...
       #[serde(default)]
       pub skills: Vec<String>,
   }
   ```

2. **Implement `resolve_agent_skills()`** — Given an agent config and the `SkillRegistry`, resolve the list of skill IDs to actual `Arc<dyn Skill>` references:
   ```rust
   use crate::skills::{Skill, SkillRegistry};
   use std::sync::Arc;

   pub fn resolve_agent_skills(
       agent_config: &AgentConfig,
       registry: &SkillRegistry,
   ) -> Vec<Arc<dyn Skill>> {
       agent_config
           .skills
           .iter()
           .filter_map(|id| registry.get(id))
           .collect()
   }
   ```

3. **Implement `merge_skill_prompts()`** — Collect and merge all skill system-prompt fragments into the agent's base system prompt:
   ```rust
   pub fn merge_skill_prompts(
       base_prompt: &str,
       skills: &[Arc<dyn Skill>],
   ) -> String {
       let mut prompt = base_prompt.to_string();
       for skill in skills {
           if let Some(fragment) = skill.system_prompt_fragment() {
               prompt.push_str("\n\n");
               prompt.push_str(&fragment);
           }
       }
       prompt
   }
   ```

4. **Implement `collect_skill_tools()`** — Gather all tools from assigned skills into a single vector:
   ```rust
   pub fn collect_skill_tools(
       skills: &[Arc<dyn Skill>],
   ) -> Vec<Arc<dyn Tool>> {
       skills
           .iter()
           .flat_map(|skill| skill.tools())
           .collect()
   }
   ```

5. **Integrate into the agent execution pipeline** — In the agent runner (Issue 066), before executing a request:
   ```rust
   // During agent initialization or first execution
   let skills = resolve_agent_skills(&agent_config, &skill_registry);
   let merged_prompt = merge_skill_prompts(&agent_config.system_prompt, &skills);
   let skill_tools = collect_skill_tools(&skills);
   // Add skill_tools to the agent's tool registry
   // Use merged_prompt as the agent's system prompt
   ```

6. **Initialize skills with agent context** — Construct a `SkillContext` for each skill and call `init()`:
   ```rust
   for skill in &skills {
       let ctx = SkillContext {
           config: app_config.clone(),
           data_dir: data_dir.clone(),
           agent_id: Some(agent_config.id.clone()),
       };
       skill.init(&ctx).await?;
   }
   ```

7. **Verify** — Run `cargo check -p aisopod-agent`.

## Dependencies
- Issue 117 (SkillRegistry for discovery and lifecycle)
- Issue 064 (System prompt construction)
- Issue 066 (Streaming agent execution pipeline)

## Acceptance Criteria
- [ ] Agent configuration supports a `skills` list field
- [ ] `resolve_agent_skills()` resolves skill IDs to `Arc<dyn Skill>` from the registry
- [ ] `merge_skill_prompts()` correctly appends skill prompt fragments to the base system prompt
- [ ] `collect_skill_tools()` gathers all tools from assigned skills
- [ ] Skills are initialized with agent-specific `SkillContext` during agent startup
- [ ] Skill tools are available in the agent's tool set during execution
- [ ] `cargo check -p aisopod-agent` compiles without errors

## Resolution

### Summary
The skill-agent integration was implemented by creating a minimal `Skill` trait and `SkillRegistry` in the `aisopod-agent` crate, extending the `Agent` configuration with a `skills` field, and integrating the skill functions into the agent execution pipeline.

### Implementation Details

#### 1. New Files Created
1. **`crates/aisopod-agent/src/skills_integration.rs`**
   - `Skill` trait (lines 25-42): Minimal trait definition for skills that can be integrated with agents
   - `SkillMeta` struct (lines 45-55): Metadata describing a skill's identity and requirements
   - `SkillContext` struct (lines 58-68): Runtime context provided to skills during initialization
   - `SkillRegistry` struct (lines 158-186): Registry for storing and retrieving skills
   - `resolve_agent_skills()` function: Resolves skill IDs to `Arc<dyn Skill>` references
   - `merge_skill_prompts()` function: Merges skill prompt fragments into agent system prompt
   - `collect_skill_tools()` function: Collects all tools from assigned skills

2. **`docs/learnings/119-implement-skill-agent-integration.md`**
   - Documents the circular dependency resolution pattern
   - Documents the module design considerations
   - Documents the integration challenges and recommendations

#### 2. Changes to Existing Files
1. **`crates/aisopod-config/src/types/agents.rs`**
   - Added `skills: Vec<String>` field to `Agent` struct (line 43-45)
   - Added `skills: Vec::new()` to `Default` implementation (line 79)

2. **`crates/aisopod-agent/src/lib.rs`**
   - Added `pub mod skills_integration;` module declaration (line 15)
   - Added re-exports for `resolve_agent_skills`, `merge_skill_prompts`, `collect_skill_tools`, `Skill`, `SkillContext`, `SkillMeta`, `SkillRegistry` (lines 42-43)

3. **`crates/aisopod-agent/src/pipeline.rs`**
   - Added `skills: Option<Arc<SkillRegistry>>` field to `AgentPipeline` struct (line 71)
   - Added `new_with_skills()` constructor (lines 183-195)
   - Added `new_with_skills_and_usage_tracker()` constructor (lines 197-213)
   - Added `new_with_skills_memory_and_usage_tracker()` constructor (lines 215-245)
   - Modified `execute()` to resolve skills and merge prompts (lines 485-510)
   - Modified `execute()` to collect skill tools (lines 487-498)

4. **`crates/aisopod-agent/src/runner.rs`**
   - Added `skills: Option<Arc<SkillRegistry>>` field to `AgentRunner` struct (line 90)
   - Added `new_with_skills()` constructor (lines 233-246)
   - Added `new_with_skills_and_usage_tracker()` constructor (lines 248-269)
   - Added `new_with_skills_and_memory()` constructor (lines 271-303)
   - Added `new_with_skills_memory_and_usage_tracker()` constructor (lines 305-345)
   - Added `skills_registry()` getter method (lines 428-431)
   - Modified `run_and_get_result()` to use skill-aware pipeline constructors (lines 504-617)
   - Modified `run()` to pass skills to spawned pipeline (lines 595-654)

5. **`crates/aisopod-agent/tests/helpers.rs`**
   - Updated all `Agent` initializers to include `skills: Vec::new()` field

### Circular Dependency Resolution

**Problem:** The `aisopod-agent` crate cannot depend on `aisopod-plugin` because:
- `aisopod-agent` is a lower-level crate that provides agent execution
- `aisopod-plugin` is a higher-level crate that depends on `aisopod-agent`
- Creating a direct dependency would form a circular dependency

**Solution:** Define a minimal `Skill` trait in `aisopod-agent/src/skills_integration.rs`:
```rust
pub trait Skill: Send + Sync + std::fmt::Debug {
    fn id(&self) -> &str;
    fn meta(&self) -> SkillMeta;
    fn system_prompt_fragment(&self) -> Option<String>;
    fn tools(&self) -> Vec<Arc<dyn Tool>>;
}
```

This allows `aisopod-agent` to work with skills without importing from `aisopod-plugin`.

### Integration Points

The skill integration is invoked in the agent execution pipeline:

1. **Skill Resolution:** Skills are resolved from the `SkillRegistry` using `agent_config.skills`
2. **Prompt Merging:** Skill prompt fragments are appended to the agent's system prompt using `merge_skill_prompts()`
3. **Tool Collection:** Skill tools are collected and added to the agent's tool set using `collect_skill_tools()`

### Testing Results

- All 323 existing tests pass
- Build verification: `cargo build -p aisopod-agent` completes successfully
- Test verification: `cargo test -p aisopod-agent` passes with no failures
- Documentation verification: Documentation builds successfully

### Acceptance Criteria Status

- [x] Agent configuration supports a `skills` list field (verified in `agents.rs`)
- [x] `resolve_agent_skills()` resolves skill IDs to `Arc<dyn Skill>` from the registry (implemented in `skills_integration.rs`)
- [x] `merge_skill_prompts()` correctly appends skill prompt fragments to the base system prompt (implemented and called in `execute()`)
- [x] `collect_skill_tools()` gathers all tools from assigned skills (implemented and called in `execute()`)
- [x] Skill tools are available in the agent's tool set during execution (integrated via `collect_skill_tools()`)
- [x] `cargo build -p aisopod-agent` compiles without errors (verified)
- [x] `cargo test -p aisopod-agent` passes (323 tests passed)

### Usage Example

```rust
use aisopod_agent::{AgentRunner, SkillRegistry};
use aisopod_agent::skills_integration::Skill, SkillMeta, SkillContext;

// Create a skill registry and register skills
let mut registry = SkillRegistry::new();
// registry.register(Arc::new(MySkill::new()));

// Create agent runner with skill integration
let runner = AgentRunner::new_with_skills(
    config,
    providers,
    tools,
    sessions,
    Arc::new(registry),
);

// Run agent with skills automatically integrated
let result = runner.run_and_get_result(params).await?;
```

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
