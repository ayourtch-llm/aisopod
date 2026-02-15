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
- [ ] Text messages are sent with Slack mrkdwn formatting
- [ ] Long messages are automatically split if necessary
- [ ] Files can be sent and received via the Slack files API
- [ ] Typing indicators or "thinking" messages are displayed while generating responses
- [ ] Thread replies use `thread_ts` correctly; thread context is detected in incoming messages
- [ ] Reactions can be added and removed; reaction events are received
- [ ] Block Kit messages are constructed and sent for rich structured responses
- [ ] Channel and user discovery returns accurate information
- [ ] Messages can be edited and deleted using timestamps
- [ ] All sending methods handle Slack API errors gracefully (rate limits, invalid tokens, channel not found, etc.)
- [ ] Unit tests cover mrkdwn formatting, Block Kit construction, threads, and file handling

---
*Created: 2026-02-15*
