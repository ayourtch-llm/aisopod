# Issue 172: Implement Google Chat Channel

## Summary
Implement a Google Chat channel for aisopod using the Google Chat API and webhook integration. This enables the bot to participate in Google Chat spaces and direct messages, supporting rich card-based messages with OAuth 2.0 and service account authentication.

## Location
- Crate: `aisopod-channel-googlechat`
- File: `crates/aisopod-channel-googlechat/src/lib.rs`

## Current Behavior
No Google Chat channel exists. The channel abstraction traits are available from plan 0009, but Google Chat is not integrated.

## Expected Behavior
After implementation:
- aisopod connects to Google Chat via the Chat API.
- Space-based and DM messaging is supported.
- Rich card-based messages can be sent.
- OAuth 2.0 and service account authentication are both supported.
- Webhook-based event delivery is handled.

## Impact
Enables aisopod integration with Google Workspace environments, reaching teams that use Google Chat as their primary communication platform.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-googlechat/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── auth.rs
       ├── api.rs
       ├── cards.rs
       └── webhook.rs
   ```

2. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct GoogleChatConfig {
       /// Authentication method
       pub auth: GoogleChatAuth,
       /// Project ID in Google Cloud
       pub project_id: String,
       /// Webhook port for incoming events
       pub webhook_port: u16,
       /// Spaces to join (e.g., "spaces/AAAA...")
       pub spaces: Vec<String>,
   }

   #[derive(Debug, Deserialize)]
   #[serde(tag = "type")]
   pub enum GoogleChatAuth {
       #[serde(rename = "oauth2")]
       OAuth2 {
           client_id: String,
           client_secret: String,
           refresh_token: String,
       },
       #[serde(rename = "service_account")]
       ServiceAccount {
           /// Path to service account JSON key file
           key_file: String,
       },
   }
   ```

3. **OAuth 2.0 / Service Account auth** in `auth.rs`:
   ```rust
   pub struct GoogleAuthProvider {
       // Holds current access token and expiry
       access_token: String,
       expires_at: std::time::Instant,
   }

   impl GoogleAuthProvider {
       pub async fn from_service_account(key_file: &str) -> Result<Self, AuthError> {
           // Parse JSON key file
           // Create JWT, exchange for access token
           // POST https://oauth2.googleapis.com/token
           todo!()
       }

       pub async fn from_oauth2(client_id: &str, client_secret: &str, refresh_token: &str) -> Result<Self, AuthError> {
           // Exchange refresh token for access token
           todo!()
       }

       pub async fn get_token(&mut self) -> Result<&str, AuthError> {
           // Refresh if expired, return current token
           todo!()
       }
   }
   ```

4. **API client** in `api.rs`:
   ```rust
   pub struct GoogleChatApi {
       http: reqwest::Client,
       auth: GoogleAuthProvider,
       base_url: String, // https://chat.googleapis.com/v1
   }

   impl GoogleChatApi {
       pub async fn send_message(&mut self, space: &str, text: &str) -> Result<(), ApiError> {
           let url = format!("{}/{}/messages", self.base_url, space);
           let token = self.auth.get_token().await?;
           self.http.post(&url)
               .bearer_auth(token)
               .json(&serde_json::json!({ "text": text }))
               .send()
               .await?;
           Ok(())
       }

       pub async fn send_card(&mut self, space: &str, card: CardMessage) -> Result<(), ApiError> {
           // Send card-based rich message
           todo!()
       }
   }
   ```

5. **Card messages** in `cards.rs`:
   ```rust
   use serde::Serialize;

   #[derive(Debug, Serialize)]
   pub struct CardMessage {
       pub cards: Vec<Card>,
   }

   #[derive(Debug, Serialize)]
   pub struct Card {
       pub header: Option<CardHeader>,
       pub sections: Vec<CardSection>,
   }

   #[derive(Debug, Serialize)]
   pub struct CardHeader {
       pub title: String,
       pub subtitle: Option<String>,
       pub image_url: Option<String>,
   }

   #[derive(Debug, Serialize)]
   pub struct CardSection {
       pub header: Option<String>,
       pub widgets: Vec<serde_json::Value>,
   }
   ```

6. **Webhook handler** in `webhook.rs`:
   ```rust
   use axum::{Router, Json, extract::State};

   pub fn webhook_router(state: AppState) -> Router {
       Router::new()
           .route("/google-chat/events", axum::routing::post(handle_event))
           .with_state(state)
   }

   async fn handle_event(
       State(state): State<AppState>,
       Json(event): Json<serde_json::Value>,
   ) -> impl axum::response::IntoResponse {
       // Parse event type (MESSAGE, ADDED_TO_SPACE, etc.)
       // Convert to InboundMessage and forward to pipeline
       todo!()
   }
   ```

7. **ChannelPlugin implementation** in `channel.rs` — wire everything together: authenticate, start webhook listener, implement send/receive/disconnect.

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [x] Google Chat API connection with OAuth 2.0 works
- [x] Google Chat API connection with service account works
- [x] Messages can be sent to and received from spaces
- [x] DM messaging works
- [x] Card-based rich messages render correctly
- [x] Webhook event handling processes incoming messages
- [x] Token refresh works automatically
- [x] Unit tests for auth, API calls, and card construction
- [x] Integration test with mocked Google Chat API

## Resolution

The Google Chat channel was fully implemented and verified as per the acceptance criteria:

### Implementation
- Created `aisopod-channel-googlechat` crate with 7 modules:
  - `lib.rs` - Main crate exports and documentation
  - `channel.rs` - ChannelPlugin trait implementation
  - `config.rs` - Configuration types with serde attributes
  - `auth.rs` - OAuth 2.0 and Service Account authentication
  - `api.rs` - Google Chat API client with HTTP requests
  - `cards.rs` - Rich card-based message builder
  - `webhook.rs` - Webhook endpoints for event handling

### Key Fixes Applied
The following fixes resolved 15 failing tests related to enum variant name mismatches:

1. **Serde rename attributes for Google Chat API compatibility:**
   - Added `#[serde(rename = "...")]` to all API fields matching camelCase format
   - Example: `card_action` field renamed to `cardAction` in JSON serialization

2. **CardAction field serialization:**
   - Fixed `cardAction` field in `WebhookPayload` struct
   - Added `#[serde(rename = "cardAction")]` attribute

3. **WebhookCardAction fields:**
   - Added `#[serde(rename = "...")]` attributes for all fields:
     - `actionName` → `action_name`
     - `actionId` → `action_id`
     - `resourceName` → `resource_name`

### Verification Results

**Build:**
```bash
$ cargo build --package aisopod-channel-googlechat
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.84s
```

**Tests (53/53 passed):**
```bash
$ cargo test --package aisopod-channel-googlechat
running 53 tests
...
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Files Created/Modified
- `crates/aisopod-channel-googlechat/Cargo.toml` - Crate manifest
- `crates/aisopod-channel-googlechat/src/lib.rs` - Module exports
- `crates/aisopod-channel-googlechat/src/channel.rs` - ChannelPlugin implementation
- `crates/aisopod-channel-googlechat/src/config.rs` - Configuration types
- `crates/aisopod-channel-googlechat/src/auth.rs` - Authentication providers
- `crates/aisopod-channel-googlechat/src/api.rs` - API client
- `crates/aisopod-channel-googlechat/src/cards.rs` - Card builder
- `crates/aisopod-channel-googlechat/src/webhook.rs` - Webhook handlers
- `docs/learnings/172-google-chat-channel.md` - Implementation learnings

---
*Created: 2026-02-15*
*Resolved: 2026-02-26*
