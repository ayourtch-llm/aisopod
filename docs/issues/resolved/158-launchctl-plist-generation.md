# Issue 158: Implement launchctl Plist Generation

## Resolution
This issue has been implemented with the following specifications:

- Changed Label from `com.aisopod.daemon` to `com.aisopod.gateway`
- Updated log directory from `/usr/local/var/log/` to `~/Library/Logs/aisopod/`
- Updated stdout path to `~/Library/Logs/aisopod/aisopod.out.log`
- Updated stderr path to `~/Library/Logs/aisopod/aisopod.err.log`
- Added log directory creation using `std::fs::create_dir_all`
- Used `$HOME` environment variable with fallback to `dirs::home_dir()`

## Summary
Implement the `aisopod daemon install` command on macOS to generate and install a LaunchAgent plist file that manages the aisopod gateway as a background service with automatic startup and keep-alive.

## Location
- Crate: `aisopod-cli`
- File: `crates/aisopod-cli/src/commands/daemon.rs`

## Current Behavior
The `aisopod daemon install` command is defined but does not generate or install a macOS LaunchAgent plist file.

## Expected Behavior
Running `aisopod daemon install` on macOS generates a plist file and installs it to `~/Library/LaunchAgents/com.aisopod.gateway.plist`, sets up a log directory, and provides instructions for loading the service.

## Impact
Enables macOS users to run aisopod as a managed background service that starts automatically on login and restarts if it crashes.

## Suggested Implementation

1. Add a function to generate the plist XML content:

```rust
fn generate_launchd_plist(binary_path: &str, log_dir: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.aisopod.gateway</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary_path}</string>
        <string>gateway</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log_dir}/aisopod.out.log</string>
    <key>StandardErrorPath</key>
    <string>{log_dir}/aisopod.err.log</string>
    <key>WorkingDirectory</key>
    <string>/tmp</string>
</dict>
</plist>
"#,
        binary_path = binary_path,
        log_dir = log_dir,
    )
}
```

2. Add the install logic for macOS:

```rust
fn install_launchd_service() -> Result<()> {
    let binary_path = std::env::current_exe()?
        .to_string_lossy()
        .to_string();
    let home = std::env::var("HOME")?;

    let log_dir = format!("{}/Library/Logs/aisopod", home);
    std::fs::create_dir_all(&log_dir)?;

    let plist_content = generate_launchd_plist(&binary_path, &log_dir);

    let plist_path = format!(
        "{}/Library/LaunchAgents/com.aisopod.gateway.plist",
        home
    );
    std::fs::write(&plist_path, plist_content)?;

    println!("Plist written to {}", plist_path);
    println!("Run: launchctl load {}", plist_path);
    Ok(())
}
```

3. Gate the implementation behind `#[cfg(target_os = "macos")]` so it only compiles on macOS.

4. In the daemon command dispatcher, call the appropriate platform function:

```rust
#[cfg(target_os = "macos")]
install_launchd_service()?;
#[cfg(target_os = "linux")]
install_systemd_service(system_level)?;
```

## Dependencies
- Issue 133 (daemon commands)

## Acceptance Criteria
- [x] `aisopod daemon install` generates a valid plist file on macOS
- [x] Plist is installed to `~/Library/LaunchAgents/com.aisopod.gateway.plist`
- [x] Label is set to `com.aisopod.gateway`
- [x] `RunAtLoad` and `KeepAlive` are both `true`
- [x] Log directory `~/Library/Logs/aisopod/` is created
- [x] Stdout and stderr are directed to log files
- [x] `launchctl load` successfully loads the service
- [x] `aisopod daemon uninstall` unloads and removes the plist

## Resolution
Implemented `aisopod daemon install` command for macOS LaunchAgent plist generation:
- Implemented `aisopod daemon install` command for macOS LaunchAgent plist generation
- Plist installed to `~/Library/LaunchAgents/com.aisopod.gateway.plist`
- Log directory `~/Library/Logs/aisopod/` created
- Plist contains: `Label=com.aisopod.gateway`, `RunAtLoad=true`, `KeepAlive=true`
- Stdout/stderr directed to `~/Library/Logs/aisopod/aisopod.out.log` and `aisopod.err.log`
- WorkingDirectory set to `/tmp`
- Platform-gated with `#[cfg(target_os = "macos")]`
- Updated `aisopod daemon uninstall` to use `launchctl bootout` command
- Added `libc = "0.2"` dependency to `Cargo.toml`
- All changes committed

---
*Created: 2026-02-15*
*Resolved: 2026-02-25*
