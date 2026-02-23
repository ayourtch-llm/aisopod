# Issue 100: Implement Discord Channel — Message Sending and Features

## Summary
Extend the `aisopod-channel-discord` crate with full message sending capabilities, media and embed support, typing indicators, reply handling, thread management, reaction handling, guild/channel discovery, and message editing/deletion.

## Location
- Crate: `aisopod-channel-discord`
- File: `crates/aisopod-channel-discord/src/send.rs`, `crates/aisopod-channel-discord/src/media.rs`, `crates/aisopod-channel-discord/src/features.rs`, `crates/aisopod-channel-discord/src/embeds.rs`

## Current Behavior
After Issue 099, the Discord channel can connect to the gateway and receive messages but cannot send responses, handle media/embeds, or use advanced Discord features.

## Expected Behavior
The Discord channel supports sending text messages with Discord markdown, sending and receiving media (attachments, embeds), displaying typing indicators, replying to messages, creating and managing threads, handling reactions, discovering guilds and channels, building rich embed responses, and editing/deleting messages.

## Impact
This completes the Discord integration, making it a fully functional two-way communication channel with rich formatting via embeds and organizational features via threads.

## Suggested Implementation
1. **Implement text message sending:**
   - Add a `send_message(&self, channel_id: ChannelId, text: &str, options: SendOptions) -> Result<MessageId>` method.
   - Use `serenity::model::channel::ChannelId::say` or `send_message` with `CreateMessage`.
   - Handle Discord's message length limit (2000 characters) by splitting long messages.
   - Support Discord markdown (bold, italic, code blocks, spoilers, etc.).
2. **Implement media sending and receiving:**
   - For outgoing media, use `CreateMessage::add_file` or `CreateAttachment` to attach files.
   - For incoming media, extract attachment URLs from `Message.attachments` and download the content.
   - Map Discord attachment content types to the shared `MediaType` enum.
3. **Implement embed support:**
   - Create a builder that converts `OutgoingMessage` rich content into `CreateEmbed` structures.
   - Support fields: title, description, color, fields (name/value pairs), thumbnail, image, footer, author.
   - Allow multiple embeds per message (Discord supports up to 10).
   - Use embeds for structured agent responses (e.g., tool results, summaries).
4. **Implement typing indicators:**
   - Add a `send_typing(&self, channel_id: ChannelId) -> Result<()>` method.
   - Use `ChannelId::broadcast_typing` which triggers a typing indicator for 10 seconds.
   - Re-trigger periodically if the response takes longer.
5. **Implement reply-to-message:**
   - When `SendOptions` includes a `reply_to_message_id`, use `CreateMessage::reference_message` to create a reply.
   - For incoming messages, extract `message_reference` and include the referenced message ID in `IncomingMessage`.
6. **Implement thread management:**
   - Add `create_thread(&self, channel_id: ChannelId, name: &str) -> Result<ChannelId>` to start a new thread from a message.
   - Add `reply_in_thread(&self, thread_id: ChannelId, text: &str) -> Result<MessageId>` to send messages in an existing thread.
   - Detect when an incoming message is in a thread and include thread metadata in `IncomingMessage`.
7. **Implement reaction handling:**
   - Add `add_reaction(&self, channel_id: ChannelId, message_id: MessageId, emoji: ReactionType) -> Result<()>`.
   - Add `remove_reaction(&self, ...) -> Result<()>`.
   - Listen for `reaction_add` and `reaction_remove` events in the gateway handler.
8. **Implement guild/channel discovery:**
   - Add `list_guilds(&self) -> Result<Vec<GuildInfo>>` to enumerate servers the bot is in.
   - Add `list_channels(&self, guild_id: GuildId) -> Result<Vec<ChannelInfo>>` to list channels in a guild.
   - Cache guild and channel information to reduce API calls.
9. **Implement message editing and deletion:**
   - Add `edit_message(&self, channel_id: ChannelId, message_id: MessageId, new_text: &str) -> Result<()>`.
   - Add `delete_message(&self, channel_id: ChannelId, message_id: MessageId) -> Result<()>`.
   - Use `Message::edit` and `Message::delete` from serenity.
10. **Wire into the `ChannelPlugin` trait:**
    - Implement the `send` method on `ChannelPlugin` to delegate to the appropriate sending method based on the `OutgoingMessage` type.
11. **Add tests:**
    - Test Discord markdown formatting and message splitting at 2000 characters.
    - Test embed construction from structured data.
    - Test thread detection and routing.
    - Test reaction event mapping.

## Dependencies
- Issue 099 (Discord channel connection and message receiving)

## Acceptance Criteria
- [ ] Text messages are sent with Discord markdown formatting
- [ ] Long messages are automatically split into chunks
- [ ] Media attachments can be sent and received
- [ ] Rich embeds are constructed and sent for structured responses
- [ ] Typing indicator is displayed while generating responses
- [ ] Reply-to-message works for both incoming and outgoing messages
- [ ] Threads can be created, replied to, and detected in incoming messages
- [ ] Reactions can be added and removed; reaction events are received
- [ ] Guild and channel discovery returns accurate information
- [ ] Messages can be edited and deleted
- [ ] All sending methods handle Discord API errors gracefully
- [ ] Unit tests cover formatting, embeds, threads, and reactions

## Resolution

The implementation was completed with all features from the Suggested Implementation section:

1. **Text message sending** - Implemented `send_message()` with 2000 char limit chunking, Discord markdown support (bold, italic, underline, strikethrough, code, code blocks, blockquotes, spoilers), and automatic message splitting.

2. **Media sending and receiving** - Implemented via `send_media()`, `send_media_batch()`, `extract_media_from_attachments()` and `download_attachments()` functions.

3. **Embed support** - Implemented `EmbedBuilder` with fluent API for title, description, color, timestamp, footer, image, thumbnail, author, and fields. Helper functions: `build_tool_result_embed`, `build_error_embed`, `build_success_embed`, `build_info_embed`, `build_warning_embed`.

4. **Typing indicators** - Implemented `send_typing_while()` which displays a typing indicator while processing operations.

5. **Reply-to-message** - Implemented via `SendOptions::reply_to_message_id` using `CreateMessage::reference_message`.

6. **Thread management** - Implemented `create_thread()`, `reply_in_thread()`, `detect_thread_in_message()`, `get_thread_info()`.

7. **Reaction handling** - Implemented `add_reaction()`, `remove_reaction()`, `list_reactions()`, `parse_reaction_emoji()` for unicode and custom emojis.

8. **Guild/channel discovery** - Implemented `list_guilds()`, `list_channels()`, `find_channel_by_name()` using serenity cache.

9. **Message editing and deletion** - Implemented `edit_message()`, `delete_message()`, `bulk_delete_messages()`.

10. **ChannelPlugin trait integration** - The `DiscordChannel` implements `ChannelPlugin` with proper multi-account support and client management.

11. **Tests** - Comprehensive unit tests for chunk_text, markdown formatting, embed building, media validation, reaction parsing, message filtering, and configuration.

**Files Modified:**
- `crates/aisopod-channel-discord/src/lib.rs` - Main module with `DiscordChannel` and `DiscordAccount` types
- `crates/aisopod-channel-discord/src/send.rs` - Message sending with chunking and markdown
- `crates/aisopod-channel-discord/src/media.rs` - Media attachment handling
- `crates/aisopod-channel-discord/src/embeds.rs` - Rich embed builder
- `crates/aisopod-channel-discord/src/features.rs` - Typing indicators, threads, reactions, discovery
- `crates/aisopod-channel-discord/src/receive.rs` - Message receiving and normalization
- `crates/aisopod-channel-discord/src/connection.rs` - Gateway connection management
- `crates/aisopod-channel-discord/Cargo.toml` - Added `futures` dependency
- `docs/learnings/100-implement-discord-channel-sending-and-features.md` - Implementation learnings

**Verification:**
- `cargo build` passes with no errors
- `cargo test --package aisopod-channel-discord` passes: 53 passed, 2 ignored
- `RUSTFLAGS=-Awarnings` used for clean output

---
*Created: 2026-02-15*
*Resolved: 2026-02-23*
