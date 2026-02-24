# Issue 145: Container Execution Implementation Learning

## Summary

This document captures key learnings from implementing the container execution sandbox feature using Docker/Podman.

## Key Implementation Decisions

### 1. CLI-Based Container Management

We chose to shell out to the Docker/Podman CLI rather than using a Rust library like `docker-api` or `podman-api`. This approach has several advantages:

**Pros:**
- No additional dependencies required
- Leverages existing Docker/Podman installations
- Simpler maintenance (less code to maintain)
- Works with both Docker and Podman without code changes

**Cons:**
- Platform-dependent (requires Docker/Podman to be installed)
- Less fine-grained control over container lifecycle
- More difficult to handle edge cases programmatically

### 2. Container Lifecycle Management

The executor provides two approaches:

**One-shot execution (`run_one_shot`):**
- Automatically creates, starts, executes, and destroys containers
- Ensures cleanup even on errors or timeouts
- Best for short-lived, independent commands

**Manual lifecycle management:**
- `create_container()`, `start_container()`, `execute()`, `destroy_container()`
- Allows container reuse for multiple commands
- Better for long-running sandboxes or multiple related operations

### 3. Timeout Enforcement

Timeouts are enforced using `tokio::time::timeout()` which:
- Wraps the entire command execution
- Automatically kills the container on timeout
- Returns a structured result indicating timeout status

```rust
let result = tokio::time::timeout(config.timeout, cmd.output()).await;
match result {
    Ok(Ok(output)) => { /* normal handling */ }
    Ok(Err(e)) => { /* command failed */ }
    Err(_) => { /* timeout */ }
}
```

### 4. Error Handling Philosophy

Container operations are best-effort for cleanup:
- `destroy_container()` doesn't fail if container doesn't exist
- `stop_container()` and `kill_container()` ignore "already stopped" errors
- This ensures cleanup code doesn't fail when the container is already gone

### 5. Workspace Mounting

The workspace is mounted as `/workspace` inside containers with configurable access:
- `ReadOnly`: `-v /path:/workspace:ro`
- `ReadWrite`: `-v /path:/workspace:rw`
- `None`: No workspace mount

This provides a predictable path for tools to access workspace files.

## Common Pitfalls

### 1. Container ID Trimming

Container IDs from CLI output may have trailing newlines:
```rust
let container_id = String::from_utf8_lossy(&output.stdout)
    .trim()  // Important: trim whitespace
    .to_string();
```

### 2. UTF-8 Conversion

Always use `String::from_utf8_lossy()` for output:
```rust
let stdout = String::from_utf8_lossy(&output.stdout).to_string();
```
This handles cases where containers produce non-UTF-8 output.

### 3. Exit Code Handling

Exit codes may not be available if the process was terminated:
```rust
let exit_code = output.status.code().unwrap_or(-1);
```

### 4. Workspace Path Display

Use `Path::display()` for mounting paths:
```rust
let mount = format!("{}:/workspace:ro", working_dir.display());
```

## Testing Considerations

### Integration Tests

Integration tests are marked `#[ignore]` because they require Docker/Podman:
```rust
#[tokio::test]
#[ignore] // Requires Docker/Podman
async fn test_run_echo_in_container() { /* ... */ }
```

Run them with:
```bash
cargo test --package aisopod-tools -- --ignored
```

### Test Images

Use small, reliable images for testing:
- `alpine:latest` - Small, widely available
- Consider caching images locally for faster tests

## Performance Considerations

### Container Startup Time

Container startup is relatively slow (100-500ms for Alpine):
- Consider container reuse for multiple commands
- One-shot execution may not be suitable for high-frequency operations

### Resource Limits

Always configure resource limits in production:
```rust
SandboxConfig {
    memory_limit: Some("256m".to_string()),
    cpu_limit: Some(0.5),
    // ...
}
```

### Network Isolation

Disable network access when not needed:
```rust
SandboxConfig {
    network_access: false,
    // ...
}
```

## Security Considerations

### Workspace Access

Be careful with `ReadWrite` access - containers can modify workspace files:
- Consider `ReadOnly` when possible
- Validate file paths to prevent escape attacks

### Resource Exhaustion

Always set resource limits to prevent:
- Memory exhaustion (OOM kills)
- CPU starvation
- Disk space issues

### Network Access

Disable network access for commands that don't need it:
- Prevents data exfiltration
- Reduces attack surface
- Faster execution (no network setup)

## Migration Notes

### From Direct Execution

When migrating from direct command execution to containerized execution:

1. **Configure Sandbox**: Enable sandboxing and configure image/runtime
2. **Path Adjustments**: Workspace is at `/workspace` inside container
3. **Environment Variables**: May need to be explicitly passed
4. **Timeouts**: May need adjustment due to startup overhead

### Example Migration

**Before:**
```rust
let output = Command::new("sh").arg("-c").arg(command).output().await?;
```

**After:**
```rust
let executor = SandboxExecutor::new(SandboxRuntime::Docker);
let config = SandboxConfig {
    enabled: true,
    image: "alpine:latest".to_string(),
    ..Default::default()
};

let result = executor
    .run_one_shot(&config, command, workspace_path)
    .await?;
```

## Future Enhancements

Potential improvements for future iterations:

1. **Container Pooling**: Reuse containers for multiple commands
2. **Image Caching**: Pre-pull images to reduce startup time
3. **Progress Reporting**: Stream output during long-running commands
4. **Signal Forwarding**: Forward SIGINT/SIGTERM to containers
5. **Health Checks**: Verify container runtime is available
6. **Metrics**: Track execution times and success rates

## Related Issues

- Issue 144: Sandbox configuration types
- Issue 052: Bash tool execution patterns
