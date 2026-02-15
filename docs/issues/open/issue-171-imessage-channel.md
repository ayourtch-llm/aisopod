# Issue 171: Implement iMessage Channel (macOS-only)

## Summary
Implement an iMessage channel for aisopod that works exclusively on macOS. The integration uses either an AppleScript bridge or the BlueBubbles API to send and receive iMessages, with proper platform gating so the crate compiles as a no-op stub on non-macOS targets.

## Location
- Crate: `aisopod-channel-imessage`
- File: `crates/aisopod-channel-imessage/src/lib.rs`

## Current Behavior
No iMessage channel exists. The channel abstraction layer provides the traits, but iMessage is not implemented. There is no platform-specific channel precedent in the project.

## Expected Behavior
After implementation:
- On macOS, aisopod can send and receive iMessages (DMs and group chats).
- On non-macOS platforms, the crate compiles but returns a clear, descriptive error at runtime.
- Media attachments (images, files) are supported.
- AppleScript bridge or BlueBubbles API is used for message transport.

## Impact
Enables aisopod to integrate with Apple's messaging ecosystem, which is particularly valuable for macOS-based deployments and users who primarily communicate via iMessage.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-imessage/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── applescript.rs
       ├── bluebubbles.rs
       └── platform.rs
   ```

2. **Platform gating** in `lib.rs`:
   ```rust
   #[cfg(target_os = "macos")]
   mod applescript;
   #[cfg(target_os = "macos")]
   mod native_channel;

   mod config;
   mod channel;

   #[cfg(not(target_os = "macos"))]
   pub fn create_channel(_config: config::IMessageConfig) -> Result<(), aisopod_channel_core::ChannelError> {
       Err(aisopod_channel_core::ChannelError::PlatformUnsupported(
           "iMessage channel is only available on macOS".into()
       ))
   }
   ```

3. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct IMessageConfig {
       /// Backend: "applescript" or "bluebubbles"
       pub backend: IMessageBackend,
       /// BlueBubbles server URL (if using BlueBubbles)
       pub bluebubbles_url: Option<String>,
       /// BlueBubbles API password
       pub bluebubbles_password: Option<String>,
       /// Poll interval in seconds
       pub poll_interval_secs: u64,
   }

   #[derive(Debug, Deserialize)]
   #[serde(rename_all = "lowercase")]
   pub enum IMessageBackend {
       AppleScript,
       BlueBubbles,
   }
   ```

4. **AppleScript bridge** in `applescript.rs` (macOS only):
   ```rust
   #[cfg(target_os = "macos")]
   use std::process::Command;

   #[cfg(target_os = "macos")]
   pub fn send_message(recipient: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
       let script = format!(
           r#"tell application "Messages"
               set targetBuddy to buddy "{}" of service "iMessage"
               send "{}" to targetBuddy
           end tell"#,
           recipient, text
       );
       Command::new("osascript")
           .args(&["-e", &script])
           .output()?;
       Ok(())
   }
   ```

5. **BlueBubbles API client** in `bluebubbles.rs`:
   ```rust
   pub struct BlueBubblesClient {
       base_url: String,
       password: String,
       http: reqwest::Client,
   }

   impl BlueBubblesClient {
       pub async fn send_message(&self, chat_guid: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
           let url = format!("{}/api/v1/message/text", self.base_url);
           self.http.post(&url)
               .query(&[("password", &self.password)])
               .json(&serde_json::json!({
                   "chatGuid": chat_guid,
                   "message": text
               }))
               .send()
               .await?;
           Ok(())
       }
   }
   ```

6. **ChannelPlugin implementation** in `channel.rs`:
   ```rust
   #[async_trait]
   impl ChannelPlugin for IMessageChannel {
       async fn connect(&mut self) -> Result<(), ChannelError> {
           // Verify platform, then initialize chosen backend
           #[cfg(not(target_os = "macos"))]
           return Err(ChannelError::PlatformUnsupported("macOS required".into()));
           // ...
           todo!()
       }
       // ... send, receive, disconnect
   }
   ```

7. **Platform detection** in `platform.rs` — provide a `check_platform()` function that returns a user-friendly error message on non-macOS systems.

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [ ] iMessage channel works on macOS via AppleScript bridge
- [ ] BlueBubbles API backend works as an alternative
- [ ] DM and group messaging supported
- [ ] Media attachments (images, files) send and receive
- [ ] `cfg(target_os = "macos")` gating compiles correctly on all platforms
- [ ] Clear, descriptive error returned on non-macOS platforms
- [ ] Unit tests for message formatting and platform detection
- [ ] Integration test on macOS with mock AppleScript output

---
*Created: 2026-02-15*
