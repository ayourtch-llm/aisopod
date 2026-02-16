# Issue 090: Define Adapter Interface Traits

## Summary
Define all 13 adapter interface traits in the `aisopod-channel` crate. Each adapter represents an optional capability that a channel plugin can implement, from message delivery to device pairing.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/src/adapters.rs`

## Current Behavior
The `aisopod-channel` crate has no adapter trait definitions. Channel plugins have no standardized interface for optional capabilities.

## Expected Behavior
The crate exports 13 async adapter traits, each with well-documented methods:

1. **OnboardingAdapter** — CLI onboarding wizard: `setup_wizard(&self, ctx: &OnboardingContext) -> Result<AccountConfig>`.
2. **OutboundAdapter** — Message delivery: `send_text(&self, target: &MessageTarget, text: &str) -> Result<()>`, `send_media(&self, target: &MessageTarget, media: &Media) -> Result<()>`.
3. **GatewayAdapter** — WebSocket/polling connection lifecycle: `connect(&self, account: &AccountConfig) -> Result<()>`, `disconnect(&self, account: &AccountConfig) -> Result<()>`, `is_connected(&self, account: &AccountConfig) -> bool`.
4. **StatusAdapter** — Health monitoring: `health_check(&self, account: &AccountConfig) -> Result<ChannelHealth>`.
5. **TypingAdapter** — Typing indicators: `send_typing(&self, target: &MessageTarget) -> Result<()>`.
6. **MessagingAdapter** — Message reactions: `react(&self, message_id: &str, emoji: &str) -> Result<()>`, `unreact(&self, message_id: &str, emoji: &str) -> Result<()>`.
7. **ThreadingAdapter** — Thread/reply support: `create_thread(&self, parent_id: &str, title: &str) -> Result<String>`, `reply_in_thread(&self, thread_id: &str, text: &str) -> Result<()>`.
8. **DirectoryAdapter** — Group/user discovery: `list_groups(&self, account: &AccountConfig) -> Result<Vec<GroupInfo>>`, `list_members(&self, group_id: &str) -> Result<Vec<MemberInfo>>`.
9. **SecurityAdapter** — Security and DM policies: `is_allowed_sender(&self, sender: &SenderInfo) -> bool`, `requires_mention_in_group(&self) -> bool`.
10. **HeartbeatAdapter** — Keep-alive mechanism: `heartbeat(&self, account: &AccountConfig) -> Result<()>`, `heartbeat_interval(&self) -> Duration`.
11. **ChannelConfigAdapter** — Account management: `list_accounts(&self) -> Result<Vec<String>>`, `resolve_account(&self, id: &str) -> Result<AccountSnapshot>`, `enable_account(&self, id: &str) -> Result<()>`, `disable_account(&self, id: &str) -> Result<()>`, `delete_account(&self, id: &str) -> Result<()>`.
12. **AuthAdapter** — Token/credential management: `authenticate(&self, config: &AccountConfig) -> Result<AuthToken>`, `refresh_token(&self, token: &AuthToken) -> Result<AuthToken>`.
13. **PairingAdapter** — Device pairing: `initiate_pairing(&self) -> Result<PairingCode>`, `complete_pairing(&self, code: &str) -> Result<AccountConfig>`.

Supporting types referenced by adapter methods (e.g., `OnboardingContext`, `AccountConfig`, `ChannelHealth`, `GroupInfo`, `MemberInfo`, `AccountSnapshot`, `AuthToken`, `PairingCode`) should be defined or re-exported as needed.

## Impact
These traits define the contract between the core system and every channel implementation. Without them, channel plugins cannot be developed against a stable API.

## Suggested Implementation
1. Open `crates/aisopod-channel/src/adapters.rs`.
2. Add `use async_trait::async_trait;` and import supporting types from `types.rs`.
3. Define any supporting types not yet defined. For example:
   - `OnboardingContext` — struct with fields relevant to onboarding (e.g., `config_dir: PathBuf`).
   - `AccountConfig` — struct with fields `id: String`, `channel: String`, `credentials: serde_json::Value`, `enabled: bool`.
   - `ChannelHealth` — enum with variants `Healthy`, `Degraded(String)`, `Disconnected(String)`.
   - `GroupInfo` — struct with `id: String`, `name: String`.
   - `MemberInfo` — struct with `id: String`, `display_name: String`.
   - `AccountSnapshot` — struct with `id: String`, `channel: String`, `enabled: bool`, `connected: bool`.
   - `AuthToken` — struct with `token: String`, `expires_at: Option<DateTime<Utc>>`.
   - `PairingCode` — struct with `code: String`, `expires_at: DateTime<Utc>`, `qr_url: Option<String>`.
4. Define each adapter trait using `#[async_trait]` with `Send + Sync` bounds. Copy the method signatures from the plan's code blocks.
5. Add doc-comments (`///`) to every trait, method, and parameter type explaining its purpose and expected behavior.
6. Re-export all adapter traits and supporting types from `crates/aisopod-channel/src/lib.rs`.
7. Run `cargo check -p aisopod-channel` to verify everything compiles.

## Dependencies
- Issue 089 (define ChannelPlugin trait and channel metadata types)

## Acceptance Criteria
- [ ] All 13 adapter traits are defined with correct method signatures
- [ ] All supporting types (`OnboardingContext`, `AccountConfig`, `ChannelHealth`, `GroupInfo`, `MemberInfo`, `AccountSnapshot`, `AuthToken`, `PairingCode`) are defined
- [ ] Every trait, method, and type has a doc-comment
- [ ] All traits require `Send + Sync` bounds
- [ ] `cargo check -p aisopod-channel` compiles without errors

---
*Created: 2026-02-15*
