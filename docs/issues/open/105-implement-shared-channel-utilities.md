# Issue 105: Implement Shared Channel Utilities

## Summary
Implement a shared utilities module within the `aisopod-channel` crate that provides cross-platform message formatting normalization, media transcoding, rate limit handling, connection state management with reconnection logic, and error mapping to common error types.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/src/util/mod.rs`, `crates/aisopod-channel/src/util/formatting.rs`, `crates/aisopod-channel/src/util/media.rs`, `crates/aisopod-channel/src/util/rate_limit.rs`, `crates/aisopod-channel/src/util/connection.rs`, `crates/aisopod-channel/src/util/errors.rs`

## Current Behavior
Each Tier 1 channel crate (Telegram, Discord, WhatsApp, Slack) would need to independently implement formatting conversion, rate limiting, reconnection logic, and error handling, leading to code duplication and inconsistent behavior.

## Expected Behavior
A shared `util` module in `aisopod-channel` provides reusable utilities that all channel implementations depend on. This ensures consistent behavior across platforms and reduces code duplication.

## Impact
Without shared utilities, each channel crate duplicates common logic, increasing maintenance burden and risk of inconsistencies. These utilities form the foundation for reliable, production-quality channel integrations.

## Suggested Implementation
1. **Message formatting normalization (`formatting.rs`):**
   - Define a `NormalizedMarkdown` intermediate representation that captures bold, italic, strikethrough, code, code blocks, links, and quotes.
   - Implement `fn from_telegram_markdown(input: &str) -> NormalizedMarkdown` — parse Telegram MarkdownV2 syntax.
   - Implement `fn from_discord_markdown(input: &str) -> NormalizedMarkdown` — parse Discord markdown syntax.
   - Implement `fn from_slack_mrkdwn(input: &str) -> NormalizedMarkdown` — parse Slack mrkdwn syntax.
   - Implement `fn from_plain_text(input: &str) -> NormalizedMarkdown` — wrap plain text.
   - Implement `fn to_telegram_markdown(&self) -> String` — render to Telegram MarkdownV2.
   - Implement `fn to_discord_markdown(&self) -> String` — render to Discord markdown.
   - Implement `fn to_slack_mrkdwn(&self) -> String` — render to Slack mrkdwn.
   - Implement `fn to_whatsapp_text(&self) -> String` — render to WhatsApp text (limited formatting).
   - Implement `fn to_plain_text(&self) -> String` — strip all formatting.
   - This enables cross-platform message forwarding with correct formatting.
2. **Media transcoding (`media.rs`):**
   - Implement `fn ensure_compatible_format(media: &MediaAttachment, target: Platform) -> Result<MediaAttachment>` that checks if a media file is compatible with the target platform and converts if necessary.
   - Define per-platform constraints: max file size, supported image formats, supported audio/video codecs.
   - For images: resize if exceeding max dimensions, convert between PNG/JPEG/WebP as needed.
   - For audio: convert between OGG/MP3/AAC as needed (shell out to `ffmpeg` if available, or use a Rust audio library).
   - For documents: pass through without conversion (all platforms support common document types).
   - Return the original media unchanged if it is already compatible.
3. **Rate limit handling (`rate_limit.rs`):**
   - Implement a `RateLimiter` struct that tracks API call counts per time window.
   - Support per-platform configuration:
     - Telegram: 30 messages/second globally, 20 messages/minute per chat.
     - Discord: varies by endpoint (default 5 requests/5 seconds per route).
     - WhatsApp Business API: 80 messages/second (business tier dependent).
     - Slack: 1 message/second per channel (Web API tier 2+).
   - Implement `async fn acquire(&self) -> Result<()>` that blocks until a request can be made without exceeding limits.
   - Parse `Retry-After` headers from API error responses and honor them.
   - Implement a sliding window algorithm using `tokio::time::Instant` for tracking.
4. **Connection state management (`connection.rs`):**
   - Define a `ConnectionState` enum: `Disconnected`, `Connecting`, `Connected`, `Reconnecting`, `Failed`.
   - Implement a `ConnectionManager` struct that tracks the current state and provides reconnection logic.
   - Implement exponential backoff for reconnection attempts: start at 1 second, double each attempt, cap at 5 minutes.
   - Add jitter (random offset) to prevent thundering herd when multiple channels reconnect simultaneously.
   - Emit state change events via a `tokio::sync::watch` channel so other components can react.
   - Implement `async fn maintain_connection<F>(&self, connect: F)` that continuously attempts to maintain a connection, calling the provided closure to establish it.
   - Reset backoff on successful connection.
   - Track total connection uptime and number of reconnections for observability.
5. **Error mapping (`errors.rs`):**
   - Define a `ChannelError` enum with variants common across all platforms:
     - `AuthenticationFailed` — invalid or expired token.
     - `RateLimited { retry_after: Duration }` — API rate limit hit.
     - `MessageTooLong { max_length: usize }` — message exceeds platform limit.
     - `MediaUnsupported { media_type: String }` — media format not supported by platform.
     - `ConnectionLost` — WebSocket or HTTP connection dropped.
     - `PermissionDenied` — bot lacks required permissions.
     - `NotFound { resource: String }` — channel, user, or message not found.
     - `PlatformError { code: String, message: String }` — platform-specific error pass-through.
   - Implement `From<TelegramError>`, `From<SerenityError>`, etc. in each channel crate to map platform errors to `ChannelError`.
   - Implement `std::fmt::Display` and `std::error::Error` for `ChannelError`.
6. **Add unit tests for each utility:**
   - Formatting: round-trip test (parse Telegram → render Discord → parse Discord → compare), test edge cases (nested formatting, escaped characters).
   - Rate limiter: test that requests are delayed when limit is reached, test `Retry-After` handling.
   - Connection manager: test state transitions, test exponential backoff timing, test reset on success.
   - Error mapping: test that each platform error variant maps to the correct `ChannelError`.

## Dependencies
- Issue 089 (define ChannelPlugin trait and channel metadata types)
- Issue 091 (define message types)

## Acceptance Criteria
- [ ] `NormalizedMarkdown` type converts correctly between Telegram, Discord, Slack, and WhatsApp formatting
- [ ] Media transcoding detects incompatible formats and converts them for the target platform
- [ ] `RateLimiter` enforces per-platform rate limits and honors `Retry-After` headers
- [ ] `ConnectionManager` implements exponential backoff with jitter for reconnection
- [ ] `ConnectionState` transitions are tracked and observable via watch channel
- [ ] `ChannelError` covers all common error scenarios across platforms
- [ ] All shared utilities are used by Tier 1 channel crates (Telegram, Discord, WhatsApp, Slack)
- [ ] Unit tests pass for formatting round-trips, rate limiting, connection management, and error mapping
- [ ] `cargo test -p aisopod-channel` passes with all utility tests green

---
*Created: 2026-02-15*
