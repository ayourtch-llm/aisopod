# Issue 002: Create aisopod-shared Library Crate

## Summary
Create the `aisopod-shared` library crate that will hold shared utilities, common types, and helper functions used across all other crates in the workspace.

## Location
- Crate: `aisopod-shared`
- File: `crates/aisopod-shared/Cargo.toml`, `crates/aisopod-shared/src/lib.rs`

## Current Behavior
The `crates/aisopod-shared/` directory does not exist. There is no shared utilities crate in the workspace.

## Expected Behavior
A new library crate `aisopod-shared` exists under `crates/aisopod-shared/` with:
- A `Cargo.toml` that inherits workspace package settings and declares relevant workspace dependencies
- A `src/lib.rs` with basic doc comments explaining the crate's purpose as the shared utilities crate
- The crate is listed in the root `Cargo.toml` workspace members

## Impact
This crate is the common foundation for all other library crates. It will contain shared types, error definitions, and utility functions that prevent code duplication across the project.

## Suggested Implementation
1. Create the directory `crates/aisopod-shared/`.
2. Create `crates/aisopod-shared/Cargo.toml` with:
   ```toml
   [package]
   name = "aisopod-shared"
   version.workspace = true
   edition.workspace = true

   [dependencies]
   anyhow.workspace = true
   thiserror.workspace = true
   serde.workspace = true
   tracing.workspace = true
   ```
3. Create `crates/aisopod-shared/src/lib.rs` with a top-level doc comment:
   ```rust
   //! # aisopod-shared
   //!
   //! Shared utilities, common types, and helper functions for the aisopod project.
   ```
4. Add `"crates/aisopod-shared"` to the `workspace.members` list in the root `Cargo.toml`.
5. Run `cargo check -p aisopod-shared` to verify the crate compiles.

## Dependencies
001

## Acceptance Criteria
- [ ] `crates/aisopod-shared/Cargo.toml` exists and inherits workspace settings
- [ ] `crates/aisopod-shared/src/lib.rs` exists with doc comments describing the crate
- [ ] The crate is listed in the root `Cargo.toml` workspace members
- [ ] `cargo check -p aisopod-shared` succeeds without errors

---
*Created: 2026-02-15*
