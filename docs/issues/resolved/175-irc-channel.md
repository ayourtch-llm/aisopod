# Issue 175: Implement IRC Channel

## Summary
Implement an IRC channel for aisopod using the `irc` crate. This enables the bot to connect to IRC servers, join channels, send and receive messages (including private messages via PRIVMSG), with support for multiple servers, NickServ authentication, and TLS encryption.

## Location
- Crate: `aisopod-channel-irc`
- File: `crates/aisopod-channel-irc/src/lib.rs`

## Current Behavior
No IRC channel exists. The channel abstraction traits are defined, but IRC protocol support is not implemented.

## Expected Behavior
After implementation:
- aisopod connects to one or more IRC servers.
- Channel and DM (PRIVMSG) messaging is supported.
- NickServ authentication is handled.
- TLS-encrypted connections are supported.
- Multiple simultaneous server connections are managed.

## Impact
Adds support for the IRC protocol, enabling aisopod to work with legacy and modern IRC networks that remain popular in developer and open-source communities.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-irc/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── client.rs
       └── auth.rs
   ```

2. **Add dependency** in `Cargo.toml`:
   ```toml
   [dependencies]
   irc = "1.0"
   ```

3. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct IrcConfig {
       /// List of IRC server connections
       pub servers: Vec<IrcServerConfig>,
   }

   #[derive(Debug, Deserialize)]
   pub struct IrcServerConfig {
       /// Server hostname (e.g., "irc.libera.chat")
       pub server: String,
       /// Server port (default: 6697 for TLS, 6667 for plain)
       pub port: u16,
       /// Use TLS encryption
       pub use_tls: bool,
       /// Bot nickname
       pub nickname: String,
       /// Optional NickServ password
       pub nickserv_password: Option<String>,
       /// Channels to join (e.g., ["#channel1", "#channel2"])
       pub channels: Vec<String>,
       /// Server password (for password-protected servers)
       pub server_password: Option<String>,
   }
   ```

4. **IRC client wrapper** in `client.rs`:
   ```rust
   use irc::client::prelude::*;

   pub struct IrcConnection {
       client: Client,
       server_name: String,
   }

   impl IrcConnection {
       pub async fn connect(config: &super::config::IrcServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
           let irc_config = Config {
               nickname: Some(config.nickname.clone()),
               server: Some(config.server.clone()),
               port: Some(config.port),
               use_tls: Some(config.use_tls),
               channels: config.channels.clone(),
               password: config.server_password.clone(),
               ..Config::default()
           };

           let client = Client::from_config(irc_config).await?;
           client.identify()?;

           Ok(Self {
               client,
               server_name: config.server.clone(),
           })
       }

       pub async fn send_privmsg(&self, target: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
           self.client.send_privmsg(target, message)?;
           Ok(())
       }

       pub fn stream(&self) -> irc::client::ClientStream {
           self.client.stream()
       }
   }
   ```

5. **NickServ authentication** in `auth.rs`:
   ```rust
   pub async fn authenticate_nickserv(
       client: &irc::client::Client,
       password: &str,
   ) -> Result<(), Box<dyn std::error::Error>> {
       // Send: PRIVMSG NickServ :IDENTIFY <password>
       client.send_privmsg("NickServ", &format!("IDENTIFY {}", password))?;
       // Optionally wait for confirmation response
       Ok(())
   }
   ```

6. **ChannelPlugin implementation** in `channel.rs`:
   ```rust
   use futures::StreamExt;

   pub struct IrcChannel {
       connections: Vec<super::client::IrcConnection>,
   }

   #[async_trait]
   impl ChannelPlugin for IrcChannel {
       async fn connect(&mut self) -> Result<(), ChannelError> {
           // Connect to each configured server
           // Authenticate with NickServ if configured
           // Join configured channels
           todo!()
       }

       async fn send(&self, msg: OutboundMessage) -> Result<(), ChannelError> {
           // Route to correct server/channel/user via PRIVMSG
           todo!()
       }

       async fn receive(&mut self) -> Result<InboundMessage, ChannelError> {
           // Select across all server streams
           // Parse PRIVMSG into InboundMessage
           // Distinguish channel messages from DMs
           todo!()
       }

       async fn disconnect(&mut self) -> Result<(), ChannelError> {
           // Send QUIT to each server, close connections
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
- [ ] IRC server connection with TLS works
- [ ] Channel join and PRIVMSG send/receive works
- [ ] DM (private PRIVMSG) works
- [ ] NickServ authentication works
- [ ] Multiple simultaneous server connections work
- [ ] Graceful reconnection on disconnect
- [ ] Unit tests for message parsing and NickServ auth
- [ ] Integration test with mock IRC server

---
*Created: 2026-02-15*
