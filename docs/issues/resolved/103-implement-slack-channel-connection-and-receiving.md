# Issue 103: Implement Slack Channel — Connection and Message Receiving

## Summary
Create the `aisopod-channel-slack` crate and implement Slack API connectivity using a custom HTTP client. Set up bot token and app token authentication with Socket Mode, receive messages from DMs, channels, and threads, and register the channel as a `ChannelPlugin` in the channel registry.

## Location
- Crate: `aisopod-channel-slack`
- File: `crates/aisopod-channel-slack/src/lib.rs`, `crates/aisopod-channel-slack/src/connection.rs`, `crates/aisopod-channel-slack/src/receive.rs`, `crates/aisopod-channel-slack/src/socket_mode.rs`

## Current Behavior
No Slack channel implementation exists. The channel abstraction layer (Issues 089–092) defines the `ChannelPlugin` trait and registry but has no concrete Slack integration.

## Expected Behavior
A new `aisopod-channel-slack` crate provides a `SlackChannel` struct that implements `ChannelPlugin`. The bot authenticates with the Slack API using a bot token and connects via Socket Mode (using an app token) to receive real-time events. Incoming messages from DMs, channels, and threads are normalized into the shared `IncomingMessage` type and forwarded to the message routing pipeline.

## Impact
Slack is the dominant platform for workplace communication. This issue delivers the foundational connectivity for Slack integration, enabling aisopod agents to receive messages in Slack workspaces.

## Suggested Implementation
1. **Create the crate scaffold:**
   - Run `cargo new --lib crates/aisopod-channel-slack` and add it to the workspace `Cargo.toml`.
   - Add dependencies: `reqwest`, `tokio`, `tokio-tungstenite` (for WebSocket), `serde`, `serde_json`, `tracing`, and workspace crates `aisopod-channel`, `aisopod-shared`, `aisopod-config`.
2. **Define the account configuration type:**
   ```rust
   pub struct SlackAccountConfig {
       pub bot_token: String,
       pub app_token: Option<String>,
       pub signing_secret: Option<String>,
       pub allowed_channels: Option<Vec<String>>,
       pub mention_required: bool,
   }
   ```
   Derive `Deserialize` so it can be loaded from config files.
3. **Implement `SlackChannel`:**
   - Create a struct `SlackChannel` that holds the HTTP client (for Slack Web API calls), the bot token, and configuration.
   - Implement a `new(config: SlackAccountConfig) -> Result<Self>` constructor that validates the bot token by calling `auth.test`.
4. **Implement Socket Mode connection:**
   - Use the app token to call `apps.connections.open` to obtain a WebSocket URL.
   - Connect to the WebSocket URL using `tokio-tungstenite`.
   - Handle the Socket Mode protocol: receive `hello` event, acknowledge `envelope_id` for each event received.
   - Implement automatic reconnection if the WebSocket connection drops.
5. **Handle incoming message events:**
   - Listen for `events_api` type envelopes containing `message` events.
   - Extract: channel ID, user ID, message text, timestamp (ts), thread timestamp (thread_ts), and any file attachments.
   - Filter out messages from the bot itself (compare `user` with the bot's own user ID obtained from `auth.test`).
   - Filter messages from channels not in the allowed list (if configured).
   - Detect thread context: if `thread_ts` is present and differs from `ts`, the message is a thread reply.
6. **Normalize incoming messages:**
   - Convert each Slack message event into the shared `IncomingMessage` type defined in Issue 091.
   - Map Slack channel IDs to channel-specific identifiers.
   - Map Slack user IDs to sender identifiers.
   - Detect whether the message is a DM (channel ID starts with `D`), channel message (starts with `C`), or group (starts with `G`).
   - Include thread_ts in metadata for thread-aware routing.
7. **Implement `ChannelPlugin` trait:**
   - Implement `id()` → return `"slack"`.
   - Implement `metadata()` → return capabilities (supports text, media, typing indicators, replies, threads, reactions, Block Kit).
   - Implement `start()` → spawn the Socket Mode WebSocket listener as a background task.
   - Implement `stop()` → close the WebSocket connection and shut down the listener.
8. **Register with the channel registry:**
   - Provide a function `register(registry: &mut ChannelRegistry, config: SlackAccountConfig)` that creates a `SlackChannel` and adds it to the registry.
9. **Add basic unit tests:**
   - Test config deserialization with valid and missing fields.
   - Test Socket Mode envelope acknowledgment construction.
   - Test message event parsing for DM, channel, and thread messages.
   - Test message normalization from Slack event types to `IncomingMessage`.
   - Test self-message filtering.

## Dependencies
- Issue 089 (define ChannelPlugin trait and channel metadata types)
- Issue 090 (define adapter interface traits)
- Issue 091 (define message types)
- Issue 092 (implement channel registry)

## Acceptance Criteria
- [x] `aisopod-channel-slack` crate is created and added to the workspace
- [x] `SlackAccountConfig` is defined and deserializable from config
- [x] Bot authenticates with Slack using bot token (`auth.test` succeeds)
- [x] Socket Mode connection is established via app token and WebSocket
- [x] Socket Mode events are acknowledged correctly
- [x] DM, channel, and thread messages are received
- [x] Bot's own messages are filtered out
- [x] Incoming Slack messages are normalized to shared `IncomingMessage` type
- [x] `SlackChannel` implements the `ChannelPlugin` trait
- [x] Channel is registered in the channel registry
- [x] `cargo build -p aisopod-channel-slack` compiles without errors

## Resolution
The Slack channel integration was implemented with the following components:

**Crate Structure:**
- Created `crates/aisopod-channel-slack/` crate with workspace integration
- Implemented modular design across multiple source files
- All required dependencies added: `reqwest`, `tokio`, `tokio-tungstenite`, `serde`, `serde_json`, `tracing`, `chrono`, `futures_util`, `anyhow`

**Core Implementation:**
- `SlackAccountConfig`: Configuration type with bot token, optional app token, channel/user filtering, mention requirement, and reconnection settings
- `SlackAccount`: Account wrapper with configuration and bot user ID storage
- `SlackChannel`: Main plugin implementation with `ChannelPlugin` trait
- `SlackSocketModeConnection`: WebSocket connection management for Socket Mode
- `SlackClientHandle`: HTTP client for Slack Web API calls

**Message Processing:**
- `receive.rs`: Message receiving, filtering, and normalization
  - Self-message filtering to avoid loops
  - Channel/user allowlist filtering
  - Mention requirement enforcement
  - Message normalization to shared `IncomingMessage` type
  - Support for DMs (channel ID starts with 'D'), channels ('C'), and groups ('G')
  - Thread context detection and reply_to population

**Socket Mode Protocol:**
- `apps.connections.open` endpoint for WebSocket URL fetch
- WebSocket connection via `tokio-tungstenite`
- Event acknowledgment via `envelope_id`
- Background task spawning with shutdown signal handling

**Tests:**
- 42 unit tests covering all modules
- Tests for config serialization, message normalization, filtering, and Socket Mode events

**Verification:**
- `cargo build -p aisopod-channel-slack`: ✅ Passed with `RUSTFLAGS=-Awarnings`
- `cargo test -p aisopod-channel-slack`: ✅ 42 tests passed

---
*Created: 2026-02-15*
*Resolved: 2026-02-23*
