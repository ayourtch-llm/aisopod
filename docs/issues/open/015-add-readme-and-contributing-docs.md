# Issue 015: Add README and CONTRIBUTING Documentation

## Summary
Update the root `README.md` with a comprehensive project overview, build instructions, and architecture diagram, and create a `CONTRIBUTING.md` with development setup and contribution guidelines.

## Location
- Crate: `(workspace root)`
- File: `README.md`, `CONTRIBUTING.md`

## Current Behavior
The repository may have a minimal or empty `README.md`. No `CONTRIBUTING.md` exists. New contributors have no guidance on project structure, setup, or contribution workflow.

## Expected Behavior
- `README.md` contains: project overview, features list, architecture overview (listing all crates and their purposes), build instructions, and usage examples
- `CONTRIBUTING.md` contains: development environment setup, coding standards, PR workflow, and testing guidelines

## Impact
Good documentation is critical for onboarding new contributors and making the project accessible. Without it, developers waste time figuring out the project structure and conventions on their own.

## Suggested Implementation
1. Update `README.md` at the repository root with:
   - Project name and brief description
   - Features / goals section
   - Architecture overview section listing all workspace crates:
     - `aisopod` — Binary entry point
     - `aisopod-shared` — Shared utilities and common types
     - `aisopod-config` — Configuration management
     - `aisopod-provider` — LLM provider abstractions
     - `aisopod-tools` — Tool-use framework
     - `aisopod-session` — Session management
     - `aisopod-memory` — Memory and context management
     - `aisopod-agent` — Core agent orchestration
     - `aisopod-channel` — Communication channels
     - `aisopod-plugin` — Plugin system
     - `aisopod-gateway` — API gateway
   - Build instructions (`cargo build`, `cargo test`, `cargo run`)
   - License section
2. Create `CONTRIBUTING.md` at the repository root with:
   - Prerequisites (Rust stable, Git)
   - Development setup steps (clone, build, test)
   - Code style guidelines (use `cargo fmt`, `cargo clippy`)
   - Branch naming conventions
   - Pull request process
   - Testing requirements (all tests must pass, add tests for new features)
3. Review both files for accuracy and completeness.

## Dependencies
001

## Acceptance Criteria
- [ ] `README.md` exists with project overview, architecture section listing all crates, and build instructions
- [ ] `CONTRIBUTING.md` exists with development setup, coding standards, and PR workflow
- [ ] Build instructions in `README.md` are accurate and work when followed
- [ ] All crate names and descriptions match the actual workspace structure

---
*Created: 2026-02-15*
