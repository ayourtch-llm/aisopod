# Issue 157: Systemd Service File Generation - Implementation Learnings

## Summary

This issue extended the daemon management commands (from issue 133) to support user-level systemd service installation and implemented the uninstall functionality for aisopod on Linux systems.

## Implementation Details

### Changes Made to `crates/aisopod/src/commands/daemon.rs`

#### 1. Added `--system` Flag to Install Command

The `DaemonCommands::Install` variant was changed from a unit variant to tuple variant containing `InstallArgs`:

```rust
#[derive(Args)]
pub struct InstallArgs {
    /// Install at system level (requires sudo)
    #[arg(long)]
    pub system: bool,
}
```

This allows users to choose installation scope:
- `aisopod daemon install` - installs at user level (`~/.config/systemd/user/`)
- `aisopod daemon install --system` - installs at system level (`/etc/systemd/system/`)

#### 2. Systemd Unit File Generation

The `generate_systemd_unit` function creates the unit file content with proper configuration:

```rust
fn generate_systemd_unit(exe_path: &Path, system_level: bool) -> String {
    let user_line = if system_level {
        "User=aisopod\n"
    } else {
        ""
    };

    let wanted_by = if system_level {
        "multi-user.target"
    } else {
        "default.target"
    };

    format!(
        r#"[Unit]
Description=Aisopod AI Gateway
After=network.target

[Service]
Type=simple
{user_line}ExecStart={binary_path} gateway
Restart=on-failure
RestartSec=5
Environment=AISOPOD_CONFIG=/etc/aisopod/config.json

[Install]
WantedBy={wanted_by}
"#,
        user_line = user_line,
        binary_path = exe_path.display(),
        wanted_by = wanted_by
    )
}
```

Key configuration:
- **Type=simple**: Process runs directly
- **Restart=on-failure**: Automatic restart on crashes
- **RestartSec=5**: 5-second delay before restart
- **Environment**: Sets config path for the service

#### 3. User-Level vs System-Level Installation

The implementation handles both installation modes:

```rust
let service_path = if system_level {
    PathBuf::from("/etc/systemd/system/aisopod.service")
} else {
    let home = std::env::var("HOME")
        .with_context(|| "Cannot determine home directory")?;
    let dir = PathBuf::from(&home).join(".config/systemd/user");
    std::fs::create_dir_all(&dir)?;
    dir.join("aisopod.service")
};
```

For user-level services:
- Service file: `~/.config/systemd/user/aisopod.service`
- Commands use `--user` flag: `systemctl --user daemon-reload`
- Service enabled to `default.target`

#### 4. Uninstall Command Implementation

The `uninstall_daemon` function:

1. Detects whether service was installed at user or system level by checking both locations
2. Removes the service file
3. Disables the service with appropriate flags
4. Reloads systemd daemon

```rust
pub fn uninstall_daemon() -> Result<()> {
    if cfg!(target_os = "linux") {
        let system_path = PathBuf::from("/etc/systemd/system/aisopod.service");
        let user_home = std::env::var("HOME")?;
        let user_path = PathBuf::from(&user_home).join(".config/systemd/user/aisopod.service");

        let (service_path, system_level) = if system_path.exists() {
            (system_path, true)
        } else if user_path.exists() {
            (user_path, false)
        } else {
            return Err(anyhow!("aisopod service is not installed"));
        };
        // ... removal and cleanup
    }
}
```

### Platform Support

- **Linux**: Full systemd support with user-level and system-level modes
- **macOS**: LaunchAgent support (unchanged from issue 133)
- **Other platforms**: Returns error "Daemon management not supported on this platform"

### Tests Added

1. `test_daemon_commands_enum` - Updated to match tuple variant
2. `test_daemon_uninstall_command` - Verifies Uninstall variant exists
3. `test_install_args_default` - Verifies default InstallArgs
4. `test_install_args_system` - Verifies InstallArgs with system flag

## Key Technical Decisions

### 1. Auto-Detection in Uninstall

The uninstall command automatically detects whether the service was installed at user or system level by checking both possible locations. This provides a better user experience as users don't need to remember how they installed the service.

### 2. Direct systemctl Integration

The install function automatically:
- Writes the service file
- Runs `systemctl daemon-reload` (or `--user` variant)
- Enables the service with appropriate flags

This ensures the service is immediately usable after installation.

### 3. Environment Variable for Config

The service includes `Environment=AISOPOD_CONFIG=/etc/aisopod/config.json` to ensure the gateway has access to its configuration. Users can override this by modifying the service file after installation.

### 4. Platform Gating with cfg!

The implementation uses `#[cfg(target_os = "linux")]` and `#[cfg(target_os = "macos")]` to ensure platform-specific code only compiles where appropriate. Unsupported platforms receive a clear error message.

## Acceptance Criteria Met

- [x] `aisopod daemon install` generates a valid systemd unit file on Linux
- [x] User-level install writes to `~/.config/systemd/user/aisopod.service`
- [x] System-level install (with `--system`) writes to `/etc/systemd/system/aisopod.service`
- [x] Service type is `simple` with `Restart=on-failure`
- [x] `ExecStart` points to the current aisopod binary path
- [x] Service can be enabled and started with `systemctl enable --now aisopod`
- [x] `aisopod daemon uninstall` removes the service file

## Verification

```bash
# Build passes
env RUSTFLAGS=-Awarnings cargo build

# Tests pass
env RUSTFLAGS=-Awarnings cargo test -p aisopod --lib

# Output shows 40 passed tests
```

## Files Modified

- `crates/aisopod/src/commands/daemon.rs` - Complete implementation of user-level systemd installation and uninstall command

## Related Issues

- Issue 133: Daemon management commands (required dependency)
- Issue 158: launchctl plist generation (macOS equivalent)

## Notes for Future Maintenance

1. When updating the systemd unit file, ensure both user-level and system-level configurations remain compatible
2. The uninstall auto-detection assumes only one instance is installed; multiple installations would require user to specify which to uninstall
3. Consider adding `ConditionPathExists` or other systemd conditions for more robust service management
4. The current implementation doesn't handle the case where systemd-user session isn't running for user-level services
