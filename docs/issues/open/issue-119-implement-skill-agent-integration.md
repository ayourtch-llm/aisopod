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

---
*Created: 2026-02-15*
