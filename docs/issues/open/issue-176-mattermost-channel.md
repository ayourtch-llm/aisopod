# Issue 176: Implement Mattermost Channel

## Summary
Implement a Mattermost channel for aisopod using the Mattermost REST API and WebSocket event streaming. This enables the bot to send and receive messages in Mattermost channels and direct messages, with bot account support and self-hosted server configuration.

## Location
- Crate: `aisopod-channel-mattermost`
- File: `crates/aisopod-channel-mattermost/src/lib.rs`

## Current Behavior
No Mattermost channel exists. The channel abstraction traits are defined, but Mattermost is not integrated.

## Expected Behavior
After implementation:
- aisopod connects to a Mattermost server via REST API and WebSocket.
- Channel and DM messaging is supported.
- Real-time event streaming via WebSocket works.
- Bot account authentication is handled.
- Self-hosted server URLs are configurable.

## Impact
Enables aisopod to work with Mattermost, a popular open-source, self-hosted team communication platform used by organizations that need data sovereignty and customization.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-mattermost/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── api.rs
       ├── websocket.rs
       └── auth.rs
   ```

2. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct MattermostConfig {
       /// Mattermost server URL (e.g., "https://mattermost.example.com")
       pub server_url: String,
       /// Authentication method
       pub auth: MattermostAuth,
       /// Channels to join (by channel name or ID)
       pub channels: Vec<String>,
       /// Team name or ID
       pub team: String,
   }

   #[derive(Debug, Deserialize)]
   #[serde(tag = "type")]
   pub enum MattermostAuth {
       #[serde(rename = "bot")]
       BotToken { token: String },
       #[serde(rename = "personal")]
       PersonalToken { token: String },
       #[serde(rename = "password")]
       Password { username: String, password: String },
   }
   ```

3. **REST API client** in `api.rs`:
   ```rust
   pub struct MattermostApi {
       base_url: String,
       token: String,
       http: reqwest::Client,
   }

   impl MattermostApi {
       pub async fn create_post(&self, channel_id: &str, message: &str) -> Result<Post, ApiError> {
           let url = format!("{}/api/v4/posts", self.base_url);
           let resp = self.http.post(&url)
               .bearer_auth(&self.token)
               .json(&serde_json::json!({
                   "channel_id": channel_id,
                   "message": message
               }))
               .send()
               .await?;
           Ok(resp.json().await?)
       }

       pub async fn get_channel_by_name(&self, team_id: &str, name: &str) -> Result<Channel, ApiError> {
           let url = format!("{}/api/v4/teams/{}/channels/name/{}", self.base_url, team_id, name);
           let resp = self.http.get(&url)
               .bearer_auth(&self.token)
               .send()
               .await?;
           Ok(resp.json().await?)
       }

       pub async fn create_direct_channel(&self, user_ids: [&str; 2]) -> Result<Channel, ApiError> {
           let url = format!("{}/api/v4/channels/direct", self.base_url);
           let resp = self.http.post(&url)
               .bearer_auth(&self.token)
               .json(&user_ids)
               .send()
               .await?;
           Ok(resp.json().await?)
       }
   }
   ```

4. **WebSocket event streaming** in `websocket.rs`:
   ```rust
   use tokio_tungstenite::tungstenite::Message;
   use futures::{StreamExt, SinkExt};

   pub struct MattermostWebSocket {
       // WebSocket connection handle
   }

   impl MattermostWebSocket {
       pub async fn connect(server_url: &str, token: &str) -> Result<Self, Box<dyn std::error::Error>> {
           let ws_url = server_url
               .replace("https://", "wss://")
               .replace("http://", "ws://");
           let url = format!("{}/api/v4/websocket", ws_url);
           // Connect and send auth challenge response
           // { "seq": 1, "action": "authentication_challenge", "data": { "token": "..." } }
           todo!()
       }

       pub async fn next_event(&mut self) -> Result<MattermostEvent, Box<dyn std::error::Error>> {
           // Read next WebSocket frame, parse as event
           // Filter for "posted" events containing new messages
           todo!()
       }
   }

   #[derive(Debug)]
   pub struct MattermostEvent {
       pub event: String,    // e.g., "posted"
       pub data: serde_json::Value,
   }
   ```

5. **ChannelPlugin implementation** in `channel.rs`:
   ```rust
   #[async_trait]
   impl ChannelPlugin for MattermostChannel {
       async fn connect(&mut self) -> Result<(), ChannelError> {
           // Authenticate, resolve channels, start WebSocket
           todo!()
       }

       async fn send(&self, msg: OutboundMessage) -> Result<(), ChannelError> {
           // Create post via REST API
           todo!()
       }

       async fn receive(&mut self) -> Result<InboundMessage, ChannelError> {
           // Read from WebSocket event stream
           todo!()
       }

       async fn disconnect(&mut self) -> Result<(), ChannelError> {
           // Close WebSocket, clean up
           todo!()
       }
   }
   ```

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [ ] Mattermost server connection with bot token works
- [ ] Channel messaging (send and receive) works
- [ ] DM messaging works
- [ ] WebSocket event streaming delivers real-time messages
- [ ] Self-hosted server URL configuration works
- [ ] Bot account and personal token auth both work
- [ ] Unit tests for API client, WebSocket parsing, and auth
- [ ] Integration test with mocked Mattermost API

---
*Created: 2026-02-15*
