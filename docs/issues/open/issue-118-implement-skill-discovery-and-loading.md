# Issue 118: Implement Skill Discovery and Loading

## Summary
Implement the skill discovery and loading pipeline that scans skill directories, parses manifest files, validates requirements, and registers discovered skills into the `SkillRegistry`. This enables both built-in (feature-gated) and user-installed skills to be automatically found and loaded at startup.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/skills/discovery.rs`, `crates/aisopod-plugin/src/skills/manifest.rs`

## Current Behavior
The `SkillRegistry` exists (Issue 117) but skills must be manually registered. There is no automatic discovery from the filesystem, no manifest parsing, and no requirement validation.

## Expected Behavior
After this issue is completed:
- A `SkillManifest` struct is parsed from `skill.toml` files found in skill directories.
- The discovery system scans `~/.aisopod/skills/` for user-installed skills and a built-in skills directory for compiled-in skills.
- Feature-gated built-in skills are conditionally included at compile time.
- Requirement validation checks that required environment variables are set, required binaries are on `$PATH`, and platform constraints are satisfied.
- Discovered and validated skills are automatically registered into the `SkillRegistry`.

## Impact
Without discovery and loading, skills must be manually wired into the system. This issue enables the plug-and-play experience where dropping a skill into a directory makes it available automatically.

## Suggested Implementation
1. **Define `SkillManifest` in `manifest.rs`:**
   ```rust
   use serde::Deserialize;
   use crate::skills::SkillCategory;

   #[derive(Debug, Deserialize)]
   pub struct SkillManifest {
       pub id: String,
       pub name: String,
       pub description: String,
       pub version: String,
       pub category: SkillCategory,
       #[serde(default)]
       pub required_env_vars: Vec<String>,
       #[serde(default)]
       pub required_binaries: Vec<String>,
       pub platform: Option<String>,
   }
   ```
   Add a function `parse_manifest(path: &Path) -> Result<SkillManifest>` that reads and parses a `skill.toml` file using `toml::from_str`.

2. **Implement requirement validation in `discovery.rs`:**
   ```rust
   use std::env;
   use std::process::Command;

   pub fn validate_requirements(manifest: &SkillManifest) -> Result<(), Vec<String>> {
       let mut errors = Vec::new();

       // Check environment variables
       for var in &manifest.required_env_vars {
           if env::var(var).is_err() {
               errors.push(format!("Missing environment variable: {}", var));
           }
       }

       // Check required binaries
       for bin in &manifest.required_binaries {
           if Command::new("which").arg(bin).output().map(|o| !o.status.success()).unwrap_or(true) {
               errors.push(format!("Missing required binary: {}", bin));
           }
       }

       // Check platform constraint
       if let Some(ref platform) = manifest.platform {
           if platform != std::env::consts::OS {
               errors.push(format!("Platform mismatch: requires {}, running {}", platform, std::env::consts::OS));
           }
       }

       if errors.is_empty() { Ok(()) } else { Err(errors) }
   }
   ```

3. **Implement directory scanning in `discovery.rs`:**
   ```rust
   use std::path::{Path, PathBuf};

   pub fn discover_skill_dirs(base_dirs: &[PathBuf]) -> Vec<PathBuf> {
       let mut skill_dirs = Vec::new();
       for base in base_dirs {
           if let Ok(entries) = std::fs::read_dir(base) {
               for entry in entries.flatten() {
                   let path = entry.path();
                   if path.is_dir() && path.join("skill.toml").exists() {
                       skill_dirs.push(path);
                   }
               }
           }
       }
       skill_dirs
   }
   ```

4. **Implement the top-level `load_skills()` function:**
   ```rust
   pub async fn load_skills(
       registry: &mut SkillRegistry,
       base_dirs: &[PathBuf],
   ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
       let mut loaded = Vec::new();
       let dirs = discover_skill_dirs(base_dirs);
       for dir in dirs {
           let manifest = parse_manifest(&dir.join("skill.toml"))?;
           match validate_requirements(&manifest) {
               Ok(()) => {
                   // Load and register the skill (implementation-specific)
                   loaded.push(manifest.id.clone());
               }
               Err(reasons) => {
                   // Register with Degraded status
                   registry.set_status(
                       &manifest.id,
                       SkillStatus::Degraded { reason: reasons.join("; ") },
                   );
               }
           }
       }
       Ok(loaded)
   }
   ```

5. **Add feature gates for built-in skills** — In `Cargo.toml`:
   ```toml
   [features]
   skill-healthcheck = []
   skill-session-logs = []
   skill-model-usage = []
   default = ["skill-healthcheck", "skill-session-logs", "skill-model-usage"]
   ```

6. **Re-export from `skills/mod.rs`** — Add `pub use discovery::load_skills;` and `pub use manifest::SkillManifest;`.

7. **Verify** — Run `cargo check -p aisopod-plugin`.

## Dependencies
- Issue 117 (SkillRegistry for discovery and lifecycle)

## Acceptance Criteria
- [ ] `SkillManifest` struct is defined and can be parsed from `skill.toml` files
- [ ] `discover_skill_dirs()` scans directories and finds skills with valid manifests
- [ ] `validate_requirements()` checks environment variables, binaries, and platform constraints
- [ ] Skills with unmet requirements are registered with `Degraded` status instead of failing
- [ ] Feature gates control inclusion of built-in skills at compile time
- [ ] `load_skills()` orchestrates the full discovery-validate-register pipeline
- [ ] `cargo check -p aisopod-plugin` compiles without errors

---
*Created: 2026-02-15*
