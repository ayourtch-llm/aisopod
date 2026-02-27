# Learning Notes: Implementing Lark/Feishu Channel

**Issue #181**: Implement Lark/Feishu Channel  
**Date**: 2026-02-27

## Summary

Successfully implemented a complete Lark/Feishu channel plugin for aisopod, following the channel abstraction pattern used by other channels in the project.

## Implementation Overview

### Crate Structure
```
crates/aisopod-channel-lark/
├── Cargo.toml
└── src/
    ├── lib.rs         # Module exports and crate documentation
    ├── channel.rs     # ChannelPlugin trait implementation
    ├── config.rs      # LarkConfig for credentials
    ├── auth.rs        # LarkAuth for token management
    ├── api.rs         # LarkApi for API calls
    ├── cards.rs       # MessageCard for rich content
    └── events.rs      # Webhook event handlers
```

### Key Components

1. **LarkConfig** (`config.rs`)
   - Contains app_id, app_secret, verification_token, encrypt_key
   - webhook_port for event subscriptions
   - use_feishu flag for China region support
   - Provides `base_url()` method for API endpoint selection

2. **LarkAuth** (`auth.rs`)
   - Manages tenant access tokens
   - Implements automatic token refresh before expiry
   - Uses caching to avoid unnecessary API calls
   - Stores token expiry with 5-minute buffer

3. **LarkApi** (`api.rs`)
   - HTTP client for Lark Open Platform API
   - Methods: `send_text()`, `send_card()`, `send_message()`, `get_user_profile()`
   - Handles authentication via bearer tokens
   - Proper error handling with ApiError enum

4. **MessageCard** (`cards.rs`)
   - Rich interactive message card builder
   - Supports: simple cards, div elements, markdown, images
   - JSON serialization for API compatibility
   - Template color selection (blue, green, orange)

5. **Events Handler** (`events.rs`)
   - Webhook endpoint for Lark event subscriptions
   - URL verification challenge handling
   - Token validation
   - Event type parsing and routing

6. **LarkChannel** (`channel.rs`)
   - Implements `ChannelPlugin` trait
   - Multi-account support
   - Integrates with aisopod message types
   - Provides config and security adapters

## Technical Decisions

### Error Handling
- Used `anyhow::Result` for async methods to allow flexible error types
- Added `AuthError` and `ApiError` enums for specific error cases
- Used `thiserror` derive macro for clean error definitions

### Token Management
- Tokens are cached and refreshed 5 minutes before expiry
- Thread-safe with `Arc<Mutex<...>>` pattern for LarkApi
- Automatic refresh on each API call via `get_tenant_access_token()`

### Axum Integration
- Events handler uses axum's `Router` and `State` for dependency injection
- URL verification challenge response requires proper JSON serialization
- Token verification against configured verification_token

### Card Element Tag Naming
- Had to rename `tag` to `tag_type` to avoid serde tag conflict
- Used `#[serde(rename = "tag")]` to maintain API compatibility

### Message Content Handling
- MessageContent doesn't have a Card variant, so cards are sent via send_card method
- Media handling requires proper String conversion to avoid borrowing issues

## Issues Encountered

1. **Axum Feature Conflict**: The `extract` feature doesn't exist in axum 0.7; removed it.

2. **Token Request Error Type**: Initial implementation tried to return `AuthError` directly, but `Result` is `anyhow::Result`.

3. **Card Element Tag Conflict**: Using `#[serde(tag = "tag")]` on enum conflicts with field named `tag`. Fixed by renaming field to `tag_type`.

4. **Json Return Type**: Must use `Json(Value)` instead of `Json(&Value)` to avoid temporary value issues.

5. **Borrowing Issues**: Using `unwrap_or(&...)` pattern caused temporary value borrowing issues. Fixed by using `unwrap_or_else` with owned `String`.

## Lessons Learned

### For Future Channel Implementations

1. **Check serde tag usage**: When using `#[serde(tag = "...")]` on enums, don't use that field name in variants.

2. **Axum JSON handling**: Prefer `Json(Value)` over `Json(&Value)` when constructing responses.

3. **Error type consistency**: Ensure error types match the `Result` type being used (`anyhow::Result` vs `Result<T, E>`).

4. **Media URL handling**: Always convert to owned `String` to avoid borrowing issues in async contexts.

5. **Test early**: Running tests after each module helps catch errors quickly.

### Code Quality

- All methods have comprehensive doc comments
- Tests cover basic functionality (14 tests)
- Follows existing channel patterns (LINE, Telegram, etc.)
- Uses tracing for logging
- Proper error messages with context

## Next Steps (Not Implemented)

Per the issue's "Receive" requirement:
- Implement actual message receiving from webhooks
- Convert Lark events to `aisopod_channel::IncomingMessage`
- Route messages to the agent engine

## Files Created/Modified

### New Files
- `crates/aisopod-channel-lark/Cargo.toml`
- `crates/aisopod-channel-lark/src/lib.rs`
- `crates/aisopod-channel-lark/src/config.rs`
- `crates/aisopod-channel-lark/src/auth.rs`
- `crates/aisopod-channel-lark/src/api.rs`
- `crates/aisopod-channel-lark/src/cards.rs`
- `crates/aisopod-channel-lark/src/events.rs`
- `crates/aisopod-channel-lark/src/channel.rs`

### Modified Files
- `Cargo.toml` - Added `crates/aisopod-channel-lark` member

## Test Results

```
running 14 tests
test auth::tests::test_is_token_valid_no_token ... ok
test auth::tests::test_new_lark_auth ... ok
test auth::tests::test_new_feishu_auth ... ok
test cards::tests::test_simple_card ... ok
test cards::tests::test_div_card ... ok
test cards::tests::test_image_card ... ok
test cards::tests::test_card_to_json ... ok
test config::tests::test_base_url ... ok
test config::tests::test_default_config ... ok
test events::tests::test_event_type_url_verification ... ok
test events::tests::test_event_type_message ... ok
test channel::tests::test_lark_channel_new ... ok
test channel::tests::test_lark_channel_feishu ... ok
test api::tests::test_lark_api_new ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Build Status

- `cargo build`: ✅ PASS
- `cargo test --package aisopod-channel-lark`: ✅ PASS (14/14)
- All tests pass with `RUSTFLAGS=-Awarnings`

---

*Document created: 2026-02-27*
