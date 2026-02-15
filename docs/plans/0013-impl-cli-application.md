# 0013 — CLI Application

**Master Plan Reference:** Section 3.11 — CLI Application  
**Phase:** 6 (User Interface)  
**Dependencies:** 0002 (Configuration), 0003 (Gateway), 0006 (Agent Engine)

---

## Objective

Implement the aisopod command-line interface using `clap`, providing all
essential commands for agent management, configuration, channel setup,
daemon control, and system administration.

---

## Deliverables

### 1. CLI Framework

Build on `clap` with derive macros:

```rust
#[derive(Parser)]
#[command(name = "aisopod", about = "Personal AI assistant gateway")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Config file path
    #[arg(long, default_value = "~/.aisopod/aisopod.json")]
    pub config: PathBuf,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the gateway server
    Gateway(GatewayArgs),
    /// Manage agents
    Agent(AgentArgs),
    /// Send a message
    Message(MessageArgs),
    /// Manage configuration
    Config(ConfigArgs),
    /// Show status
    Status(StatusArgs),
    // ... more commands
}
```

### 2. Command Groups

Port all command groups from OpenClaw's `src/commands/`:

**Gateway commands:**
- `aisopod gateway` — Start the gateway server
- `aisopod gateway --bind <addr>` — Bind to specific address
- `aisopod gateway --allow-unconfigured` — Allow unconfigured startup

**Agent commands:**
- `aisopod agent list` — List configured agents
- `aisopod agent add <id>` — Add a new agent
- `aisopod agent delete <id>` — Remove an agent
- `aisopod agent identity <id>` — Show agent identity

**Message commands:**
- `aisopod message <text>` — Send a message to the active agent
- `aisopod message --channel <ch> <text>` — Send to specific channel

**Config commands:**
- `aisopod config show` — Display current configuration
- `aisopod config set <key> <value>` — Update a config value
- `aisopod config wizard` — Interactive setup wizard
- `aisopod config channels` — Configure channels interactively

**Status commands:**
- `aisopod status` — Show system status
- `aisopod health` — Health check
- `aisopod dashboard` — Live dashboard view

**Model commands:**
- `aisopod models list` — List available models
- `aisopod models switch <model>` — Switch primary model

**Channel commands:**
- `aisopod channels list` — List configured channels
- `aisopod channels setup <channel>` — Setup wizard for a channel

**Session commands:**
- `aisopod sessions list` — List active sessions
- `aisopod sessions clear` — Clear session history
- `aisopod reset` — Reset all sessions

**Daemon commands:**
- `aisopod daemon install` — Install as system service
- `aisopod daemon start` — Start the daemon
- `aisopod daemon stop` — Stop the daemon
- `aisopod daemon status` — Daemon status
- `aisopod daemon logs` — View daemon logs

**Diagnostic commands:**
- `aisopod doctor` — Run diagnostics
- `aisopod sandbox explain` — Explain sandbox setup

**Auth commands:**
- `aisopod auth setup` — Interactive auth setup
- `aisopod auth status` — Show auth profile status

### 3. Interactive Onboarding Wizard

Port the guided setup experience:
- Welcome screen with aisopod branding
- Auth provider selection (API key, OAuth)
- Model selection
- Channel setup (optional, per-channel wizards)
- Configuration summary and confirmation
- First test message

### 4. Gateway Client

For commands that communicate with a running gateway:
- WebSocket client connection to gateway
- RPC method invocation
- Streaming response handling
- Auth token management

### 5. Output Formatting

- Colored terminal output (using `colored` or `termcolor`)
- Table formatting for list commands (using `tabled` or `comfy-table`)
- JSON output mode (`--json` flag) for scripting
- Progress indicators for long-running operations
- Markdown rendering in terminal (for help/docs)

### 6. Shell Completions

- Generate completions for bash, zsh, fish, PowerShell
- `aisopod completions <shell>` command
- Dynamic completions for agent names, channels, models

### 7. Daemon Management

**Linux:** systemd service file generation and management
**macOS:** launchctl plist generation and management
**Windows:** Windows Service support (future)

---

## Acceptance Criteria

- [ ] CLI parses all commands and arguments correctly
- [ ] Gateway starts and listens on configured port
- [ ] Agent CRUD commands work
- [ ] Config wizard completes setup successfully
- [ ] Status/health commands display accurate information
- [ ] Channel setup wizards complete for Tier 1 channels
- [ ] Daemon install/start/stop/status works on Linux and macOS
- [ ] JSON output mode works for all list commands
- [ ] Shell completions generate for bash/zsh/fish
- [ ] Interactive onboarding guides new users through setup
- [ ] Error messages are clear and actionable
