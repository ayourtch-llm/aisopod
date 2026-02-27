# Getting Started

Welcome to Aisopod! This guide will help you install Aisopod, complete the onboarding wizard, and send your first message within 10 minutes.

## System Requirements

| Component       | Minimum           | Recommended        |
|-----------------|-------------------|--------------------|
| OS              | Linux, macOS, Windows (WSL2) | Linux or macOS |
| Rust toolchain  | 1.75+             | latest stable      |
| Memory          | 512 MB            | 2 GB               |
| Disk            | 100 MB            | 500 MB             |
| Network         | Required for LLM API calls | Broadband     |

You will also need an API key from at least one supported LLM provider
(OpenAI, Anthropic, etc.).

## Installation

Aisopod can be installed in several ways depending on your platform and preference.

### Pre-built Binary (Recommended)

Download the latest release for your platform:

```bash
curl -fsSL https://aisopod.dev/install.sh | sh
```

### Homebrew (macOS / Linux)

```bash
brew install aisopod/tap/aisopod
```

### Cargo Install (from source)

```bash
cargo install aisopod
```

### Docker

```bash
docker run -it --rm -v ~/.config/aisopod:/config ghcr.io/aisopod/aisopod:latest
```

### Verify Installation

After installation, verify that Aisopod is correctly installed:

```bash
aisopod --version
```

You should see output like:

```
aisopod 0.1.0
```

## First Run

Run the onboarding wizard to set up your first configuration:

```bash
aisopod onboarding
```

The wizard will guide you through:

1. **Choose your default LLM provider**: Select from supported providers like OpenAI, Anthropic, or Ollama
2. **Enter your API key**: Securely stored in your OS keychain
3. **Configure your default model**: Set the model to use for conversations (e.g., `gpt-4`)
4. **Set up a channel (optional)**: Connect a messaging platform like Telegram, Discord, or Slack

Your configuration is saved to `~/.config/aisopod/config.toml`.

### Manual Configuration

If you prefer to configure manually, create `~/.config/aisopod/config.toml` with:

```toml
[agents.default]
model = "gpt-4"

[providers.openai]
api_key = "your-api-key-here"

[gateway.bind]
address = "127.0.0.1"

[gateway.server]
port = 3000
```

## Quick Start

### Send a Message via CLI

Send a message directly from the terminal:

```bash
aisopod onboarding
```

Then start the gateway in one terminal:

```bash
aisopod gateway
```

And send a message in another terminal:

```bash
aisopod message "Hello, what can you do?"
```

### REST API via Gateway

Start the gateway server:

```bash
aisopod gateway
```

The gateway will start on `http://127.0.0.1:3000`.

#### Using cURL

```bash
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages": [{"role": "user", "content": "Hello!"}]}'
```

#### Using JavaScript

```javascript
const response = await fetch('http://localhost:3000/v1/chat/completions', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    messages: [{ role: 'user', content: 'Hello!' }]
  })
});
const data = await response.json();
console.log(data.choices[0].message.content);
```

## Architecture Overview

Aisopod is organized around three core concepts:

- **Gateway**: The HTTP/WebSocket server that receives messages and routes them to agents.
- **Agent**: An LLM-backed conversational entity with a system prompt, model selection, and bound skills.
- **Channel**: An integration with an external messaging platform (Telegram, Discord, Slack, etc.).

```
Channel → Gateway → Agent → LLM Provider
             ↕
          Skills / Tools
```

### How It Works

1. A message arrives via a **Channel** (e.g., Telegram, Discord)
2. The **Gateway** receives and authenticates the request
3. The message is routed to the appropriate **Agent**
4. The **Agent** processes the message using the configured LLM provider
5. The response flows back through the gateway to the channel

### Gateway

The gateway provides:
- HTTP REST API for programmatic access
- WebSocket support for real-time streaming
- Authentication and authorization
- Rate limiting and security middleware

### Agent

Each agent includes:
- A system prompt defining behavior
- Model selection (different models for different tasks)
- Skills and tools for enhanced capabilities
- Session management for conversation history

### Channel

Supported channels include:
- Telegram
- Discord
- Slack
- WhatsApp
- Matrix
- IRC
- And more...

## What's Next?

- [Configuration](./configuration.md): Learn about configuration options
- [Agents, Channels & Skills](./agents-channels-skills.md): Understand the core concepts
- [Developer Guide](./developer-guide.md): Dive into the architecture
- [CLI Reference](./cli-reference.md): Explore all available commands

## Troubleshooting

If you encounter issues:

1. Check the [Troubleshooting](./troubleshooting.md) guide
2. Run `aisopod doctor` for system diagnostics
3. Visit the [GitHub Discussions](https://github.com/aisopod/aisopod/discussions)
