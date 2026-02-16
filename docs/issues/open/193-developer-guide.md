# Issue 193: Write Developer Guide (Architecture, Contributing, Plugin/Channel Dev)

## Summary
Create a comprehensive developer guide covering the internal architecture of Aisopod, contribution workflows, and step-by-step tutorials for developing plugins and custom channels.

## Location
- Crate: N/A (documentation)
- File: `docs/book/src/developer-guide.md`

## Current Behavior
Architecture knowledge exists only in developers' heads and scattered code comments. There is no onboarding path for new contributors, and no documentation for building plugins or custom channels.

## Expected Behavior
A developer guide at `docs/book/src/developer-guide.md` that enables a new contributor to understand the codebase, set up a development environment, submit quality PRs, and build a working plugin or custom channel from scratch.

## Impact
Developer documentation is critical for open-source sustainability. Without it, new contributors face a steep learning curve, plugins cannot be developed by the community, and the bus factor remains dangerously low.

## Suggested Implementation

1. **Create** `docs/book/src/developer-guide.md` with the following sections:

2. **Architecture Overview section:**
   ```markdown
   ## Architecture

   ### Crate Dependency Graph
   \```
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
   \```

   ### Module Organization
   Each crate follows a consistent structure:
   \```
   crates/aisopod-example/
   ├── Cargo.toml
   ├── src/
   │   ├── lib.rs          # Public API and re-exports
   │   ├── config.rs       # Crate-specific configuration
   │   ├── error.rs        # Error types
   │   └── ...             # Feature modules
   └── tests/
       └── integration.rs  # Integration tests
   \```
   ```

3. **Key Design Patterns section:**
   ```markdown
   ### Key Design Patterns

   **Trait-based abstraction**: Core extension points are defined as traits:
   - `Channel` — implement to add a messaging platform
   - `Skill` — implement to add a tool/capability
   - `ModelProvider` — implement to add an LLM provider
   - `StorageBackend` — implement to add persistence

   **Async-first**: All I/O-bound operations use `async`/`await` with
   `tokio` as the runtime. Traits use `async_trait` where needed.

   **Streaming**: LLM responses are streamed via `tokio::sync::mpsc`
   channels and exposed as SSE or WebSocket events.

   **Error handling**: Each crate defines its own error enum via `thiserror`,
   with `From` conversions for clean propagation.
   ```

4. **Contributing Guide section:**
   ```markdown
   ## Contributing

   ### Development Environment Setup
   \```bash
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
   \```

   ### Build & Test
   \```bash
   # Build in release mode
   cargo build --release

   # Run specific crate tests
   cargo test -p aisopod-gateway

   # Run with logging
   RUST_LOG=debug cargo run -- gateway start
   \```

   ### Code Style
   - Follow `rustfmt` defaults (no custom config)
   - All public items must have doc comments (`///`)
   - Prefer `thiserror` for error types
   - Use `tracing` for logging (not `log` or `println!`)
   - Keep functions under 50 lines where possible

   ### Pull Request Process
   1. Fork the repository and create a feature branch
   2. Write tests for new functionality
   3. Ensure `cargo test --workspace` passes
   4. Ensure `cargo clippy --workspace` is clean
   5. Write a clear PR description referencing the issue number
   6. Request review from a maintainer
   ```

5. **Plugin Development Tutorial section:**
   ```markdown
   ## Plugin Development

   ### Implementing the `Skill` Trait

   \```rust
   use aisopod_skill::{Skill, SkillContext, SkillResult};
   use async_trait::async_trait;

   pub struct WeatherSkill {
       api_key: String,
   }

   #[async_trait]
   impl Skill for WeatherSkill {
       fn name(&self) -> &str {
           "weather"
       }

       fn description(&self) -> &str {
           "Get current weather for a location"
       }

       fn parameters_schema(&self) -> serde_json::Value {
           serde_json::json!({
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

       async fn execute(&self, ctx: &SkillContext) -> SkillResult {
           let location = ctx.param_str("location")?;
           let weather = self.fetch_weather(&location).await?;
           Ok(serde_json::json!({"weather": weather}))
       }
   }
   \```

   ### Plugin Manifest
   Create a `plugin.toml` in your plugin crate:
   \```toml
   [plugin]
   name = "weather"
   version = "0.1.0"
   description = "Weather information skill"
   author = "Your Name"

   [[skills]]
   name = "weather"
   \```

   ### Registering with Aisopod
   Add the plugin to your config:
   \```toml
   [[plugins]]
   path = "./plugins/weather"
   \```
   ```

6. **Channel Development Tutorial section:**
   ```markdown
   ## Channel Development

   ### Implementing the `Channel` Trait

   \```rust
   use aisopod_channel::{Channel, ChannelConfig, ChannelMessage, ChannelResult};
   use async_trait::async_trait;

   pub struct MatrixChannel {
       homeserver: String,
       access_token: String,
   }

   #[async_trait]
   impl Channel for MatrixChannel {
       fn name(&self) -> &str {
           "matrix"
       }

       async fn connect(&mut self, config: &ChannelConfig) -> ChannelResult<()> {
           // Initialize Matrix SDK client
           // Join configured rooms
           Ok(())
       }

       async fn receive(&mut self) -> ChannelResult<ChannelMessage> {
           // Poll for new Matrix messages
           // Convert to ChannelMessage
           todo!()
       }

       async fn send(&self, message: ChannelMessage) -> ChannelResult<()> {
           // Send message back to Matrix room
           Ok(())
       }

       async fn disconnect(&mut self) -> ChannelResult<()> {
           // Clean shutdown
           Ok(())
       }
   }
   \```

   ### Channel Adapter Pattern
   Channels implement a common `Channel` trait, allowing the gateway to
   treat all channels uniformly. Your adapter is responsible for:
   1. Connecting to the external platform
   2. Converting platform-specific messages to `ChannelMessage`
   3. Converting `ChannelMessage` back to platform-specific format
   4. Handling reconnection and error recovery
   ```

7. **Update `SUMMARY.md`** to link to this page.

## Dependencies
- Issue 187 (documentation infrastructure)
- Issue 115 (plugin system tests validate trait interface)
- Issue 096 (channel system tests validate trait interface)

## Acceptance Criteria
- [ ] `docs/book/src/developer-guide.md` exists and is linked from `SUMMARY.md`
- [ ] Architecture section includes crate dependency graph and module structure
- [ ] Key design patterns (traits, async, streaming, errors) are explained
- [ ] Contributing section covers: dev setup, build/test, code style, PR process
- [ ] Plugin development tutorial has a complete `Skill` trait implementation example
- [ ] Plugin manifest format is documented
- [ ] Channel development tutorial has a complete `Channel` trait implementation example
- [ ] A new developer can create a basic plugin or channel by following the guide
- [ ] `mdbook build` succeeds with this page included

---
*Created: 2026-02-15*
