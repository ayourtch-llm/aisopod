# Issue 053: Implement File Operations Tool

## Summary
Implement a built-in file operations tool that provides read, write, search, listing, and metadata capabilities, all enforced within a workspace path restriction to prevent unauthorized file access.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/builtins/file.rs`

## Current Behavior
No file operations tool exists in the crate.

## Expected Behavior
After this issue is completed:
- The `FileTool` struct implements the `Tool` trait.
- It supports multiple operations selected via an `operation` parameter: `read`, `write`, `search`, `list`, and `metadata`.
- `read` — Returns the contents of a file as a string.
- `write` — Writes or creates a file with the given content.
- `search` — Searches for files matching glob patterns or text content matching a grep-like pattern.
- `list` — Lists the contents of a directory.
- `metadata` — Returns file size, permissions, and modification time.
- All file paths are resolved relative to the workspace path from `ToolContext` and path traversal outside the workspace is rejected.

## Impact
File operations are essential for agents that read code, write files, and navigate project directories. Workspace restriction is critical for security in multi-agent environments.

## Suggested Implementation
1. **Create `file.rs`** — Add `crates/aisopod-tools/src/builtins/file.rs`.

2. **Define `FileTool`**:
   ```rust
   pub struct FileTool;
   ```

3. **Implement `Tool` for `FileTool`**:
   - `name()` → `"file"`
   - `description()` → `"Read, write, search, list, and inspect files"`
   - `parameters_schema()` → JSON Schema with:
     - `operation` (enum: `read`, `write`, `search`, `list`, `metadata`) — required
     - `path` (string) — required for `read`, `write`, `list`, `metadata`
     - `content` (string) — required for `write`
     - `pattern` (string) — required for `search`
     - `glob` (string) — optional for `search`
   - `execute()`:
     1. Parse the `operation` parameter.
     2. Call the corresponding helper function.
     3. Wrap the result in a `ToolResult`.

4. **Implement workspace path enforcement**:
   - Create a helper `resolve_path(base: &Path, requested: &str) -> Result<PathBuf>` that:
     1. Joins the requested path to the workspace base.
     2. Canonicalizes the result.
     3. Checks that the canonical path starts with the canonical workspace base.
     4. Returns an error if the path is outside the workspace.
   - Call this helper before every file operation.

5. **Implement each operation**:
   - `read` — Use `tokio::fs::read_to_string()`.
   - `write` — Use `tokio::fs::write()`. Create parent directories if needed with `tokio::fs::create_dir_all()`.
   - `search` — Use the `glob` crate for pattern matching and basic line-by-line text search for grep-like functionality.
   - `list` — Use `tokio::fs::read_dir()` and collect entry names, types, and sizes.
   - `metadata` — Use `tokio::fs::metadata()` to return size, permissions (as octal), and modification time (as ISO 8601).

6. **Register the tool** — Ensure the tool can be registered with the `ToolRegistry`.

7. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 050 (Tool registry)

## Acceptance Criteria
- [x] `FileTool` implements the `Tool` trait.
- [x] `read` operation returns file contents.
- [x] `write` operation creates or overwrites files.
- [x] `search` operation finds files by glob pattern and text content.
- [x] `list` operation returns directory contents.
- [x] `metadata` operation returns file size, permissions, and modification time.
- [x] Workspace path restriction prevents access outside the workspace.
- [x] `parameters_schema()` returns a valid JSON Schema.
- [x] `cargo check -p aisopod-tools` compiles without errors.
- [x] All tests pass (`cargo test -p aisopod-tools`).

## Resolution

This issue was resolved by implementing the complete `FileTool` in `crates/aisopod-tools/src/builtins/file.rs`. The implementation includes:

- **Tool Implementation**: `FileTool` struct implementing the `Tool` trait with:
  - `name()` returning `"file"`
  - `description()` returning `"Read, write, search, list, and inspect files"`
  - `parameters_schema()` returning a JSON Schema with all operation parameters

- **Operations**:
  - `read`: Reads file contents as text
  - `write`: Writes content to a file, creating parent directories as needed
  - `search`: Searches for files matching glob patterns using `walkdir` and `glob_match`
  - `list`: Lists directory contents with metadata (size, permissions, modification time)
  - `metadata`: Returns detailed file/directory metadata in JSON format

- **Security**: All file operations resolve paths relative to the workspace directory and reject path traversal attempts outside the workspace.

- **Tests**: Comprehensive test suite covering all operations, edge cases, and security checks.

- **Registration**: The tool is registered in `builtins/mod.rs` and `lib.rs`.

The implementation passed all acceptance criteria with `cargo build` and `cargo test` at the project root.

---
*Created: 2026-02-15*
*Resolved: 2026-02-19*
