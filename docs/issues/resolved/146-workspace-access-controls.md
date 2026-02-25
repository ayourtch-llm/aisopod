# Issue 146: Implement Workspace Access Controls

## Summary
Enforce workspace access controls for sandboxed containers based on the `WorkspaceAccess` setting. Ensure that `ReadOnly` mounts are truly read-only, `None` prevents any workspace mount, and `ReadWrite` allows full access. Validate that paths do not escape the workspace root.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/sandbox/workspace.rs` (new)

## Current Behavior
No workspace access enforcement exists. The sandbox executor from Issue 145 accepts a `working_dir` path and mounts it, but there is no validation layer to prevent path traversal or to verify that mount flags are correctly applied.

## Expected Behavior
After this issue is completed:
- A `WorkspaceGuard` validates and prepares workspace mounts before container creation.
- `ReadOnly` mode mounts the workspace with the `:ro` flag in the container runtime.
- `None` mode skips the workspace mount entirely.
- `ReadWrite` mode mounts with full read-write access.
- Path validation rejects any path that escapes the configured workspace root (e.g., `../../etc/passwd`).
- Symlinks that point outside the workspace are detected and rejected.

## Impact
Without proper access controls, a sandboxed agent could read or modify files outside its designated workspace, defeating the purpose of isolation. This is a critical security boundary.

## Suggested Implementation

1. **Create the workspace validation module** (`crates/aisopod-tools/src/sandbox/workspace.rs`):
   ```rust
   use std::path::{Path, PathBuf};
   use crate::sandbox::config::WorkspaceAccess;

   #[derive(Debug)]
   pub struct WorkspaceGuard {
       root: PathBuf,
       access: WorkspaceAccess,
   }

   #[derive(Debug, thiserror::Error)]
   pub enum WorkspaceError {
       #[error("path escapes workspace root: {0}")]
       PathEscape(PathBuf),

       #[error("symlink points outside workspace: {0}")]
       SymlinkEscape(PathBuf),

       #[error("workspace root does not exist: {0}")]
       RootNotFound(PathBuf),
   }
   ```

2. **Implement path validation:**
   ```rust
   impl WorkspaceGuard {
       pub fn new(root: PathBuf, access: WorkspaceAccess) -> Result<Self, WorkspaceError> {
           if !root.exists() {
               return Err(WorkspaceError::RootNotFound(root));
           }
           let root = root
               .canonicalize()
               .map_err(|_| WorkspaceError::RootNotFound(root.clone()))?;
           Ok(Self { root, access })
       }

       /// Validate that a path is within the workspace root.
       pub fn validate_path(&self, path: &Path) -> Result<PathBuf, WorkspaceError> {
           let resolved = if path.is_absolute() {
               path.to_path_buf()
           } else {
               self.root.join(path)
           };

           // Canonicalize to resolve symlinks and ..
           let canonical = resolved
               .canonicalize()
               .map_err(|_| WorkspaceError::PathEscape(resolved.clone()))?;

           if !canonical.starts_with(&self.root) {
               return Err(WorkspaceError::PathEscape(canonical));
           }

           Ok(canonical)
       }

       pub fn access(&self) -> &WorkspaceAccess {
           &self.access
       }

       pub fn root(&self) -> &Path {
           &self.root
       }
   }
   ```

3. **Implement mount argument generation** for the container runtime:
   ```rust
   impl WorkspaceGuard {
       /// Generate the Docker/Podman volume mount arguments.
       /// Returns `None` if access is `None` (no mount).
       pub fn mount_args(&self) -> Option<Vec<String>> {
           match self.access {
               WorkspaceAccess::None => None,
               WorkspaceAccess::ReadOnly => Some(vec![
                   "-v".to_string(),
                   format!("{}:/workspace:ro", self.root.display()),
               ]),
               WorkspaceAccess::ReadWrite => Some(vec![
                   "-v".to_string(),
                   format!("{}:/workspace:rw", self.root.display()),
               ]),
           }
       }
   }
   ```

4. **Integrate with `SandboxExecutor`** from Issue 145:
   ```rust
   // In SandboxExecutor::create_container() or execute()
   let guard = WorkspaceGuard::new(working_dir.to_path_buf(), config.workspace_access.clone())?;

   if let Some(mount_args) = guard.mount_args() {
       cmd.args(&mount_args);
   }
   ```

5. **Add unit tests:**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use std::fs;
       use tempfile::TempDir;

       #[test]
       fn test_valid_path_within_workspace() {
           let tmp = TempDir::new().unwrap();
           let subdir = tmp.path().join("subdir");
           fs::create_dir(&subdir).unwrap();

           let guard = WorkspaceGuard::new(
               tmp.path().to_path_buf(),
               WorkspaceAccess::ReadWrite,
           ).unwrap();

           assert!(guard.validate_path(&subdir).is_ok());
       }

       #[test]
       fn test_path_escape_rejected() {
           let tmp = TempDir::new().unwrap();
           let guard = WorkspaceGuard::new(
               tmp.path().to_path_buf(),
               WorkspaceAccess::ReadWrite,
           ).unwrap();

           let escape = tmp.path().join("../../etc/passwd");
           assert!(guard.validate_path(&escape).is_err());
       }

       #[test]
       fn test_none_access_no_mount() {
           let tmp = TempDir::new().unwrap();
           let guard = WorkspaceGuard::new(
               tmp.path().to_path_buf(),
               WorkspaceAccess::None,
           ).unwrap();

           assert!(guard.mount_args().is_none());
       }

       #[test]
       fn test_readonly_mount_has_ro_flag() {
           let tmp = TempDir::new().unwrap();
           let guard = WorkspaceGuard::new(
               tmp.path().to_path_buf(),
               WorkspaceAccess::ReadOnly,
           ).unwrap();

           let args = guard.mount_args().unwrap();
           assert!(args[1].ends_with(":ro"));
       }

       #[test]
       fn test_readwrite_mount_has_rw_flag() {
           let tmp = TempDir::new().unwrap();
           let guard = WorkspaceGuard::new(
               tmp.path().to_path_buf(),
               WorkspaceAccess::ReadWrite,
           ).unwrap();

           let args = guard.mount_args().unwrap();
           assert!(args[1].ends_with(":rw"));
       }
   }
   ```

## Dependencies
- Issue 144 (sandbox configuration types — `WorkspaceAccess` enum)
- Issue 145 (container execution — `SandboxExecutor` integration)

## Resolution
The implementation was completed with the following changes:

1. **Created `crates/aisopod-tools/src/sandbox/workspace.rs`** with:
   - `WorkspaceGuard` struct that validates workspace paths
   - `WorkspaceError` enum for validation errors
   - `new()` constructor that canonicalizes the root path
   - `validate_path()` method that:
     - Resolves symlinks in path components
     - Checks each component for symlinks pointing outside workspace
     - Rejects paths that escape the workspace root
   - `mount_args()` method that generates Docker/Podman volume mount arguments
   - `bind_mount()` helper method for direct mount specification
   - Comprehensive unit tests covering all access modes and edge cases

2. **Exported WorkspaceGuard** from the sandbox module in `crates/aisopod-tools/src/sandbox/mod.rs`

3. **Integrated with SandboxExecutor** in `crates/aisopod-tools/src/sandbox/executor.rs`:
   - Updated `execute()` to use `WorkspaceGuard` for validation
   - Updated `run_one_shot()` to validate workspace before container creation
   - Both methods now reject paths that escape the workspace root

4. **Added top-level exports** in `crates/aisopod-tools/src/lib.rs` for `WorkspaceGuard` and `WorkspaceError`

## Acceptance Criteria
- [x] `WorkspaceGuard` validates paths and rejects traversals outside the workspace root
- [x] Symlinks pointing outside the workspace are detected and rejected
- [x] `ReadOnly` access produces `:ro` mount flags
- [x] `ReadWrite` access produces `:rw` mount flags
- [x] `None` access skips the workspace mount entirely
- [x] Integration with `SandboxExecutor` uses `WorkspaceGuard` for all mount decisions
- [x] Unit tests cover valid paths, escape attempts, and all three access modes

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
