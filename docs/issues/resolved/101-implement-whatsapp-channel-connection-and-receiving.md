# Issue 101: Implement WhatsApp Channel — Connection and Message Receiving

## Summary
Create the `aisopod-channel-whatsapp` crate and implement WhatsApp Business API connectivity. Set up API key authentication, webhook handler for receiving messages, and register the channel as a `ChannelPlugin` in the channel registry.

## Location
- Crate: `aisopod-channel-whatsapp`
- File: `crates/aisopod-channel-whatsapp/src/lib.rs`, `crates/aisopod-channel-whatsapp/src/connection.rs`, `crates/aisopod-channel-whatsapp/src/receive.rs`, `crates/aisopod-channel-whatsapp/src/webhook.rs`

## Current Behavior
No WhatsApp channel implementation exists. The channel abstraction layer (Issues 089–092) defines the `ChannelPlugin` trait and registry but has no concrete WhatsApp integration.

## Expected Behavior
A new `aisopod-channel-whatsapp` crate provides a `WhatsAppChannel` struct that implements `ChannelPlugin`. The channel authenticates with the WhatsApp Business API using an API token, exposes a webhook endpoint for receiving inbound messages, and normalizes incoming messages from DMs and groups into the shared `IncomingMessage` type.

## Impact
WhatsApp is the world's most widely used messaging platform. This issue delivers the foundational connectivity for WhatsApp Business API mode, enabling aisopod agents to receive messages from WhatsApp users.

## Suggested Implementation
1. **Create the crate scaffold:**
   - Run `cargo new --lib crates/aisopod-channel-whatsapp` and add it to the workspace `Cargo.toml`.
   - Add dependencies: `reqwest` (HTTP client), `tokio`, `serde`, `serde_json`, `tracing`, and workspace crates `aisopod-channel`, `aisopod-shared`, `aisopod-config`.
2. **Define the account configuration type:**
   ```rust
   pub struct WhatsAppAccountConfig {
       pub mode: WhatsAppMode, // BusinessApi (initially)
       pub api_token: Option<String>,
       pub phone_number_id: Option<String>,
       pub business_account_id: Option<String>,
       pub webhook_verify_token: Option<String>,
       pub allowed_numbers: Option<Vec<String>>,
   }
   
   pub enum WhatsAppMode {
       BusinessApi,
       BaileysBridge, // future support
   }
   ```
   Derive `Deserialize` so it can be loaded from config files.
3. **Implement `WhatsAppChannel`:**
   - Create a struct `WhatsAppChannel` that holds the HTTP client, API token, and configuration.
   - Implement a `new(config: WhatsAppAccountConfig) -> Result<Self>` constructor that validates the API token by calling a test endpoint (e.g., fetching the phone number details).
4. **Implement webhook verification:**
   - WhatsApp Business API requires a webhook verification handshake. Implement a GET handler at the webhook path that responds to the `hub.challenge` parameter when `hub.verify_token` matches the configured `webhook_verify_token`.
   - This endpoint is registered with the gateway HTTP server (Issue 026).
5. **Implement webhook message receiver:**
   - Implement a POST handler at the webhook path that receives incoming message notifications from WhatsApp.
   - Parse the JSON payload which contains a `messages` array under `entry[].changes[].value.messages[]`.
   - Extract message type (text, image, document, audio, video, sticker, location, contacts), sender phone number, timestamp, and message body.
   - Handle both individual and group messages.
6. **Normalize incoming messages:**
   - Convert each WhatsApp message into the shared `IncomingMessage` type defined in Issue 091.
   - Map WhatsApp phone numbers to sender identifiers.
   - Map WhatsApp conversation IDs to channel-specific identifiers.
   - Detect whether the message is a DM or group message.
7. **Implement `ChannelPlugin` trait:**
   - Implement `id()` → return `"whatsapp"`.
   - Implement `metadata()` → return capabilities (supports text, media, typing indicators, replies, read receipts).
   - Implement `start()` → register the webhook endpoints with the gateway.
   - Implement `stop()` → deregister the webhook endpoints.
8. **Register with the channel registry:**
   - Provide a function `register(registry: &mut ChannelRegistry, config: WhatsAppAccountConfig)` that creates a `WhatsAppChannel` and adds it to the registry.
9. **Add basic unit tests:**
   - Test config deserialization with valid and missing fields.
   - Test webhook verification handshake with correct and incorrect verify tokens.
   - Test JSON payload parsing for various message types (text, image, group).
   - Test message normalization from WhatsApp types to `IncomingMessage`.

## Dependencies
- Issue 026 (axum HTTP server skeleton — for webhook endpoint registration)
- Issue 089 (define ChannelPlugin trait and channel metadata types)
- Issue 090 (define adapter interface traits)
- Issue 091 (define message types)
- Issue 092 (implement channel registry)

## Resolution

This issue was implemented by creating the `aisopod-channel-whatsapp` crate with full WhatsApp Business API integration.

### Changes Made:

1. **Created the crate scaffold:**
   - Created `crates/aisopod-channel-whatsapp` with `cargo new --lib`
   - Added to workspace `Cargo.toml`
   - Added dependencies: `reqwest`, `tokio`, `serde`, `serde_json`, `tracing`, `axum`, `anyhow`, `chrono`

2. **Implemented configuration types:**
   - `crates/aisopod-channel-whatsapp/src/connection.rs` - `WhatsAppMode` enum and `WhatsAppAccountConfig` struct with `Deserialize` derive

3. **Implemented ChannelPlugin trait:**
   - `crates/aisopod-channel-whatsapp/src/lib.rs` - `WhatsAppChannel` struct and `ChannelPlugin` trait implementation with `id()`, `metadata()`, `start()`, `stop()` methods
   - `register()` function to add WhatsApp channel to the channel registry

4. **Implemented webhook handlers:**
   - `crates/aisopod-channel-whatsapp/src/webhook.rs` - GET handler for verification (responds to `hub.challenge` with verify token check) and POST handler for receiving messages

5. **Implemented message receiving and normalization:**
   - `crates/aisopod-channel-whatsapp/src/receive.rs` - Parse WhatsApp JSON payload, extract messages from `entry[].changes[].value.messages[]`, handle text, image, audio, location, and group messages
   - Normalize to shared `IncomingMessage` type with sender ID mapping and DM/group detection

6. **Implemented message sending:**
   - `crates/aisopod-channel-whatsapp/src/send.rs` - Send text messages, images, and other media via WhatsApp Business API

7. **Implemented media handling:**
   - `crates/aisopod-channel-whatsapp/src/media.rs` - Upload and download media files with proper content type handling

8. **Added unit tests (19 tests):**
   - Webhook verification with correct and incorrect tokens
   - Message parsing for text, image, audio, location, and group messages
   - `WhatsAppMode` and `WhatsAppAccountConfig` serialization/deserialization
   - Channel registration with the registry
   - Sender filtering for allowed numbers

### Verification:
- `cargo build`: Passes
- `cargo test`: All tests pass (116 + 15 + 16 + 21 + ... + 19 WhatsApp tests)
- `cargo test -p aisopod-provider`: 107 tests passed
- `cargo test -p aisopod-tools`: 137 tests passed

---
*Created: 2026-02-15*
*Resolved: 2026-02-23*
