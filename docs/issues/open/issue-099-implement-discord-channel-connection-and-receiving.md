# Issue 099: Implement Discord Channel — Connection and Message Receiving

## Summary
Create the `aisopod-channel-discord` crate and implement Discord Bot API connectivity using the `serenity` crate. Set up bot token authentication, gateway connection for receiving events, and register the channel as a `ChannelPlugin` in the channel registry.

## Location
- Crate: `aisopod-channel-discord`
- File: `crates/aisopod-channel-discord/src/lib.rs`, `crates/aisopod-channel-discord/src/connection.rs`, `crates/aisopod-channel-discord/src/receive.rs`

## Current Behavior
No Discord channel implementation exists. The channel abstraction layer (Issues 089–092) defines the `ChannelPlugin` trait and registry but has no concrete Discord integration.

## Expected Behavior
A new `aisopod-channel-discord` crate provides a `DiscordChannel` struct that implements `ChannelPlugin`. The bot authenticates with the Discord API using a bot token, connects to the Discord gateway via WebSocket, and receives incoming messages from DMs, server channels, and threads. Received messages are normalized into the shared `IncomingMessage` type and forwarded to the message routing pipeline.

## Impact
Discord is a widely used platform for community and team interactions. This issue delivers the foundational connectivity required before any sending or advanced features can be built.

## Suggested Implementation
1. **Create the crate scaffold:**
   - Run `cargo new --lib crates/aisopod-channel-discord` and add it to the workspace `Cargo.toml`.
   - Add dependencies: `serenity`, `tokio`, `serde`, `tracing`, and workspace crates `aisopod-channel`, `aisopod-shared`, `aisopod-config`.
2. **Define the account configuration type:**
   ```rust
   pub struct DiscordAccountConfig {
       pub bot_token: String,
       pub application_id: Option<String>,
       pub allowed_guilds: Option<Vec<u64>>,
       pub allowed_channels: Option<Vec<u64>>,
       pub mention_required_in_channels: bool,
   }
   ```
   Derive `Deserialize` so it can be loaded from config files.
3. **Implement `DiscordChannel`:**
   - Create a struct `DiscordChannel` that holds the `serenity::Client` builder configuration and the runtime client handle.
   - Implement a `new(config: DiscordAccountConfig) -> Result<Self>` constructor that validates the bot token.
4. **Implement gateway connection:**
   - Use `serenity::Client::builder` with the bot token and appropriate `GatewayIntents` (at minimum: `GUILD_MESSAGES`, `DIRECT_MESSAGES`, `MESSAGE_CONTENT`).
   - Implement `serenity::EventHandler` on a handler struct to receive `message` events.
   - Start the client in a background tokio task.
5. **Handle incoming messages:**
   - In the `message` event handler, extract: channel type (DM, guild text, thread), author information, message content, and any reply reference.
   - Filter messages from the bot itself to avoid self-loops.
   - Filter messages from guilds/channels not in the allowed lists (if configured).
6. **Normalize incoming messages:**
   - Convert each `serenity::model::channel::Message` into the shared `IncomingMessage` type defined in Issue 091.
   - Map Discord channel IDs and guild IDs to channel-specific identifiers.
   - Map Discord user IDs to sender identifiers.
   - Detect whether the message is a DM or guild message based on the channel type.
7. **Implement `ChannelPlugin` trait:**
   - Implement `id()` → return `"discord"`.
   - Implement `metadata()` → return capabilities (supports text, media, embeds, typing indicators, replies, threads, reactions).
   - Implement `start()` → spawn the gateway client as a background task.
   - Implement `stop()` → send a shutdown signal to the gateway client.
8. **Register with the channel registry:**
   - Provide a function `register(registry: &mut ChannelRegistry, config: DiscordAccountConfig)` that creates a `DiscordChannel` and adds it to the registry.
9. **Add basic unit tests:**
   - Test config deserialization with valid and invalid tokens.
   - Test message normalization from serenity types to `IncomingMessage`.
   - Test self-message filtering.
   - Test guild/channel allowlist filtering.

## Dependencies
- Issue 089 (define ChannelPlugin trait and channel metadata types)
- Issue 090 (define adapter interface traits)
- Issue 091 (define message types)
- Issue 092 (implement channel registry)

## Acceptance Criteria
- [ ] `aisopod-channel-discord` crate is created and added to the workspace
- [ ] `DiscordAccountConfig` is defined and deserializable from config
- [ ] Bot authenticates with Discord using bot token
- [ ] Gateway connection is established and receives message events
- [ ] DM, server channel, and thread messages are received
- [ ] Bot's own messages are filtered out
- [ ] Incoming Discord messages are normalized to shared `IncomingMessage` type
- [ ] `DiscordChannel` implements the `ChannelPlugin` trait
- [ ] Channel is registered in the channel registry
- [ ] `cargo build -p aisopod-channel-discord` compiles without errors

---
*Created: 2026-02-15*
