# Issue 049: Define Tool Trait and Core Types

## Summary
Define the foundational `Tool` trait and its associated types (`ToolContext`, `ToolResult`) in the `aisopod-tools` crate. These form the core abstraction that all built-in and plugin-contributed tools implement.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/trait.rs` (or `crates/aisopod-tools/src/lib.rs`)

## Current Behavior
The `aisopod-tools` crate exists as an empty scaffold with no tool abstraction defined.

## Expected Behavior
After this issue is completed:
- A `Tool` async trait is defined with four methods: `name()`, `description()`, `parameters_schema()`, and `execute()`.
- A `ToolContext` struct is defined containing `agent_id`, `session_key`, `workspace_path`, `sandbox_config`, and `approval_handler`.
- A `ToolResult` struct is defined containing `content`, `is_error`, and `metadata`.
- All types are well-documented with rustdoc comments explaining their purpose and usage.

## Impact
Every tool in the system depends on this trait. It is the single most foundational type in the tool subsystem — nothing else in plan 0005 can proceed without it.

## Suggested Implementation
1. **Add dependencies** — In `crates/aisopod-tools/Cargo.toml`, add dependencies on `async-trait`, `serde`, `serde_json`, and `anyhow` (or the project's shared error type from `aisopod-shared`).

2. **Define `ToolResult`** — Create a struct with three fields:
   ```rust
   pub struct ToolResult {
       pub content: String,
       pub is_error: bool,
       pub metadata: Option<serde_json::Value>,
   }
   ```
   Add a doc comment explaining that `content` is the textual result returned to the AI model, `is_error` indicates whether the tool call failed, and `metadata` holds optional structured data for internal use.

3. **Define `ToolContext`** — Create a struct with five fields:
   ```rust
   pub struct ToolContext {
       pub agent_id: String,
       pub session_key: String,
       pub workspace_path: Option<PathBuf>,
       pub sandbox_config: Option<SandboxConfig>,
       pub approval_handler: Option<Arc<dyn ApprovalHandler>>,
   }
   ```
   Use `std::path::PathBuf` for the workspace path. `SandboxConfig` and `ApprovalHandler` can initially be placeholder types (empty struct / empty trait) that later issues flesh out. Add doc comments for each field.

4. **Define the `Tool` trait** — Use the `#[async_trait]` macro:
   ```rust
   #[async_trait]
   pub trait Tool: Send + Sync {
       fn name(&self) -> &str;
       fn description(&self) -> &str;
       fn parameters_schema(&self) -> serde_json::Value;
       async fn execute(
           &self,
           params: serde_json::Value,
           ctx: &ToolContext,
       ) -> Result<ToolResult>;
   }
   ```
   Add doc comments on the trait itself and on each method explaining its contract.

5. **Re-export from `lib.rs`** — Make sure `Tool`, `ToolContext`, and `ToolResult` are publicly accessible from the crate root.

6. **Verify** — Run `cargo check -p aisopod-tools` to confirm everything compiles.

## Dependencies
- Issue 005 (Create aisopod-tools crate)
- Issue 016 (Define core configuration types)

## Acceptance Criteria
- [x] `Tool` trait is defined with `name()`, `description()`, `parameters_schema()`, and `execute()` methods.
- [x] `ToolContext` struct is defined with all five fields.
- [x] `ToolResult` struct is defined with `content`, `is_error`, and `metadata`.
- [x] All public types have rustdoc comments.
- [x] `cargo check -p aisopod-tools` compiles without errors.
- [x] `cargo build` passes at project root.
- [x] `cargo test` passes at project root.

## Resolution

The Tool trait and core types were implemented in the `aisopod-tools` crate. The implementation includes:

### Changes Made
- **File**: `crates/aisopod-tools/src/lib.rs`
  - Defined `Tool` async trait with four methods: `name()`, `description()`, `parameters_schema()`, and `execute()`
  - Defined `ToolContext` struct with fields: `agent_id`, `session_key`, `workspace_path`, `sandbox_config`, `approval_handler`, and `metadata`
  - Defined `ToolResult` struct with fields: `content`, `is_error`, and `metadata`
  - Implemented associated methods for `ToolResult` (`success()`, `error()`, `with_metadata()`)
  - Implemented helper methods for `ToolContext` (`new()`, `with_workspace_path()`, `with_sandbox_config()`, `with_approval_handler()`, `with_metadata()`, `metadata_get()`)
  - Added comprehensive rustdoc documentation for all types and methods
  - Re-exported `Tool`, `ToolContext`, and `ToolResult` from crate root

### Dependencies Implemented
- Issue 005: aisopod-tools crate (already created)
- Issue 016: Core configuration types (SandboxConfig, SandboxIsolationMode defined as part of this issue)

### Verification
- `cargo build` passes at project root
- `cargo test` passes at project root (137 tests in aisopod-tools crate)
- `cargo check -p aisopod-tools` compiles without errors

---
*Created: 2026-02-15*
*Resolved: 2026-02-19*
