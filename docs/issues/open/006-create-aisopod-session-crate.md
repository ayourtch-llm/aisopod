# Issue 006: Create aisopod-session Library Crate

## Summary
Create the `aisopod-session` library crate that will manage conversation sessions, state tracking, and session lifecycle for the aisopod project.

## Location
- Crate: `aisopod-session`
- File: `crates/aisopod-session/Cargo.toml`, `crates/aisopod-session/src/lib.rs`

## Current Behavior
The `crates/aisopod-session/` directory does not exist. There is no session management crate in the workspace.

## Expected Behavior
A new library crate `aisopod-session` exists under `crates/aisopod-session/` with:
- A `Cargo.toml` that inherits workspace package settings and depends on `aisopod-shared`
- A `src/lib.rs` with basic doc comments explaining the crate's purpose
- The crate is listed in the root `Cargo.toml` workspace members

## Impact
This crate will handle all session-related logic, including creating, persisting, and restoring conversation sessions. It is essential for maintaining stateful interactions with users.

## Suggested Implementation
1. Create the directory `crates/aisopod-session/`.
2. Create `crates/aisopod-session/Cargo.toml` with:
   ```toml
   [package]
   name = "aisopod-session"
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
3. Create `crates/aisopod-session/src/lib.rs` with a top-level doc comment:
   ```rust
   //! # aisopod-session
   //!
   //! Session management, state tracking, and session lifecycle for conversations.
   ```
4. Add `"crates/aisopod-session"` to the `workspace.members` list in the root `Cargo.toml`.
5. Run `cargo check -p aisopod-session` to verify the crate compiles.

## Dependencies
001, 002

## Acceptance Criteria
- [ ] `crates/aisopod-session/Cargo.toml` exists and inherits workspace settings
- [ ] `crates/aisopod-session/src/lib.rs` exists with doc comments describing the crate
- [ ] The crate depends on `aisopod-shared` via path dependency
- [ ] The crate is listed in the root `Cargo.toml` workspace members
- [ ] `cargo check -p aisopod-session` succeeds without errors

---
*Created: 2026-02-15*
