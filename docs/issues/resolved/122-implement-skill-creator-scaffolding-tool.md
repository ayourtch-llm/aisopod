# Issue 122: Implement Skill Creator Scaffolding Tool

## Summary
Implement a CLI command or tool that scaffolds new skills by generating a template directory with a `skill.toml` manifest, `lib.rs` with a basic `Skill` trait implementation, and a README file. This lowers the barrier for creating new skills.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/skills/scaffold.rs`
- CLI: `crates/aisopod/src/cli/mod.rs`

## Current Behavior
There was no automated way to create a new skill. Developers must manually create directory structures, manifest files, and trait implementations from scratch, which is error-prone and time-consuming.

## Expected Behavior
After this issue is completed:
- A `scaffold_skill()` function accepts a skill name, description, and category.
- It generates a complete skill directory with:
  - `skill.toml` — Pre-filled manifest with the provided metadata.
  - `src/lib.rs` — A skeleton `Skill` trait implementation that compiles.
  - `README.md` — Basic documentation with usage instructions.
- The generated skill compiles and can be loaded by the discovery system (Issue 118).

## Resolution

### Changes Made

1. **Created `scaffold.rs` module** (`crates/aisopod-plugin/src/skills/scaffold.rs`)
   - Implemented `ScaffoldOptions` struct with name, description, category, and output_dir
   - Implemented `scaffold_skill()` function that generates complete skill directory structure
   - Implemented `to_pascal_case()` helper function for converting kebab-case to PascalCase

2. **Updated `mod.rs`** (`crates/aisopod-plugin/src/skills/mod.rs`)
   - Added `pub use` for scaffold functions and types
   - Re-exported `scaffold_skill`, `ScaffoldOptions`, and `to_pascal_case`

3. **Updated `lib.rs`** (`crates/aisopod-plugin/src/lib.rs`)
   - Added `scaffold` module to public exports
   - Added re-exports for scaffolding functionality

4. **Implemented CLI subcommand** (`crates/aisopod/src/cli/mod.rs`)
   - Added `ScaffoldSkillArgs` for command-line argument parsing
   - Implemented `run_scaffold_skill()` function to handle CLI invocation
   - Added category validation and path expansion for `~` support

5. **Fixed scaffold template** (`crates/aisopod-plugin/src/skills/scaffold.rs`)
   - Added `#[derive(Debug)]` to generated struct definition
   - Critical fix: The `Skill` trait requires `Debug` trait bound

### Key Implementation Details

**Scaffold Output Structure:**
```
{skill-name}/
├── skill.toml      # TOML manifest with metadata
├── src/
│   └── lib.rs      # Skill implementation with Skill trait
└── README.md       # Documentation with usage instructions
```

**Generated Files:**
- `skill.toml`: Contains id, name, description, version, category, required_env_vars, required_binaries
- `src/lib.rs`: Includes Debug derive, Skill trait implementation with all required methods
- `README.md`: Documents category, usage, and development steps

**CLI Usage:**
```bash
aisopod scaffold-skill \
  --name my-skill \
  --description "My skill description" \
  --category utility \
  --output-dir ~/.aisopod/skills
```

### Verification Results

All acceptance criteria met:

✅ **scaffold_skill() creates a complete skill directory structure**
- Creates `{skill-name}/` directory with src/ subdirectory
- Generates all three required files: skill.toml, src/lib.rs, README.md

✅ **Generated skill.toml is valid and parseable by parse_manifest()**
- Manifest correctly uses TOML format
- All required fields (id, name, description, version, category) are present
- Parse test: `test_scaffold_skill_manifest_content`

✅ **Generated src/lib.rs contains a compilable Skill trait implementation**
- Includes all required imports (async_trait, Skill, SkillMeta, SkillCategory)
- Struct has Debug derive (critical fix)
- Default trait implementation included
- Skill trait implementation with all required methods
- Compiles as standalone project: verified in `/tmp/test_lib_compile`

✅ **Generated README.md includes usage instructions**
- Documents category and description
- Provides usage examples in TOML format
- Includes development steps

✅ **Skill name, description, and category correctly substituted into templates**
- All three values are dynamically substituted in all templates
- Test: `test_scaffold_skill_different_categories`

✅ **to_pascal_case() correctly converts kebab-case names to PascalCase**
- `my-skill` → `MySkill`
- `test` → `Test`
- `a-b-c` → `ABC`
- Test: `test_to_pascal_case_basic`, `test_to_pascal_case_mixed_case`

✅ **Generated skill can be loaded by discovery system (Issue 118)**
- Manifest format matches expected structure
- Directory structure follows discovery conventions
- `discover_skill_dirs()` correctly finds generated skills
- Test: `test_discover_skill_dirs_with_valid_skill`

### Test Results

All 152 unit tests pass:
```
running 152 tests
test result: ok. 152 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Scaffold-specific tests (10 tests):
- `test_to_pascal_case_basic`
- `test_to_pascal_case_empty`
- `test_to_pascal_case_mixed_case`
- `test_scaffold_skill_creates_directory`
- `test_scaffold_skill_creates_files`
- `test_scaffold_skill_creates_src_directory`
- `test_scaffold_skill_manifest_content`
- `test_scaffold_skill_lib_content`
- `test_scaffold_skill_readme_content`
- `test_scaffold_skill_different_categories`

All 22 integration tests pass.

### Build Verification

```bash
cargo build -p aisopod-plugin  # ✅ Success
cargo test -p aisopod-plugin   # ✅ All 152 tests pass
cargo run -- aisopod scaffold-skill ...  # ✅ CLI works correctly
```

Generated skill compilation verified in standalone project.

## Impact
Scaffolding reduces the time to create a new skill from minutes of manual boilerplate to a single command. It ensures consistency across skill implementations and encourages community contribution.

## Dependencies
- Issue 116 (Skill trait, SkillMeta, and SkillCategory types)
- Issue 117 (SkillRegistry for discovery and lifecycle)

## Acceptance Criteria
- [x] `scaffold_skill()` creates a complete skill directory structure
- [x] Generated `skill.toml` is valid and parseable by `parse_manifest()`
- [x] Generated `src/lib.rs` contains a compilable `Skill` trait implementation
- [x] Generated `README.md` includes usage instructions
- [x] Skill name, description, and category are correctly substituted into templates
- [x] `to_pascal_case()` correctly converts kebab-case names to PascalCase struct names
- [x] Generated skill can be loaded by the discovery system (Issue 118)

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*

## Additional Notes

### Critical Bug Fix
The initial implementation was missing `#[derive(Debug)]` on the generated struct. The `Skill` trait requires `std::fmt::Debug` as a trait bound:
```rust
pub trait Skill: Send + Sync + std::fmt::Debug
```

This was discovered when trying to compile the generated skill in a standalone project. The fix added the derive attribute to the template.

### Learnings Document
See `docs/learnings/122-implement-skill-creator-scaffolding-tool.md` for detailed learnings and observations from this implementation.
