# Issue 102: Implement WhatsApp Channel — Message Sending and Features

## Summary
Extend the `aisopod-channel-whatsapp` crate with full message sending capabilities, media support, typing indicators, reply handling, read receipts, and allowed number filtering.

## Location
- Crate: `aisopod-channel-whatsapp`
- File: `crates/aisopod-channel-whatsapp/src/send.rs`, `crates/aisopod-channel-whatsapp/src/media.rs`, `crates/aisopod-channel-whatsapp/src/features.rs`

## Current Behavior
After Issue 101, the WhatsApp channel can authenticate and receive messages via webhook but cannot send responses, handle media, or use advanced WhatsApp features.

## Expected Behavior
The WhatsApp channel supports sending text messages, sending and receiving media (photos, documents, audio, video, stickers), displaying typing indicators ("recording" state), replying to specific messages, sending read receipts, and filtering messages by allowed phone numbers.

## Impact
This completes the WhatsApp integration, making it a fully functional two-way communication channel. Users can interact with aisopod agents through WhatsApp with media support and delivery feedback.

## Suggested Implementation
1. **Implement text message sending:**
   - Add a `send_message(&self, to: &str, text: &str, options: SendOptions) -> Result<MessageId>` method.
   - Use the WhatsApp Business API `POST /v17.0/{phone_number_id}/messages` endpoint.
   - Build the JSON payload:
     ```json
     {
       "messaging_product": "whatsapp",
       "to": "<recipient_phone>",
       "type": "text",
       "text": { "body": "<message_text>" }
     }
     ```
   - Include the API token in the `Authorization: Bearer` header.
   - Handle WhatsApp's message length limit by splitting long messages if needed.
2. **Implement media sending:**
   - For photos: set `type` to `"image"` and include `image.link` or upload via the media API.
   - For documents: set `type` to `"document"` with `document.link`, `document.filename`, and optional `document.caption`.
   - For audio: set `type` to `"audio"` with `audio.link`.
   - For video: set `type` to `"video"` with `video.link` and optional `video.caption`.
   - For stickers: set `type` to `"sticker"` with `sticker.link`.
   - Implement media upload via `POST /v17.0/{phone_number_id}/media` for local files, then reference the returned media ID.
3. **Implement media receiving:**
   - When an incoming webhook message has type `image`, `document`, `audio`, `video`, or `sticker`, extract the media ID.
   - Download the media using `GET /v17.0/{media_id}` to get the URL, then fetch the binary content.
   - Attach the downloaded media to the `IncomingMessage` as a media attachment with the appropriate `MediaType`.
4. **Implement typing indicators:**
   - WhatsApp does not have a direct "typing" API, but the Business API supports "recording" status.
   - Send a `POST /v17.0/{phone_number_id}/messages` with `type: "reaction"` or use status updates to indicate activity.
   - As an alternative, mark messages as "read" promptly to signal acknowledgment.
5. **Implement reply-to-message:**
   - When `SendOptions` includes a `reply_to_message_id`, add `context.message_id` to the JSON payload:
     ```json
     {
       "context": { "message_id": "<original_message_id>" }
     }
     ```
   - For incoming messages, extract `context.id` (the replied-to message ID) and include it in `IncomingMessage`.
6. **Implement read receipts:**
   - Send read receipts by calling `POST /v17.0/{phone_number_id}/messages` with:
     ```json
     {
       "messaging_product": "whatsapp",
       "status": "read",
       "message_id": "<message_id>"
     }
     ```
   - Send read receipts automatically when a message is received and processing begins.
7. **Implement allowed number filtering:**
   - On each incoming message, check the sender's phone number against `config.allowed_numbers`.
   - If `allowed_numbers` is `Some` and the sender is not in the list, silently drop the message.
   - If `allowed_numbers` is `None`, allow all numbers (open access).
   - Log filtered messages at `debug` level for troubleshooting.
8. **Wire into the `ChannelPlugin` trait:**
   - Implement the `send` method on `ChannelPlugin` to delegate to the appropriate sending method based on the `OutgoingMessage` type.
   - Map `OutgoingMessage` fields to the WhatsApp-specific API calls.
9. **Add tests:**
   - Test text message payload construction.
   - Test media upload and download URL generation.
   - Test reply-to-message payload with `context.message_id`.
   - Test read receipt payload construction.
   - Test allowed number filtering with matching, non-matching, and no-filter scenarios.

## Dependencies
- Issue 101 (WhatsApp channel connection and message receiving)

## Acceptance Criteria
- [ ] Text messages are sent via WhatsApp Business API
- [ ] Photos, documents, audio, video, and stickers can be sent and received
- [ ] Media is uploaded and downloaded correctly via the media API
- [ ] Typing/recording indicators are sent to signal activity
- [ ] Reply-to-message includes the context reference in outgoing messages
- [ ] Read receipts are sent when messages are received
- [ ] Allowed number filtering correctly blocks unauthorized senders
- [ ] All sending methods handle WhatsApp API errors gracefully (rate limits, invalid tokens, etc.)
- [ ] Unit tests cover payload construction, media handling, and number filtering

---
*Created: 2026-02-15*
