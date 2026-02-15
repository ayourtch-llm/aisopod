# Issue 096: Add Channel Abstraction Unit Tests

## Summary
Add comprehensive unit tests for all channel abstraction functionality: channel registry operations, message routing logic, security enforcement, and media handling utilities.

## Location
- Crate: `aisopod-channel`
- File: `crates/aisopod-channel/tests/` and/or `crates/aisopod-channel/src/*.rs` (inline tests)

## Current Behavior
The channel abstraction crate has implementation code but no automated tests to verify correctness.

## Expected Behavior
A full test suite exercises every public API in the `aisopod-channel` crate, covering happy paths, edge cases, and error conditions. Mock implementations of `ChannelPlugin`, `SecurityAdapter`, and other adapters are used to test routing and security in isolation.

## Impact
Without tests, regressions can be introduced silently. Tests provide confidence that the channel system behaves correctly and serve as living documentation for expected behavior.

## Suggested Implementation
1. Create a test helper `MockChannelPlugin` that implements `ChannelPlugin` with configurable ID, metadata, and capabilities. This allows tests to register channels with specific behaviors.
2. Create a test helper `MockSecurityAdapter` that implements `SecurityAdapter` with configurable allowlist and mention requirement settings.
3. **Channel registry tests** (`test_registry.rs` or inline in `registry.rs`):
   - `test_register_and_get` — register a channel, look it up by ID, verify it returns the correct plugin.
   - `test_get_unknown_channel` — look up an unregistered ID, verify `None` is returned.
   - `test_list_preserves_order` — register channels A, B, C in order, call `list()`, verify the order is A, B, C.
   - `test_normalize_id_lowercase` — register a channel with ID `"Telegram"`, look it up with `"telegram"`, verify it resolves.
   - `test_alias_resolution` — register a channel, add an alias, look up by alias, verify it resolves to the canonical plugin.
   - `test_alias_for_unknown_channel` — add an alias for a channel that doesn't exist, verify an error is returned.
   - `test_register_duplicate` — register two channels with the same ID, verify the second overwrites the first.
4. **Message routing tests** (`test_router.rs` or inline in `router.rs`):
   - `test_route_happy_path` — create a mock setup with a registered channel, valid account, and bound agent. Route a message and verify it reaches the agent runner.
   - `test_route_unknown_channel` — route a message with an unregistered channel ID, verify an error is returned.
   - `test_route_disabled_account` — route a message with a disabled account, verify an error is returned.
   - `test_route_unauthorized_sender` — route a message from a sender not on the allowlist, verify it is rejected.
   - `test_route_group_without_mention` — route a group message without @mention when mention is required, verify it is silently skipped.
   - `test_route_group_with_mention` — route a group message with @mention when mention is required, verify it is processed.
   - `test_route_dm_no_mention_required` — route a DM, verify mention requirement is not checked.
5. **Security enforcement tests** (`test_security.rs` or inline in `security.rs`):
   - `test_check_sender_allowed` — check an allowed sender, verify `Ok(())`.
   - `test_check_sender_blocked` — check a blocked sender, verify an error is returned.
   - `test_check_sender_no_adapter` — check with no security adapter, verify `Ok(())` (open access).
   - `test_check_mention_required_and_present` — verify `Allowed` when mention is found.
   - `test_check_mention_required_and_missing` — verify `SkipSilently` when mention is absent.
   - `test_check_mention_not_required` — verify `Allowed` when mention is not required.
   - `test_check_mention_dm_skips_check` — verify mention check is not applied to DMs.
   - `test_check_dm_policy_allowed` — verify an allowed sender can DM.
   - `test_check_dm_policy_blocked` — verify a blocked sender cannot DM.
6. **Media handling tests** (`test_media.rs` or inline in `media.rs`):
   - `test_detect_media_type_jpeg` — pass JPEG magic bytes, verify `MediaType::Image`.
   - `test_detect_media_type_png` — pass PNG magic bytes, verify `MediaType::Image`.
   - `test_detect_media_type_pdf` — pass PDF magic bytes, verify `MediaType::Document`.
   - `test_detect_media_type_by_extension` — pass unknown bytes with a `.mp3` filename, verify `MediaType::Audio`.
   - `test_detect_media_type_unknown` — pass unrecognized bytes with no filename, verify `MediaType::Other`.
   - `test_resize_image_larger_than_max` — create a 1000x1000 image, resize to max 500x500, verify output dimensions.
   - `test_resize_image_smaller_than_max` — create a 200x200 image, resize to max 500x500, verify dimensions are unchanged.
   - `test_validate_media_supported` — validate media with a supported type, verify `Ok(())`.
   - `test_validate_media_unsupported` — validate media with an unsupported type, verify an error is returned.
   - `test_validate_media_disabled` — validate media when `supports_media` is false, verify an error is returned.
7. Run all tests with `cargo test -p aisopod-channel` and verify they pass.

## Dependencies
- Issue 089 (define ChannelPlugin trait and channel metadata types)
- Issue 090 (define adapter interface traits)
- Issue 091 (define message types)
- Issue 092 (implement channel registry)
- Issue 093 (implement message routing pipeline)
- Issue 094 (implement security and allowlist enforcement)
- Issue 095 (implement media handling utilities)

## Acceptance Criteria
- [ ] `MockChannelPlugin` and `MockSecurityAdapter` test helpers are created
- [ ] Channel registry tests cover register, get, list, normalize, alias, and duplicate scenarios
- [ ] Message routing tests cover happy path, unknown channel, disabled account, unauthorized sender, mention required/present/absent, and DM routing
- [ ] Security enforcement tests cover sender allowlist, mention checks, DM policies, and no-adapter fallback
- [ ] Media handling tests cover type detection (magic bytes and extension), image resizing, format conversion, and validation against capabilities
- [ ] All tests use mock implementations to avoid external dependencies
- [ ] `cargo test -p aisopod-channel` passes with all tests green

---
*Created: 2026-02-15*
