# Configuration

Aisopod uses a flexible configuration system that supports both TOML and JSON5 formats. This guide covers all configuration options, environment variables, and file formats.

## Configuration File Format

Aisopod supports **TOML** and **JSON5** as configuration formats.

### Default Configuration Location

The default configuration file is searched in this order:

1. Path specified by `--config` CLI flag
2. Environment variable `AISOPOD_CONFIG`
3. Platform-specific default locations:
   - **Linux/macOS**: `~/.config/aisopod/config.json5`
   - **Docker**: `/etc/aisopod/config.json`
   - **Current directory**: `aisopod-config.json5`

### Supported File Extensions

| Extension | Format | Description |
|-----------|--------|-------------|
| `.json5`  | JSON5  | JSON with comments, trailing commas, and unquoted keys (recommended) |
| `.json`   | JSON   | Standard JSON format |
| `.toml`   | TOML   | TOML format |

### Configuration Override Methods

Configuration can be overridden in the following order (highest priority first):

1. **CLI arguments**: `aisopod gateway --config /path/to/config.json5`
2. **Environment variables**: `AISOPOD_CONFIG=/path/to/config.json5`
3. **Default config file**: Located in platform-specific directories

## Environment Variables

All environment variables use the `AISOPOD_` prefix and override corresponding config file values.

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `AISOPOD_CONFIG` | Path to configuration file | Platform-specific default | `/etc/aisopod/config.json5` |
| `AISOPOD_LOG` | Log level: `trace`, `debug`, `info`, `warn`, `error` | `info` | `debug` |
| `AISOPOD_BUILD_UI` | Build UI during compilation (any value enables) | Not set | `1` |
| `AISOPOD_MODELS_DEFAULT_PROVIDER` | Default model provider name | Empty string | `openai` |
| `AISOPOD_MODELS_PROVIDERS_0_API_KEY` | API key for first provider | Empty string | `sk-...` |
| `AISOPOD_MODELS_PROVIDERS_0_ENDPOINT` | API endpoint for first provider | Provider default | `https://api.openai.com/v1` |
| `AISOPOD_MODELS_PROVIDERS_0_NAME` | Name for first provider | Provider name | `openai` |
| `AISOPOD_GATEWAY_SERVER_PORT` | Gateway server port | `8080` | `3000` |
| `AISOPOD_GATEWAY_BIND_ADDRESS` | Gateway bind address | `127.0.0.1` | `0.0.0.0` |
| `AISOPOD_TOOLS_BASH_ENABLED` | Enable bash tool | `false` | `true` |
| `AISOPOD_SESSION_ENABLED` | Enable session persistence | Not directly set | Via config |
| `AISOPOD_MEMORY_ENABLED` | Enable memory backend | Not directly set | Via config |
| `AISOPOD_TEST_URL` | Test server URL for conformance tests | `ws://127.0.0.1:8080/ws` | `ws://localhost:8080/ws` |
| `AISOPOD_TEST_TOKEN` | Test authentication token | `test-token` | `my-token` |

### Environment Variable Substitution

In configuration files, you can reference environment variables using the `${VAR}` or `${VAR:-default}` syntax:

```toml
[models.providers]
api_key = "${AISOPOD_OPENAI_API_KEY}"
```

If the variable is not set and no default is provided, configuration loading will fail.

## Config Sections Explained

The root configuration object contains the following sections:

### `[meta]` — Metadata

Configuration metadata.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `version` | string | `"1.0"` | Configuration schema version |

### `[auth]` — Authentication

Authentication configuration for the gateway.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `api_keys` | array of strings | `[]` | API keys for external services |
| `profiles` | array of objects | `[]` | Authentication profiles |
| `gateway_mode` | string | `"none"` | Gateway auth mode: `token`, `password`, or `none` |
| `tokens` | array of objects | `[]` | Token-based credentials |
| `passwords` | array of objects | `[]` | Password-based credentials |

**Auth Profile Fields:**
- `name`: Profile name
- `api_key`: Reference to API key (uses `${VAR}` syntax)
- `provider`: Provider type
- `endpoint`: Optional provider endpoint

### `[env]` — Environment

Environment-specific settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `debug` | boolean | `false` | Enable debug mode |

### `[gateway]` — Gateway Server

Gateway server configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `server` | object | (see below) | HTTP server settings |
| `bind` | object | (see below) | Bind address configuration |
| `tls` | object | (see below) | TLS configuration |
| `web_ui` | object | (see below) | Web UI configuration |
| `handshake_timeout` | integer | `5` | WebSocket handshake timeout (seconds) |
| `rate_limit` | object | (see below) | Rate limiting configuration |
| `request_size_limits` | object | (see below) | Request size limits |
| `pairing_cleanup_interval` | integer | `300` | Pairing cleanup interval (seconds) |

#### `[gateway.server]` — HTTP Server

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | `""` | Server name |
| `port` | integer | `8080` | Port to bind to |
| `graceful_shutdown` | boolean | `false` | Enable graceful shutdown |

#### `[gateway.bind]` — Bind Address

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `address` | string | `"127.0.0.1"` | IP address to bind to |
| `ipv6` | boolean | `false` | Enable IPv6 |

#### `[gateway.tls]` — TLS Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable TLS |
| `cert_path` | string | `""` | Certificate file path |
| `key_path` | string | `""` | Private key file path |

#### `[gateway.web_ui]` — Web UI

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable static file serving |
| `dist_path` | string | `"../web-ui/dist"` | Web UI assets directory |
| `cors_origins` | array of strings | `["http://localhost:8080", "http://localhost:5173"]` | Allowed CORS origins |

#### `[gateway.rate_limit]` — Rate Limiting

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_requests` | integer | `100` | Maximum requests in window |
| `window` | integer | `60` | Sliding window duration (seconds) |

#### `[gateway.request_size_limits]` — Request Size Limits

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_body_size` | integer | `10485760` (10MB) | Maximum request body size |
| `max_headers_size` | integer | `8192` (8KB) | Maximum headers size |
| `max_headers_count` | integer | `100` | Maximum header count |

### `[agents]` — Agent Definitions

Agent configuration with default settings and list of agents.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `agents` | array of objects | `[]` | List of agents |
| `default` | object | (see below) | Default agent configuration |

#### Agent Object

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | string | Required | Agent ID (unique identifier) |
| `name` | string | `""` | Human-readable name |
| `model` | string | Required | Model to use (must match a configured model) |
| `workspace` | string | `""` | Workspace path for tools |
| `sandbox` | boolean | `false` | Enable sandboxed execution |
| `subagents` | array of strings | `[]` | Subagent IDs this agent can spawn |
| `system_prompt` | string | `""` | System prompt for this agent |
| `max_subagent_depth` | integer | `3` | Maximum subagent spawning depth |
| `subagent_allowed_models` | array of strings | `null` | Allowlist of subagent models |
| `skills` | array of strings | `[]` | Skill IDs to assign |

#### `[agents.default]` — Default Agent Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `model` | string | `""` | Default model |
| `workspace` | string | `""` | Default workspace |
| `sandbox` | boolean | `false` | Default sandbox setting |

### `[models]` — Model Provider Configuration

Model definitions and provider settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `models` | array of objects | `[]` | Model definitions |
| `providers` | array of objects | `[]` | Provider configurations |
| `fallbacks` | array of objects | `[]` | Model fallback configurations |
| `default_provider` | string | `""` | Default provider name |

#### Model Object

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | string | Required | Model ID (unique identifier) |
| `name` | string | `""` | Human-readable name |
| `provider` | string | `""` | Provider name |
| `capabilities` | array of strings | `[]` | Model capabilities (e.g., `["chat"]`) |

#### Model Provider Object

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | Required | Provider name |
| `endpoint` | string | `""` | API endpoint URL |
| `api_key` | string | `""` | API key reference (uses `${VAR}` syntax) |

### `[channels]` — Channel Integrations

Channel configuration for messaging platform integrations.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `channels` | array of objects | `[]` | Channel definitions |
| `default` | object | (see below) | Default channel settings |
| `telegram` | object | (see below) | Telegram-specific config |
| `discord` | object | (see below) | Discord-specific config |
| `whatsapp` | object | (see below) | WhatsApp-specific config |
| `slack` | object | (see below) | Slack-specific config |
| `github` | object | (see below) | GitHub-specific config |
| `gitlab` | object | (see below) | GitLab-specific config |
| `bitbucket` | object | (see below) | Bitbucket-specific config |
| `mattermost` | object | (see below) | Mattermost-specific config |
| `matrix` | object | (see below) | Matrix-specific config |
| `msteams` | object | (see below) | Microsoft Teams-specific config |

#### Channel Object

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | string | Required | Channel ID (unique identifier) |
| `name` | string | `""` | Human-readable name |
| `channel_type` | string | `""` | Channel type (e.g., `telegram`, `discord`) |
| `connection` | object | (see below) | Connection settings |

#### Channel Connection Object

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `endpoint` | string | `""` | Connection endpoint |
| `token` | string | `""` | Authentication token (uses `${VAR}` syntax) |

#### Platform-Specific Configurations

##### Telegram

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `token` | string | `null` | Telegram bot token |

##### Discord

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `token` | string | `null` | Discord bot token |
| `client_secret` | string | `null` | Discord client secret |

##### WhatsApp

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `access_token` | string | `null` | WhatsApp access token |
| `phone_number_id` | string | `null` | Phone number ID |

##### Slack

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `token` | string | `null` | Slack bot token (xoxb-) |
| `signing_secret` | string | `null` | Slack signing secret |
| `verification_token` | string | `null` | Slack verification token |
| `app_token` | string | `null` | Slack app-level token |

##### Matrix

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `access_token` | string | `null` | Matrix access token |
| `home_server` | string | `null` | Matrix home server URL |
| `user_id` | string | `null` | Matrix user ID |

##### Microsoft Teams

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `tenant_id` | string | `null` | Azure AD tenant ID |
| `client_id` | string | `null` | Azure AD client ID |
| `client_secret` | string | `null` | Azure AD client secret |
| `bot_app_id` | string | `null` | Bot framework app ID |
| `bot_app_password` | string | `null` | Bot framework app password |

### `[tools]` — Tool / Skill Configuration

Tool configuration for agent capabilities.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bash` | object | (see below) | Bash execution tool |
| `exec` | object | (see below) | Exec tool |
| `filesystem` | object | (see below) | File system tool |

#### `[tools.bash]` — Bash Tool

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable bash tool |
| `working_dir` | string | `""` | Working directory for execution |
| `timeout` | integer | `300` | Timeout in seconds |

#### `[tools.exec]` — Exec Tool

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable exec tool |
| `allowed_commands` | array of strings | `[]` | List of allowed commands |

#### `[tools.filesystem]` — File System Tool

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable filesystem tool |
| `root` | string | `""` | Root directory for file operations |
| `operations` | array of strings | `[]` | Allowed operations |

### `[skills]` — Skills Configuration

Skill module and settings configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `modules` | array of objects | `[]` | Skill modules |
| `settings` | object | (see below) | Skill settings |

#### Skill Module Object

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | string | Required | Module ID |
| `name` | string | `""` | Module name |
| `path` | string | `""` | Module path |
| `enabled` | boolean | `false` | Enable module |

#### `[skills.settings]` — Skill Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `timeout` | integer | `60` | Default timeout in seconds |
| `max_executions` | integer | `0` | Maximum execution count |

### `[plugins]` — Plugins Configuration

Plugin registry and settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `registry` | array of objects | `[]` | Plugin registry entries |
| `settings` | object | (see below) | Plugin settings |

#### Plugin Entry Object

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | string | Required | Plugin ID |
| `name` | string | `""` | Plugin name |
| `version` | string | `""` | Plugin version |
| `enabled` | boolean | `false` | Enable plugin |

#### `[plugins.settings]` — Plugin Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `auto_load` | boolean | `false` | Auto-load plugins |
| `plugin_dir` | string | `""` | Plugin directory path |
| `load_timeout` | integer | `30` | Load timeout in seconds |

### `[session]` — Session Configuration

Session and message handling configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `messages` | object | (see below) | Message handling settings |
| `compaction` | object | (see below) | Session compaction settings |

#### `[session.messages]` — Message Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_messages` | integer | `0` | Maximum messages in session |
| `retention` | string | `""` | Message retention policy |
| `format` | string | `""` | Message formatting |

#### `[session.compaction]` — Compaction Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable compaction |
| `min_messages` | integer | `0` | Minimum messages before compaction |
| `interval` | integer | `3600` | Compaction interval (seconds) |

### `[bindings]` — Agent Bindings

Agent-to-channel binding rules.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_id` | string | Yes | Agent ID to bind |
| `channel_type` | string | Yes | Channel type to bind to |

### `[memory]` — Memory Configuration

Memory backend configuration for conversation history.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `backend` | object | (see below) | Memory backend configuration |
| `settings` | object | (see below) | Memory settings |

#### `[memory.backend]` — Backend Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `type` | string | `""` | Backend type (e.g., `sqlite`, `qdrant`) |
| `connection` | string | `""` | Connection string |
| `database` | string | `""` | Database name |

#### `[memory.settings]` — Memory Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `memory_limit` | integer | `0` | Memory limit in MB |
| `eviction_policy` | string | `""` | Eviction policy (e.g., `lru`) |
| `ttl` | integer | `86400` | Time-to-live in seconds (default: 1 day) |

## Example Configurations

### Minimal Setup (Single Agent, No Channels)

This is a minimal configuration for testing with a single agent and no external channels.

```toml
[meta]
version = "1.0"

[auth]
gateway_mode = "none"

[[agents.agents]]
id = "default"
name = "Default Agent"
model = "gpt-4"

[models.models]
id = "gpt-4"
name = "GPT-4"
provider = "openai"
capabilities = ["chat"]

[models.providers]
name = "openai"
endpoint = "https://api.openai.com/v1"
api_key = "${AISOPOD_OPENAI_API_KEY}"
```

Save this to `~/.config/aisopod/config.toml` and set:

```bash
export AISOPOD_OPENAI_API_KEY="sk-..."
aisopod gateway
```

### Development Setup (Single Agent, Telegram Channel)

This configuration sets up a development environment with Telegram integration.

```toml
[meta]
version = "1.0"

[auth]
gateway_mode = "none"

[env]
debug = true

[agents.agents]
id = "dev-agent"
name = "Development Agent"
model = "gpt-4"
system_prompt = "You are a helpful development assistant."

[agents.default]
model = "gpt-4"
workspace = "/tmp/aisopod-workspace"
sandbox = false

[[models.models]]
id = "gpt-4"
name = "GPT-4"
provider = "openai"
capabilities = ["chat"]

[models.providers]
name = "openai"
endpoint = "https://api.openai.com/v1"
api_key = "${AISOPOD_OPENAI_API_KEY}"

[models.default_provider] = "openai"

[[channels.channels]]
id = "telegram-dev"
name = "Telegram Development"
channel_type = "telegram"

[[bindings]]
agent_id = "dev-agent"
channel_type = "telegram"
```

### Production Setup (Multi-Agent, Multiple Channels)

This configuration shows a production-ready setup with multiple agents, multiple channels, and proper authentication.

```toml
[meta]
version = "1.0"

[auth]
gateway_mode = "token"

[[auth.tokens]]
token = "${AISOPOD_API_TOKEN}"
role = "admin"
scopes = ["*"]

[env]
debug = false

[server]
name = "aisopod-production"
port = 3080

[gateway.server]
graceful_shutdown = true

[gateway.bind]
address = "0.0.0.0"
ipv6 = false

[gateway.rate_limit]
max_requests = 1000
window = 60

[gateway.request_size_limits]
max_body_size = 10485760
max_headers_size = 8192
max_headers_count = 100

[[agents.agents]]
id = "general"
name = "General Assistant"
model = "gpt-4o"
system_prompt = "You are a helpful and knowledgeable assistant."
skills = ["web_search"]

[[agents.agents]]
id = "coder"
name = "Coding Assistant"
model = "claude-3-opus"
system_prompt = "You are an expert coding assistant. Write clean, efficient, and well-documented code."
skills = ["code_exec", "web_search"]

[[agents.agents]]
id = "analyst"
name = "Data Analyst"
model = "gpt-4-turbo"
system_prompt = "You are a data analysis expert. Help users analyze data and generate insights."
skills = ["data_analysis"]

[agents.default]
model = "gpt-4o"
workspace = "/var/lib/aisopod/workspace"
sandbox = true

[[models.models]]
id = "gpt-4o"
name = "GPT-4o"
provider = "openai"
capabilities = ["chat", "vision"]

[[models.models]]
id = "claude-3-opus"
name = "Claude 3 Opus"
provider = "anthropic"
capabilities = ["chat"]

[[models.models]]
id = "gpt-4-turbo"
name = "GPT-4 Turbo"
provider = "openai"
capabilities = ["chat"]

[models.providers]
name = "openai"
endpoint = "https://api.openai.com/v1"
api_key = "${AISOPOD_OPENAI_API_KEY}"

[models.providers]
name = "anthropic"
endpoint = "https://api.anthropic.com/v1"
api_key = "${AISOPOD_ANTHROPIC_API_KEY}"

[models.default_provider] = "openai"

[[channels.channels]]
id = "telegram-main"
name = "Telegram Main Channel"
channel_type = "telegram"

[[channels.channels]]
id = "discord-main"
name = "Discord Main Server"
channel_type = "discord"

[[channels.channels]]
id = "slack-team"
name = "Slack Team Workspace"
channel_type = "slack"

[[bindings]]
agent_id = "general"
channel_type = "telegram"

[[bindings]]
agent_id = "general"
channel_type = "discord"

[[bindings]]
agent_id = "coder"
channel_type = "slack"

[channels.telegram.token] = "${AISOPOD_TELEGRAM_TOKEN}"
[channels.discord.token] = "${AISOPOD_DISCORD_TOKEN}"
[channels.slack.token] = "${AISOPOD_SLACK_TOKEN}"

[tools.bash]
enabled = true
working_dir = "/tmp/aisopod"
timeout = 300

[tools.exec]
enabled = false
allowed_commands = []

[tools.filesystem]
enabled = false
root = "/var/lib/aisopod/files"
operations = ["read", "write", "delete"]

[sessions.messages]
max_messages = 1000
retention = "7d"

[sessions.compaction]
enabled = true
min_messages = 50
interval = 3600

[memory]
enabled = true
provider = "sqlite"
embedding_provider = "openai"
storage_path = "./data/memory"

[memory.backend]
type = "sqlite"
connection = "./data/memory.db"
database = "aisopod"

[memory.settings]
memory_limit = 512
eviction_policy = "lru"
ttl = 86400
```

### Docker Setup

For Docker deployments, use a volume mount for the configuration directory:

```bash
docker run -d \
  --name aisopod \
  -v $(pwd)/config:/config \
  -e AISOPOD_OPENAI_API_KEY="sk-..." \
  -p 3080:3080 \
  ghcr.io/aisopod/aisopod:latest
```

And create `config/config.json`:

```json5
{
  // Aisopod Docker configuration
  meta: {
    version: "1.0",
  },
  auth: {
    gateway_mode: "none",
  },
  gateway: {
    server: {
      port: 3080,
    },
    bind: {
      address: "0.0.0.0",
    },
  },
  agents: {
    agents: [{
      id: "default",
      name: "Default Agent",
      model: "gpt-4",
    }],
    default: {
      model: "gpt-4",
    },
  },
  models: {
    providers: [{
      name: "openai",
      api_key: "${AISOPOD_OPENAI_API_KEY}",
    }],
    default_provider: "openai",
  },
}
```

## Configuration File Structure (JSON5)

Here's a complete example in JSON5 format:

```json5
{
  // Configuration schema version
  meta: {
    version: "1.0",
  },

  // Authentication settings
  auth: {
    gateway_mode: "none",  // or "token", "password"
  },

  // Environment settings
  env: {
    debug: false,
  },

  // Gateway server configuration
  gateway: {
    server: {
      name: "aisopod-gateway",
      port: 8080,
    },
    bind: {
      address: "127.0.0.1",
      ipv6: false,
    },
    tls: {
      enabled: false,
      cert_path: "",
      key_path: "",
    },
    web_ui: {
      enabled: true,
      dist_path: "../web-ui/dist",
      cors_origins: [
        "http://localhost:8080",
        "http://localhost:5173",
      ],
    },
    handshake_timeout: 5,
    rate_limit: {
      max_requests: 100,
      window: 60,
    },
    request_size_limits: {
      max_body_size: 10485760,  // 10MB
      max_headers_size: 8192,   // 8KB
      max_headers_count: 100,
    },
    pairing_cleanup_interval: 300,
  },

  // Agent definitions
  agents: {
    agents: [{
      id: "default",
      name: "Default Agent",
      model: "gpt-4",
      workspace: "/workspace",
      sandbox: false,
      subagents: [],
      system_prompt: "You are a helpful assistant.",
      max_subagent_depth: 3,
      skills: [],
    }],
    default: {
      model: "gpt-4",
      workspace: "/workspace",
      sandbox: false,
    },
  },

  // Model definitions
  models: {
    models: [{
      id: "gpt-4",
      name: "GPT-4",
      provider: "openai",
      capabilities: ["chat"],
    }],
    providers: [{
      name: "openai",
      endpoint: "https://api.openai.com/v1",
      api_key: "${AISOPOD_OPENAI_API_KEY}",
    }],
    fallbacks: [],
    default_provider: "openai",
  },

  // Channel definitions
  channels: {
    channels: [],
    default: {
      channel_type: "websocket",
    },
    telegram: {
      token: null,
    },
    discord: {
      token: null,
    },
  },

  // Tool configurations
  tools: {
    bash: {
      enabled: false,
      working_dir: "",
      timeout: 300,
    },
    exec: {
      enabled: false,
      allowed_commands: [],
    },
    filesystem: {
      enabled: false,
      root: "",
      operations: [],
    },
  },

  // Skill configurations
  skills: {
    modules: [],
    settings: {
      timeout: 60,
      max_executions: 0,
    },
  },

  // Plugin configurations
  plugins: {
    registry: [],
    settings: {
      auto_load: false,
      plugin_dir: "",
      load_timeout: 30,
    },
  },

  // Session configurations
  session: {
    messages: {
      max_messages: 1000,
      retention: "",
      format: "",
    },
    compaction: {
      enabled: false,
      min_messages: 0,
      interval: 3600,
    },
  },

  // Agent bindings
  bindings: [],

  // Memory configuration
  memory: {
    backend: {
      type: "",
      connection: "",
      database: "",
    },
    settings: {
      memory_limit: 0,
      eviction_policy: "",
      ttl: 86400,
    },
  },
}
```

## Common Configuration Patterns

### Using Environment Variables for Secrets

```toml
[models.providers]
api_key = "${AISOPOD_API_KEY}"

[channels.telegram]
token = "${AISOPOD_TELEGRAM_TOKEN}"

[auth.tokens]
token = "${AISOPOD_ADMIN_TOKEN}"
```

### Enabling Sandbox Execution

```toml
[agents.default]
sandbox = true

[agents.agents]
sandbox = true

[tools.bash]
enabled = true
working_dir = "/tmp/aisopod"
timeout = 300
```

### Configuring Rate Limiting

```toml
[gateway.rate_limit]
max_requests = 100
window = 60  # seconds

[gateway.request_size_limits]
max_body_size = 10485760  # 10MB
max_headers_size = 8192   # 8KB
max_headers_count = 100
```

### Setting Up Session Compaction

```toml
[session.compaction]
enabled = true
min_messages = 50
interval = 3600  # every hour
```

### Configuring TLS

```toml
[gateway.tls]
enabled = true
cert_path = "/etc/ssl/certs/aisopod.crt"
key_path = "/etc/ssl/private/aisopod.key"
```

## Configuration Validation

Aisopod validates configuration on startup. Common validation errors include:

1. **Missing required fields**: All required fields must be present
2. **Invalid values**: Values must be within allowed ranges
3. **Circular references**: Subagent references cannot be circular
4. **Duplicate IDs**: All IDs must be unique
5. **Unresolved environment variables**: Required env vars must be set

Run `aisopod doctor` to check your configuration.

## See Also

- [CLI Command Reference](./cli-reference.md) - CLI options including `--config`
- [Agents, Channels & Skills](./agents-channels-skills.md) - Core concepts
- [Troubleshooting](./troubleshooting.md) - Common configuration issues
