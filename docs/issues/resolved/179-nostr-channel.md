# Issue 179: Implement Nostr Channel

## Summary
Implement a Nostr channel for aisopod using the Nostr protocol (NIP-01 for basic protocol, NIP-04 for encrypted DMs). This enables the bot to connect to Nostr relays, post public messages, send and receive encrypted direct messages, and manage cryptographic keys.

## Location
- Crate: `aisopod-channel-nostr`
- File: `crates/aisopod-channel-nostr/src/lib.rs`

## Current Behavior
No Nostr channel exists. The channel abstraction traits are defined, but Nostr protocol support is not implemented.

## Expected Behavior
After implementation:
- aisopod connects to one or more Nostr relays.
- Public channel posting (kind 1 events) works.
- Encrypted DMs (NIP-04) are supported.
- Relay connection management handles multiple relays.
- Key management supports nsec/npub formats.

## Impact
Adds support for the Nostr decentralized protocol, enabling aisopod to participate in a censorship-resistant, relay-based messaging network popular in Bitcoin and freedom-of-speech communities.

## Suggested Implementation

1. **Create crate scaffold:**
   ```
   crates/aisopod-channel-nostr/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── channel.rs
       ├── config.rs
       ├── relay.rs
       ├── keys.rs
       ├── events.rs
       └── nip04.rs
   ```

2. **Configuration** in `config.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct NostrConfig {
       /// Private key in nsec or hex format
       pub private_key: String,
       /// Relay URLs to connect to
       pub relays: Vec<String>,
       /// Enable NIP-04 encrypted DMs
       pub enable_dms: bool,
       /// Public channels to follow (by event ID or pubkey)
       pub channels: Vec<String>,
   }
   ```

3. **Key management** in `keys.rs`:
   ```rust
   use secp256k1::{SecretKey, PublicKey, Secp256k1};

   pub struct NostrKeys {
       secret_key: SecretKey,
       public_key: PublicKey,
   }

   impl NostrKeys {
       pub fn from_nsec(nsec: &str) -> Result<Self, KeyError> {
           // Decode bech32 nsec format to raw bytes
           // Derive public key from secret key
           todo!()
       }

       pub fn from_hex(hex: &str) -> Result<Self, KeyError> {
           let secp = Secp256k1::new();
           let secret_key = SecretKey::from_slice(&hex::decode(hex)?)?;
           let public_key = PublicKey::from_secret_key(&secp, &secret_key);
           Ok(Self { secret_key, public_key })
       }

       pub fn npub(&self) -> String {
           // Encode public key as bech32 npub
           todo!()
       }

       pub fn pubkey_hex(&self) -> String {
           hex::encode(self.public_key.serialize()[1..].to_vec())
       }
   }
   ```

4. **Event creation and signing** in `events.rs`:
   ```rust
   use serde::{Serialize, Deserialize};

   #[derive(Debug, Serialize, Deserialize)]
   pub struct NostrEvent {
       pub id: String,
       pub pubkey: String,
       pub created_at: u64,
       pub kind: u32,
       pub tags: Vec<Vec<String>>,
       pub content: String,
       pub sig: String,
   }

   impl NostrEvent {
       pub fn new_text_note(keys: &super::keys::NostrKeys, content: &str) -> Result<Self, EventError> {
           // Kind 1: text note
           // Serialize [0, pubkey, created_at, kind, tags, content]
           // SHA256 hash for id
           // Sign with secret key
           todo!()
       }

       pub fn new_dm(
           keys: &super::keys::NostrKeys,
           recipient_pubkey: &str,
           plaintext: &str,
       ) -> Result<Self, EventError> {
           // Kind 4: encrypted DM (NIP-04)
           // Encrypt content with shared secret
           // Tag recipient: ["p", recipient_pubkey]
           todo!()
       }
   }
   ```

5. **NIP-04 encryption** in `nip04.rs`:
   ```rust
   pub fn encrypt(
       sender_secret: &secp256k1::SecretKey,
       recipient_pubkey: &secp256k1::PublicKey,
       plaintext: &str,
   ) -> Result<String, Box<dyn std::error::Error>> {
       // Compute shared secret via ECDH
       // AES-256-CBC encrypt with random IV
       // Return base64(ciphertext) + "?iv=" + base64(iv)
       todo!()
   }

   pub fn decrypt(
       recipient_secret: &secp256k1::SecretKey,
       sender_pubkey: &secp256k1::PublicKey,
       ciphertext: &str,
   ) -> Result<String, Box<dyn std::error::Error>> {
       // Parse base64 ciphertext and IV
       // Compute shared secret, decrypt
       todo!()
   }
   ```

6. **Relay connection management** in `relay.rs`:
   ```rust
   use tokio_tungstenite::tungstenite::Message;
   use futures::StreamExt;

   pub struct RelayPool {
       relays: Vec<RelayConnection>,
   }

   pub struct RelayConnection {
       url: String,
       // WebSocket connection
   }

   impl RelayPool {
       pub async fn connect(urls: &[String]) -> Result<Self, Box<dyn std::error::Error>> {
           // Connect to each relay via WebSocket
           // wss://relay.example.com
           todo!()
       }

       pub async fn publish(&mut self, event: &super::events::NostrEvent) -> Result<(), Box<dyn std::error::Error>> {
           // Send ["EVENT", event] to all relays
           todo!()
       }

       pub async fn subscribe(&mut self, filters: Vec<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
           // Send ["REQ", subscription_id, ...filters] to all relays
           todo!()
       }

       pub async fn next_event(&mut self) -> Result<super::events::NostrEvent, Box<dyn std::error::Error>> {
           // Read from any relay, parse ["EVENT", sub_id, event]
           todo!()
       }
   }
   ```

7. **ChannelPlugin implementation** in `channel.rs` — load keys, connect to relays, subscribe to events, implement send/receive for both public and DM messages.

## Dependencies
- Issue 089: Channel trait definitions
- Issue 090: Inbound message pipeline
- Issue 091: Outbound message pipeline
- Issue 092: Channel lifecycle management

## Acceptance Criteria
- [ ] Relay connections via WebSocket work
- [ ] Public posting (kind 1 text notes) works
- [ ] Encrypted DMs (NIP-04) send and receive correctly
- [ ] Key management supports nsec and hex formats
- [ ] Multiple relay connections are managed concurrently
- [ ] Event signing and verification work correctly
- [ ] Unit tests for key management, event creation, and NIP-04 encryption
- [ ] Integration test with mock relay

---
*Created: 2026-02-15*
