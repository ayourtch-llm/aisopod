# Issue 133: Implement Daemon Management Commands

## Summary
Implement the `aisopod daemon` subcommands for installing, starting, stopping, checking status of, and tailing logs for the aisopod background service on Linux (systemd) and macOS (launchctl).

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/commands/daemon.rs`

## Current Behavior
The daemon subcommand is a stub that panics with `todo!`. There is no way to manage aisopod as a background service.

## Expected Behavior
Users can install aisopod as a system service, manage its lifecycle (start/stop/status), and tail its logs. The implementation auto-detects the platform and generates the appropriate service configuration (systemd unit file on Linux, launchctl plist on macOS).

## Impact
Running as a daemon is the recommended production deployment mode. This enables aisopod to start automatically on boot and run continuously in the background.

## Suggested Implementation

1. Define the daemon subcommand and its nested subcommands:

```rust
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub command: DaemonCommands,
}

#[derive(Subcommand)]
pub enum DaemonCommands {
    /// Install aisopod as a system service
    Install,
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Show daemon status
    Status,
    /// Tail daemon logs
    Logs {
        /// Number of lines to show
        #[arg(long, default_value_t = 50)]
        lines: usize,

        /// Follow log output
        #[arg(long, short)]
        follow: bool,
    },
}
```

2. Implement the install handler with platform detection:

```rust
use std::process::Command;

pub fn install_daemon() -> anyhow::Result<()> {
    let exe_path = std::env::current_exe()?;

    if cfg!(target_os = "linux") {
        install_systemd_service(&exe_path)?;
    } else if cfg!(target_os = "macos") {
        install_launchctl_service(&exe_path)?;
    } else {
        anyhow::bail!("Daemon installation not supported on this platform");
    }

    Ok(())
}

fn install_systemd_service(exe_path: &std::path::Path) -> anyhow::Result<()> {
    let unit = format!(
        r#"[Unit]
Description=aisopod AI Agent Orchestration Platform
After=network.target

[Service]
Type=simple
ExecStart={} gateway
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
"#,
        exe_path.display()
    );

    let service_path = "/etc/systemd/system/aisopod.service";
    std::fs::write(service_path, unit)?;

    Command::new("systemctl").args(["daemon-reload"]).status()?;
    Command::new("systemctl").args(["enable", "aisopod"]).status()?;

    println!("Systemd service installed and enabled.");
    println!("Start with: aisopod daemon start");
    Ok(())
}

fn install_launchctl_service(exe_path: &std::path::Path) -> anyhow::Result<()> {
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.aisopod.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>gateway</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/usr/local/var/log/aisopod.log</string>
    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/aisopod.err</string>
</dict>
</plist>
"#,
        exe_path.display()
    );

    let plist_path = dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents/com.aisopod.daemon.plist");
    std::fs::write(&plist_path, plist)?;

    println!("LaunchAgent plist installed at {}", plist_path.display());
    println!("Start with: aisopod daemon start");
    Ok(())
}
```

3. Implement start/stop/status:

```rust
pub fn start_daemon() -> anyhow::Result<()> {
    if cfg!(target_os = "linux") {
        Command::new("systemctl").args(["start", "aisopod"]).status()?;
    } else if cfg!(target_os = "macos") {
        Command::new("launchctl").args(["load", &plist_path()]).status()?;
    }
    println!("Daemon started.");
    Ok(())
}

pub fn stop_daemon() -> anyhow::Result<()> {
    if cfg!(target_os = "linux") {
        Command::new("systemctl").args(["stop", "aisopod"]).status()?;
    } else if cfg!(target_os = "macos") {
        Command::new("launchctl").args(["unload", &plist_path()]).status()?;
    }
    println!("Daemon stopped.");
    Ok(())
}

pub fn daemon_status() -> anyhow::Result<()> {
    if cfg!(target_os = "linux") {
        Command::new("systemctl").args(["status", "aisopod"]).status()?;
    } else if cfg!(target_os = "macos") {
        Command::new("launchctl").args(["list", "com.aisopod.daemon"]).status()?;
    }
    Ok(())
}
```

4. Implement log tailing:

```rust
pub fn tail_logs(lines: usize, follow: bool) -> anyhow::Result<()> {
    if cfg!(target_os = "linux") {
        let mut args = vec!["--unit=aisopod", &format!("--lines={}", lines)];
        if follow { args.push("--follow"); }
        Command::new("journalctl").args(&args).status()?;
    } else if cfg!(target_os = "macos") {
        let log_path = "/usr/local/var/log/aisopod.log";
        let mut args = vec!["-n", &lines.to_string(), log_path];
        if follow { args.push("-f"); }
        Command::new("tail").args(&args).status()?;
    }
    Ok(())
}
```

## Dependencies
- Issue 124 (clap CLI framework)

## Acceptance Criteria
- [ ] `aisopod daemon install` generates and installs a systemd service file on Linux
- [ ] `aisopod daemon install` generates and installs a launchctl plist on macOS
- [ ] `aisopod daemon start` starts the background service
- [ ] `aisopod daemon stop` stops the background service
- [ ] `aisopod daemon status` shows whether the daemon is running
- [ ] `aisopod daemon logs` shows recent log output
- [ ] `aisopod daemon logs --follow` streams log output in real time
- [ ] Unsupported platforms produce a clear error message

---
*Created: 2026-02-15*
