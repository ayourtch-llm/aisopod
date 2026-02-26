# Issue #158: Launchctl Plist Generation for macOS Daemon

## Summary

This document captures the learnings and implementation details for the `aisopod daemon install` command on macOS, which generates and installs a LaunchAgent plist file to manage the aisopod gateway as a background service.

## Key Implementation Details

### Plist File Structure

The LaunchAgent plist file is generated at `~/Library/LaunchAgents/com.aisopod.gateway.plist` with the following key properties:

- **Label**: `com.aisopod.gateway` - Identifies the service to launchd
- **ProgramArguments**: Array containing the binary path and `gateway` argument
- **RunAtLoad**: `true` - Starts the service when the user logs in
- **KeepAlive**: `true` - Automatically restarts the service if it crashes
- **WorkingDirectory**: `/tmp` - Sets the working directory for the service
- **StandardOutPath**: `~/Library/Logs/aisopod/aisopod.out.log` - Log file for stdout
- **StandardErrorPath**: `~/Library/Logs/aisopod/aisopod.err.log` - Log file for stderr

### Platform-Specific Commands

#### Installation
- Uses `launchctl load` to load the plist after writing it to disk
- Creates the log directory `~/Library/Logs/aisopod/` automatically

#### Starting/Stopping
- **Start**: `launchctl load <plist_path>`
- **Stop**: `launchctl bootout <plist_path>` (preferred over unload)
- **Uninstall**: `launchctl bootout <plist_path>` then delete the plist file

#### Status and Logs
- **Status**: `launchctl list com.aisopod.gateway`
- **Logs**: `tail -n <lines> ~/Library/Logs/aisopod/aisopod.out.log`

### macOS launchctl Commands

- `load`: Loads a plist file and registers it with launchd
- `bootout`: Unloads a service and removes it from launchd (preferred over `unload`)
- `unload`: Just unloads a service (less thorough than `bootout`)
- `list`: Lists running services or queries a specific service

### Platform Detection

All macOS-specific code should be gated behind `#[cfg(target_os = "macos")]` to ensure cross-platform compatibility.

## Code Structure

The implementation follows this pattern:

```rust
fn install_launchctl_service(exe_path: &Path) -> Result<()> {
    // 1. Get home directory with fallback
    let home = std::env::var("HOME")
        .or_else(|_| dirs::home_dir()...)
    
    // 2. Create log directory
    let log_dir = format!("{}/Library/Logs/aisopod", home);
    std::fs::create_dir_all(&log_dir)?;
    
    // 3. Generate plist content
    let plist = generate_plist_content(&exe_path, &log_dir);
    
    // 4. Write plist to disk
    let plist_path = format!("{}/Library/LaunchAgents/com.aisopod.gateway.plist", home);
    std::fs::write(&plist_path, plist)?;
    
    // 5. Print instructions
    println!("Plist written to {}", plist_path);
    println!("Run: launchctl load {}", plist_path);
    Ok(())
}
```

## Common Pitfalls

1. **Home directory detection**: Always use `std::env::var("HOME")` with fallback to `dirs::home_dir()` as not all environments have HOME set
2. **Path formatting**: Use `format!` or `PathBuf` for constructing paths, never string concatenation
3. **Platform detection**: Always gate macOS-specific code with `#[cfg(target_os = "macos")]`
4. **Error handling**: Use `anyhow::Context` to provide helpful error messages for file system operations

## Testing Considerations

- Tests should be platform-aware and skip on non-macOS systems
- Mock file system operations for unit tests
- Consider using `tempfile` crate for testing plist generation
- Integration tests should verify plist content matches expected format

## Future Improvements

1. Add `LaunchOnlyOnce` key if the gateway should only start once per session
2. Consider adding `LowPriorityIO` key for background priority I/O
3. Add `ThrottleInterval` to control restart frequency
4. Consider adding `EnvironmentVariables` for custom configuration
5. Implement `aisopod daemon reinstall` to update existing installations

## References

- [Apple Launchd Documentation](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html)
- [launchd man page](https://man7.org/linux/man-pages/man8/systemd.8.html)
- [Property List Format](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/PropertyLists Introduction to Property Lists)
