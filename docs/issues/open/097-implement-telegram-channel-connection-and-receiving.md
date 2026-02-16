# Issue 097: Implement Telegram Channel — Connection and Message Receiving

## Summary
Create the `aisopod-channel-telegram` crate and implement Telegram Bot API connectivity using the `teloxide` crate. Set up bot token authentication, message receiving via long-polling or webhook mode, and register the channel as a `ChannelPlugin` in the channel registry.

## Location
- Crate: `aisopod-channel-telegram`
- File: `crates/aisopod-channel-telegram/src/lib.rs`, `crates/aisopod-channel-telegram/src/connection.rs`, `crates/aisopod-channel-telegram/src/receive.rs`

## Current Behavior
No Telegram channel implementation exists. The channel abstraction layer (Issues 089–092) defines the `ChannelPlugin` trait and registry but has no concrete Telegram integration.

## Expected Behavior
A new `aisopod-channel-telegram` crate provides a `TelegramChannel` struct that implements `ChannelPlugin`. The bot authenticates with the Telegram Bot API using a bot token, connects via long-polling or webhook mode (configurable), and receives incoming messages from DMs, groups, and supergroups. Received messages are normalized into the shared `IncomingMessage` type and forwarded to the message routing pipeline.

## Impact
Telegram is one of the most commonly used messaging platforms for bot interactions. This issue delivers the foundational connectivity required before any sending or advanced features can be built.

## Suggested Implementation
1. **Create the crate scaffold:**
   - Run `cargo new --lib crates/aisopod-channel-telegram` and add it to the workspace `Cargo.toml`.
   - Add dependencies: `teloxide`, `tokio`, `serde`, `tracing`, and workspace crates `aisopod-channel`, `aisopod-shared`, `aisopod-config`.
2. **Define the account configuration type:**
   ```rust
   pub struct TelegramAccountConfig {
       pub bot_token: String,
       pub webhook_url: Option<String>,
       pub allowed_users: Option<Vec<i64>>,
       pub allowed_groups: Option<Vec<i64>>,
       pub parse_mode: ParseMode,
   }
   ```
   Derive `Deserialize` so it can be loaded from config files.
3. **Implement `TelegramChannel`:**
   - Create a struct `TelegramChannel` that holds the `teloxide::Bot` instance and configuration.
   - Implement a `new(config: TelegramAccountConfig) -> Result<Self>` constructor that validates the bot token by calling the `getMe` API endpoint.
4. **Implement long-polling receiver:**
   - Use `teloxide::dispatching::Dispatcher` or `teloxide::repls` to set up a long-polling loop.
   - Register an update handler that matches on `Message` updates.
   - For each incoming Telegram `Message`, extract the chat type (private, group, supergroup), sender information, message text, and any reply-to metadata.
5. **Implement webhook receiver (optional mode):**
   - If `webhook_url` is set in config, register the webhook with Telegram using `setWebhook`.
   - Expose an HTTP endpoint (coordinate with `aisopod-gateway`) that receives POST updates from Telegram.
   - Parse the JSON body into `teloxide::types::Update`.
6. **Normalize incoming messages:**
   - Convert each Telegram `Message` into the shared `IncomingMessage` type defined in Issue 091.
   - Map Telegram chat IDs to channel-specific identifiers.
   - Map Telegram user IDs to sender identifiers.
   - Detect whether the message is a DM or group message based on `chat.type`.
7. **Implement `ChannelPlugin` trait:**
   - Implement `id()` → return `"telegram"`.
   - Implement `metadata()` → return capabilities (supports text, media, typing indicators, replies).
   - Implement `start()` → spawn the long-polling or webhook listener as a background task.
   - Implement `stop()` → gracefully shut down the listener.
8. **Register with the channel registry:**
   - In the crate's public API, provide a function `register(registry: &mut ChannelRegistry, config: TelegramAccountConfig)` that creates a `TelegramChannel` and adds it to the registry.
9. **Add basic unit tests:**
   - Test config deserialization with valid and invalid bot tokens.
   - Test message normalization from Telegram types to `IncomingMessage`.
   - Use mock HTTP responses to test the `getMe` validation call.

## Dependencies
- Issue 089 (define ChannelPlugin trait and channel metadata types)
- Issue 090 (define adapter interface traits)
- Issue 091 (define message types)
- Issue 092 (implement channel registry)

## Acceptance Criteria
- [ ] `aisopod-channel-telegram` crate is created and added to the workspace
- [ ] `TelegramAccountConfig` is defined and deserializable from config
- [ ] Bot authenticates with Telegram using bot token (`getMe` succeeds)
- [ ] Long-polling mode receives messages from DMs, groups, and supergroups
- [ ] Webhook mode is supported as an alternative to long-polling
- [ ] Incoming Telegram messages are normalized to shared `IncomingMessage` type
- [ ] `TelegramChannel` implements the `ChannelPlugin` trait
- [ ] Channel is registered in the channel registry
- [ ] `cargo build -p aisopod-channel-telegram` compiles without errors

---
*Created: 2026-02-15*
