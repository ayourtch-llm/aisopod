# Agents, Channels & Skills

This guide covers Aisopod's three core domain concepts: agents, channels, and skills. These components work together to enable multi-agent, multi-channel conversational AI systems.

## Agents

### What is an Agent?

An agent is an LLM-backed conversational entity that serves as the brain of your Aisopod system. Each agent has:

- **ID** (unique identifier) - Used internally to reference the agent
- **Name** - Human-readable name for display purposes
- **Model** - Which LLM provider and model to use (e.g., `gpt-4o`, `claude-3-5-sonnet`)
- **System prompt** - Personality, instructions, and behavior definition
- **Workspace** - File system path for agent operations
- **Skills** - Tools and capabilities the agent can invoke
- **Subagents** - List of other agents this agent can spawn
- **Max subagent depth** - How deep the subagent hierarchy can go (default: 3)

### Agent Lifecycle

1. **Configuration** — Agent is defined in `config.toml` or added via CLI
2. **Loading** — Gateway loads agent configuration at startup
3. **Activation** — Agent becomes ready to receive messages
4. **Message handling** — Receives messages from channels, processes with LLM, generates responses
5. **Deactivation** — Agent stops when gateway shuts down

### Creating an Agent

Agents can be created in two ways:

#### Via Config File

Add to your `config.toml`:

```toml
[[agents]]
id = "research-assistant"
name = "Research Assistant"
model = "gpt-4o"
workspace = "/workspace/research"
sandbox = false
system_prompt = """
You are a research assistant. Help users find and summarize
information from the web. Always cite your sources accurately.
When searching for information, use the web_search skill.
"""
skills = ["web_search", "summarize"]
```

#### Via CLI

```bash
aisopod agent add my-agent
# Interactive prompts for name, model, system prompt, and other settings
```

### Agent ID vs Name

- **ID**: Machine-readable identifier, used for routing and configuration. Should use kebab-case (e.g., `research-assistant`).
- **Name**: Human-readable display name. Can contain spaces and special characters (e.g., `Research Assistant`).

### Model Selection

Each agent specifies which model to use via the `model` field:

```toml
[[agents]]
id = "primary"
model = "gpt-4o"
```

Common model identifiers:
- OpenAI: `gpt-4o`, `gpt-4-turbo`, `gpt-3.5-turbo`
- Anthropic: `claude-3-5-sonnet`, `claude-3-opus`, `claude-3-haiku`
- Google: `gemini-1.5-pro`, `gemini-1.5-flash`
- Ollama: `llama3`, `mistral`, `codellama`

### System Prompt Customization

System prompts support several features:

#### Multi-line Strings

Use TOML's `"""` syntax for multi-line prompts:

```toml
system_prompt = """
You are a helpful assistant.
You specialize in coding tasks.
Always explain your reasoning.
"""
```

#### Variable Interpolation

The following variables are available in system prompts:

- `{user_name}` - The name of the user
- `{channel_name}` - The name of the channel

#### File References

For complex prompts, store them in separate files:

```toml
system_prompt_file = "prompts/assistant.md"
```

The file is read relative to the config file location.

### Agent Workspaces

Each agent has a workspace directory for file operations:

```toml
[[agents]]
id = "data-analyst"
workspace = "/workspace/analyst"
```

The workspace is where:
- Temporary files are created
- File upload processing occurs
- Agent-specific data is stored

### Sandbox Mode

Enable sandboxing for untrusted tool execution:

```toml
[[agents]]
id = "sandboxed-agent"
sandbox = true
```

When enabled, the agent's tool execution runs in an isolated container.

## Channels

Channels connect Aisopod to messaging platforms. Each channel receives messages from users and sends responses back.

### Supported Channels

Aisopod supports multiple messaging platforms:

| Channel | Type | Description |
|---------|------|-------------|
| Telegram | `telegram` | Instant messaging service |
| Discord | `discord` | Gaming and community platform |
| WhatsApp | `whatsapp` | Business messaging API |
| Slack | `slack` | Team collaboration |
| Signal | `signal` | Encrypted messaging |
| IRC | `irc` | Internet Relay Chat |
| Matrix | `matrix` | Decentralized communication |
| LINE | `line` | Japanese messaging app |
| WeChat | `wechat` | Chinese messaging platform |
| Zalo | `zalo` | Vietnamese messaging app |

### Channel Configuration

Each channel requires:
- **ID** - Unique identifier for the channel
- **Name** - Human-readable name
- **Channel Type** - The platform type (e.g., `telegram`)
- **Connection** - Authentication and endpoint settings

#### Basic Channel Template

```toml
[[channels]]
id = "my-telegram"
name = "My Telegram Bot"
channel_type = "telegram"
[channels.connection]
endpoint = "https://api.telegram.org"
token = "${AISOPOD_TELEGRAM_TOKEN}"
```

### Tier 1 Channel Setup Guides

#### Telegram Setup

1. **Create a Bot**
   - Open Telegram and search for [@BotFather](https://t.me/BotFather)
   - Send `/newbot` command
   - Follow the prompts to create your bot
   - Copy the bot token (looks like `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

2. **Configure Aisopod**
   Add to your `config.toml`:

   ```toml
   [[channels]]
   id = "telegram-primary"
   name = "Primary Telegram Bot"
   channel_type = "telegram"
   [channels.connection]
   endpoint = "https://api.telegram.org"
   token = "${AISOPOD_TELEGRAM_TOKEN}"
   ```

3. **Set Environment Variable**

   ```bash
   export AISOPOD_TELEGRAM_TOKEN="123456789:ABCdef..."
   ```

4. **Start the Gateway**

   ```bash
   aisopod gateway start
   ```

5. **Test Your Bot**
   - Message your bot on Telegram
   - It should respond if connected properly

#### Discord Setup

1. **Create an Application**
   - Visit [Discord Developer Portal](https://discord.com/developers/applications)
   - Click "New Application" and give it a name
   - Go to the "Bot" section
   - Click "Add Bot"

2. **Configure Bot Settings**
   - Under "Privileged Gateway Intents", enable:
     - Message Content Intent
     - Server Members Intent (if needed)
   - Click "Reset Token" and copy the bot token

3. **Invite Bot to Server**
   - Go to "OAuth2" → "URL Generator"
   - Select scopes: `bot`
   - Select bot permissions: `Send Messages`, `Read Message History`
   - Copy the generated URL and open it in your browser
   - Select your server and authorize

4. **Configure Aisopod**

   ```toml
   [[channels]]
   id = "discord-primary"
   name = "Primary Discord Bot"
   channel_type = "discord"
   [channels.connection]
   endpoint = "https://discord.com/api"
   token = "${AISOPOD_DISCORD_TOKEN}"
   ```

5. **Set Environment Variable**

   ```bash
   export AISOPOD_DISCORD_TOKEN="your-bot-token-here"
   ```

6. **Start the Gateway**

   ```bash
   aisopod gateway start
   ```

#### WhatsApp Setup

1. **Set Up WhatsApp Business API**
   - Sign up for a WhatsApp Business Account at [Meta](https://www.meta.com/business/platform/whatsapp-business-api/)
   - Create a WhatsApp Business API account
   - Create a product in Meta for WhatsApp

2. **Configure Webhook**
   - Get your webhook URL from Aisopod gateway (default: `http://localhost:3000/webhooks/whatsapp`)
   - Configure the webhook in Meta's dashboard
   - Set the verify token (you'll need this for Aisopod config)

3. **Configure Aisopod**

   ```toml
   [[channels]]
   id = "whatsapp-primary"
   name = "Primary WhatsApp"
   channel_type = "whatsapp"
   [channels.connection]
   endpoint = "https://graph.facebook.com"
   phone_number_id = "${AISOPOD_WHATSAPP_PHONE_ID}"
   access_token = "${AISOPOD_WHATSAPP_ACCESS_TOKEN}"
   ```

4. **Set Environment Variables**

   ```bash
   export AISOPOD_WHATSAPP_PHONE_ID="your-phone-number-id"
   export AISOPOD_WHATSAPP_ACCESS_TOKEN="your-access-token"
   ```

5. **Verify Webhook Connection**
   - The gateway will automatically verify the webhook when it starts
   - Test by sending a message to your WhatsApp business number

#### Slack Setup

1. **Create a Slack App**
   - Visit [api.slack.com/apps](https://api.slack.com/apps)
   - Click "Create New App"
   - Choose "From scratch" and give it a name

2. **Configure Socket Mode (Recommended)**
   - Go to "Socket Mode" and enable it
   - Click "Generate Token and Install to Workspace"
   - Copy the App-Level Token

3. **Add Bot Token Scopes**
   - Go to "OAuth & Permissions"
   - Add these bot token scopes:
     - `chat:write` - Send messages
     - `app_mentions:read` - Read mentions
     - `im:history` - Read direct messages
     - `channels:history` - Read channel messages (if needed)
   - Click "Reinstall to Workspace"

4. **Configure Aisopod**

   ```toml
   [[channels]]
   id = "slack-primary"
   name = "Primary Slack"
   channel_type = "slack"
   [channels.connection]
   endpoint = "https://slack.com/api"
   bot_token = "${AISOPOD_SLACK_BOT_TOKEN}"
   app_token = "${AISOPOD_SLACK_APP_TOKEN}"
   ```

5. **Set Environment Variables**

   ```bash
   export AISOPOD_SLACK_BOT_TOKEN="xoxb-..."
   export AISOPOD_SLACK_APP_TOKEN="xapp-..."
   ```

6. **Start the Gateway**

   ```bash
   aisopod gateway start
   ```

### Channel Connection Settings

The `connection` section varies by channel type:

#### Telegram

```toml
[channels.connection]
endpoint = "https://api.telegram.org"
token = "${AISOPOD_TELEGRAM_TOKEN}"
```

#### Discord

```toml
[channels.connection]
endpoint = "https://discord.com/api"
token = "${AISOPOD_DISCORD_TOKEN}"
```

#### WhatsApp

```toml
[channels.connection]
endpoint = "https://graph.facebook.com"
phone_number_id = "${AISOPOD_WHATSAPP_PHONE_ID}"
access_token = "${AISOPOD_WHATSAPP_ACCESS_TOKEN}"
```

#### Slack

```toml
[channels.connection]
endpoint = "https://slack.com/api"
bot_token = "${AISOPOD_SLACK_BOT_TOKEN}"
app_token = "${AISOPOD_SLACK_APP_TOKEN}"
```

### Multiple Channels

You can configure multiple channels, each with its own settings:

```toml
[[channels]]
id = "telegram-primary"
name = "Primary Telegram"
channel_type = "telegram"
[channels.connection]
endpoint = "https://api.telegram.org"
token = "${AISOPOD_TELEGRAM_TOKEN}"

[[channels]]
id = "discord-primary"
name = "Primary Discord"
channel_type = "discord"
[channels.connection]
endpoint = "https://discord.com/api"
token = "${AISOPOD_DISCORD_TOKEN}"
```

### Channel-Specific Configuration

Each channel type has additional configuration options:

#### Telegram

```toml
[channels.telegram]
token = "${AISOPOD_TELEGRAM_TOKEN}"
```

#### Discord

```toml
[channels.discord]
token = "${AISOPOD_DISCORD_TOKEN}"
client_secret = "${AISOPOD_DISCORD_CLIENT_SECRET}"
```

#### WhatsApp

```toml
[channels.whatsapp]
access_token = "${AISOPOD_WHATSAPP_ACCESS_TOKEN}"
phone_number_id = "${AISOPOD_WHATSAPP_PHONE_ID}"
```

#### Slack

```toml
[channels.slack]
token = "${AISOPOD_SLACK_BOT_TOKEN}"
app_token = "${AISOPOD_SLACK_APP_TOKEN}"
signing_secret = "${AISOPOD_SLACK_SIGNING_SECRET}"
```

## Agent Binding (Routing Channels to Agents)

Agent binding defines which channels send messages to which agents. This enables:
- Routing different channels to different specialized agents
- Multiple channels sharing the same agent
- Priority-based routing

### Basic Binding

In your configuration, bind channels to agents:

```toml
[[bindings]]
agent_id = "research-assistant"
channels = ["telegram-primary", "discord-primary"]
priority = 100
```

This routes messages from both Telegram and Discord to the `research-assistant` agent.

### Multiple Agents

You can define different agents for different purposes:

```toml
[[agents]]
id = "general"
name = "General Assistant"
model = "gpt-4o"
skills = ["summarize", "file_read"]

[[agents]]
id = "coder"
name = "Code Assistant"
model = "claude-3-5-sonnet"
skills = ["code_exec"]

[[bindings]]
agent_id = "general"
channels = ["telegram-primary"]
priority = 100

[[bindings]]
agent_id = "coder"
channels = ["discord-primary"]
priority = 100
```

### Binding Priority

When multiple bindings match a channel, the one with higher priority wins:

```toml
[[bindings]]
agent_id = "premium"
channels = ["telegram-primary"]
priority = 200

[[bindings]]
agent_id = "basic"
channels = ["telegram-primary"]
priority = 100
```

The `premium` binding will be used for `telegram-primary`.

### Default Agent

You can specify a default agent that handles unbound channels:

```toml
[[bindings]]
agent_id = "default-agent"
channels = []
priority = 0
```

## Skills

Skills are reusable bundles of tools and capabilities that can be assigned to agents. Each skill provides:

- **Tools** - Functions the agent can invoke (e.g., `web_search`, `code_exec`)
- **System prompt fragments** - Added to the agent's system prompt
- **Runtime initialization** - Setup code that runs when the agent loads

### Built-in Skills

Aisopod comes with several built-in skills:

| Skill | Category | Description | Default |
|-------|----------|-------------|---------|
| `healthcheck` | System | Health check endpoint and diagnostics | On |
| `session_logs` | System | Logs all agent sessions to database | On |
| `model_usage` | System | Tracks token usage and costs | On |
| `web_search` | Integration | Search the web via configured provider | Off |
| `code_exec` | Utility | Execute code in sandboxed environment | Off |
| `summarize` | Utility | Summarize long texts or URLs | On |
| `memory` | Productivity | Persistent memory across sessions | Off |
| `image_gen` | Utility | Generate images via DALL-E / SD | Off |
| `file_read` | Utility | Read uploaded files | On |

### Skill Categories

Skills are categorized as:

- **Messaging**: Email, chat, social media integration
- **Productivity**: Calendar, file management, note-taking
- **AiMl**: Model orchestration, data preprocessing, training
- **Integration**: HTTP clients, webhook handlers, service connectors
- **Utility**: Logging, formatting, validation helpers
- **System**: Configuration, security, diagnostics

### Enabling Skills

Skills are enabled by adding them to an agent's `skills` list:

```toml
[[agents]]
id = "assistant"
name = "Assistant"
model = "gpt-4o"
skills = ["web_search", "summarize", "file_read"]
```

### Disabling Skills

Skills are disabled by omitting them from the `skills` list:

```toml
[[agents]]
id = "restricted"
name = "Restricted Bot"
model = "gpt-4o"
# Only skills not listed here are available
skills = ["summarize"]  # web_search, code_exec are disabled
```

### Skill Configuration

Some skills require configuration:

```toml
[[skills.modules]]
id = "web_search"
path = "builtin/web_search"
enabled = true

[[skills.modules]]
id = "code_exec"
path = "builtin/code_exec"
enabled = true

[skills.settings]
timeout = 60
max_executions = 100
```

### Creating Custom Skills

See the [Developer Guide](./developer-guide.md) for implementing custom skills via the `Skill` trait.

## Best Practices

### Agent Design

1. **Single Purpose**: Create agents with focused responsibilities
2. **Clear System Prompts**: Be explicit about behavior and boundaries
3. **Appropriate Models**: Use cheaper models for simple tasks, expensive ones for complex reasoning
4. **Sandbox When Needed**: Enable sandboxing for untrusted tool execution

### Channel Management

1. **Separate Environments**: Use different channels for dev/staging/production
2. **Environment Variables**: Store sensitive tokens in environment variables
3. **Rate Limiting**: Consider rate limits for each channel type
4. **Error Handling**: Implement fallback behavior for channel failures

### Skill Selection

1. **Minimal Skills**: Start with fewer skills, add as needed
2. **Security**: Disable code execution for untrusted agents
3. **Cost**: Be aware of tool costs (web search, code execution)
4. **Testing**: Test skills in isolation before combining

## Troubleshooting

### Agent Not Responding

1. Check gateway logs: `aisopod gateway logs`
2. Verify agent exists: `aisopod agent list`
3. Check model configuration: `aisopod models list`
4. Verify agent binding: Check `bindings` in config

### Channel Connection Issues

1. Verify token is set: `echo $AISOPOD_TELEGRAM_TOKEN`
2. Check network connectivity
3. Review channel-specific error messages in logs
4. Ensure proper scopes/permissions are configured

### Skill Not Available

1. Check skill is enabled in config
2. Verify skill module is loaded: `aisopod plugins list`
3. Check skill requirements (env vars, binaries)
4. Review skill logs for initialization errors
