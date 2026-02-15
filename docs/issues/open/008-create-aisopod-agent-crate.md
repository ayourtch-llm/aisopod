# Issue 008: Create aisopod-agent Library Crate

## Summary
Create the `aisopod-agent` library crate that will implement the core agent loop, orchestration logic, and agent lifecycle management for the aisopod project.

## Location
- Crate: `aisopod-agent`
- File: `crates/aisopod-agent/Cargo.toml`, `crates/aisopod-agent/src/lib.rs`

## Current Behavior
The `crates/aisopod-agent/` directory does not exist. There is no agent orchestration crate in the workspace.

## Expected Behavior
A new library crate `aisopod-agent` exists under `crates/aisopod-agent/` with:
- A `Cargo.toml` that inherits workspace package settings and depends on `aisopod-shared`
- A `src/lib.rs` with basic doc comments explaining the crate's purpose
- The crate is listed in the root `Cargo.toml` workspace members

## Impact
This is the central crate of the project, implementing the agent loop that coordinates between providers, tools, memory, and sessions. It is the heart of the agentic system.

## Suggested Implementation
1. Create the directory `crates/aisopod-agent/`.
2. Create `crates/aisopod-agent/Cargo.toml` with:
   ```toml
   [package]
   name = "aisopod-agent"
   version.workspace = true
   edition.workspace = true

   [dependencies]
   aisopod-shared = { path = "../aisopod-shared" }
   serde.workspace = true
   serde_json.workspace = true
   anyhow.workspace = true
   thiserror.workspace = true
   tracing.workspace = true
   tokio.workspace = true
   ```
3. Create `crates/aisopod-agent/src/lib.rs` with a top-level doc comment:
   ```rust
   //! # aisopod-agent
   //!
   //! Core agent loop, orchestration logic, and agent lifecycle management.
   ```
4. Add `"crates/aisopod-agent"` to the `workspace.members` list in the root `Cargo.toml`.
5. Run `cargo check -p aisopod-agent` to verify the crate compiles.

## Dependencies
001, 002

## Acceptance Criteria
- [ ] `crates/aisopod-agent/Cargo.toml` exists and inherits workspace settings
- [ ] `crates/aisopod-agent/src/lib.rs` exists with doc comments describing the crate
- [ ] The crate depends on `aisopod-shared` via path dependency
- [ ] The crate is listed in the root `Cargo.toml` workspace members
- [ ] `cargo check -p aisopod-agent` succeeds without errors

---
*Created: 2026-02-15*
