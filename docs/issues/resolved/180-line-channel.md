# Issue 180: Implement LINE Channel

## Summary
Implement a LINE channel for aisopod using the LINE Messaging API. This enables the bot to send and receive messages with LINE users and groups, support rich Flex Messages, handle webhooks for incoming events, and manage channel access tokens.

## Location
- Crate: `aisopod-channel-line`
- File: `crates/aisopod-channel-line/src/lib.rs`

## Current Behavior
No LINE channel exists. The channel abstraction traits are defined, but LINE Messaging API integration is not implemented.

## Expected Behavior
After implementation:
- aisopod connects to the LINE Messaging API.
- User and group messaging is supported.
- Rich Flex Messages can be sent.
- Webhook handling processes incoming events.
- Channel access tokens are managed and refreshed.

## Impact
Enables aisopod to reach LINE's massive user base, particularly in Japan, Thailand, Taiwan, and Indonesia, where LINE is the dominant messaging platform.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-line/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── api.rs
       ├── flex.rs
       ├── webhook.rs
       └── auth.rs
   ```

2. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct LineConfig {
       /// Channel access token (long-lived or stateless)
       pub channel_access_token: String,
       /// Channel secret (for webhook signature verification)
       pub channel_secret: String,
       /// Webhook port
       pub webhook_port: u16,
       /// Webhook endpoint path
       pub webhook_path: String,
   }
   ```

3. **Messaging API client** in `api.rs`:
   ```rust
   pub struct LineApi {
       token: String,
       http: reqwest::Client,
       base_url: String, // https://api.line.me/v2/bot
   }

   impl LineApi {
       pub async fn reply_message(&self, reply_token: &str, messages: Vec<LineMessage>) -> Result<(), ApiError> {
           let url = format!("{}/message/reply", self.base_url);
           self.http.post(&url)
               .bearer_auth(&self.token)
               .json(&serde_json::json!({
                   "replyToken": reply_token,
                   "messages": messages
               }))
               .send()
               .await?;
           Ok(())
       }

       pub async fn push_message(&self, to: &str, messages: Vec<LineMessage>) -> Result<(), ApiError> {
           let url = format!("{}/message/push", self.base_url);
           self.http.post(&url)
               .bearer_auth(&self.token)
               .json(&serde_json::json!({
                   "to": to,
                   "messages": messages
               }))
               .send()
               .await?;
           Ok(())
       }
   }

   #[derive(Debug, serde::Serialize)]
   #[serde(tag = "type")]
   pub enum LineMessage {
       #[serde(rename = "text")]
       Text { text: String },
       #[serde(rename = "image")]
       Image {
           #[serde(rename = "originalContentUrl")]
           original_content_url: String,
           #[serde(rename = "previewImageUrl")]
           preview_image_url: String,
       },
       #[serde(rename = "flex")]
       Flex {
           #[serde(rename = "altText")]
           alt_text: String,
           contents: serde_json::Value,
       },
   }
   ```

4. **Flex Message builder** in `flex.rs`:
   ```rust
   use serde::Serialize;

   #[derive(Debug, Serialize)]
   pub struct FlexContainer {
       #[serde(rename = "type")]
       pub container_type: String, // "bubble" or "carousel"
       pub body: Option<FlexComponent>,
       pub header: Option<FlexComponent>,
       pub footer: Option<FlexComponent>,
   }

   #[derive(Debug, Serialize)]
   pub struct FlexComponent {
       #[serde(rename = "type")]
       pub component_type: String,
       #[serde(flatten)]
       pub properties: serde_json::Value,
   }

   impl FlexContainer {
       pub fn simple_bubble(text: &str) -> Self {
           Self {
               container_type: "bubble".into(),
               body: Some(FlexComponent {
                   component_type: "box".into(),
                   properties: serde_json::json!({
                       "layout": "vertical",
                       "contents": [{
                           "type": "text",
                           "text": text,
                           "wrap": true
                       }]
                   }),
               }),
               header: None,
               footer: None,
           }
       }
   }
   ```

5. **Webhook handler** in `webhook.rs`:
   ```rust
   use axum::{Router, Json, extract::State, http::HeaderMap};
   use hmac::{Hmac, Mac};
   use sha2::Sha256;

   pub fn line_router(state: AppState) -> Router {
       Router::new()
           .route("/line/webhook", axum::routing::post(handle_webhook))
           .with_state(state)
   }

   async fn handle_webhook(
       State(state): State<AppState>,
       headers: HeaderMap,
       body: String,
   ) -> impl axum::response::IntoResponse {
       // Verify X-Line-Signature header using channel secret
       // Parse webhook events (message, follow, unfollow, join, leave, etc.)
       // Convert message events to InboundMessage
       todo!()
   }

   fn verify_signature(channel_secret: &str, body: &str, signature: &str) -> bool {
       let mut mac = Hmac::<Sha256>::new_from_slice(channel_secret.as_bytes()).unwrap();
       mac.update(body.as_bytes());
       let expected = base64::encode(mac.finalize().into_bytes());
       expected == signature
   }
   ```

6. **Channel access token management** in `auth.rs`:
   ```rust
   pub async fn issue_stateless_token(
       client_id: &str,
       client_secret: &str,
   ) -> Result<String, AuthError> {
       // POST https://api.line.me/oauth2/v3/token
       // grant_type=client_credentials
       todo!()
   }
   ```

7. **ChannelPlugin implementation** in `channel.rs` — initialize API client, start webhook listener, implement send (push/reply) and receive.

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [ ] LINE Messaging API connection works
- [ ] User messaging (push and reply) works
- [ ] Group messaging works
- [ ] Flex Messages render correctly
- [ ] Webhook signature verification works
- [ ] Webhook event parsing handles message, follow, and other events
- [ ] Channel access token management works
- [ ] Unit tests for API client, Flex builder, and webhook verification
- [ ] Integration test with mocked LINE API

---
*Created: 2026-02-15*
