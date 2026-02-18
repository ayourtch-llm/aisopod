# Issue 041: Implement Anthropic Claude Provider

## Summary
Implement the `ModelProvider` trait for the Anthropic Messages API, supporting streaming SSE chat completions, tool use / function calling, system prompt handling, vision (image) support, and API key authentication.

## Location
- Crate: `aisopod-provider-anthropic` (new crate, or module at `crates/aisopod-provider/src/providers/anthropic.rs`)
- File: `crates/aisopod-provider/src/providers/anthropic.rs` (if module approach)

## Current Behavior
There is no Anthropic provider implementation. The system cannot communicate with the Anthropic Messages API.

## Expected Behavior
After this issue is completed:
- An `AnthropicProvider` struct implements `ModelProvider`.
- Chat completions are sent to `/v1/messages` with streaming SSE responses parsed into `ChatCompletionChunk` values.
- Tool use (function calling) is supported via Anthropic's tool-use format.
- System prompts are sent using Anthropic's dedicated `system` parameter (not as a message).
- Vision support allows `ContentPart::Image` to be converted to Anthropic's image format.
- API key authentication is handled via the `x-api-key` header.

## Impact
Anthropic Claude is a primary model provider for the system. This implementation enables all Claude model interactions, including Opus, Sonnet, and Haiku variants.

## Suggested Implementation
1. If creating a new crate, run `cargo new --lib crates/aisopod-provider-anthropic` and add it to the workspace `Cargo.toml`. Otherwise, create the module directory `crates/aisopod-provider/src/providers/` with a `mod.rs` and `anthropic.rs`.
2. Add dependencies: `reqwest` (with `stream` and `json` features), `tokio`, `serde`, `serde_json`, `futures-core`, `async-trait`, `tracing`, and the `aisopod-provider` crate (for the trait and types).
3. Define `AnthropicProvider`:
   ```rust
   pub struct AnthropicProvider {
       client: reqwest::Client,
       api_key: String,
       base_url: String, // default "https://api.anthropic.com"
   }
   ```
4. Implement `ModelProvider` for `AnthropicProvider`:
   - `id()` → `"anthropic"`.
   - `list_models()` → return a hardcoded or API-fetched list of Claude models with capability metadata.
   - `chat_completion()`:
     a. Convert `ChatCompletionRequest` into the Anthropic request JSON format.
     b. Extract any `Role::System` message and place its text into the top-level `system` field.
     c. Convert `ToolDefinition` entries into Anthropic's `tools` array format.
     d. Convert `ContentPart::Image` into Anthropic's `image` content block with `type`, `media_type`, and base64 `data`.
     e. POST to `{base_url}/v1/messages` with headers `x-api-key`, `anthropic-version: 2023-06-01`, `Content-Type: application/json`.
     f. Parse the SSE stream line by line, deserializing `event: content_block_delta`, `message_delta`, and `message_stop` events into `ChatCompletionChunk`.
     g. Return the chunk stream as `Pin<Box<dyn Stream<…>>>`.
   - `health_check()` → send a minimal request or check the API endpoint availability.
5. Write Anthropic-specific request/response serde types in a private submodule (e.g., `anthropic/api_types.rs`) to handle the Anthropic JSON format.
6. Add tracing spans and log statements for request/response lifecycle.
7. Run `cargo check` to confirm compilation.

## Dependencies
- Issue 038 (ModelProvider trait and core types)
- Issue 039 (Provider registry — for registration)
- Issue 040 (Auth profile management — for API key rotation)

## Acceptance Criteria
- [x] `AnthropicProvider` implements `ModelProvider`.
- [x] Streaming SSE responses from `/v1/messages` are parsed into `ChatCompletionChunk` values.
- [x] Tool use / function calling is supported in requests and responses.
- [x] System prompts are sent via Anthropic's `system` parameter, not as a message.
- [x] Vision (image) content is correctly formatted for Anthropic's API.
- [x] API key is sent via the `x-api-key` header.
- [x] `cargo check` passes for the provider crate/module.

## Resolution

The Anthropic Claude provider was implemented as a module under `aisopod-provider` crate:

**Files Created/Modified:**
- `crates/aisopod-provider/src/providers/anthropic.rs` - Main provider implementation
- `crates/aisopod-provider/src/providers/anthropic/api_types.rs` - Private submodule for Anthropic API types
- `crates/aisopod-provider/src/providers/mod.rs` - Module declaration
- `crates/aisopod-provider/src/lib.rs` - Public exports

**Implementation Highlights:**
- `AnthropicProvider` struct with `reqwest::Client`, API key, and base URL
- `ModelProvider` trait implementation with all required methods
- `chat_completion()` - streaming SSE implementation with proper parsing
- `list_models()` - returns hardcoded list of Claude models (Opus, Sonnet, Haiku)
- `health_check()` - verifies API endpoint availability
- System prompt extraction from `Role::System` messages to top-level `system` field
- Tool use support via Anthropic's `tools` array format
- Vision support via `ContentPart::Image` to Anthropic image format conversion
- API key authentication via `x-api-key` header

**Test Coverage:**
- `test_extract_system_prompt` - validates system prompt extraction
- `test_build_anthropic_request` - validates request JSON construction
- `test_convert_message_text` - validates message text conversion

**Verification:**
- `cargo check`: ✅ Passed
- `cargo build`: ✅ Passed
- `cargo test -p aisopod-provider`: ✅ 49 tests passed (including 3 Anthropic-specific tests)

---
*Created: 2026-02-15*
*Resolved: 2026-02-18*
