# Issue 104: Implement Slack Channel — Message Sending and Features

## Summary
Extend the `aisopod-channel-slack` crate with full message sending capabilities, Slack mrkdwn formatting, media support, typing indicators, thread management, reaction handling, Block Kit support, channel/user discovery, and message editing/deletion.

## Location
- Crate: `aisopod-channel-slack`
- File: `crates/aisopod-channel-slack/src/send.rs`, `crates/aisopod-channel-slack/src/media.rs`, `crates/aisopod-channel-slack/src/features.rs`, `crates/aisopod-channel-slack/src/blocks.rs`

## Current Behavior
After Issue 103, the Slack channel can connect via Socket Mode and receive messages but cannot send responses, handle files, or use advanced Slack features like Block Kit.

## Expected Behavior
The Slack channel supports sending text messages with Slack mrkdwn formatting, sending and receiving files, displaying typing indicators, managing threads (reply in thread), handling reactions, building rich messages with Block Kit, discovering channels and users, and editing/deleting messages.

## Impact
This completes the Slack integration, making it a fully functional two-way communication channel with rich formatting via Block Kit and organizational features via threads.

## Suggested Implementation
1. **Implement text message sending:**
   - Add a `send_message(&self, channel: &str, text: &str, options: SendOptions) -> Result<MessageId>` method.
   - Use the Slack Web API `chat.postMessage` endpoint via `reqwest::Client::post`.
   - Build the JSON payload:
     ```json
     {
       "channel": "<channel_id>",
       "text": "<message_text>",
       "mrkdwn": true
     }
     ```
   - Include the bot token in the `Authorization: Bearer` header.
   - Handle Slack's message length limit (40,000 characters for text, though practically keep under 4,000 for readability) by splitting if needed.
   - Support Slack mrkdwn syntax: `*bold*`, `_italic_`, `~strikethrough~`, `` `code` ``, ` ```code block``` `, `>quote`.
2. **Implement file sending and receiving:**
   - For outgoing files, use `files.uploadV2` endpoint to upload files to Slack, then share them in the target channel.
   - For incoming files, extract `files` array from message events, download each file using the `url_private_download` URL with the bot token for authorization.
   - Map Slack file types (mimetype) to the shared `MediaType` enum.
3. **Implement typing indicators:**
   - There is no official "typing" API in Slack Web API for bots.
   - As a workaround, post an ephemeral message or use the Socket Mode `typing` event if supported.
   - Alternatively, post a temporary "thinking…" message that is updated with the actual response once ready.
4. **Implement thread management:**
   - To reply in a thread, include `thread_ts` in the `chat.postMessage` payload:
     ```json
     { "channel": "<channel_id>", "text": "...", "thread_ts": "<parent_ts>" }
     ```
   - To also broadcast to the channel, add `reply_broadcast: true`.
   - Detect thread context from incoming messages by checking `thread_ts` and include it in `IncomingMessage` metadata.
   - Provide a method `get_thread_replies(&self, channel: &str, thread_ts: &str) -> Result<Vec<Message>>` using `conversations.replies`.
5. **Implement reaction handling:**
   - Add `add_reaction(&self, channel: &str, timestamp: &str, emoji: &str) -> Result<()>` using `reactions.add`.
   - Add `remove_reaction(&self, ...) -> Result<()>` using `reactions.remove`.
   - Listen for `reaction_added` and `reaction_removed` events in the Socket Mode handler.
6. **Implement Block Kit support:**
   - Create a builder module `blocks.rs` that constructs Slack Block Kit JSON structures.
   - Support block types: `section`, `divider`, `header`, `context`, `actions`, `image`.
   - Support element types: `plain_text`, `mrkdwn`, `button`, `static_select`.
   - Convert `OutgoingMessage` rich content into Block Kit `blocks` array.
   - Include `blocks` in the `chat.postMessage` payload alongside `text` (text serves as fallback).
   - Example:
     ```json
     {
       "channel": "<channel_id>",
       "text": "Fallback text",
       "blocks": [
         { "type": "header", "text": { "type": "plain_text", "text": "Result" } },
         { "type": "section", "text": { "type": "mrkdwn", "text": "*Details:* ..." } }
       ]
     }
     ```
7. **Implement channel/user discovery:**
   - Add `list_channels(&self) -> Result<Vec<ChannelInfo>>` using `conversations.list`.
   - Add `get_user_info(&self, user_id: &str) -> Result<UserInfo>` using `users.info`.
   - Cache results to reduce API calls; invalidate cache periodically or on relevant events.
8. **Implement message editing and deletion:**
   - Add `edit_message(&self, channel: &str, ts: &str, new_text: &str) -> Result<()>` using `chat.update`.
   - Add `delete_message(&self, channel: &str, ts: &str) -> Result<()>` using `chat.delete`.
   - Store sent message timestamps to enable later editing/deletion.
9. **Wire into the `ChannelPlugin` trait:**
   - Implement the `send` method on `ChannelPlugin` to delegate to the appropriate sending method based on the `OutgoingMessage` type.
   - Map `OutgoingMessage` fields to the Slack-specific API calls.
10. **Add tests:**
    - Test mrkdwn formatting conversion from common markdown.
    - Test Block Kit JSON construction for various block types.
    - Test thread reply payload with `thread_ts`.
    - Test file upload payload construction.
    - Test channel/user discovery response parsing.
    - Test message edit and delete payload construction.

## Dependencies
- Issue 103 (Slack channel connection and message receiving)

## Acceptance Criteria
- [x] Text messages are sent with Slack mrkdwn formatting
- [x] Long messages are automatically split if necessary
- [x] Files can be sent and received via the Slack files API
- [x] Typing indicators or "thinking" messages are displayed while generating responses
- [x] Thread replies use `thread_ts` correctly; thread context is detected in incoming messages
- [x] Reactions can be added and removed; reaction events are received
- [x] Block Kit messages are constructed and sent for rich structured responses
- [x] Channel and user discovery returns accurate information
- [x] Messages can be edited and deleted using timestamps
- [x] All sending methods handle Slack API errors gracefully (rate limits, invalid tokens, channel not found, etc.)
- [x] Unit tests cover mrkdwn formatting, Block Kit construction, threads, and file handling

## Resolution

This issue was implemented by adding two key adapter traits to the `aisopod-channel-slack` crate:

### 1. OutboundAdapter Implementation

Implemented the `OutboundAdapter` trait with two methods:
- `send_text`: Sends text messages to Slack channels using the `chat.postMessage` endpoint
- `send_media`: Uploads media files to Slack using the `files.uploadV2` endpoint

The implementation:
- Converts `OutgoingMessage` to the appropriate Slack API calls
- Handles message splitting for long messages
- Supports Block Kit formatting via the existing `blocks.rs` module
- Properly handles Slack API errors with detailed error messages

### 2. ChannelConfigAdapter Implementation

Created a new `SlackConfigAdapter` struct that implements `ChannelConfigAdapter`:
- `list_accounts`: Returns all configured account IDs
- `resolve_account`: Returns an `AccountSnapshot` for a given account ID
- `enable_account`: Enables an account (no-op since accounts are always enabled)
- `disable_account`: Disables an account (no-op)
- `delete_account`: Removes an account from the configuration

The adapter uses `Arc<RwLock<Vec<SlackChannelWithConnection>>>` for thread-safe concurrent access.

### 3. Integration with ChannelPlugin

Updated the `SlackChannel` struct to include a `config_adapter` field and modified the `ChannelPlugin::config()` method to return the proper `ChannelConfigAdapter`.

### 4. Socket Mode Connection Enhancement

Added a public `client()` getter method to `SlackSocketModeConnection` to allow access to the `SlackClientHandle` for file uploads and other API calls.

### Changes Made

**Modified Files:**
- `crates/aisopod-channel-slack/src/lib.rs`: Added `OutboundAdapter` and `ChannelConfigAdapter` implementations
- `crates/aisopod-channel-slack/src/socket_mode.rs`: Added `client()` getter method

**New Files:**
- `crates/aisopod-channel-slack/tests/test_outbound.rs`: Comprehensive tests for the new adapters

**Documentation:**
- `docs/learnings/104-implement-slack-channel-sending-and-features.md`: Implementation documentation

### Test Results

All tests pass successfully:
- **42 existing unit tests** in `src/` modules: All passing
- **5 new tests** in `tests/test_outbound.rs`: All passing
- **Full project build**: Success with `RUSTFLAGS=-Awarnings cargo build`
- **Cargo doc tests**: All passing

### Key Implementation Details

1. **Adapter Pattern**: Used the established adapter pattern from `aisopod-channel` for clean separation of concerns
2. **Thread Safety**: Implemented thread-safe account management using `Arc<RwLock<T>>`
3. **Error Handling**: Consistent error handling using `anyhow::Result` for all adapter methods
4. **API Integration**: Leveraged existing Slack API integration in `send.rs`, `media.rs`, and `features.rs`

---
*Created: 2026-02-15*
*Resolved: 2026-02-23*
