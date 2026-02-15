# Issue 177: Implement Nextcloud Talk Channel

## Summary
Implement a Nextcloud Talk channel for aisopod using the Nextcloud Talk API. This enables the bot to participate in Nextcloud Talk rooms with room-based messaging, file sharing integration, and Nextcloud authentication.

## Location
- Crate: `aisopod-channel-nextcloud`
- File: `crates/aisopod-channel-nextcloud/src/lib.rs`

## Current Behavior
No Nextcloud Talk channel exists. The channel abstraction traits are defined, but Nextcloud Talk is not integrated.

## Expected Behavior
After implementation:
- aisopod connects to a Nextcloud instance and participates in Talk rooms.
- Room-based messaging (send and receive) is supported.
- File sharing via Nextcloud's file system works.
- Nextcloud credentials or app passwords are used for authentication.

## Impact
Enables aisopod to integrate with Nextcloud Talk, a self-hosted collaboration platform popular with organizations that prioritize data ownership and run their own infrastructure.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-nextcloud/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── api.rs
       ├── polling.rs
       └── files.rs
   ```

2. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct NextcloudConfig {
       /// Nextcloud server URL (e.g., "https://cloud.example.com")
       pub server_url: String,
       /// Username
       pub username: String,
       /// App password or regular password
       pub password: String,
       /// Rooms to join (by room token)
       pub rooms: Vec<String>,
       /// Poll interval in seconds for new messages
       pub poll_interval_secs: u64,
   }
   ```

3. **Talk API client** in `api.rs`:
   ```rust
   pub struct NextcloudTalkApi {
       base_url: String,
       auth: (String, String), // (username, password) for Basic auth
       http: reqwest::Client,
   }

   impl NextcloudTalkApi {
       pub async fn send_message(&self, room_token: &str, message: &str) -> Result<(), ApiError> {
           let url = format!(
               "{}/ocs/v2.php/apps/spreed/api/v1/chat/{}",
               self.base_url, room_token
           );
           self.http.post(&url)
               .basic_auth(&self.auth.0, Some(&self.auth.1))
               .header("OCS-APIRequest", "true")
               .json(&serde_json::json!({ "message": message }))
               .send()
               .await?;
           Ok(())
       }

       pub async fn receive_messages(
           &self,
           room_token: &str,
           last_known_id: i64,
       ) -> Result<Vec<TalkMessage>, ApiError> {
           let url = format!(
               "{}/ocs/v2.php/apps/spreed/api/v1/chat/{}",
               self.base_url, room_token
           );
           let resp = self.http.get(&url)
               .basic_auth(&self.auth.0, Some(&self.auth.1))
               .header("OCS-APIRequest", "true")
               .query(&[
                   ("lookIntoFuture", "1"),
                   ("lastKnownMessageId", &last_known_id.to_string()),
               ])
               .send()
               .await?;
           // Parse OCS response envelope
           todo!()
       }

       pub async fn get_rooms(&self) -> Result<Vec<TalkRoom>, ApiError> {
           let url = format!(
               "{}/ocs/v2.php/apps/spreed/api/v4/room",
               self.base_url
           );
           let resp = self.http.get(&url)
               .basic_auth(&self.auth.0, Some(&self.auth.1))
               .header("OCS-APIRequest", "true")
               .send()
               .await?;
           todo!()
       }
   }

   #[derive(Debug, serde::Deserialize)]
   pub struct TalkMessage {
       pub id: i64,
       pub actor_id: String,
       pub message: String,
       pub timestamp: i64,
   }

   #[derive(Debug, serde::Deserialize)]
   pub struct TalkRoom {
       pub token: String,
       pub name: String,
       #[serde(rename = "type")]
       pub room_type: i32,
   }
   ```

4. **Polling for new messages** in `polling.rs`:
   ```rust
   pub struct MessagePoller {
       api: super::api::NextcloudTalkApi,
       rooms: Vec<String>,
       last_known_ids: std::collections::HashMap<String, i64>,
       poll_interval: std::time::Duration,
   }

   impl MessagePoller {
       pub async fn poll_once(&mut self) -> Result<Vec<(String, super::api::TalkMessage)>, Box<dyn std::error::Error>> {
           let mut new_messages = Vec::new();
           for room in &self.rooms {
               let last_id = self.last_known_ids.get(room).copied().unwrap_or(0);
               let messages = self.api.receive_messages(room, last_id).await?;
               for msg in messages {
                   self.last_known_ids.insert(room.clone(), msg.id);
                   new_messages.push((room.clone(), msg));
               }
           }
           Ok(new_messages)
       }
   }
   ```

5. **File sharing** in `files.rs`:
   ```rust
   pub async fn share_file(
       api: &super::api::NextcloudTalkApi,
       room_token: &str,
       file_path: &str,
   ) -> Result<(), Box<dyn std::error::Error>> {
       // Upload file to Nextcloud via WebDAV
       // Share file link in Talk room
       todo!()
   }
   ```

6. **ChannelPlugin implementation** in `channel.rs` — authenticate, join rooms, start polling loop, implement send/receive/disconnect.

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [ ] Nextcloud Talk API connection with authentication works
- [ ] Room-based messaging (send and receive) works
- [ ] Room listing and joining works
- [ ] File sharing integration works
- [ ] Polling for new messages works reliably
- [ ] Unit tests for API client and message polling
- [ ] Integration test with mocked Nextcloud Talk API

---
*Created: 2026-02-15*
