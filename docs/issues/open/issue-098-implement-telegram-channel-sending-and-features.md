# Issue 098: Implement Telegram Channel — Message Sending and Features

## Summary
Extend the `aisopod-channel-telegram` crate with full message sending capabilities, media support, typing indicators, reply handling, group mention detection, message editing/deletion, and multiple bot account support.

## Location
- Crate: `aisopod-channel-telegram`
- File: `crates/aisopod-channel-telegram/src/send.rs`, `crates/aisopod-channel-telegram/src/media.rs`, `crates/aisopod-channel-telegram/src/features.rs`

## Current Behavior
After Issue 097, the Telegram channel can connect and receive messages but cannot send responses, handle media, or use advanced Telegram features.

## Expected Behavior
The Telegram channel supports sending text messages with Markdown formatting, sending and receiving media (photos, documents, audio, video), displaying typing indicators, replying to specific messages, detecting @botname mentions in groups, editing and deleting messages, and managing multiple bot accounts.

## Impact
This completes the Telegram integration, making it a fully functional two-way communication channel. Users can interact with aisopod agents through Telegram with rich media and formatting support.

## Suggested Implementation
1. **Implement text message sending:**
   - Add a `send_message(&self, chat_id: ChatId, text: &str, options: SendOptions) -> Result<MessageId>` method.
   - Use `teloxide::Bot::send_message` with the configured `parse_mode` (MarkdownV2 or HTML).
   - Handle Telegram's message length limit (4096 characters) by splitting long messages into chunks.
   - Preserve formatting across chunks by tracking open/close Markdown delimiters.
2. **Implement media sending and receiving:**
   - Add methods: `send_photo`, `send_document`, `send_audio`, `send_video`.
   - For each, accept the media as bytes or a URL and use the corresponding `teloxide` method.
   - For incoming media, extract the file ID from the Telegram message, download via `getFile`, and attach to the `IncomingMessage` as a media attachment.
   - Map Telegram media types to the shared `MediaType` enum from Issue 091.
3. **Implement typing indicators:**
   - Add a `send_typing(&self, chat_id: ChatId) -> Result<()>` method.
   - Use `teloxide::Bot::send_chat_action(ChatAction::Typing)`.
   - Start a background task that re-sends the typing indicator every 4 seconds (Telegram's indicator expires after 5 seconds) until the response is ready.
4. **Implement reply-to-message:**
   - When `SendOptions` includes a `reply_to_message_id`, use `reply_to_message_id` in the Telegram API call.
   - For incoming messages, extract `reply_to_message` and include the referenced message ID in `IncomingMessage`.
5. **Implement group mention detection:**
   - When a message arrives in a group or supergroup, check if the message text contains `@botusername`.
   - Also check for replies to the bot's own messages as implicit mentions.
   - Use `teloxide::Bot::get_me()` to retrieve the bot's username at startup and cache it.
   - If mention is required (per config) and not detected, skip the message silently.
6. **Implement message editing and deletion:**
   - Add `edit_message(&self, chat_id: ChatId, message_id: MessageId, new_text: &str) -> Result<()>`.
   - Add `delete_message(&self, chat_id: ChatId, message_id: MessageId) -> Result<()>`.
   - Use `teloxide::Bot::edit_message_text` and `teloxide::Bot::delete_message`.
7. **Implement multiple bot account support:**
   - Change `TelegramChannel` to hold a `Vec<TelegramAccount>` where each account has its own `Bot` instance and config.
   - Route outgoing messages to the correct account based on the session's originating account.
   - Each account runs its own polling/webhook listener.
   - Provide a method to list active accounts.
8. **Wire into the `ChannelPlugin` trait:**
   - Implement the `send` method on `ChannelPlugin` to delegate to the appropriate sending method based on the `OutgoingMessage` type.
   - Map `OutgoingMessage` fields to the Telegram-specific send calls.
9. **Add tests:**
   - Test Markdown formatting and message splitting at the 4096 character boundary.
   - Test mention detection with various `@botname` placements.
   - Test media type mapping from shared types to Telegram types.
   - Test multi-account routing logic.

## Dependencies
- Issue 097 (Telegram channel connection and message receiving)

## Acceptance Criteria
- [ ] Text messages are sent with Markdown formatting
- [ ] Long messages are automatically split into chunks preserving formatting
- [ ] Photos, documents, audio, and video can be sent and received
- [ ] Typing indicator is displayed while generating responses
- [ ] Reply-to-message works for both incoming and outgoing messages
- [ ] Group mention detection correctly identifies `@botname` mentions
- [ ] Messages can be edited and deleted
- [ ] Multiple bot accounts can be configured and run simultaneously
- [ ] All sending methods handle Telegram API errors gracefully
- [ ] Unit tests cover formatting, media mapping, mention detection, and multi-account routing

---
*Created: 2026-02-15*
