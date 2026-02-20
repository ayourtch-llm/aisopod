# Issue 065: Implement Message Transcript Repair

## Summary
Implement provider-specific message transcript repair to handle the different turn-ordering requirements of Anthropic, OpenAI, and Google Gemini. Insert synthetic messages where needed to produce valid message sequences for each provider.

## Location
- Crate: `aisopod-agent`
- File: `crates/aisopod-agent/src/transcript.rs`

## Current Behavior
No transcript repair logic exists. Raw message sequences from sessions may violate provider-specific ordering rules, causing API errors.

## Expected Behavior
After this issue is completed:
- A `repair_transcript()` function takes a message list and a target provider, then returns a corrected message list.
- Anthropic repair: ensures strictly alternating user/assistant turns; merges or splits consecutive same-role messages.
- OpenAI/Gemini repair: handles their respective turn rules (OpenAI is more flexible; Gemini has specific ordering requirements).
- Synthetic messages (e.g., a placeholder user message like "[continued]") are inserted to fill gaps in the sequence.
- The repair process preserves the semantic content of the original messages.

## Impact
Without transcript repair, the agent will encounter API errors from providers that enforce strict turn ordering. This is especially critical for Anthropic, which rejects requests with consecutive same-role messages. Repair logic ensures reliable execution regardless of the session's raw message history.

## Suggested Implementation
1. **Create `crates/aisopod-agent/src/transcript.rs`:**
   - Define a `TranscriptRepairer` or a standalone function:
     ```rust
     pub fn repair_transcript(
         messages: &[Message],
         provider: ProviderKind,
     ) -> Vec<Message>
     ```
   - Define `ProviderKind` enum (or reuse from provider crate): `Anthropic`, `OpenAI`, `Google`, `Other`.

2. **Anthropic repair strategy:**
   - Walk through the message list.
   - If two consecutive messages have the same role, insert a synthetic message with the opposite role between them.
   - Ensure the sequence starts with a user message (insert a synthetic one if it starts with assistant).
   - Synthetic messages should have a recognizable marker (e.g., content: `"[continued]"`, or a metadata flag).

3. **OpenAI repair strategy:**
   - OpenAI is more flexible, but still check for edge cases.
   - Ensure system messages are at the beginning.
   - Remove or merge duplicate system messages.

4. **Gemini repair strategy:**
   - Handle Gemini's specific turn ordering rules.
   - Ensure alternating user/model turns similar to Anthropic.

5. **Fallback / `Other` strategy:**
   - Pass through messages unchanged (no-op repair).

6. **Update `crates/aisopod-agent/src/lib.rs`:**
   - Add `pub mod transcript;`.

7. **Add unit tests:**
   - Test Anthropic repair with consecutive user messages → synthetic assistant inserted.
   - Test Anthropic repair with consecutive assistant messages → synthetic user inserted.
   - Test Anthropic repair with a sequence starting with assistant → synthetic user prepended.
   - Test OpenAI repair preserves system message at the start.
   - Test Gemini repair with alternating turn violations.
   - Test that a valid sequence passes through unchanged for each provider.
   - Test that synthetic messages have recognizable markers.

8. **Verify** — Run `cargo test -p aisopod-agent`.

## Dependencies
- Issue 062 (Agent types — provides `Message` type)
- Issue 038 (ModelProvider trait and core types — provides provider kind information)

## Acceptance Criteria
- [x] `repair_transcript()` function exists and handles Anthropic, OpenAI, and Gemini providers.
- [x] Anthropic repair enforces alternating user/assistant turns.
- [x] Synthetic messages are inserted where needed with recognizable markers.
- [x] Valid sequences pass through unchanged.
- [x] Unit tests cover all repair strategies and edge cases.
- [x] `cargo check -p aisopod-agent` succeeds without errors.

## Resolution

### Changes Made

**File: `crates/aisopod-agent/src/transcript.rs`**
- Created complete `repair_transcript()` function that takes a message slice and target provider
- Implemented `ProviderKind` enum with variants: `Anthropic`, `OpenAI`, `Google`, `Other`
- Implemented `repair_for_anthropic()` - ensures strictly alternating user/assistant turns
  - Prepends synthetic user message if sequence starts with assistant
  - Inserts synthetic assistant message between consecutive user messages
  - Inserts synthetic user message between consecutive assistant messages
- Implemented `repair_for_openai()` - handles OpenAI's more flexible requirements
  - Ensures system messages are at the start
  - Merges multiple system messages into one with combined content
- Implemented `repair_for_google()` - similar to Anthropic for Gemini requirements
- Implemented helper functions: `synthetic_user_message()` and `synthetic_assistant_message()`
  - Both create messages with `"[continued]"` text marker for easy identification
- Implemented comprehensive unit tests (16 test cases) covering all scenarios

**File: `crates/aisopod-agent/src/lib.rs`**
- Added `pub mod transcript;`
- Exported `repair_transcript` and `ProviderKind` from crate root

### Test Results
All 108 tests in `aisopod-agent` crate pass, including 16 new transcript repair tests covering:
- Anthropic: consecutive messages, start/end edge cases, valid sequences, empty sequences
- OpenAI: system message ordering, multiple system message deduplication
- Gemini: similar to Anthropic with proper user/model alternation
- Other provider: pass-through behavior

### Verification
```bash
$ RUSTFLAGS=-Awarnings cargo build
$ RUSTFLAGS=-Awarnings cargo test -p aisopod-agent
$ cargo test -p aisopod-agent transcript::tests  # 16 passed
```

---
*Created: 2026-02-15*
*Resolved: 2026-02-20*
