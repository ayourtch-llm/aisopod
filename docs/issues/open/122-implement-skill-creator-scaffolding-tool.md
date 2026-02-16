# Issue 122: Implement Skill Creator Scaffolding Tool

## Summary
Implement a CLI command or tool that scaffolds new skills by generating a template directory with a `skill.toml` manifest, `lib.rs` with a basic `Skill` trait implementation, and a README file. This lowers the barrier for creating new skills.

## Location
- Crate: `aisopod` (binary crate) or `aisopod-plugin`
- File: `crates/aisopod-plugin/src/skills/scaffold.rs` (or `crates/aisopod/src/cli/scaffold_skill.rs`)

## Current Behavior
There is no automated way to create a new skill. Developers must manually create directory structures, manifest files, and trait implementations from scratch, which is error-prone and time-consuming.

## Expected Behavior
After this issue is completed:
- A `scaffold_skill()` function (or CLI subcommand) accepts a skill name, description, and category.
- It generates a complete skill directory with:
  - `skill.toml` — Pre-filled manifest with the provided metadata.
  - `src/lib.rs` — A skeleton `Skill` trait implementation that compiles.
  - `README.md` — Basic documentation with usage instructions.
- The generated skill compiles and can be loaded by the discovery system (Issue 118).

## Impact
Scaffolding reduces the time to create a new skill from minutes of manual boilerplate to a single command. It ensures consistency across skill implementations and encourages community contribution.

## Suggested Implementation
1. **Define `ScaffoldOptions`:**
   ```rust
   pub struct ScaffoldOptions {
       pub name: String,
       pub description: String,
       pub category: SkillCategory,
       pub output_dir: PathBuf,
   }
   ```

2. **Implement `scaffold_skill()`:**
   ```rust
   use std::fs;
   use std::path::PathBuf;

   pub fn scaffold_skill(opts: &ScaffoldOptions) -> Result<PathBuf, Box<dyn std::error::Error>> {
       let skill_dir = opts.output_dir.join(&opts.name);
       fs::create_dir_all(skill_dir.join("src"))?;

       // Generate skill.toml
       let manifest = format!(
           r#"id = "{name}"
   name = "{name}"
   description = "{description}"
   version = "0.1.0"
   category = "{category:?}"
   required_env_vars = []
   required_binaries = []
   "#,
           name = opts.name,
           description = opts.description,
           category = opts.category,
       );
       fs::write(skill_dir.join("skill.toml"), manifest)?;

       // Generate src/lib.rs
       let lib_rs = format!(
           r#"use async_trait::async_trait;
   use std::sync::Arc;

   pub struct {struct_name} {{
       meta: SkillMeta,
   }}

   impl {struct_name} {{
       pub fn new() -> Self {{
           Self {{
               meta: SkillMeta {{
                   name: "{name}".to_string(),
                   description: "{description}".to_string(),
                   version: "0.1.0".to_string(),
                   category: SkillCategory::{category:?},
                   required_env_vars: vec![],
                   required_binaries: vec![],
                   platform: None,
               }},
           }}
       }}
   }}

   #[async_trait]
   impl Skill for {struct_name} {{
       fn id(&self) -> &str {{
           "{name}"
       }}

       fn meta(&self) -> &SkillMeta {{
           &self.meta
       }}

       fn system_prompt_fragment(&self) -> Option<String> {{
           Some("TODO: Describe what this skill provides to the agent.".to_string())
       }}

       fn tools(&self) -> Vec<Arc<dyn Tool>> {{
           vec![]
       }}

       async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>> {{
           Ok(())
       }}
   }}
   "#,
           struct_name = to_pascal_case(&opts.name),
           name = opts.name,
           description = opts.description,
           category = opts.category,
       );
       fs::write(skill_dir.join("src/lib.rs"), lib_rs)?;

       // Generate README.md
       let readme = format!(
           r#"# {name}

   {description}

   ## Category
   {category:?}

   ## Usage
   Add `"{name}"` to your agent's `skills` list in the configuration:

   ```toml
   [[agents]]
   id = "my-agent"
   skills = ["{name}"]
   ```

   ## Development
   1. Edit `src/lib.rs` to implement your skill's tools and system prompt.
   2. Update `skill.toml` with any required environment variables or binaries.
   3. Place this directory in `~/.aisopod/skills/` to make it available.
   "#,
           name = opts.name,
           description = opts.description,
           category = opts.category,
       );
       fs::write(skill_dir.join("README.md"), readme)?;

       Ok(skill_dir)
   }
   ```

3. **Implement `to_pascal_case()` helper:**
   ```rust
   fn to_pascal_case(s: &str) -> String {
       s.split('-')
           .map(|word| {
               let mut chars = word.chars();
               match chars.next() {
                   None => String::new(),
                   Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
               }
           })
           .collect()
   }
   ```

4. **Wire into CLI** — If the binary crate has a CLI subcommand system, add a `scaffold-skill` subcommand that calls `scaffold_skill()` with arguments parsed from the command line.

5. **Verify** — Run the scaffold function, then verify the generated skill can be parsed by the manifest parser (Issue 118) and compiles as a standalone project.

## Dependencies
- Issue 116 (Skill trait, SkillMeta, and SkillCategory types)
- Issue 117 (SkillRegistry for discovery and lifecycle)

## Acceptance Criteria
- [ ] `scaffold_skill()` creates a complete skill directory structure
- [ ] Generated `skill.toml` is valid and parseable by `parse_manifest()`
- [ ] Generated `src/lib.rs` contains a compilable `Skill` trait implementation
- [ ] Generated `README.md` includes usage instructions
- [ ] Skill name, description, and category are correctly substituted into templates
- [ ] `to_pascal_case()` correctly converts kebab-case names to PascalCase struct names
- [ ] Generated skill can be loaded by the discovery system (Issue 118)

---
*Created: 2026-02-15*
