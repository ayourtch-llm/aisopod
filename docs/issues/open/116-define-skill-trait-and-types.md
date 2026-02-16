# Issue 116: Define Skill Trait, SkillMeta, and SkillCategory Types

## Summary
Define the core `Skill` trait, `SkillMeta` struct, `SkillCategory` enum, and `SkillContext` struct in the `aisopod-plugin` crate (or a new `aisopod-skills` module within it). These types form the foundation of the skills system, establishing the interface that every skill must implement and the metadata that describes each skill's identity and capabilities.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/skills/trait.rs`, `crates/aisopod-plugin/src/skills/meta.rs`, `crates/aisopod-plugin/src/skills/context.rs`

## Current Behavior
The `aisopod-plugin` crate has the `Plugin` trait (Issue 107) and tool system types (Issue 049) but no skill abstraction exists. There is no way to define reusable bundles of system-prompt fragments and tools that can be assigned to agents.

## Expected Behavior
After this issue is completed:
- A `Skill` async trait is defined with methods: `id()`, `meta()`, `system_prompt_fragment()`, `tools()`, and `init()`.
- A `SkillMeta` struct describes each skill's name, description, version, category, required environment variables, required binaries, and platform constraints.
- A `SkillCategory` enum classifies skills into: `Messaging`, `Productivity`, `AiMl`, `Integration`, `Utility`, and `System`.
- A `SkillContext` struct provides runtime context to skills during initialization.
- All types are well-documented with rustdoc comments.

## Impact
Every other skills system issue depends on these type definitions. The `Skill` trait is the single most foundational type in the skills subsystem — nothing else in plan 0012 can proceed without it.

## Suggested Implementation
1. **Create the `skills` module directory** — Add `crates/aisopod-plugin/src/skills/mod.rs` and the individual files.

2. **Define `SkillCategory` in `meta.rs`:**
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
   pub enum SkillCategory {
       Messaging,
       Productivity,
       AiMl,
       Integration,
       Utility,
       System,
   }
   ```

3. **Define `SkillMeta` in `meta.rs`:**
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct SkillMeta {
       pub name: String,
       pub description: String,
       pub version: String,
       pub category: SkillCategory,
       pub required_env_vars: Vec<String>,
       pub required_binaries: Vec<String>,
       pub platform: Option<String>,
   }
   ```
   Add a doc comment explaining that `required_env_vars` lists environment variables the skill needs at runtime, `required_binaries` lists executables that must be on `$PATH`, and `platform` optionally constrains the skill to a specific OS (e.g. `"linux"`, `"macos"`).

4. **Define `SkillContext` in `context.rs`:**
   ```rust
   use std::sync::Arc;
   use std::path::PathBuf;

   /// Runtime context provided to skills during initialization.
   pub struct SkillContext {
       pub config: Arc<serde_json::Value>,
       pub data_dir: PathBuf,
       pub agent_id: Option<String>,
   }
   ```

5. **Define the `Skill` trait in `trait.rs`:**
   ```rust
   use async_trait::async_trait;
   use crate::skills::{SkillMeta, SkillContext};

   #[async_trait]
   pub trait Skill: Send + Sync {
       /// Returns the unique identifier for this skill.
       fn id(&self) -> &str;

       /// Returns metadata describing this skill.
       fn meta(&self) -> &SkillMeta;

       /// Returns a system prompt fragment to be merged into the agent's system prompt.
       fn system_prompt_fragment(&self) -> Option<String>;

       /// Returns the set of tools this skill provides.
       fn tools(&self) -> Vec<Arc<dyn Tool>>;

       /// Called during skill initialization with runtime context.
       async fn init(&self, ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>>;
   }
   ```

6. **Re-export from `skills/mod.rs`:**
   ```rust
   mod r#trait;
   mod meta;
   mod context;

   pub use r#trait::Skill;
   pub use meta::{SkillMeta, SkillCategory};
   pub use context::SkillContext;
   ```

7. **Re-export `skills` module from `lib.rs`** — Add `pub mod skills;` to the crate root.

8. **Verify** — Run `cargo check -p aisopod-plugin` to confirm everything compiles.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 107 (Plugin trait and PluginMeta types)

## Acceptance Criteria
- [ ] `Skill` trait is defined with `id()`, `meta()`, `system_prompt_fragment()`, `tools()`, and `init()` methods
- [ ] `SkillMeta` struct includes name, description, version, category, required_env_vars, required_binaries, and platform fields
- [ ] `SkillCategory` enum includes `Messaging`, `Productivity`, `AiMl`, `Integration`, `Utility`, and `System` variants
- [ ] `SkillContext` struct provides runtime context including config, data directory, and optional agent ID
- [ ] All public types and methods have rustdoc documentation comments
- [ ] `cargo check -p aisopod-plugin` compiles without errors
- [ ] `cargo doc -p aisopod-plugin` generates documentation without warnings

---
*Created: 2026-02-15*
