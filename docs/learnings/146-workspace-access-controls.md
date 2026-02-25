# Workspace Access Controls Implementation

## Summary
This document captures learnings from implementing workspace access controls for sandboxed containers in the aisopod project.

## Key Decisions

### Path Validation Strategy
The `WorkspaceGuard::validate_path()` method uses a two-phase validation approach:

1. **Symlink Check**: Before canonicalizing the full path, we iterate through each component and check if any component is a symlink. If a symlink points outside the workspace, we reject it immediately.

2. **Canonicalization**: After symlink checks, we canonicalize the path to resolve `..` components and verify the final path is within the workspace root.

This approach ensures that even if a path doesn't exist yet, we can still detect symlinks that would escape the workspace once the path is resolved.

### Error Handling
The `WorkspaceError` enum distinguishes between:
- `PathEscape`: The path resolves to a location outside the workspace
- `SymlinkEscape`: A symlink in the path points outside the workspace  
- `RootNotFound`: The workspace root directory doesn't exist

This granularity helps callers understand why validation failed and take appropriate action.

### Mount Argument Generation
We provide two methods for generating mount specifications:
- `mount_args()`: Returns `Option<Vec<String>>` for passing to container runtime CLI
- `bind_mount()`: Returns `Option<String>` for direct path mapping

The `mount_args()` method is used by `SandboxExecutor` for Docker/Podman CLI commands, while `bind_mount()` could be useful for API-based container management.

## Security Considerations

### Path Traversal Prevention
The implementation prevents path traversal attacks by:
1. Canonicalizing all paths to absolute form
2. Checking if the canonical path starts with the workspace root
3. Rejecting any path that would resolve outside the workspace

### Symlink Security
Symlinks are checked at each component level:
- If a symlink component resolves to an absolute path outside the workspace, it's rejected
- If a symlink component resolves to a relative path, it's resolved relative to the symlink's parent directory
- The resolved target is then canonicalized and checked against the workspace root

This prevents scenarios like:
```
/workspace/symlink -> /etc/passwd
```

### Working Directory Validation
The `SandboxExecutor` validates the working directory before executing commands:
- In `execute()`: Validates and mounts the workspace
- In `run_one_shot()`: Validates before container creation

This ensures that even if an attacker somehow passes a malicious working directory path, it will be caught before execution.

## Testing Approach

### Unit Tests
The test suite covers:
- Valid paths within the workspace
- Path escape attempts (e.g., `../../etc/passwd`)
- Absolute paths outside the workspace
- Symlink escape detection
- All three access modes (None, ReadOnly, ReadWrite)
- Mount argument generation for each mode

### Test Patterns
- Use `std::env::temp_dir()` for real filesystem operations
- Test both absolute and relative paths
- Verify error types match expected behavior

## Integration with SandboxExecutor

The `WorkspaceGuard` is integrated into `SandboxExecutor` at the appropriate points:

1. **Validation Point**: Before mounting the workspace in `execute()` and `run_one_shot()`
2. **Mount Point**: Using `guard.mount_args()` to generate CLI arguments
3. **Error Propagation**: Returning `WorkspaceError` to signal validation failures

This integration ensures that all workspace mounts go through validation, preventing unauthorized access.

## Code Organization

### Module Structure
```
crates/aisopod-tools/src/sandbox/
├── config.rs      # Re-exports SandboxConfig types
├── executor.rs    # SandboxExecutor implementation
└── workspace.rs   # WorkspaceGuard (NEW)
```

### Public Exports
The workspace module exports are available at multiple levels:
- `aisopod_tools::sandbox::workspace::WorkspaceGuard`
- `aisopod_tools::sandbox::WorkspaceGuard` (via mod re-export)
- `aisopod_tools::WorkspaceGuard` (top-level re-export)

This provides flexibility for callers while maintaining clear ownership of the types.

## Common Pitfalls

### Canonicalization Failures
When a path doesn't exist, `canonicalize()` returns an error. The implementation treats this as a `PathEscape` error, which is correct because we can only validate paths that exist. This is acceptable because the workspace root should always exist for valid use cases.

### Relative Path Handling
Relative paths are joined with the workspace root before validation. This means `validate_path(Path::new("foo"))` is equivalent to `validate_path(Path::new("/workspace/foo"))`. This behavior is intentional and matches the expected semantics of a working directory.

### Empty Paths
The implementation doesn't explicitly handle empty paths. In practice, `PathBuf::new()` joined with a workspace root produces the root itself, which should be valid. If needed, explicit empty path handling could be added.

## Future Enhancements

### Additional Validation
Potential future improvements:
1. Whitelist/blacklist of specific paths or patterns
2. Size limits for mounted directories
3. File type restrictions (e.g., block special files)

### Performance Optimizations
For very large directory trees:
1. Cache canonicalized paths
2. Implement incremental validation for repeated operations
3. Lazy symlink resolution

### API Enhancements
1. Add method to get relative path within workspace
2. Support for multiple workspace roots
3. Configurable validation rules per use case

## Related Issues
- Issue 144: Sandbox configuration types (provides `WorkspaceAccess` enum)
- Issue 145: Container execution (integrated with `SandboxExecutor`)

## References
- Docker volume mount documentation: https://docs.docker.com/storage/volumes/
- Podman volume documentation: https://docs.podman.io/en/latest/markdown/podman-run.1.html#volume-v
