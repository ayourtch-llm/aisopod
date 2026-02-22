# Issue 094: Implement Security and Allowlist Enforcement

## Summary
Implement the security enforcement layer that integrates with `SecurityAdapter` to enforce sender allowlists, @mention requirements for group messages, and DM security policies.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/src/security.rs`

## Current Behavior
There is no security enforcement for incoming messages. Any sender from any channel can reach the agent engine without access control.

## Expected Behavior
A `SecurityEnforcer` struct provides reusable security checks that the message routing pipeline calls before passing messages to agents:

- **Sender allowlist** — integrates with `SecurityAdapter::is_allowed_sender()` to verify the sender is on the allowlist. If the channel plugin does not implement `SecurityAdapter`, all senders are allowed by default.
- **Mention requirement** — for group/channel messages, checks `SecurityAdapter::requires_mention_in_group()`. If true, scans the message content for a bot @mention. Messages without the mention are silently skipped.
- **DM security policies** — for direct messages, checks if the sender is permitted to DM the bot. This may use the same allowlist or a separate DM-specific policy.

The enforcer exposes methods:
```rust
pub fn check_sender(&self, adapter: Option<&dyn SecurityAdapter>, sender: &SenderInfo) -> Result<()>;
pub fn check_mention(&self, adapter: Option<&dyn SecurityAdapter>, message: &IncomingMessage, bot_identifiers: &[String]) -> MentionCheckResult;
pub fn check_dm_policy(&self, adapter: Option<&dyn SecurityAdapter>, sender: &SenderInfo) -> Result<()>;
```

`MentionCheckResult` is an enum: `Allowed`, `SkipSilently`, `Blocked(String)`.

## Impact
Without security enforcement, the bot processes messages from unauthorized users and responds in groups where it was not explicitly addressed, leading to spam and potential abuse.

## Suggested Implementation
1. Open `crates/aisopod-channel/src/security.rs`.
2. Define `MentionCheckResult` enum with variants `Allowed`, `SkipSilently`, `Blocked(String)`.
3. Define `SecurityEnforcer` struct (it may be stateless or hold configuration like default policies).
4. Implement `SecurityEnforcer::new()`.
5. Implement `check_sender()`:
   - If `adapter` is `None`, return `Ok(())` (no security adapter means open access).
   - If `adapter` is `Some`, call `adapter.is_allowed_sender(sender)`.
   - If not allowed, return an error with a descriptive message including the sender's ID.
6. Implement `check_mention()`:
   - If `adapter` is `None`, return `MentionCheckResult::Allowed`.
   - If the message's peer kind is not `Group` or `Channel`, return `Allowed` (mention checks only apply to groups).
   - Call `adapter.requires_mention_in_group()`. If false, return `Allowed`.
   - Scan the message content for any of the `bot_identifiers` (e.g., `@botname`). For `MessageContent::Text`, search the string. For `Mixed`, search each text part.
   - If a mention is found, return `Allowed`. Otherwise, return `SkipSilently`.
7. Implement `check_dm_policy()`:
   - If `adapter` is `None`, return `Ok(())`.
   - Call `adapter.is_allowed_sender(sender)` (reuse the allowlist for DMs).
   - If not allowed, return an error.
8. Add doc-comments to every type, method, and variant.
9. Re-export `SecurityEnforcer` and `MentionCheckResult` from `crates/aisopod-channel/src/lib.rs`.
10. Run `cargo check -p aisopod-channel` to verify everything compiles.

## Dependencies
- Issue 090 (define adapter interface traits — provides `SecurityAdapter`)
- Issue 093 (implement message routing pipeline — consumes security checks)

## Acceptance Criteria
- [x] `SecurityEnforcer` struct is defined with `check_sender()`, `check_mention()`, and `check_dm_policy()` methods
- [x] `MentionCheckResult` enum is defined with `Allowed`, `SkipSilently`, and `Blocked` variants
- [x] Unauthorized senders are rejected with a descriptive error
- [x] Group messages without required @mention return `SkipSilently`
- [x] When no `SecurityAdapter` is provided, all checks pass (open access)
- [x] DM security policies are enforced
- [x] Every public type and method has a doc-comment
- [x] `cargo check -p aisopod-channel` compiles without errors

## Resolution

Implemented the security enforcement layer as described in the issue:

### Implementation Details

**Created `SecurityEnforcer` struct** in `crates/aisopod-channel/src/security.rs`:
- Stateful enforcer with optional `DmPolicy` configuration
- `new()` constructor for default configuration
- `with_dm_policy()` constructor for custom DM policy
- `Default` trait implementation

**Created `MentionCheckResult` enum** with variants:
- `Allowed` - Message passes mention check
- `SkipSilently` - Message without required @mention
- `Blocked(String)` - Message explicitly blocked with reason

**Created `DmPolicy` enum** for DM security policies:
- `Allowlist` - Use same allowlist as regular messages
- `Open` - Allow all DMs (no restrictions)
- `Custom` - Custom policy implementation

### Methods Implemented

1. **`check_sender()`** - Verifies sender is on allowlist
   - Uses `SecurityAdapter::is_allowed_sender()` when available
   - Allows all senders when no adapter is provided
   - Returns descriptive error with sender ID for unauthorized senders

2. **`check_mention()`** - Checks @mention requirement for group messages
   - Applies only to Group/Channel peer kinds
   - Respects `SecurityAdapter::requires_mention_in_group()`
   - Scans message content for `@identifier` or `<@identifier>` format
   - Returns `Allowed`, `SkipSilently`, or `Blocked` variant

3. **`check_dm_policy()`** - Enforces DM security policies
   - Uses allowlist check via `SecurityAdapter::is_allowed_sender()`
   - Allows all senders when no adapter is provided

### Integration

**SecurityEnforcer is integrated into `MessageRouter`**:
- Step 4 in routing pipeline: calls `check_sender()`
- Step 5 in routing pipeline: calls `check_mention()` for group messages
- Properly handles `MentionCheckResult` variants
- Skips silently or blocks messages as appropriate

### Code Quality

- Comprehensive doc-comments on all public types and methods
- 39 unit tests covering all methods and edge cases
- All tests pass (`cargo test -p aisopod-channel`)
- Code compiles without errors (`cargo check -p aisopod-channel`)
- Re-exports `SecurityEnforcer` and `MentionCheckResult` from `lib.rs`

---
*Created: 2026-02-15*
*Resolved: 2026-02-22*
