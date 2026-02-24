# Daemon Management Commands Implementation

## Summary

This learning document captures key insights from implementing the `aisopod daemon` subcommand family for managing the aisopod background service on Linux (systemd) and macOS (launchctl).

## Platform Detection

### Rust's `cfg` Attributes

Rust provides compile-time platform detection through `cfg` attributes:

```rust
if cfg!(target_os = "linux") {
    // Linux-specific code
} else if cfg!(target_os = "macos") {
    // macOS-specific code
} else {
    anyhow::bail!("Unsupported platform");
}
```

These attributes are evaluated at compile time, allowing different code paths for different platforms while maintaining a single codebase.

### Platform-Specific Considerations

#### Linux (systemd)

- Service files are placed in `/etc/systemd/system/`
- Requires root privileges for installation and management
- Commands used:
  - `systemctl daemon-reload` - Reload systemd configuration
  - `systemctl enable aisopod` - Enable auto-start on boot
  - `systemctl start aisopod` - Start the service
  - `systemctl stop aisopod` - Stop the service
  - `systemctl status aisopod` - Check service status
  - `journalctl --unit=aisopod --lines=N --follow` - Tail logs

#### macOS (launchctl)

- Service definitions are placed in user's `~/Library/LaunchAgents/`
- Uses Property List (plist) format for configuration
- Commands used:
  - `launchctl load ~/Library/LaunchAgents/com.aisopod.daemon.plist` - Load and start
  - `launchctl unload ~/Library/LaunchAgents/com.aisopod.daemon.plist` - Stop and unload
  - `launchctl list com.aisopod.daemon` - Check status
  - `tail -n N -f /usr/local/var/log/aisopod.log` - Tail logs

## Command Line Argument Parsing with Clap

### Args and Subcommand Patterns

Clap's `Args` and `Subcommand` traits enable elegant CLI design:

```rust
#[derive(Args)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub command: DaemonCommands,
}

#[derive(Subcommand)]
pub enum DaemonCommands {
    Install,
    Start,
    Stop,
    Status,
    Logs {
        #[arg(long, default_value_t = 50)]
        lines: usize,
        #[arg(long, short)]
        follow: bool,
    },
}
```

### Default Values and Short Options

- `default_value_t` provides type-safe default values
- `short` attribute enables single-character flags (e.g., `-f` for `--follow`)

## lifetime Management in Command Arguments

### Temporary Value Issue

When using temporary values in vectors, Rust's borrow checker can flag lifetime issues:

```rust
// ❌ Incorrect - temporary value dropped
let mut args = vec!["--unit=aisopod", &format!("--lines={}", lines)];
```

### Solution

Store temporary values in variables before using them:

```rust
// ✅ Correct - binding extends lifetime
let lines_arg = format!("--lines={}", lines);
let mut args = vec!["--unit=aisopod", &lines_arg];
```

This ensures the temporary value lives long enough to be used by the command.

## Error Handling with Anyhow

### Contextual Error Messages

The `anyhow` crate provides `with_context` for adding helpful context:

```rust
Command::new("systemctl")
    .args(["start", "aisopod"])
    .status()
    .with_context(|| "Failed to start aisopod service")?;
```

This produces error messages like:
```
Failed to start aisopod service: No such file or directory (os error 2)
```

### Early Exit for Unsupported Platforms

Using `anyhow::bail!` for clear error messages:

```rust
anyhow::bail!("Daemon installation not supported on this platform");
```

## File System Operations

### Writing Service Files

```rust
std::fs::write(service_path, unit)
    .with_context(|| format!("Failed to write systemd service file to {}", service_path))?;
```

### Home Directory Detection

The `dirs` crate provides cross-platform home directory detection:

```rust
use dirs;

let home_dir = dirs::home_dir()
    .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
```

## Testing Strategy

### Unit Tests for Command Parsing

Tests verify CLI argument parsing works correctly:

```rust
#[test]
fn test_daemon_logs_args() {
    let args = DaemonArgs {
        command: DaemonCommands::Logs {
            lines: 100,
            follow: true,
        },
    };

    match args.command {
        DaemonCommands::Logs { lines, follow } => {
            assert_eq!(lines, 100);
            assert!(follow);
        }
        _ => assert!(false),
    }
}
```

## Deployment Considerations

### Service File Locations

| Platform | Service File Path |
|----------|-------------------|
| Linux | `/etc/systemd/system/aisopod.service` |
| macOS | `~/Library/LaunchAgents/com.aisopod.daemon.plist` |

### Prerequisites

- **Linux**: User must have sudo privileges to write to `/etc/systemd/system/` and run systemctl
- **macOS**: User must have write access to their `~/Library/LaunchAgents/` directory

### Service Configuration

The generated service files:
- Run as `gateway` subcommand
- Auto-restart on failure (Linux)
- Keep-alive enabled (macOS)
- Log to appropriate system logs

## Code Organization

### Module Structure

```
crates/aisopod/
├── src/
│   ├── cli.rs              # CLI entry point, dispatches to commands
│   ├── commands/
│   │   ├── mod.rs          # Module exports
│   │   ├── daemon.rs       # Daemon management commands
│   │   ├── agent.rs        # Agent management
│   │   └── ...             # Other commands
│   └── main.rs             # Binary entry point
```

### Dispatch Pattern

```rust
Commands::Daemon(args) => {
    crate::commands::daemon::run(args).expect("Daemon command failed");
}
```

This pattern maintains consistency across all command handlers.

## Future Enhancements

Potential improvements for future work:

1. **Configuration file path handling**: Allow specifying custom config paths for daemon mode
2. **Service status parsing**: Parse and display structured status information
3. **Service file templates**: Make service files configurable via command options
4. **Health checks**: Add integration with gateway health endpoints
5. **Multi-user support**: Support installing system-wide vs user-level services

## References

- [systemd Documentation](https://www.freedesktop.org/software/systemd/manual/systemd.syntax.html)
- [launchd Documentation](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html)
- [Clap Documentation](https://docs.rs/clap/latest/clap/)
- [Anyhow Documentation](https://docs.rs/anyhow/latest/anyhow/)
