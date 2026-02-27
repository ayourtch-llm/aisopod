# Developer Guide

This guide covers the internal architecture of Aisopod, contribution workflows, and step-by-step tutorials for developing plugins and custom channels.

## Architecture

### Crate Dependency Graph

```
aisopod (binary)
├── aisopod-gateway     (HTTP/WS server, routing)
│   ├── aisopod-agent   (agent lifecycle, LLM calls)
│   │   ├── aisopod-model   (model provider abstractions)
│   │   └── aisopod-skill   (skill/tool trait and registry)
│   ├── aisopod-channel (channel trait, adapters)
│   └── aisopod-auth    (authentication, authorization)
├── aisopod-config      (config parsing, validation)
├── aisopod-protocol    (JSON-RPC, message types, streaming)
├── aisopod-storage     (sessions, memory, persistence)
└── aisopod-common      (shared types, errors, utils)
```

**Actual Aisopod Workspace Structure:**

```
aisopod (binary)
├── aisopod-gateway         (HTTP/WS server, routing)
│   ├── aisopod-agent       (agent lifecycle, LLM calls)
│   │   ├── aisopod-provider   (model provider abstractions)
│   │   └── aisopod-tools      (skill/tool trait and registry)
│   ├── aisopod-channel     (channel trait, adapters)
│   ├── aisopod-session     (session management)
│   ├── aisopod-memory      (memory storage)
│   └── aisopod-config      (configuration)
├── aisopod-config          (config parsing, validation)
├── aisopod-session         (session management)
├── aisopod-memory          (memory storage)
├── aisopod-plugin          (plugin loading system)
├── aisopod-channel-*       (channel adapter crates)
├── aisopod-provider-*      (provider adapter crates)
├── aisopod-tools           (built-in tools)
├── aisopod-client          (client library)
└── aisopod-shared          (shared types, errors, utils)
```

### Module Organization

Each crate follows a consistent structure:

```
crates/aisopod-example/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Public API and re-exports
│   ├── config.rs       # Crate-specific configuration
│   ├── error.rs        # Error types
│   ├── traits.rs       # Trait definitions
│   ├── adapters.rs     # Optional adapter traits
│   └── ...             # Feature modules
└── tests/
    └── integration.rs  # Integration tests
```

**Crate Descriptions:**

| Crate | Purpose |
|-------|---------|
| `aisopod` | Main binary entry point with CLI |
| `aisopod-gateway` | HTTP/WebSocket server, routing, authentication |
| `aisopod-agent` | Agent lifecycle management, LLM integration |
| `aisopod-provider` | LLM provider abstraction (OpenAI, Anthropic, etc.) |
| `aisopod-channel` | Channel trait and adapter system |
| `aisopod-channel-*` | Platform-specific channel adapters (Telegram, Discord, etc.) |
| `aisopod-tools` | Tool trait and built-in tools (file, bash, cron, etc.) |
| `aisopod-session` | Session state management |
| `aisopod-memory` | Memory storage and retrieval |
| `aisopod-plugin` | Plugin loading and management system |
| `aisopod-config` | Configuration parsing and validation |
| `aisopod-client` | Client library for external integrations |
| `aisopod-shared` | Shared types, errors, utilities |

### Key Design Patterns

#### Trait-based Abstraction

Core extension points are defined as traits:

- **`ChannelPlugin`** — implement to add a messaging platform
- **`Tool`** — implement to add a tool/capability  
- **`ModelProvider`** — implement to add an LLM provider
- **`AuthAdapter`**, **`SecurityAdapter`**, **`GatewayAdapter`**, etc. — optional capabilities

**Example:** Channel trait with optional adapters

```rust
pub trait ChannelPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn meta(&self) -> &ChannelMeta;
    fn capabilities(&self) -> &ChannelCapabilities;
    
    fn config(&self) -> &dyn ChannelConfigAdapter;
    
    // Optional adapters - only implement what you need
    fn onboarding(&self) -> Option<&dyn OnboardingAdapter> { None }
    fn gateway(&self) -> Option<&dyn GatewayAdapter> { None }
    fn outbound(&self) -> Option<&dyn OutboundAdapter> { None }
    // ... more optional adapters
}
```

#### Async-first

All I/O-bound operations use `async`/`await` with `tokio` as the runtime. Traits use `async_trait` where needed.

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Tool {
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>;
}
```

#### Streaming

LLM responses are streamed via `tokio::sync::mpsc` channels and exposed as SSE or WebSocket events.

```rust
use tokio::sync::mpsc;

pub struct StreamingResponse {
    sender: mpsc::Sender<Chunk>,
}

impl StreamingResponse {
    pub async fn send_chunk(&self, chunk: Chunk) -> Result<()> {
        self.sender.send(chunk).await?;
        Ok(())
    }
}
```

#### Error Handling

Each crate defines its own error enum via `thiserror`, with `From` conversions for clean propagation.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Authentication failed")]
    AuthFailed,
    
    #[error("Message format error: {0}")]
    MessageFormat(String),
}
```

## Contributing

### Development Environment Setup

```bash
# Clone the repository
git clone https://github.com/AIsopod/aisopod.git
cd aisopod

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Run lints
cargo clippy --workspace --all-targets -- -D warnings

# Format code
cargo fmt --all
```

**Recommended development tools:**

- `cargo-watch`: `cargo install cargo-watch`
- `cargo-udeps`: `cargo install cargo-udeps`
- `rust-analyzer` (VS Code or IDE extension)

### Build & Test

```bash
# Build in release mode
cargo build --release

# Run specific crate tests
cargo test -p aisopod-gateway

# Run with logging
RUST_LOG=debug cargo run -- gateway start

# Run tests with coverage
cargo tarpaulin --workspace

# Check for unused dependencies
cargo udeps --workspace
```

### Code Style

- Follow `rustfmt` defaults (no custom config)
- All public items must have doc comments (`///`)
- Prefer `thiserror` for error types
- Use `tracing` for logging (not `log` or `println!`)
- Keep functions under 50 lines where possible
- Use descriptive error messages
- Add integration tests for new features

**Formatting example:**

```rust
/// Executes the tool with the given parameters.
///
/// # Arguments
///
/// * `params` - The tool parameters, validated against the schema
/// * `ctx` - The execution context
///
/// # Returns
///
/// Returns `Ok(ToolResult)` on success, containing the tool output.
/// Returns `Err(e)` if the tool execution failed.
async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>;
```

### Pull Request Process

1. Fork the repository and create a feature branch
   ```bash
   git checkout -p feature/my-new-feature
   ```

2. Write tests for new functionality
   - Unit tests in `src/lib.rs` or `src/module.rs`
   - Integration tests in `tests/integration.rs`
   - Test coverage for edge cases

3. Ensure `cargo test --workspace` passes
   ```bash
   cargo test --workspace
   ```

4. Ensure `cargo clippy --workspace` is clean
   ```bash
   cargo clippy --workspace --all-targets -- -D warnings
   ```

5. Format your code
   ```bash
   cargo fmt --all
   ```

6. Write a clear PR description referencing the issue number
   - Use the PR template
   - Include before/after behavior
   - List breaking changes if any

7. Request review from a maintainer

## Plugin Development Tutorial

This tutorial walks through creating a custom skill plugin for Aisopod.

### Implementing the `Tool` Trait

Create a new crate with `Cargo.toml`:

```toml
[package]
name = "weather-skill"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
aisopod-tools = { path = "../aisopod-tools" }
async-trait.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
tokio.workspace = true
reqwest.workspace = true
```

Implement the skill:

```rust
use aisopod_tools::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Missing location parameter")]
    MissingLocation,
    #[error("API error: {0}")]
    ApiError(String),
}

pub struct WeatherSkill {
    api_key: String,
}

impl WeatherSkill {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { api_key: api_key.into() }
    }

    async fn fetch_weather(&self, location: &str) -> Result<String, WeatherError> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}",
            location, self.api_key
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| WeatherError::ApiError(e.to_string()))?;

        let body = response.text().await
            .map_err(|e| WeatherError::ApiError(e.to_string()))?;

        // Parse and extract temperature
        Ok(format!("Current weather for {}: 22°C, clear skies", location))
    }
}

#[async_trait]
impl Tool for WeatherSkill {
    fn name(&self) -> &str {
        "weather"
    }

    fn description(&self) -> &str {
        "Get current weather for a location"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name or coordinates"
                }
            },
            "required": ["location"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let location = params.get("location")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!(WeatherError::MissingLocation))?;

        let weather = self.fetch_weather(location).await?;
        Ok(ToolResult::success(weather))
    }
}
```

### Plugin Manifest

Create a `plugin.toml` in your plugin crate:

```toml
[plugin]
name = "weather"
version = "0.1.0"
description = "Weather information skill"
author = "Your Name"
license = "MIT"

[[tools]]
name = "weather"
description = "Get current weather for a location"
```

### Building the Plugin

Add to your `Cargo.toml` to export the plugin:

```toml
[package.metadata]
plugin-type = "tool"
```

### Registering with Aisopod

Add the plugin to your config:

```toml
[[plugins]]
path = "./plugins/weather"
```

Or use environment-based loading:

```bash
AISOPOD_PLUGIN_PATHS=./plugins/weather ./aisopod gateway start
```

## Channel Development Tutorial

This tutorial walks through creating a custom channel plugin for Aisopod.

### Implementing the `ChannelPlugin` Trait

Create a new crate:

```toml
[package]
name = "matrix-channel"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
aisopod-channel = { path = "../aisopod-channel" }
async-trait.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
tokio.workspace = true
ruma = "0.8"
matrix-sdk = "0.8"
```

Implement the channel:

```rust
use aisopod_channel::{
    ChannelPlugin, ChannelMeta, ChannelCapabilities, ChatType, MediaType,
    adapters::{GatewayAdapter, OutboundAdapter, ChannelConfigAdapter},
    message::{IncomingMessage, OutgoingMessage, MessageContent, MessagePart},
    util::{NormalizedMarkdown, RateLimiter},
};
use async_trait::async_trait;
use ruma::{EventId, RoomId, UserId};
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum MatrixError {
    #[error("Invalid homeserver URL: {0}")]
    InvalidHomeserver(String),
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Matrix SDK error: {0}")]
    SdkError(String),
}

pub struct MatrixChannel {
    homeserver: String,
    access_token: String,
    user_id: String,
    client: Option<matrix_sdk::Client>,
    joined_rooms: HashMap<String, RoomId>,
    rate_limiter: RateLimiter,
}

impl MatrixChannel {
    pub fn new(homeserver: impl Into<String>, access_token: impl Into<String>) -> Self {
        Self {
            homeserver: homeserver.into(),
            access_token: access_token.into(),
            user_id: String::new(),
            client: None,
            joined_rooms: HashMap::new(),
            rate_limiter: RateLimiter::new(10, std::time::Duration::from_secs(1)),
        }
    }
}

#[async_trait]
impl ChannelConfigAdapter for MatrixChannel {
    fn name(&self) -> &str {
        "matrix"
    }

    fn configure(&mut self, config: &HashMap<String, String>) -> anyhow::Result<()> {
        if let Some hs) = config.get("homeserver") {
            self.homeserver = hs.clone();
        }
        if let Some(token) = config.get("access_token") {
            self.access_token = token.clone();
        }
        Ok(())
    }

    fn to_config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("homeserver".to_string(), self.homeserver.clone());
        config.insert("access_token".to_string(), self.access_token.clone());
        config
    }
}

#[async_trait]
impl GatewayAdapter for MatrixChannel {
    async fn connect(&mut self) -> anyhow::Result<()> {
        let client = matrix_sdk::Client::builder()
            .homeserver_url(&self.homeserver)
            .build()
            .await
            .map_err(|e| anyhow::anyhow!(MatrixError::SdkError(e.to_string())))?;

        client
            .restore_login(matrix_sdk::LoginRestore::new(&self.access_token))
            .await
            .map_err(|_| anyhow::anyhow!(MatrixError::AuthFailed))?;

        self.user_id = client.user_id()
            .map(|u| u.to_string())
            .unwrap_or_default();
        
        self.client = Some(client);
        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        if let Some(client) = &self.client {
            client.logout().await.ok();
        }
        self.client = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}

#[async_trait]
impl OutboundAdapter for MatrixChannel {
    async fn send(&self, message: OutgoingMessage) -> anyhow::Result<()> {
        let rate_limited = self.rate_limiter.check().await?;
        if !rate_limited {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let client = self.client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected"))?;

        let room_id = self.joined_rooms.get(&message.target)
            .ok_or_else(|| anyhow::anyhow!("Not joined room: {}", message.target))?;

        let content = NormalizedMarkdown::from(message.content);
        
        client
            .room(room_id)
            .ok_or_else(|| anyhow::anyhow!("Room not found"))?
            .send(content.to_string().into(), None)
            .await
            .map_err(|e| anyhow::anyhow!(MatrixError::SdkError(e.to_string())))?;

        Ok(())
    }
}

#[async_trait]
impl ChannelPlugin for MatrixChannel {
    fn id(&self) -> &str {
        "matrix"
    }

    fn meta(&self) -> &ChannelMeta {
        &ChannelMeta {
            name: "Matrix".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Matrix channel adapter".to_string(),
        }
    }

    fn capabilities(&self) -> &ChannelCapabilities {
        &ChannelCapabilities {
            chat_types: vec![ChatType::OneOnOne, ChatType::Group],
            media_types: vec![MediaType::Image, MediaType::Video, MediaType::Audio, MediaType::File],
            has_user_list: true,
            has_threading: true,
            has_typing: true,
            has_read_receipts: true,
        }
    }

    fn config(&self) -> &dyn ChannelConfigAdapter {
        self
    }

    fn gateway(&self) -> Option<&dyn GatewayAdapter> {
        Some(self)
    }

    fn outbound(&self) -> Option<&dyn OutboundAdapter> {
        Some(self)
    }
}
```

### Channel Adapter Pattern

Channels implement a common `ChannelPlugin` trait with optional adapters. Your adapter is responsible for:

1. **Connecting** to the external platform (authentication, session setup)
2. **Converting** platform-specific messages to `ChannelMessage`
3. **Converting** `ChannelMessage` back to platform-specific format
4. **Handling** reconnection and error recovery
5. **Rate limiting** API calls appropriately

### Building the Channel Plugin

Add to your `Cargo.toml`:

```toml
[package.metadata]
plugin-type = "channel"
```

### Registering with Aisopod

Add to your config:

```toml
[[plugins]]
path = "./plugins/matrix-channel"
```

## Appendix

### Common Error Patterns

#### Async Trait Implementation

```rust
use async_trait::async_trait;

#[async_trait]
pub trait MyTrait {
    async fn my_method(&self) -> Result<()>;
}
```

#### Error Conversion

```rust
#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
```

### Useful Crates

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `async-trait` | Async trait methods |
| `thiserror` | Error definitions |
| `tracing` | Structured logging |
| `serde` / `serde_json` | Serialization |
| `anyhow` | Error handling |
| `futures` | Async utilities |

### Debugging Tips

1. Set `RUST_LOG=debug` for detailed logging
2. Use `tokio-console` for async debugging
3. Enable backtrace with `RUST_BACKTRACE=1`
4. Check error chain with `.source()` method
