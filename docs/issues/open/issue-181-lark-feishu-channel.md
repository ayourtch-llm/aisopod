# Issue 181: Implement Lark/Feishu Channel

## Summary
Implement a Lark (Feishu) channel for aisopod using the Lark Open Platform API. This enables the bot to send and receive messages in Lark groups and direct messages, support rich message cards, handle event subscriptions, and manage app credentials.

## Location
- Crate: `aisopod-channel-lark`
- File: `crates/aisopod-channel-lark/src/lib.rs`

## Current Behavior
No Lark/Feishu channel exists. The channel abstraction traits are defined, but Lark Open Platform integration is not implemented.

## Expected Behavior
After implementation:
- aisopod connects to the Lark Open Platform API.
- Group and DM messaging is supported.
- Rich message cards can be sent.
- Event subscription webhook handles incoming events.
- App credentials (App ID / App Secret) are managed for token refresh.

## Impact
Enables aisopod to work with Lark/Feishu, a popular enterprise collaboration platform used extensively in Asia-Pacific markets and by ByteDance ecosystem companies.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-lark/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── api.rs
       ├── cards.rs
       ├── events.rs
       └── auth.rs
   ```

2. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct LarkConfig {
       /// App ID from Lark Developer Console
       pub app_id: String,
       /// App Secret from Lark Developer Console
       pub app_secret: String,
       /// Verification token for event subscriptions
       pub verification_token: String,
       /// Encrypt key for event encryption (optional)
       pub encrypt_key: Option<String>,
       /// Webhook port for event subscriptions
       pub webhook_port: u16,
       /// Use Feishu domain instead of Lark (for China region)
       pub use_feishu: bool,
   }
   ```

3. **Authentication and token management** in `auth.rs`:
   ```rust
   pub struct LarkAuth {
       app_id: String,
       app_secret: String,
       base_url: String, // https://open.larksuite.com or https://open.feishu.cn
       tenant_access_token: Option<String>,
       token_expiry: Option<std::time::Instant>,
   }

   impl LarkAuth {
       pub async fn get_tenant_access_token(&mut self) -> Result<String, AuthError> {
           if self.is_token_valid() {
               return Ok(self.tenant_access_token.clone().unwrap());
           }
           // POST {base_url}/open-apis/auth/v3/tenant_access_token/internal
           // Body: { "app_id": "...", "app_secret": "..." }
           // Response: { "tenant_access_token": "...", "expire": 7200 }
           todo!()
       }

       fn base_url(use_feishu: bool) -> String {
           if use_feishu {
               "https://open.feishu.cn".into()
           } else {
               "https://open.larksuite.com".into()
           }
       }
   }
   ```

4. **API client** in `api.rs`:
   ```rust
   pub struct LarkApi {
       auth: LarkAuth,
       http: reqwest::Client,
       base_url: String,
   }

   impl LarkApi {
       pub async fn send_message(
           &mut self,
           receive_id: &str,
           receive_id_type: &str, // "open_id", "chat_id", "user_id", "email"
           msg_type: &str,
           content: &str,
       ) -> Result<(), ApiError> {
           let token = self.auth.get_tenant_access_token().await?;
           let url = format!(
               "{}/open-apis/im/v1/messages?receive_id_type={}",
               self.base_url, receive_id_type
           );
           self.http.post(&url)
               .bearer_auth(&token)
               .json(&serde_json::json!({
                   "receive_id": receive_id,
                   "msg_type": msg_type,
                   "content": content
               }))
               .send()
               .await?;
           Ok(())
       }

       pub async fn send_text(&mut self, chat_id: &str, text: &str) -> Result<(), ApiError> {
           let content = serde_json::json!({ "text": text }).to_string();
           self.send_message(chat_id, "chat_id", "text", &content).await
       }

       pub async fn send_card(&mut self, chat_id: &str, card: serde_json::Value) -> Result<(), ApiError> {
           let content = card.to_string();
           self.send_message(chat_id, "chat_id", "interactive", &content).await
       }
   }
   ```

5. **Rich message cards** in `cards.rs`:
   ```rust
   use serde::Serialize;

   #[derive(Debug, Serialize)]
   pub struct MessageCard {
       pub config: CardConfig,
       pub header: CardHeader,
       pub elements: Vec<serde_json::Value>,
   }

   #[derive(Debug, Serialize)]
   pub struct CardConfig {
       pub wide_screen_mode: bool,
   }

   #[derive(Debug, Serialize)]
   pub struct CardHeader {
       pub title: CardText,
       pub template: Option<String>, // color: "blue", "green", "red", etc.
   }

   #[derive(Debug, Serialize)]
   pub struct CardText {
       pub tag: String, // "plain_text" or "lark_md"
       pub content: String,
   }

   impl MessageCard {
       pub fn simple(title: &str, body: &str) -> Self {
           Self {
               config: CardConfig { wide_screen_mode: true },
               header: CardHeader {
                   title: CardText { tag: "plain_text".into(), content: title.into() },
                   template: Some("blue".into()),
               },
               elements: vec![serde_json::json!({
                   "tag": "div",
                   "text": { "tag": "lark_md", "content": body }
               })],
           }
       }
   }
   ```

6. **Event subscription handler** in `events.rs`:
   ```rust
   use axum::{Router, Json, extract::State};

   pub fn lark_router(state: AppState) -> Router {
       Router::new()
           .route("/lark/events", axum::routing::post(handle_event))
           .with_state(state)
   }

   async fn handle_event(
       State(state): State<AppState>,
       Json(payload): Json<serde_json::Value>,
   ) -> impl axum::response::IntoResponse {
       // Handle URL verification challenge:
       // If payload has "challenge", return { "challenge": value }
       // Handle event callback:
       // Verify token, decrypt if needed
       // Parse event type (im.message.receive_v1, etc.)
       // Convert to InboundMessage
       todo!()
   }
   ```

7. **ChannelPlugin implementation** in `channel.rs` — authenticate with app credentials, start event subscription webhook, implement send/receive.

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [ ] Lark Open Platform API connection works
- [ ] Tenant access token management and refresh works
- [ ] Group messaging (send and receive) works
- [ ] DM messaging works
- [ ] Rich message cards render correctly
- [ ] Event subscription webhook handles incoming messages
- [ ] Feishu domain support works for China region
- [ ] Unit tests for auth, API client, and card construction
- [ ] Integration test with mocked Lark API

---
*Created: 2026-02-15*
