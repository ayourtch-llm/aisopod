# Issue 106: Add Tier 1 Channel Integration Tests

## Summary
Create comprehensive integration tests that verify the end-to-end message flow for each Tier 1 channel (Telegram, Discord, WhatsApp, Slack), including connection lifecycle, error handling, reconnection, and rate limiting.

## Location
- Crate: `aisopod-channel` (integration test suite)
- File: `crates/aisopod-channel/tests/integration/mod.rs`, `crates/aisopod-channel/tests/integration/telegram.rs`, `crates/aisopod-channel/tests/integration/discord.rs`, `crates/aisopod-channel/tests/integration/whatsapp.rs`, `crates/aisopod-channel/tests/integration/slack.rs`

## Current Behavior
Individual channel crates have unit tests for their internal logic, but there are no integration tests that verify the full message lifecycle from receiving a message through the channel abstraction, routing, and sending a response back.

## Expected Behavior
An integration test suite exercises each Tier 1 channel's full message flow using mock API servers. Tests verify that messages are received, normalized, routed, and that responses are sent back correctly. Additional tests cover error scenarios, reconnection behavior, and rate limit compliance.

## Impact
Integration tests catch issues that unit tests miss — such as incorrect wiring between components, serialization mismatches, and lifecycle bugs. They provide confidence that each channel works end-to-end before deployment.

## Suggested Implementation
1. **Set up mock API servers:**
   - Create a `MockTelegramServer` using `axum` or `wiremock` that simulates the Telegram Bot API. It should handle `getMe`, `getUpdates`, `sendMessage`, `sendPhoto`, etc.
   - Create a `MockDiscordGateway` that simulates the Discord WebSocket gateway (send `HELLO`, `READY` events) and a `MockDiscordApi` for REST calls.
   - Create a `MockWhatsAppApi` that simulates the WhatsApp Business API webhook verification and message endpoints.
   - Create a `MockSlackApi` that simulates Slack's `auth.test`, `apps.connections.open` (returning a mock WebSocket URL), and `chat.postMessage` endpoints, plus a mock Socket Mode WebSocket.
   - Each mock server should record received requests for assertion.

2. **Telegram integration tests (`telegram.rs`):**
   - `test_telegram_connect_and_receive` — Start the Telegram channel with the mock server. Simulate an incoming update via the mock's `getUpdates` response. Verify the message is normalized to `IncomingMessage` with correct fields.
   - `test_telegram_send_text` — Trigger a send operation. Verify the mock server receives a `sendMessage` request with correct chat_id and text.
   - `test_telegram_send_media` — Trigger a media send. Verify the mock server receives a `sendPhoto`/`sendDocument` request.
   - `test_telegram_error_recovery` — Simulate a 429 (rate limit) response from the mock. Verify the channel retries after the `Retry-After` delay.
   - `test_telegram_reconnect` — Simulate a network error during polling. Verify the channel reconnects with exponential backoff.

3. **Discord integration tests (`discord.rs`):**
   - `test_discord_gateway_connect` — Start the Discord channel with the mock gateway. Verify the bot sends `IDENTIFY` and receives `READY`.
   - `test_discord_receive_message` — Simulate a `MESSAGE_CREATE` event via the mock gateway. Verify the message is normalized correctly.
   - `test_discord_send_text` — Trigger a send. Verify the mock REST API receives a `POST /channels/{id}/messages` request.
   - `test_discord_send_embed` — Trigger an embed send. Verify the mock receives the correct embed JSON structure.
   - `test_discord_reconnect` — Close the mock gateway connection. Verify the channel reconnects and resumes.

4. **WhatsApp integration tests (`whatsapp.rs`):**
   - `test_whatsapp_webhook_verification` — Send a GET request to the webhook endpoint with the correct verify token. Verify the challenge is echoed back.
   - `test_whatsapp_receive_message` — Send a POST request to the webhook endpoint with a message payload. Verify the message is normalized correctly.
   - `test_whatsapp_send_text` — Trigger a send. Verify the mock API receives a POST to `/messages` with correct payload.
   - `test_whatsapp_send_media` — Trigger a media send. Verify media upload and message send calls.
   - `test_whatsapp_allowed_number_filter` — Send a message from a number not in the allowed list. Verify it is silently dropped.

5. **Slack integration tests (`slack.rs`):**
   - `test_slack_socket_mode_connect` — Start the Slack channel with the mock. Verify `auth.test` is called, `apps.connections.open` is called, and WebSocket connects.
   - `test_slack_receive_message` — Simulate a message event via the mock Socket Mode WebSocket. Verify acknowledgment is sent and message is normalized.
   - `test_slack_send_text` — Trigger a send. Verify the mock receives a `chat.postMessage` request.
   - `test_slack_send_blocks` — Trigger a Block Kit send. Verify the mock receives correct `blocks` JSON.
   - `test_slack_thread_reply` — Trigger a thread reply. Verify `thread_ts` is included in the request.
   - `test_slack_reconnect` — Close the mock WebSocket. Verify the channel reconnects.

6. **Cross-channel tests:**
   - `test_rate_limit_compliance` — For each channel, send messages rapidly and verify the rate limiter delays requests appropriately.
   - `test_connection_state_transitions` — For each channel, verify state transitions: `Disconnected` → `Connecting` → `Connected` → (simulate drop) → `Reconnecting` → `Connected`.
   - `test_error_mapping` — Simulate various API errors from each mock server and verify they map to the correct `ChannelError` variants.

7. **Test infrastructure:**
   - Use `#[tokio::test]` for all async tests.
   - Use `tracing-test` or `tracing-subscriber` with a test writer for log capture.
   - Ensure each test starts and stops its own mock server to avoid port conflicts (use port 0 for OS-assigned ports).
   - Add a `#[ignore]` attribute to tests that require real API tokens, with instructions for running them manually.

## Dependencies
- Issue 097 (Telegram channel connection and message receiving)
- Issue 098 (Telegram channel message sending and features)
- Issue 099 (Discord channel connection and message receiving)
- Issue 100 (Discord channel message sending and features)
- Issue 101 (WhatsApp channel connection and message receiving)
- Issue 102 (WhatsApp channel message sending and features)
- Issue 103 (Slack channel connection and message receiving)
- Issue 104 (Slack channel message sending and features)
- Issue 105 (shared channel utilities)

## Acceptance Criteria
- [ ] Mock API servers are implemented for Telegram, Discord, WhatsApp, and Slack
- [ ] Telegram integration tests cover connect, receive, send (text + media), error recovery, and reconnection
- [ ] Discord integration tests cover gateway connect, receive, send (text + embed), and reconnection
- [ ] WhatsApp integration tests cover webhook verification, receive, send (text + media), and allowed number filtering
- [ ] Slack integration tests cover Socket Mode connect, receive, send (text + Block Kit), thread replies, and reconnection
- [ ] Cross-channel tests verify rate limit compliance, connection state transitions, and error mapping
- [ ] All tests use mock servers (no real API calls in CI)
- [ ] Tests run with `cargo test -p aisopod-channel --test integration` and pass
- [ ] Test infrastructure supports optional real-API tests gated behind `#[ignore]`

---
*Created: 2026-02-15*
