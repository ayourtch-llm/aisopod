# Issue 157: Implement systemd Service File Generation

## Summary
Implement the `aisopod daemon install` command on Linux to generate and install a systemd `.service` file that manages the aisopod gateway as a system service with automatic restart on failure.

## Location
- Crate: `aisopod-cli`
- File: `crates/aisopod-cli/src/commands/daemon.rs`

## Current Behavior
The `aisopod daemon install` command is defined but does not generate or install a systemd service file on Linux systems.

## Expected Behavior
Running `aisopod daemon install` on Linux generates a systemd `.service` file and installs it to the appropriate location:
- User-level: `~/.config/systemd/user/aisopod.service`
- System-level (with `--system` flag): `/etc/systemd/system/aisopod.service`

## Impact
Enables Linux users to run aisopod as a managed background service with automatic restarts, proper logging integration, and system boot startup.

## Suggested Implementation

1. Add a function to generate the systemd unit file content:

```rust
fn generate_systemd_unit(binary_path: &str, system_level: bool) -> String {
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
        user_line = if system_level { "User=aisopod\n" } else { "" },
        binary_path = binary_path,
        wanted_by = if system_level {
            "multi-user.target"
        } else {
            "default.target"
        },
    )
}
```

2. Add the install logic in the daemon command handler:

```rust
fn install_systemd_service(system_level: bool) -> Result<()> {
    let binary_path = std::env::current_exe()?
        .to_string_lossy()
        .to_string();

    let unit_content = generate_systemd_unit(&binary_path, system_level);

    let service_path = if system_level {
        PathBuf::from("/etc/systemd/system/aisopod.service")
    } else {
        let home = std::env::var("HOME")?;
        let dir = PathBuf::from(&home).join(".config/systemd/user");
        std::fs::create_dir_all(&dir)?;
        dir.join("aisopod.service")
    };

    std::fs::write(&service_path, unit_content)?;
    println!("Service file written to {}", service_path.display());
    println!("Run: systemctl {} enable --now aisopod",
        if system_level { "" } else { "--user " });
    Ok(())
}
```

3. Gate the implementation behind `#[cfg(target_os = "linux")]` so it only compiles on Linux.

## Dependencies
- Issue 133 (daemon commands)

## Acceptance Criteria
- [x] `aisopod daemon install` generates a valid systemd unit file on Linux
- [x] User-level install writes to `~/.config/systemd/user/aisopod.service`
- [x] System-level install (with `--system`) writes to `/etc/systemd/system/aisopod.service`
- [x] Service type is `simple` with `Restart=on-failure`
- [x] `ExecStart` points to the current aisopod binary path
- [x] Service can be enabled and started with `systemctl enable --now aisopod`
- [x] `aisopod daemon uninstall` removes the service file

## Resolution
Implemented `aisopod daemon install` command with user/system-level installation support:
- Added `--system` flag for system-level installation
- User-level install: writes to `~/.config/systemd/user/aisopod.service`
- System-level install (with `--system`): writes to `/etc/systemd/system/aisopod.service`
- Service unit file includes: `Type=simple`, `Restart=on-failure`, `RestartSec=5`
- `ExecStart` uses `std::env::current_exe()` to get the aisopod binary path
- Environment variable `AISOPOD_CONFIG=/etc/aisopod/config.json` set
- Platform-gated with `#[cfg(target_os = "linux")]`
- Implemented `aisopod daemon uninstall` command to remove service file
- All changes committed in commit 49c1764

---
*Created: 2026-02-15*
*Resolved: 2026-02-25*
