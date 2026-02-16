# Issue 174: Implement Matrix Channel

## Summary
Implement a Matrix channel for aisopod using the Matrix client-server API. This enables the bot to participate in Matrix rooms and direct messages, with optional end-to-end encryption support, configurable homeserver connections, and SSO/token authentication.

## Location
- Crate: `aisopod-channel-matrix`
- File: `crates/aisopod-channel-matrix/src/lib.rs`

## Current Behavior
No Matrix channel exists. The channel abstraction traits are defined, but Matrix protocol support is not implemented.

## Expected Behavior
After implementation:
- aisopod connects to a Matrix homeserver.
- Room and DM messaging is supported.
- End-to-end encryption (E2EE) is optionally enabled.
- Homeserver URL is configurable.
- SSO and access token authentication are both supported.

## Impact
Adds support for the Matrix decentralized communication protocol, enabling aisopod to work with self-hosted and federated messaging environments popular in open-source and privacy-focused communities.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-matrix/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── client.rs
       ├── encryption.rs
       └── auth.rs
   ```

2. **Add dependency** in `Cargo.toml`:
   ```toml
   [dependencies]
   matrix-sdk = { version = "0.7", features = ["e2e-encryption"] }
   ```

3. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct MatrixConfig {
       /// Homeserver URL (e.g., "https://matrix.org")
       pub homeserver_url: String,
       /// Authentication method
       pub auth: MatrixAuth,
       /// Enable end-to-end encryption
       pub enable_e2ee: bool,
       /// Rooms to join (e.g., ["!room:matrix.org"])
       pub rooms: Vec<String>,
       /// Path to store encryption keys and sync state
       pub state_store_path: Option<String>,
   }

   #[derive(Debug, Deserialize)]
   #[serde(tag = "type")]
   pub enum MatrixAuth {
       #[serde(rename = "password")]
       Password { username: String, password: String },
       #[serde(rename = "token")]
       AccessToken { access_token: String },
       #[serde(rename = "sso")]
       SSO { token: String },
   }
   ```

4. **Matrix client wrapper** in `client.rs`:
   ```rust
   use matrix_sdk::{Client, config::SyncSettings, room::Room};

   pub struct MatrixClient {
       client: Client,
   }

   impl MatrixClient {
       pub async fn connect(config: &super::config::MatrixConfig) -> Result<Self, Box<dyn std::error::Error>> {
           let client = Client::builder()
               .homeserver_url(&config.homeserver_url)
               .build()
               .await?;

           match &config.auth {
               super::config::MatrixAuth::Password { username, password } => {
                   client.matrix_auth()
                       .login_username(username, password)
                       .send()
                       .await?;
               }
               super::config::MatrixAuth::AccessToken { access_token } => {
                   // Restore session with access token
                   todo!()
               }
               super::config::MatrixAuth::SSO { token } => {
                   // SSO login flow
                   todo!()
               }
           }

           Ok(Self { client })
       }

       pub async fn send_text(&self, room_id: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
           // Look up room, send text message
           todo!()
       }

       pub async fn start_sync(&self) -> Result<(), Box<dyn std::error::Error>> {
           // Start long-polling sync loop
           self.client.sync(SyncSettings::default()).await?;
           Ok(())
       }
   }
   ```

5. **Encryption support** in `encryption.rs`:
   ```rust
   pub async fn setup_e2ee(client: &matrix_sdk::Client, store_path: &str) -> Result<(), Box<dyn std::error::Error>> {
       // Configure Olm/Megolm key storage
       // Handle key verification if needed
       // Enable encryption for joined rooms
       todo!()
   }
   ```

6. **ChannelPlugin implementation** in `channel.rs`:
   ```rust
   #[async_trait]
   impl ChannelPlugin for MatrixChannel {
       async fn connect(&mut self) -> Result<(), ChannelError> {
           // Initialize client, authenticate, optionally set up E2EE
           // Join configured rooms
           // Start sync loop in background task
           todo!()
       }

       async fn send(&self, msg: OutboundMessage) -> Result<(), ChannelError> {
           // Send to appropriate room or DM
           todo!()
       }

       async fn receive(&mut self) -> Result<InboundMessage, ChannelError> {
           // Process events from sync loop
           todo!()
       }

       async fn disconnect(&mut self) -> Result<(), ChannelError> {
           // Stop sync, clean up
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
- [ ] Matrix homeserver connection with password auth works
- [ ] Access token authentication works
- [ ] Room messaging (send and receive) works
- [ ] DM messaging works
- [ ] E2EE can be optionally enabled and functions correctly
- [ ] Homeserver URL is configurable
- [ ] Room join/leave lifecycle is managed
- [ ] Unit tests for client setup, auth, and message handling
- [ ] Integration test with mocked Matrix server responses

---
*Created: 2026-02-15*
