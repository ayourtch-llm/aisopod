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

## Resolution
The implementation was completed with the following files and changes:

### New Files Created
1. **`crates/aisopod-plugin/src/skills/manifest.rs`**
   - `SkillManifest` struct with all required fields (id, name, description, version, category, required_env_vars, required_binaries, platform)
   - `ManifestError` enum for error handling
   - `parse_manifest()` function to read and parse `skill.toml` files
   - `validate()` method for manifest validation
   - Comprehensive unit tests (10 tests)

2. **`crates/aisopod-plugin/src/skills/discovery.rs`**
   - `DiscoveryError` enum for discovery-related errors
   - `DiscoveredSkill` struct to hold discovered skill information
   - `discover_skill_dirs()` function to scan directories for skills with valid manifests
   - `validate_requirements()` function to check environment variables, binaries, and platform constraints
   - `is_binary_available()` helper function using platform-appropriate binary checking
   - `load_skills()` async function that orchestrates the full discovery-validate-register pipeline
   - Skills with unmet requirements are registered with `SkillStatus::Degraded` instead of failing
   - Comprehensive unit tests (12 tests)

3. **`crates/aisopod-plugin/src/skills/mod.rs`**
   - Added `mod manifest` and `mod discovery` module declarations
   - Re-exported `SkillManifest`, `ManifestError`, `parse_manifest`
   - Re-exported `discover_skill_dirs`, `validate_requirements`, `load_skills`
   - Updated documentation to include manifest and discovery types

### Changes to Existing Files
1. **`crates/aisopod-plugin/Cargo.toml`**
   - Added feature gates for built-in skills: `skill-healthcheck`, `skill-session-logs`, `skill-model-usage`
   - Added `all-skills` meta-feature that enables all skill plugins
   - Added `futures` to dev-dependencies for testing

2. **`Cargo.toml` (workspace)**
   - Added `futures = "0.3"` to workspace dependencies

### Acceptance Criteria Status
- [x] `SkillManifest` struct is defined and can be parsed from `skill.toml` files
- [x] `discover_skill_dirs()` scans directories and finds skills with valid manifests
- [x] `validate_requirements()` checks environment variables, binaries, and platform constraints
- [x] Skills with unmet requirements are registered with `Degraded` status instead of failing
- [x] Feature gates control inclusion of built-in skills at compile time
- [x] `load_skills()` orchestrates the full discovery-validate-register pipeline
- [x] `cargo check -p aisopod-plugin` compiles without errors

### Testing Results
- All 142 unit tests pass (12 new tests for the discovery module)
- All 22 integration tests pass
- All 56 doc tests pass
- Documentation builds without errors

### Usage Example
```rust
use aisopod_plugin::skills::{SkillRegistry, SkillContext, load_skills};
use std::path::PathBuf;

let mut registry = SkillRegistry::new();
let context = SkillContext::new(config, data_dir, agent_id);
let base_dirs = vec![PathBuf::from("~/.aisopod/skills")];

match load_skills(&mut registry, &base_dirs, &context).await {
    Ok(statuses) => {
        for (skill_id, status) in statuses {
            println!("Skill {} status: {:?}", skill_id, status);
        }
    }
    Err(e) => eprintln!("Discovery failed: {}", e),
}
```

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
