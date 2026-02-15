# Issue 182: Implement Zalo Channel

## Summary
Implement a Zalo channel for aisopod using the Zalo Official Account (OA) API. This enables the bot to send and receive messages with Zalo users, support media attachments, handle OAuth authentication, and process webhook events.

## Location
- Crate: `aisopod-channel-zalo`
- File: `crates/aisopod-channel-zalo/src/lib.rs`

## Current Behavior
No Zalo channel exists. The channel abstraction traits are defined, but Zalo OA API integration is not implemented.

## Expected Behavior
After implementation:
- aisopod connects to the Zalo OA API.
- User messaging (send and receive) is supported.
- Media attachments (images, files) are supported.
- OAuth authentication manages access tokens.
- Webhook handles incoming events from Zalo.

## Impact
Enables aisopod to reach Zalo's user base, the dominant messaging platform in Vietnam with over 70 million users, opening aisopod to the Vietnamese market.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-zalo/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── api.rs
       ├── auth.rs
       └── webhook.rs
   ```

2. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct ZaloConfig {
       /// Zalo OA App ID
       pub app_id: String,
       /// Zalo OA App Secret
       pub app_secret: String,
       /// OAuth refresh token
       pub refresh_token: String,
       /// Webhook port
       pub webhook_port: u16,
       /// OA Secret Key (for webhook verification)
       pub oa_secret_key: String,
   }
   ```

3. **OAuth authentication** in `auth.rs`:
   ```rust
   pub struct ZaloAuth {
       app_id: String,
       app_secret: String,
       access_token: Option<String>,
       refresh_token: String,
       token_expiry: Option<std::time::Instant>,
   }

   impl ZaloAuth {
       pub async fn get_access_token(&mut self) -> Result<String, AuthError> {
           if self.is_token_valid() {
               return Ok(self.access_token.clone().unwrap());
           }
           // POST https://oauth.zaloapp.com/v4/oa/access_token
           // Body: { "app_id": "...", "grant_type": "refresh_token",
           //         "refresh_token": "..." }
           // Header: secret_key: <app_secret>
           // Response: { "access_token": "...", "refresh_token": "...", "expires_in": 3600 }
           // Update self.refresh_token with new value
           todo!()
       }
   }
   ```

4. **OA API client** in `api.rs`:
   ```rust
   pub struct ZaloApi {
       auth: ZaloAuth,
       http: reqwest::Client,
       base_url: String, // https://openapi.zalo.me/v3.0/oa
   }

   impl ZaloApi {
       pub async fn send_text_message(&mut self, user_id: &str, text: &str) -> Result<(), ApiError> {
           let token = self.auth.get_access_token().await?;
           let url = format!("{}/message/cs", self.base_url);
           self.http.post(&url)
               .bearer_auth(&token)
               .json(&serde_json::json!({
                   "recipient": { "user_id": user_id },
                   "message": {
                       "text": text
                   }
               }))
               .send()
               .await?;
           Ok(())
       }

       pub async fn send_image_message(
           &mut self,
           user_id: &str,
           image_url: &str,
       ) -> Result<(), ApiError> {
           let token = self.auth.get_access_token().await?;
           let url = format!("{}/message/cs", self.base_url);
           self.http.post(&url)
               .bearer_auth(&token)
               .json(&serde_json::json!({
                   "recipient": { "user_id": user_id },
                   "message": {
                       "attachment": {
                           "type": "template",
                           "payload": {
                               "template_type": "media",
                               "elements": [{
                                   "media_type": "image",
                                   "url": image_url
                               }]
                           }
                       }
                   }
               }))
               .send()
               .await?;
           Ok(())
       }

       pub async fn send_file_message(
           &mut self,
           user_id: &str,
           file_token: &str,
       ) -> Result<(), ApiError> {
           let token = self.auth.get_access_token().await?;
           let url = format!("{}/message/cs", self.base_url);
           self.http.post(&url)
               .bearer_auth(&token)
               .json(&serde_json::json!({
                   "recipient": { "user_id": user_id },
                   "message": {
                       "attachment": {
                           "type": "file",
                           "payload": {
                               "token": file_token
                           }
                       }
                   }
               }))
               .send()
               .await?;
           Ok(())
       }
   }
   ```

5. **Webhook handler** in `webhook.rs`:
   ```rust
   use axum::{Router, Json, extract::State, http::HeaderMap};

   pub fn zalo_router(state: AppState) -> Router {
       Router::new()
           .route("/zalo/webhook", axum::routing::post(handle_webhook))
           .with_state(state)
   }

   async fn handle_webhook(
       State(state): State<AppState>,
       headers: HeaderMap,
       Json(payload): Json<serde_json::Value>,
   ) -> impl axum::response::IntoResponse {
       // Verify webhook using OA secret key
       // Parse event_name: "user_send_text", "user_send_image", "follow", "unfollow"
       // Extract sender user_id and message content
       // Convert to InboundMessage and forward to pipeline
       todo!()
   }
   ```

6. **ChannelPlugin implementation** in `channel.rs` — authenticate, start webhook listener, implement send/receive/disconnect.

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [ ] Zalo OA API connection with OAuth works
- [ ] Access token refresh (including rotating refresh tokens) works
- [ ] Text message send and receive works
- [ ] Media (image, file) messaging works
- [ ] Webhook event handling processes incoming messages
- [ ] Webhook verification with OA secret key works
- [ ] Unit tests for auth, API client, and webhook verification
- [ ] Integration test with mocked Zalo API

---
*Created: 2026-02-15*
