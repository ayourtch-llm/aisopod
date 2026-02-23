# Issue 117: Implement SkillRegistry for Discovery and Lifecycle

## Summary
Implement a central `SkillRegistry` struct that stores registered skills as `Arc<dyn Skill>` keyed by ID, manages per-agent skill assignments, and provides status reporting for skill health and availability. The registry is the single access point for all skill lookup and lifecycle operations.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/skills/registry.rs`

## Current Behavior
The `Skill` trait and associated types are defined (Issue 116) but there is no mechanism to register, look up, enumerate, or manage skills at runtime.

## Expected Behavior
After this issue is completed:
- A `SkillRegistry` struct holds a `HashMap<String, Arc<dyn Skill>>` of all registered skills.
- An `agent_skills` mapping (`HashMap<String, Vec<String>>`) tracks which skills are assigned to which agents.
- Skills can be registered with `register()`, looked up by ID with `get()`, and enumerated with `list()`.
- Per-agent skill lists can be retrieved with `skills_for_agent()`.
- A `SkillStatus` type reports each skill's health and availability.
- The `status()` method returns the current `SkillStatus` for a given skill.

## Impact
The registry is required by skill discovery (Issue 118), skill-agent integration (Issue 119), and all built-in skills (Issues 120–121). It is the central coordination point for the entire skills subsystem.

## Suggested Implementation
1. **Define `SkillStatus` in `registry.rs`:**
   ```rust
   use serde::{Deserialize, Serialize};

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
   ```

2. **Define `SkillRegistry`:**
   ```rust
   use std::collections::HashMap;
   use std::sync::Arc;
   use crate::skills::Skill;

   pub struct SkillRegistry {
       skills: HashMap<String, Arc<dyn Skill>>,
       agent_skills: HashMap<String, Vec<String>>,
       statuses: HashMap<String, SkillStatus>,
   }
   ```

3. **Implement `SkillRegistry::new()`** — Return an empty registry.

4. **Implement `register()`** — Accept an `Arc<dyn Skill>`, extract its `id()`, and insert it into the `skills` map. Set the initial status to `SkillStatus::Ready`. If a skill with the same ID already exists, log a warning and overwrite it.
   ```rust
   pub fn register(&mut self, skill: Arc<dyn Skill>) {
       let id = skill.id().to_string();
       self.statuses.insert(id.clone(), SkillStatus::Ready);
       self.skills.insert(id, skill);
   }
   ```

5. **Implement `get()`** — Accept a skill ID (`&str`) and return `Option<Arc<dyn Skill>>`.

6. **Implement `list()`** — Return a `Vec<&str>` of all registered skill IDs.

7. **Implement `assign_to_agent()`** — Accept an `agent_id` and a `Vec<String>` of skill IDs and store the mapping in `agent_skills`.

8. **Implement `skills_for_agent()`** — Accept an `agent_id` and return a `Vec<Arc<dyn Skill>>` by looking up each assigned skill ID in the `skills` map.
   ```rust
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
   ```

9. **Implement `status()`** — Accept a skill ID and return `Option<&SkillStatus>` from the `statuses` map.

10. **Implement `set_status()`** — Accept a skill ID and a `SkillStatus` to update the status map.

11. **Re-export from `skills/mod.rs`** — Add `pub use registry::{SkillRegistry, SkillStatus};`.

12. **Verify** — Run `cargo check -p aisopod-plugin`.

## Dependencies
- Issue 116 (Skill trait, SkillMeta, and SkillCategory types)

## Acceptance Criteria
- [x] `SkillRegistry` struct is defined and publicly accessible
- [x] `register()` adds a skill and sets its initial status to `Ready`
- [x] `get()` returns a skill by ID
- [x] `list()` returns all registered skill IDs
- [x] `skills_for_agent()` returns the correct skills assigned to an agent
- [x] `SkillStatus` enum supports `Ready`, `Degraded`, `Failed`, and `Unloaded` variants
- [x] `status()` returns the current status for a given skill
- [x] `cargo check -p aisopod-plugin` compiles without errors

## Resolution

### Implementation Summary

The `SkillRegistry` was implemented in `crates/aisopod-plugin/src/skills/registry.rs` with the following components:

1. **SkillStatus enum** - All four lifecycle states (Ready, Degraded, Failed, Unloaded) with Serde serialization support

2. **SkillRegistry struct** - Manages skills HashMap, agent_skills mapping, and statuses HashMap

3. **Core Methods**:
   - `new()` - Creates empty registry
   - `register()` - Adds skills with Ready status
   - `get()` - Retrieves skills by ID
   - `list()` - Returns all registered skill IDs
   - `assign_to_agent()` - Maps skills to agents
   - `skills_for_agent()` - Returns skills assigned to an agent
   - `status()` - Returns current skill status
   - `set_status()` - Updates skill status

4. **Re-exports** - `SkillRegistry` and `SkillStatus` re-exported from `skills/mod.rs`

5. **Comprehensive Tests** - 14 unit tests covering all functionality including edge cases

### Verification Results

- **Build**: `cargo build -p aisopod-plugin` - ✅ PASS
- **Check**: `cargo check -p aisopod-plugin` - ✅ PASS
- **Tests**: 122 tests passed (including 14 new registry tests)
- **Documentation**: `cargo doc -p aisopod-plugin` - ✅ PASS (3 warnings unrelated to registry)

### Files Modified

- `crates/aisopod-plugin/src/skills/registry.rs` - Created with full implementation
- `crates/aisopod-plugin/src/skills/mod.rs` - Added re-exports
- `docs/issues/resolved/117-implement-skill-registry.md` - Updated with resolution
- `docs/learnings/117-implement-skill-registry.md` - Created learning documentation

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
