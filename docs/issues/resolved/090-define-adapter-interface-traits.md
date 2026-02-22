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
- [x] All 13 adapter traits are defined with correct method signatures
- [x] All supporting types (`OnboardingContext`, `AccountConfig`, `ChannelHealth`, `GroupInfo`, `MemberInfo`, `AccountSnapshot`, `AuthToken`, `PairingCode`) are defined
- [x] Every trait, method, and type has a doc-comment
- [x] All traits require `Send + Sync` bounds
- [x] `cargo check -p aisopod-channel` compiles without errors

## Resolution
**Resolved: 2026-02-22**

The adapter interface traits were successfully implemented in `crates/aisopod-channel/src/adapters.rs`:

### Supporting Types (8 types)
- `OnboardingContext` — Context struct with `config_dir: PathBuf` and `channel_config_dir: Option<PathBuf>` for onboarding wizard flow
- `AccountConfig` — Account configuration with `id`, `channel`, `credentials` (serde_json::Value), and `enabled` boolean
- `ChannelHealth` — Enum with `Healthy`, `Degraded(String)`, and `Disconnected(String)` variants for connection status
- `GroupInfo` — Group information with `id` and `name` fields
- `MemberInfo` — Member information with `id` and `display_name` fields
- `AccountSnapshot` — Account state snapshot with `id`, `channel`, `enabled`, and `connected` fields
- `AuthToken` — Authentication token with `token` string and optional `expires_at: DateTime<Utc>`
- `PairingCode` — Pairing code with `code` string, `expires_at: DateTime<Utc>`, and optional `qr_url`

### Adapter Traits (13 traits)
All traits use `#[async_trait]` and require `Send + Sync` bounds with comprehensive doc-comments:

1. **OnboardingAdapter** — `setup_wizard(&self, ctx: &OnboardingContext) -> Result<AccountConfig, anyhow::Error>`
2. **OutboundAdapter** — `send_text(&self, target: &MessageTarget, text: &str) -> Result<(), anyhow::Error>`, `send_media(&self, target: &MessageTarget, media: &Media) -> Result<(), anyhow::Error>`
3. **GatewayAdapter** — `connect(&self, account: &AccountConfig) -> Result<(), anyhow::Error>`, `disconnect(&self, account: &AccountConfig) -> Result<(), anyhow::Error>`, `is_connected(&self, account: &AccountConfig) -> bool`
4. **StatusAdapter** — `health_check(&self, account: &AccountConfig) -> Result<ChannelHealth, anyhow::Error>`
5. **TypingAdapter** — `send_typing(&self, target: &MessageTarget) -> Result<(), anyhow::Error>`
6. **MessagingAdapter** — `react(&self, message_id: &str, emoji: &str) -> Result<(), anyhow::Error>`, `unreact(&self, message_id: &str, emoji: &str) -> Result<(), anyhow::Error>`
7. **ThreadingAdapter** — `create_thread(&self, parent_id: &str, title: &str) -> Result<String, anyhow::Error>`, `reply_in_thread(&self, thread_id: &str, text: &str) -> Result<(), anyhow::Error>`
8. **DirectoryAdapter** — `list_groups(&self, account: &AccountConfig) -> Result<Vec<GroupInfo>, anyhow::Error>`, `list_members(&self, group_id: &str) -> Result<Vec<MemberInfo>, anyhow::Error>`
9. **SecurityAdapter** — `is_allowed_sender(&self, sender: &SenderInfo) -> bool`, `requires_mention_in_group(&self) -> bool`
10. **HeartbeatAdapter** — `heartbeat(&self, account: &AccountConfig) -> Result<(), anyhow::Error>`, `heartbeat_interval(&self) -> Duration`
11. **ChannelConfigAdapter** — `list_accounts(&self) -> Result<Vec<String>, anyhow::Error>`, `resolve_account(&self, id: &str) -> Result<AccountSnapshot, anyhow::Error>`, `enable_account(&self, id: &str) -> Result<(), anyhow::Error>`, `disable_account(&self, id: &str) -> Result<(), anyhow::Error>`, `delete_account(&self, id: &str) -> Result<(), anyhow::Error>`
12. **AuthAdapter** — `authenticate(&self, config: &AccountConfig) -> Result<AuthToken, anyhow::Error>`, `refresh_token(&self, token: &AuthToken) -> Result<AuthToken, anyhow::Error>`
13. **PairingAdapter** — `initiate_pairing(&self) -> Result<PairingCode, anyhow::Error>`, `complete_pairing(&self, code: &str) -> Result<AccountConfig, anyhow::Error>`

### Verification
- `cargo check -p aisopod-channel` — ✅ Compiles without errors
- `cargo test -p aisopod-channel` — ✅ All 21 tests pass
- All traits and types are properly re-exported from `aisopod_channel` crate root

The implementation satisfies all acceptance criteria with comprehensive documentation on every trait, method, and parameter.

---
*Created: 2026-02-15*
*Resolved: 2026-02-22*
