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

## Acceptance Criteria
- [ ] `aisopod-channel-whatsapp` crate is created and added to the workspace
- [ ] `WhatsAppAccountConfig` is defined and deserializable from config
- [ ] API token authentication is validated on startup
- [ ] Webhook verification (GET) responds correctly to WhatsApp challenge
- [ ] Webhook receiver (POST) parses incoming message notifications
- [ ] DM and group messages are received and distinguished
- [ ] Incoming WhatsApp messages are normalized to shared `IncomingMessage` type
- [ ] `WhatsAppChannel` implements the `ChannelPlugin` trait
- [ ] Channel is registered in the channel registry
- [ ] `cargo build -p aisopod-channel-whatsapp` compiles without errors

---
*Created: 2026-02-15*
