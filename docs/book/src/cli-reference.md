# CLI Command Reference

The `aisopod` CLI is the primary interface for interacting with the aisopod AI agent orchestration platform. This guide documents all commands, subcommands, flags, and options available.

## CLI Overview

```
aisopod [OPTIONS] <COMMAND>
```

### Global Options

| Option | Description |
|--------|-------------|
| `-c, --config <PATH>` | Path to configuration file |
| `-v, --verbose` | Enable verbose output |
| `--json` | Output in JSON format |
| `--version` | Print version |
| `-h, --help` | Print help |

### Commands

| Command | Description |
|---------|-------------|
| [`aisopod gateway`](#aisopod-gateway) | Manage the gateway server |
| [`aisopod agent`](#aisopod-agent) | Manage agents |
| [`aisopod message`](#aisopod-message) | Send a message to an agent |
| [`aisopod config`](#aisopod-config) | Manage configuration |
| [`aisopod status`](#aisopod-status) | Show system status |
| [`aisopod health`](#aisopod-health) | Run health check |
| [`aisopod dashboard`](#aisopod-dashboard) | Display live dashboard |
| [`aisopod models`](#aisopod-models) | Manage models |
| [`aisopod channels`](#aisopod-channels) | Manage channels |
| [`aisopod sessions`](#aisopod-sessions) | Manage sessions |
| [`aisopod daemon`](#aisopod-daemon) | Manage background daemon |
| [`aisopod doctor`](#aisopod-doctor) | Run system diagnostics |
| [`aisopod auth`](#aisopod-auth) | Manage authentication |
| [`aisopod reset`](#aisopod-reset) | Reset all sessions |
| [`aisopod completions`](#aisopod-completions) | Generate shell completions |
| [`aisopod onboarding`](#aisopod-onboarding) | Run onboarding wizard |
| [`aisopod migrate`](#aisopod-migrate) | Migrate from OpenClaw |

## Commands

### `aisopod gateway`

Manage the Aisopod gateway server.

#### `aisopod gateway`

Start the gateway server.

```
aisopod gateway [OPTIONS]
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--bind <ADDR>` | Address to bind the server to | `127.0.0.1` |
| `--port <PORT>` | Port to listen on | `3000` |
| `--allow-unconfigured` | Allow requests to unconfigured agents | (not set) |

**Environment Variables:**

- `AISOPOD_CONFIG` - Path to configuration file

**Examples:**

```bash
# Start with defaults
aisopod gateway

# Start on all interfaces, custom port
aisopod gateway --bind 0.0.0.0 --port 8080

# Allow unconfigured agents
aisopod gateway --allow-unconfigured
```

---

### `aisopod agent`

Manage agents.

#### `aisopod agent list`

List all configured agents.

```
aisopod agent list
```

**Examples:**

```bash
# List all agents
aisopod agent list
```

---

#### `aisopod agent add <ID>`

Add a new agent with interactive prompts.

```
aisopod agent add <ID>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Unique agent identifier |

**Examples:**

```bash
# Add a new agent interactively
aisopod agent add my-agent
```

---

#### `aisopod agent delete <ID>`

Delete an agent by ID.

```
aisopod agent delete <ID>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Agent identifier to remove |

**Examples:**

```bash
# Delete an agent
aisopod agent delete my-agent
```

---

#### `aisopod agent identity <ID>`

Show agent identity and configuration.

```
aisopod agent identity <ID>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Agent identifier to inspect |

**Examples:**

```bash
# Show agent details
aisopod agent identity my-agent
```

---

### `aisopod message`

Send a one-shot message to an agent.

```
aisopod message <TEXT> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<TEXT>` | Message text to send |

**Options:**

| Option | Description |
|--------|-------------|
| `--channel <CHANNEL>` | Target channel |
| `--agent <AGENT>` | Target agent ID (uses default if not specified) |

**Examples:**

```bash
# Send a message to the default agent
aisopod message "Hello, world!"

# Send to a specific channel
aisopod message "Hello!" --channel telegram

# Send to a specific agent
aisopod message "Hello!" --agent my-agent
```

---

### `aisopod config`

Manage configuration.

#### `aisopod config show`

Display current configuration with sensitive fields redacted.

```
aisopod config show
```

**Examples:**

```bash
# Show current configuration
aisopod config show
```

---

#### `aisopod config set <KEY> <VALUE>`

Set a configuration value by key path.

```
aisopod config set <KEY> <VALUE>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<KEY>` | Configuration key (dot-separated path) |
| `<VALUE>` | New value |

**Examples:**

```bash
# Set gateway port
aisopod config set gateway.server.port 8080

# Set model provider endpoint
aisopod config set models.providers.0.endpoint https://api.openai.com/v1
```

---

#### `aisopod config wizard`

Run interactive setup wizard for first-time configuration.

```
aisopod config wizard
```

**Examples:**

```bash
# Run configuration wizard
aisopod config wizard
```

---

#### `aisopod config channels`

Interactive channel configuration helper.

```
aisopod config channels
```

**Examples:**

```bash
# Run channel configuration helper
aisopod config channels
```

---

#### `aisopod config init [TEMPLATE] [OPTIONS]`

Initialize a new configuration file from a template.

```
aisopod config init [TEMPLATE] [OPTIONS]
```

**Arguments:**

| Argument | Description | Default |
|----------|-------------|---------|
| `<TEMPLATE>` | Template name (dev, production, docker) | `dev` |

**Options:**

| Option | Description |
|--------|-------------|
| `-o, --output <PATH>` | Output file path |

**Examples:**

```bash
# Initialize with dev template
aisopod config init

# Initialize with production template to specific path
aisopod config init production --output /etc/aisopod/config.json

# Initialize with default template to specific path
aisopod config init -o my-config.json
```

---

### `aisopod status`

Show system status.

```
aisopod status [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--detailed` | Show extended details |

**Examples:**

```bash
# Show basic status
aisopod status

# Show detailed status
aisopod status --detailed
```

---

### `aisopod health`

Run health check.

```
aisopod health [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--json` | Output in JSON format |

**Examples:**

```bash
# Run health check
aisopod health

# Output as JSON
aisopod health --json
```

---

### `aisopod dashboard`

Display live dashboard with continuously updating status.

```
aisopod dashboard
```

**Examples:**

```bash
# Start dashboard
aisopod dashboard
```

---

### `aisopod models`

Manage models.

#### `aisopod models list [OPTIONS]`

List available models across all providers.

```
aisopod models list [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--provider <PROVIDER>` | Filter by provider name |
| `--json` | Output in JSON format |

**Examples:**

```bash
# List all models
aisopod models list

# List models from a specific provider
aisopod models list --provider openai

# Output as JSON
aisopod models list --json
```

---

#### `aisopod models switch <MODEL> [OPTIONS]`

Switch the primary model for the default agent.

```
aisopod models switch <MODEL> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<MODEL>` | Model identifier (e.g., gpt-4, claude-3-opus) |

**Options:**

| Option | Description |
|--------|-------------|
| `--json` | Output in JSON format |

**Examples:**

```bash
# Switch to gpt-4
aisopod models switch gpt-4

# Switch with JSON output
aisopod models switch claude-3-opus --json
```

---

### `aisopod channels`

Manage channels.

#### `aisopod channels list`

List configured channels and their status.

```
aisopod channels list
```

**Examples:**

```bash
# List all channels
aisopod channels list
```

---

#### `aisopod channels setup <CHANNEL>`

Interactive channel setup wizard.

```
aisopod channels setup <CHANNEL>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<CHANNEL>` | Channel type (telegram, discord, whatsapp, slack, nextcloud) |

**Examples:**

```bash
# Setup Telegram channel
aisopod channels setup telegram

# Setup Discord channel
aisopod channels setup discord
```

---

#### `aisopod channels create <NAME>`

Create a new channel plugin from template.

```
aisopod channels create <NAME>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<NAME>` | Channel name in kebab-case (e.g., "my-channel") |

**Examples:**

```bash
# Create a new channel
aisopod channels create my-channel
```

---

### `aisopod sessions`

Manage sessions.

#### `aisopod sessions list [OPTIONS]`

List active sessions with metadata.

```
aisopod sessions list [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--agent <AGENT>` | Filter by agent ID |
| `--channel <CHANNEL>` | Filter by channel |

**Examples:**

```bash
# List all sessions
aisopod sessions list

# Filter by agent
aisopod sessions list --agent my-agent

# Filter by channel
aisopod sessions list --channel telegram
```

---

#### `aisopod sessions clear [OPTIONS]`

Clear session history.

```
aisopod sessions clear [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--id <ID>` | Specific session ID to clear (clears all if omitted) |

**Examples:**

```bash
# Clear all sessions
aisopod sessions clear

# Clear specific session
aisopod sessions clear --id session-123
```

---

### `aisopod daemon`

Manage background daemon service.

#### `aisopod daemon install [OPTIONS]`

Install aisopod as a system service.

```
aisopod daemon install [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--system` | Install at system level (requires sudo) |

**Examples:**

```bash
# Install as user service
aisopod daemon install

# Install as system service
aisopod daemon install --system
```

---

#### `aisopod daemon uninstall`

Remove aisopod system service.

```
aisopod daemon uninstall
```

**Examples:**

```bash
# Uninstall daemon
aisopod daemon uninstall
```

---

#### `aisopod daemon start`

Start the daemon.

```
aisopod daemon start
```

**Examples:**

```bash
# Start daemon
aisopod daemon start
```

---

#### `aisopod daemon stop`

Stop the daemon.

```
aisopod daemon stop
```

**Examples:**

```bash
# Stop daemon
aisopod daemon stop
```

---

#### `aisopod daemon status`

Show daemon status.

```
aisopod daemon status
```

**Examples:**

```bash
# Check daemon status
aisopod daemon status
```

---

#### `aisopod daemon logs [OPTIONS]`

Tail daemon logs.

```
aisopod daemon logs [OPTIONS]
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--lines <N>` | Number of lines to show | `50` |
| `-f, --follow` | Follow log output | (not set) |

**Examples:**

```bash
# Show last 50 log lines
aisopod daemon logs

# Follow logs in real-time
aisopod daemon logs --follow

# Show last 100 lines
aisopod daemon logs --lines 100
```

---

### `aisopod doctor`

Run system diagnostics.

```
aisopod doctor [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--verbose` | Run extended diagnostics |

**Examples:**

```bash
# Run basic diagnostics
aisopod doctor

# Run extended diagnostics
aisopod doctor --verbose
```

---

### `aisopod auth`

Manage authentication.

#### `aisopod auth setup`

Interactive authentication setup.

```
aisopod auth setup
```

**Examples:**

```bash
# Setup authentication
aisopod auth setup
```

---

#### `aisopod auth status`

Show current auth status.

```
aisopod auth status
```

**Examples:**

```bash
# Check auth status
aisopod auth status
```

---

### `aisopod reset`

Reset all sessions and conversation history.

```
aisopod reset
```

**Examples:**

```bash
# Reset all sessions
aisopod reset
```

---

### `aisopod completions`

Generate shell completions.

```
aisopod completions <SHELL>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<SHELL>` | Shell to generate completions for (bash, zsh, fish, powershell, elvish) |

**Examples:**

```bash
# Generate bash completions
aisopod completions bash

# Generate zsh completions
aisopod completions zsh
```

---

### `aisopod onboarding`

Interactive onboarding wizard for first-time users.

```
aisopod onboarding [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--config <PATH>` | Path to configuration file |

**Examples:**

```bash
# Run onboarding wizard
aisopod onboarding

# Run with specific config path
aisopod onboarding --config /path/to/config.json
```

---

### `aisopod migrate`

Migrate configuration from other formats.

#### `aisopod migrate from-openclaw [OPTIONS]`

Migrate configuration from OpenClaw to aisopod format.

```
aisopod migrate from-openclaw [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-i, --input <PATH>` | Path to input OpenClaw configuration file |
| `-o, --output <PATH>` | Path to output aisopod configuration file |

**Examples:**

```bash
# Migrate from OpenClaw config
aisopod migrate from-openclaw --input openclaw-config.json5 --output aisopod-config.json
```

---

## Shell Completions

Generate completion scripts for your shell:

### Bash

```bash
aisopod completions bash > ~/.local/share/bash-completion/completions/aisopod
```

### Zsh

```bash
aisopod completions zsh > ~/.zfunc/_aisopod
# Add to ~/.zshrc:
fpath=(~/.zfunc $fpath)
autoload -Uz compinit
compinit
```

### Fish

```bash
aisopod completions fish > ~/.config/fish/completions/aisopod.fish
```

### PowerShell

```powershell
aisopod completions powershell >> $PROFILE
```

---

## Environment Variables

The following environment variables can be used to override configuration settings:

| Variable | Description |
|----------|-------------|
| `AISOPOD_CONFIG` | Path to configuration file |
| `AISOPOD_HOST` | Gateway host/bind address (overrides `--host`) |
| `AISOPOD_PORT` | Gateway port (overrides `--port`) |
| `AISOPOD_*` | Provider API keys and settings |

See the documentation for each command for command-specific environment variables.

---

## Exit Codes

| Code | Description |
|------|-------------|
| `0` | Success |
| `1` | Error or health check failure |
| `2` | Invalid command or arguments |
