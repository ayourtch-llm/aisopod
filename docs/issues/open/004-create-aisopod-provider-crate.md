# Issue 004: Create aisopod-provider Library Crate

## Summary
Create the `aisopod-provider` library crate that will define provider abstractions and implementations for interacting with LLM backends and other AI service providers.

## Location
- Crate: `aisopod-provider`
- File: `crates/aisopod-provider/Cargo.toml`, `crates/aisopod-provider/src/lib.rs`

## Current Behavior
The `crates/aisopod-provider/` directory does not exist. There is no provider abstraction crate in the workspace.

## Expected Behavior
A new library crate `aisopod-provider` exists under `crates/aisopod-provider/` with:
- A `Cargo.toml` that inherits workspace package settings and depends on `aisopod-shared`
- A `src/lib.rs` with basic doc comments explaining the crate's purpose
- The crate is listed in the root `Cargo.toml` workspace members

## Impact
This crate will house all provider-related logic including trait definitions for LLM providers, making it possible to support multiple AI backends through a uniform interface.

## Suggested Implementation
1. Create the directory `crates/aisopod-provider/`.
2. Create `crates/aisopod-provider/Cargo.toml` with:
   ```toml
   [package]
   name = "aisopod-provider"
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
3. Create `crates/aisopod-provider/src/lib.rs` with a top-level doc comment:
   ```rust
   //! # aisopod-provider
   //!
   //! Provider abstractions and implementations for LLM backends and AI service providers.
   ```
4. Add `"crates/aisopod-provider"` to the `workspace.members` list in the root `Cargo.toml`.
5. Run `cargo check -p aisopod-provider` to verify the crate compiles.

## Dependencies
001, 002

## Acceptance Criteria
- [ ] `crates/aisopod-provider/Cargo.toml` exists and inherits workspace settings
- [ ] `crates/aisopod-provider/src/lib.rs` exists with doc comments describing the crate
- [ ] The crate depends on `aisopod-shared` via path dependency
- [ ] The crate is listed in the root `Cargo.toml` workspace members
- [ ] `cargo check -p aisopod-provider` succeeds without errors

---
*Created: 2026-02-15*
