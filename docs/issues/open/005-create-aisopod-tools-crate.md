# Issue 005: Create aisopod-tools Library Crate

## Summary
Create the `aisopod-tools` library crate that will define tool abstractions, registries, and implementations for function-calling and tool-use capabilities.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/Cargo.toml`, `crates/aisopod-tools/src/lib.rs`

## Current Behavior
The `crates/aisopod-tools/` directory does not exist. There is no tools crate in the workspace.

## Expected Behavior
A new library crate `aisopod-tools` exists under `crates/aisopod-tools/` with:
- A `Cargo.toml` that inherits workspace package settings and depends on `aisopod-shared`
- A `src/lib.rs` with basic doc comments explaining the crate's purpose
- The crate is listed in the root `Cargo.toml` workspace members

## Impact
This crate will provide the tool-use framework, enabling agents to call external tools and functions. It is a core component of the agentic architecture.

## Suggested Implementation
1. Create the directory `crates/aisopod-tools/`.
2. Create `crates/aisopod-tools/Cargo.toml` with:
   ```toml
   [package]
   name = "aisopod-tools"
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
3. Create `crates/aisopod-tools/src/lib.rs` with a top-level doc comment:
   ```rust
   //! # aisopod-tools
   //!
   //! Tool abstractions, registries, and implementations for function-calling and tool-use capabilities.
   ```
4. Add `"crates/aisopod-tools"` to the `workspace.members` list in the root `Cargo.toml`.
5. Run `cargo check -p aisopod-tools` to verify the crate compiles.

## Dependencies
001, 002

## Acceptance Criteria
- [ ] `crates/aisopod-tools/Cargo.toml` exists and inherits workspace settings
- [ ] `crates/aisopod-tools/src/lib.rs` exists with doc comments describing the crate
- [ ] The crate depends on `aisopod-shared` via path dependency
- [ ] The crate is listed in the root `Cargo.toml` workspace members
- [ ] `cargo check -p aisopod-tools` succeeds without errors

---
*Created: 2026-02-15*
