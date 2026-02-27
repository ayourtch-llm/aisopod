# Migration from OpenClaw to Aisopod

Aisopod is the successor to OpenClaw, rebuilt from the ground up in Rust. This guide helps you migrate your existing OpenClaw installation to Aisopod with minimal disruption.

## Migration Overview

### What's New

Aisopod brings significant improvements over OpenClaw:

- **Rust-native binary** - No Node.js runtime required for improved performance and reduced dependencies
- **TOML configuration** - Replacing JSON5 for better readability and maintainability
- **WebSocket support** - Real-time streaming alongside REST API
- **Plugin system** - Sandboxed execution environment for custom functionality
- **Multi-agent architecture** - Support for multiple specialized agents
- **Enhanced session storage** - SQLite-based persistence for reliability
- **New channels** - Added Slack channel support

### Migration Steps

Follow these steps to migrate your OpenClaw installation to Aisopod:

1. **Install Aisopod** - Download and install the Aisopod binary
2. **Convert your configuration** - Migrate from JSON5 to TOML format
3. **Update environment variables** - Rename from `OPENCLAW_*` to `AISOPOD_*`
4. **Migrate session data** - Import OpenClaw sessions into Aisopod's database
5. **Update deployment scripts** - Modify any scripts or CI/CD pipelines
6. **Verify functionality** - Test all channels and features

## Configuration Migration

OpenClaw uses JSON5; Aisopod uses TOML as its primary format. You can use the automated migration tool or convert manually.

### Automated Migration

Use the `aisopod migrate` command to automatically convert your OpenClaw configuration:

```bash
aisopod migrate --from-openclaw /path/to/openclaw/config.json5
```

This generates an equivalent `config.toml` file in the current directory.

### Manual Migration

#### OpenClaw JSON5 Configuration

```json5
{
  server: {
    port: 3000,
    host: "0.0.0.0"
  },
  auth: {
    type: "bearer",
    token: "sk-..."
  },
  models: {
    default: {
      provider: "openai",
      model: "gpt-4"
    }
  },
  channels: [
    {
      type: "telegram",
      token: "bot-token",
      name: "telegram"
    }
  ]
}
```

#### Aisopod TOML Configuration

```toml
[server]
port = 3080          # note: default port changed from 3000 to 3080
bind_address = "0.0.0.0"

[auth]
mode = "token"       # "type" → "mode", "bearer" → "token"
api_keys = ["sk-..."]

[[models.providers]]
name = "openai"
api_key = "sk-..."

[[models]]
name = "default"
provider = "openai"
model = "gpt-4"

[[channels]]
type = "telegram"
token = "bot-token"
name = "telegram"
agent = "default"    # new: channels must bind to an agent
```

### Configuration Format Comparison

| OpenClaw (JSON5) | Aisopod (TOML) | Notes |
|------------------|----------------|-------|
| `server.port` | `[server].port` | Same structure, TOML format |
| `auth.type = "bearer"` | `[auth].mode = "token"` | Key renamed, value simplified |
| `models.default.provider` | `[[models.providers]].name` | Provider defined separately |
| `models.default.model` | `[[models]].model` | Agent/model structure |
| `channels[].type` | `[[channels]].type` | Same array structure |
| `channels[].agent` | `[[channels]].agent` | **New required field** |

## Environment Variable Mapping

All environment variables have been renamed from `OPENCLAW_*` to `AISOPOD_*` with some naming changes for consistency.

### Complete Environment Variable Table

| OpenClaw Variable | Aisopod Variable | Default | Notes |
|-------------------|------------------|---------|-------|
| `OPENCLAW_PORT` | `AISOPOD_PORT` | `3000` → `8080` | Gateway port changed |
| `OPENCLAW_HOST` | `AISOPOD_HOST` | `127.0.0.1` | Unchanged behavior |
| `OPENCLAW_AUTH_TOKEN` | `AISOPOD_AUTH_TOKEN` | - | For legacy auth mode |
| `OPENCLAW_OPENAI_KEY` | `AISOPOD_OPENAI_API_KEY` | - | API key suffix renamed |
| `OPENCLAW_ANTHROPIC_KEY` | `AISOPOD_ANTHROPIC_API_KEY` | - | API key suffix renamed |
| `OPENCLAW_TELEGRAM_TOKEN` | `AISOPOD_TELEGRAM_TOKEN` | - | Same name, TOML config |
| `OPENCLAW_DISCORD_TOKEN` | `AISOPOD_DISCORD_TOKEN` | - | Same name, TOML config |
| `OPENCLAW_LOG_LEVEL` | `AISOPOD_LOG` | `info` | Shortened name |
| `OPENCLAW_DATA_DIR` | `AISOPOD_DATA_DIR` | Platform-specific | Same purpose |
| `OPENCLAW_CONFIG` | `AISOPOD_CONFIG` | Platform-specific | Same purpose |
| `OPENCLAW_WEBHOOK_URL` | `AISOPOD_WEBHOOK_URL` | - | For webhook channels |

### Quick Rename Script

Use this script to update your `.env` file or deployment scripts:

```bash
# In your .env or deployment scripts, replace:
sed -i 's/OPENCLAW_/AISOPOD_/g' .env
sed -i 's/OPENAI_KEY/OPENAI_API_KEY/g' .env
sed -i 's/ANTHROPIC_KEY/ANTHROPIC_API_KEY/g' .env
sed -i 's/LOG_LEVEL/LOG/g' .env
```

### Environment Variable Usage

Environment variables can be used in TOML configuration files with substitution syntax:

```toml
[models.providers]
api_key = "${AISOPOD_OPENAI_API_KEY}"

[channels]
token = "${AISOPOD_TELEGRAM_TOKEN}"
```

## Feature Parity Checklist

### Supported Features

| Feature | OpenClaw | Aisopod | Notes |
|---------|----------|---------|-------|
| Chat completions API | ✅ | ✅ | Same endpoint path (`/v1/chat/completions`) |
| Streaming responses | ✅ | ✅ | Now also via WebSocket |
| Telegram channel | ✅ | ✅ | Fully compatible |
| Discord channel | ✅ | ✅ | Fully compatible |
| WhatsApp channel | ✅ | ✅ | Fully compatible |
| Slack channel | ❌ | ✅ | New in Aisopod |
| WebSocket API | ❌ | ✅ | New in Aisopod (`/ws`) |
| Multi-agent | ❌ | ✅ | New in Aisopod |
| Plugin system | ❌ | ✅ | Sandboxed execution |
| Session memory | ✅ | ✅ | Storage format changed |
| Code execution sandbox | ✅ | ✅ | Now uses Docker/Wasm |
| Web search | ✅ | ✅ | |
| Custom system prompts | ✅ | ✅ | File reference support added |
| Rate limiting | ✅ | ✅ | Enhanced with configurable limits |
| Request logging | ✅ | ✅ | Improved format |

### New Features in Aisopod

Aisopod introduces several new features not available in OpenClaw:

1. **WebSocket API** - Real-time bidirectional communication
   - Subscribe to chat events
   - Stream responses in real-time
   - Support for multi-modal data

2. **Multi-Agent Architecture** - Multiple specialized agents
   - Create agents for different purposes
   - Route channels to specific agents
   - Each agent has its own model configuration

3. **Plugin System** - Extend functionality safely
   - Sandboxed execution
   - WASM and Docker backends
   - Custom tools and integrations

4. **Slack Channel** - Native Slack integration
   - Full message formatting support
   - File uploads
   - Thread support

## Breaking Changes and Workarounds

### Default Port Changed

- **Old:** `3000` (OpenClaw)
- **New:** `8080` (Aisopod gateway)
- **Workaround:** Set `AISOPOD_PORT=3000` or update your reverse proxy and client configurations

### Channel Agent Binding Required

- **Old:** Channels auto-routed to default model
- **New:** Each channel must explicitly set `agent = "name"`
- **Workaround:** Create a `default` agent and bind all channels to it

**OpenClaw (no agent binding):**
```json5
{
  channels: [
    { type: "telegram", token: "bot-token" }
  ]
}
```

**Aisopod (agent binding required):**
```toml
[[agents]]
name = "default"
model = "gpt-4"

[[channels]]
type = "telegram"
token = "bot-token"
agent = "default"
```

### Auth Config Key Renamed

- **Old:** `auth.type = "bearer"`
- **New:** `auth.mode = "token"`
- **Workaround:** Update configuration to use `mode` instead of `type`

### Session Storage Format Changed

- **Old:** JSON files in `~/.openclaw/sessions/`
- **New:** SQLite database at `~/.local/share/aisopod/sessions.db`
- **Workaround:** Use `aisopod migrate --sessions` to import existing sessions

**Data locations:**
- **OpenClaw:** `~/.openclaw/` (Linux) or `%APPDATA%\openclaw\` (Windows)
- **Aisopod:** `~/.local/share/aisopod/` (Linux) or `%LOCALAPPDATA%\aisopod\` (Windows)

### CLI Command Structure

- **Old:** `openclaw gateway` or `openclaw channel`
- **New:** `aisopod gateway` or `aisopod channel`
- **Workaround:** Update scripts and aliases to use `aisopod`

## Data Migration

### Session History

Migrate your OpenClaw session history to Aisopod:

```bash
aisopod migrate --sessions --from ~/.openclaw/sessions/
```

This command imports OpenClaw session JSON files into Aisopod's SQLite database.

### Memory / Knowledge Base

Migrate your OpenClaw memory and knowledge base:

```bash
aisopod migrate --memory --from ~/.openclaw/memory/
```

This imports memory entries and knowledge base documents.

### Full Data Migration

To migrate all data at once:

```bash
aisopod migrate --from-openclaw ~/.openclaw/
```

This command will:
1. Convert configuration to TOML
2. Import session history
3. Import memory and knowledge base
4. Generate a report of any issues found

### Verification

After migration, verify the data integrity:

```bash
aisopod doctor --check-data
```

This command checks:
- Database integrity
- Configuration validity
- Channel connectivity
- Model availability

### Migration Report

The `aisopod migrate` command generates a detailed report:

```
Migration Report
================
Configuration: ✅ Converted to TOML
Sessions:      ✅ 42 sessions imported
Memory:        ✅ 15 memory entries imported
Warnings:      ⚠️  2 channels missing agent binding (see details below)
Errors:        ❌ 0
```

## Aisopod Migration Utility

The `aisopod migrate` command provides comprehensive migration support.

### Usage

```bash
# Migrate from OpenClaw configuration
aisopod migrate --from-openclaw /path/to/openclaw/config.json5

# Migrate sessions only
aisopod migrate --sessions --from ~/.openclaw/sessions/

# Migrate memory only
aisopod migrate --memory --from ~/.openclaw/memory/

# Full OpenClaw migration
aisopod migrate --from-openclaw ~/.openclaw/

# Dry run (no changes)
aisopod migrate --dry-run --from-openclaw ~/.openclaw/
```

### Options

| Option | Description |
|--------|-------------|
| `--from-openclaw <PATH>` | Migrate from OpenClaw directory |
| `--sessions` | Migrate session history |
| `--memory` | Migrate memory/knowledge base |
| `--config` | Output configuration file path |
| `--dry-run` | Show what would be migrated without changes |
| `--verbose` | Show detailed progress |

## Deployment Considerations

### Docker Migration

If using Docker, update your `docker-compose.yml`:

**Before (OpenClaw):**
```yaml
services:
  openclaw:
    image: openclaw/openclaw:latest
    ports:
      - "3000:3000"
    environment:
      - OPENCLAW_PORT=3000
      - OPENCLAW_OPENAI_KEY=${OPENAI_KEY}
```

**After (Aisopod):**
```yaml
services:
  aisopod:
    image: aisopod/aisopod:latest
    ports:
      - "8080:8080"
    environment:
      - AISOPOD_PORT=8080
      - AISOPOD_OPENAI_API_KEY=${OPENAI_KEY}
    volumes:
      - aisopod-data:/data

volumes:
  aisopod-data:
```

### Kubernetes Migration

Update your Kubernetes manifests to use `AISOPOD_*` variables and the new port.

## Troubleshooting

### Common Issues

**Issue:** Channel not receiving messages
- **Cause:** Channel not bound to an agent
- **Fix:** Add `agent = "default"` to channel configuration

**Issue:** 401 Unauthorized errors
- **Cause:** Old `auth.type` config not recognized
- **Fix:** Change `auth.type` to `auth.mode` with appropriate mode

**Issue:** Sessions not loading
- **Cause:** Migration not performed
- **Fix:** Run `aisopod migrate --sessions --from ~/.openclaw/sessions/`

**Issue:** Port already in use
- **Cause:** Aisopod uses port 8080 by default
- **Fix:** Set `AISOPOD_PORT=3000` or stop existing service

### Getting Help

If you encounter issues during migration:

1. Check the troubleshooting section of the documentation
2. Run `aisopod doctor` to diagnose common issues
3. Join the community Discord for real-time support
