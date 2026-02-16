# Issue 173: Implement Microsoft Teams Channel

## Summary
Implement a Microsoft Teams channel for aisopod using the Microsoft Bot Framework. This enables the bot to send and receive messages in Teams channels and direct messages, with support for Adaptive Cards, Azure AD authentication, and webhook/connector integration.

## Location
- Crate: `aisopod-channel-msteams`
- File: `crates/aisopod-channel-msteams/src/lib.rs`

## Current Behavior
No Microsoft Teams channel exists. The channel abstraction layer provides the traits, but Teams is not integrated.

## Expected Behavior
After implementation:
- aisopod connects to Microsoft Teams via the Bot Framework.
- Channel and DM messaging is supported.
- Adaptive Cards are used for rich content.
- Azure AD authentication is handled.
- Incoming webhooks and connectors are supported as an alternative.

## Impact
Enables aisopod to work in Microsoft 365 / Teams environments, reaching enterprise teams that rely on Teams for daily communication and collaboration.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-msteams/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── auth.rs
       ├── botframework.rs
       ├── adaptive_cards.rs
       └── webhook.rs
   ```

2. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct TeamsConfig {
       /// Azure Bot registration App ID
       pub app_id: String,
       /// Azure Bot registration App Password
       pub app_password: String,
       /// Azure AD Tenant ID (for single-tenant apps)
       pub tenant_id: Option<String>,
       /// Webhook endpoint port
       pub webhook_port: u16,
       /// Messaging endpoint path (e.g., "/api/messages")
       pub messaging_endpoint: String,
   }
   ```

3. **Azure AD authentication** in `auth.rs`:
   ```rust
   pub struct BotFrameworkAuth {
       app_id: String,
       app_password: String,
       tenant_id: Option<String>,
       access_token: Option<String>,
       token_expiry: Option<std::time::Instant>,
   }

   impl BotFrameworkAuth {
       pub async fn get_token(&mut self) -> Result<String, AuthError> {
           if self.is_token_valid() {
               return Ok(self.access_token.clone().unwrap());
           }
           // POST to https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token
           // with grant_type=client_credentials, client_id, client_secret,
           // scope=https://api.botframework.com/.default
           todo!()
       }
   }
   ```

4. **Bot Framework client** in `botframework.rs`:
   ```rust
   pub struct BotFrameworkClient {
       auth: BotFrameworkAuth,
       http: reqwest::Client,
   }

   impl BotFrameworkClient {
       pub async fn send_activity(
           &mut self,
           service_url: &str,
           conversation_id: &str,
           activity: Activity,
       ) -> Result<(), BotError> {
           let token = self.auth.get_token().await?;
           let url = format!(
               "{}/v3/conversations/{}/activities",
               service_url, conversation_id
           );
           self.http.post(&url)
               .bearer_auth(&token)
               .json(&activity)
               .send()
               .await?;
           Ok(())
       }
   }

   #[derive(serde::Serialize)]
   pub struct Activity {
       #[serde(rename = "type")]
       pub activity_type: String, // "message"
       pub text: Option<String>,
       pub attachments: Option<Vec<Attachment>>,
   }
   ```

5. **Adaptive Cards** in `adaptive_cards.rs`:
   ```rust
   use serde::Serialize;

   #[derive(Debug, Serialize)]
   pub struct AdaptiveCard {
       #[serde(rename = "$schema")]
       pub schema: String, // "http://adaptivecards.io/schemas/adaptive-card.json"
       #[serde(rename = "type")]
       pub card_type: String, // "AdaptiveCard"
       pub version: String,   // "1.4"
       pub body: Vec<serde_json::Value>,
       pub actions: Option<Vec<serde_json::Value>>,
   }

   impl AdaptiveCard {
       pub fn simple_text(text: &str) -> Self {
           Self {
               schema: "http://adaptivecards.io/schemas/adaptive-card.json".into(),
               card_type: "AdaptiveCard".into(),
               version: "1.4".into(),
               body: vec![serde_json::json!({
                   "type": "TextBlock",
                   "text": text,
                   "wrap": true
               })],
               actions: None,
           }
       }
   }
   ```

6. **Webhook handler** in `webhook.rs`:
   ```rust
   use axum::{Router, Json, extract::State};

   pub fn teams_router(state: AppState) -> Router {
       Router::new()
           .route("/api/messages", axum::routing::post(handle_activity))
           .with_state(state)
   }

   async fn handle_activity(
       State(state): State<AppState>,
       Json(activity): Json<serde_json::Value>,
   ) -> impl axum::response::IntoResponse {
       // Validate Bot Framework JWT token
       // Parse activity type (message, conversationUpdate, etc.)
       // Convert to InboundMessage and forward
       todo!()
   }
   ```

7. **ChannelPlugin implementation** in `channel.rs` — authenticate with Azure AD, start webhook listener, implement send/receive using Bot Framework activities.

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [ ] Azure AD authentication and token refresh work
- [ ] Messages can be sent to and received from Teams channels
- [ ] DM messaging works
- [ ] Adaptive Cards render correctly in Teams
- [ ] Webhook endpoint processes incoming Bot Framework activities
- [ ] Bot Framework JWT token validation works
- [ ] Webhook/connector alternative path works
- [ ] Unit tests for auth, activity parsing, and Adaptive Card construction
- [ ] Integration test with mocked Bot Framework API

---
*Created: 2026-02-15*
