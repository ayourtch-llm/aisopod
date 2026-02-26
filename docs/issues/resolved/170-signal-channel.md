# Issue 170: Implement Signal Channel

## Summary
Implement a Signal messaging channel for aisopod, enabling the bot to send and receive messages via Signal's secure messaging platform. Integration is achieved through Signal CLI or a libsignal subprocess bridge, supporting both direct messages and group conversations.

## Location
- Crate: `aisopod-channel-signal`
- File: `crates/aisopod-channel-signal/src/lib.rs`

## Current Behavior
No Signal channel implementation exists. The channel abstraction layer (plan 0009) and Tier 1 channels (plan 0010) provide the foundational traits and patterns, but Signal is not yet supported.

## Expected Behavior
After implementation, aisopod can:
- Connect to Signal via Signal CLI or libsignal subprocess.
- Send and receive direct messages (DMs) and group messages.
- Handle media attachments (images, documents).
- Detect and respect disappearing message timers.
- Identify users by phone number.

## Impact
Adds support for one of the most popular privacy-focused messaging platforms, expanding aisopod's reach to security-conscious users and organizations that rely on Signal for team communication.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-signal/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── outbound.rs
       ├── gateway.rs
       └── runtime.rs
   ```

2. **Define configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct SignalConfig {
       /// Path to signal-cli binary
       pub signal_cli_path: String,
       /// Phone number registered with Signal (e.g., "+1234567890")
       pub phone_number: String,
       /// Optional: list of group IDs to join
       pub groups: Vec<String>,
       /// Poll interval in seconds for checking new messages
       pub poll_interval_secs: u64,
   }
   ```

3. **Implement the ChannelPlugin trait** in `channel.rs`:
   ```rust
   use async_trait::async_trait;
   use aisopod_channel_core::{ChannelPlugin, InboundMessage, OutboundMessage, ChannelError};
   use crate::config::SignalConfig;

   pub struct SignalChannel {
       config: SignalConfig,
       // subprocess handle for signal-cli daemon
   }

   #[async_trait]
   impl ChannelPlugin for SignalChannel {
       async fn connect(&mut self) -> Result<(), ChannelError> {
           // Spawn signal-cli daemon in JSON-RPC mode:
           // signal-cli -u <phone> daemon --json
           todo!()
       }

       async fn send(&self, msg: OutboundMessage) -> Result<(), ChannelError> {
           // Use signal-cli send command or JSON-RPC call
           // signal-cli -u <phone> send -m "text" <recipient>
           // For groups: signal-cli -u <phone> send -m "text" -g <groupId>
           todo!()
       }

       async fn receive(&mut self) -> Result<InboundMessage, ChannelError> {
           // Parse JSON output from signal-cli daemon
           // Map Signal envelope to InboundMessage
           todo!()
       }

       async fn disconnect(&mut self) -> Result<(), ChannelError> {
           // Kill the signal-cli subprocess
           todo!()
       }
   }
   ```

4. **Subprocess management** in `runtime.rs`:
   ```rust
   use tokio::process::{Command, Child};

   pub struct SignalCliProcess {
       child: Child,
   }

   impl SignalCliProcess {
       pub async fn spawn(cli_path: &str, phone: &str) -> Result<Self, std::io::Error> {
           let child = Command::new(cli_path)
               .args(&["-u", phone, "daemon", "--json"])
               .stdout(std::process::Stdio::piped())
               .stdin(std::process::Stdio::piped())
               .spawn()?;
           Ok(Self { child })
       }
   }
   ```

5. **Media handling** in `outbound.rs`:
   ```rust
   pub async fn send_attachment(
       cli_path: &str,
       phone: &str,
       recipient: &str,
       file_path: &str,
   ) -> Result<(), Box<dyn std::error::Error>> {
       // signal-cli -u <phone> send -a <file_path> <recipient>
       todo!()
   }
   ```

6. **Disappearing messages** — parse the `expiresInSeconds` field from Signal envelopes and include it in message metadata so downstream handlers are aware.

7. **Phone number identity** — map Signal's phone-number-based addressing to aisopod's user identity model in the gateway layer.

## Resolution

The Signal channel implementation was completed successfully with the following changes:

- Created `aisopod-channel-signal` crate with full Signal channel implementation
- Implemented `ChannelPlugin` trait with `connect`, `send`, `receive`, and `disconnect` methods
- Spawns signal-cli daemon in JSON-RPC mode via subprocess: `signal-cli -u <phone> daemon --json`
- Handles both direct messages and group messages
- Supports media attachments (images, audio, video, documents)
- Detects disappearing message timers via the `expires_in` field in Signal envelopes
- Maps phone number identities with E.164 validation for proper formatting
- All functionality covered by 18 unit tests plus doc tests
- All tests pass with no regressions
- Changes committed in commits by the implementer
- Learning document created at `docs/learnings/170-signal-channel.md` documenting the implementation process

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [ ] Signal CLI subprocess spawns and connects successfully
- [ ] Direct messages can be sent and received
- [ ] Group messages can be sent and received
- [ ] Media attachments (images, documents) send and receive correctly
- [ ] Disappearing message timers are detected and included in metadata
- [ ] Phone number-based identity maps to aisopod user model
- [ ] Graceful error handling when signal-cli is unavailable
- [ ] Unit tests for message parsing and subprocess management
- [ ] Integration test with mock Signal CLI output

---
*Created: 2026-02-15*
*Resolved: 2026-02-25*
