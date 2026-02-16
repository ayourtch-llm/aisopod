# Issue 178: Implement Twitch Channel

## Summary
Implement a Twitch channel for aisopod using Twitch IRC (TMI) integration. This enables the bot to participate in Twitch chat, detect moderator and subscriber status, authenticate via OAuth, and support whispers for direct messaging.

## Location
- Crate: `aisopod-channel-twitch`
- File: `crates/aisopod-channel-twitch/src/lib.rs`

## Current Behavior
No Twitch channel exists. The channel abstraction traits are defined, but Twitch chat integration is not implemented.

## Expected Behavior
After implementation:
- aisopod connects to Twitch chat via IRC (TMI).
- Chat channel messaging is supported.
- Moderator and subscriber badge/status detection works.
- OAuth authentication is handled.
- Whispers are supported for direct messaging.

## Impact
Enables aisopod to integrate with the Twitch streaming platform, allowing bots to interact with chat participants in live streams — a unique real-time messaging use case.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-twitch/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── tmi.rs
       ├── auth.rs
       └── badges.rs
   ```

2. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct TwitchConfig {
       /// Bot username on Twitch
       pub username: String,
       /// OAuth token (e.g., "oauth:abc123...")
       pub oauth_token: String,
       /// Channels to join (e.g., ["#channel1", "#channel2"])
       pub channels: Vec<String>,
       /// Enable whisper support (requires verified bot)
       pub enable_whispers: bool,
       /// Client ID for Twitch API calls
       pub client_id: Option<String>,
   }
   ```

3. **TMI (Twitch Messaging Interface) client** in `tmi.rs`:
   ```rust
   use tokio::net::TcpStream;
   use tokio_tungstenite::WebSocketStream;

   pub struct TmiClient {
       // WebSocket connection to irc-ws.chat.twitch.tv
   }

   impl TmiClient {
       pub async fn connect(username: &str, oauth_token: &str) -> Result<Self, Box<dyn std::error::Error>> {
           // Connect to wss://irc-ws.chat.twitch.tv:443
           // Send: CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands
           // Send: PASS oauth:<token>
           // Send: NICK <username>
           todo!()
       }

       pub async fn join_channel(&mut self, channel: &str) -> Result<(), Box<dyn std::error::Error>> {
           // Send: JOIN #<channel>
           todo!()
       }

       pub async fn send_message(&mut self, channel: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
           // Send: PRIVMSG #<channel> :<message>
           todo!()
       }

       pub async fn send_whisper(&mut self, username: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
           // Send: PRIVMSG #<bot_channel> :/w <username> <message>
           todo!()
       }

       pub async fn read_message(&mut self) -> Result<TwitchMessage, Box<dyn std::error::Error>> {
           // Parse incoming IRC message with Twitch tags
           // Handle PING with PONG
           todo!()
       }
   }

   #[derive(Debug)]
   pub struct TwitchMessage {
       pub channel: String,
       pub username: String,
       pub text: String,
       pub tags: TwitchTags,
       pub is_whisper: bool,
   }

   #[derive(Debug)]
   pub struct TwitchTags {
       pub display_name: String,
       pub badges: Vec<String>,
       pub is_mod: bool,
       pub is_subscriber: bool,
       pub user_id: String,
   }
   ```

4. **Badge/status detection** in `badges.rs`:
   ```rust
   pub fn parse_badges(badge_str: &str) -> Vec<Badge> {
       // Parse "moderator/1,subscriber/12" format
       badge_str.split(',')
           .filter_map(|b| {
               let mut parts = b.split('/');
               let name = parts.next()?;
               let version = parts.next()?;
               Some(Badge { name: name.to_string(), version: version.to_string() })
           })
           .collect()
   }

   pub fn is_moderator(badges: &[Badge]) -> bool {
       badges.iter().any(|b| b.name == "moderator" || b.name == "broadcaster")
   }

   pub fn is_subscriber(badges: &[Badge]) -> bool {
       badges.iter().any(|b| b.name == "subscriber")
   }

   #[derive(Debug)]
   pub struct Badge {
       pub name: String,
       pub version: String,
   }
   ```

5. **OAuth handling** in `auth.rs`:
   ```rust
   pub async fn validate_token(oauth_token: &str, client_id: &str) -> Result<TokenInfo, AuthError> {
       // GET https://id.twitch.tv/oauth2/validate
       // Authorization: OAuth <token>
       todo!()
   }
   ```

6. **ChannelPlugin implementation** in `channel.rs` — connect via TMI, join channels, handle PING/PONG, route messages and whispers.

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [ ] Twitch IRC (TMI) connection with OAuth works
- [ ] Chat channel messaging (send and receive) works
- [ ] Moderator and subscriber status detection works
- [ ] Whisper support for direct messaging works
- [ ] PING/PONG keepalive handling works
- [ ] Multiple channel joins work simultaneously
- [ ] Unit tests for message parsing, badge detection, and auth
- [ ] Integration test with mock TMI server

---
*Created: 2026-02-15*
