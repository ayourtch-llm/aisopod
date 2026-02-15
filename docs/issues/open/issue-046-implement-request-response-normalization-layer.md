# Issue 046: Implement Request/Response Normalization Layer

## Summary
Build a normalization layer that converts between the internal message format and each provider's API-specific format, handles provider quirks, maps provider error codes to standard error types, and aggregates token usage across providers.

## Location
- Crate: `aisopod-provider`
- File: `crates/aisopod-provider/src/normalize.rs`

## Current Behavior
Each provider implementation independently converts between internal types and provider-specific formats. There is no shared normalization logic, which leads to duplicated code and inconsistent error handling.

## Expected Behavior
After this issue is completed:
- A `NormalizationLayer` (or set of utility functions) provides shared conversion logic between internal types and provider-specific formats.
- Anthropic-specific quirks are handled (e.g., enforcing alternating user/assistant message turns, merging consecutive same-role messages).
- Provider error codes (HTTP status codes and error response bodies) are mapped to a standard `ProviderError` enum.
- Token usage from provider responses is aggregated into a consistent `TokenUsage` struct regardless of provider.

## Impact
Normalization ensures consistent behavior across all providers, reduces duplication in provider implementations, and gives callers a uniform error handling experience.

## Suggested Implementation
1. Create `crates/aisopod-provider/src/normalize.rs`.
2. Define a standard error type:
   ```rust
   pub enum ProviderError {
       AuthenticationFailed { provider: String, message: String },
       RateLimited { provider: String, retry_after: Option<Duration> },
       InvalidRequest { provider: String, message: String },
       ModelNotFound { provider: String, model: String },
       ContextLengthExceeded { provider: String, max_tokens: u32 },
       ServerError { provider: String, status: u16, message: String },
       NetworkError { source: reqwest::Error },
       StreamClosed,
       Unknown { provider: String, message: String },
   }
   ```
3. Implement `map_http_error(provider: &str, status: u16, body: &str) -> ProviderError`:
   - 401 / 403 → `AuthenticationFailed`
   - 429 → `RateLimited` (parse `retry-after` header if present)
   - 400 → `InvalidRequest`
   - 404 → `ModelNotFound`
   - 5xx → `ServerError`
4. Implement message normalization utilities:
   - `enforce_alternating_turns(messages: &mut Vec<Message>)` — merge consecutive messages with the same role (required by Anthropic).
   - `extract_system_prompt(messages: &mut Vec<Message>) -> Option<String>` — remove and return any `Role::System` message for providers that handle system prompts separately.
5. Implement token usage aggregation:
   - `aggregate_usage(chunks: &[ChatCompletionChunk]) -> TokenUsage` — sum up token usage from a sequence of streaming chunks, using the last chunk's usage if only one reports it.
6. Re-export `ProviderError` and normalization functions from `lib.rs`.
7. Add unit tests:
   - Test that alternating-turn enforcement merges consecutive same-role messages.
   - Test that system prompt extraction works correctly.
   - Test that HTTP status codes map to the correct `ProviderError` variant.
   - Test token usage aggregation.
8. Run `cargo check -p aisopod-provider` and `cargo test -p aisopod-provider`.

## Dependencies
- Issue 038 (ModelProvider trait and core types)
- Issue 041 (Anthropic provider — drives quirk requirements)
- Issue 042 (OpenAI provider — drives format requirements)
- Issue 043 (Gemini provider — drives format requirements)

## Acceptance Criteria
- [ ] `ProviderError` enum covers authentication, rate-limiting, invalid request, model not found, context length, server error, network error, and stream-closed cases.
- [ ] `map_http_error()` correctly maps HTTP status codes to `ProviderError` variants.
- [ ] `enforce_alternating_turns()` merges consecutive same-role messages.
- [ ] `extract_system_prompt()` removes and returns the system message.
- [ ] `aggregate_usage()` produces correct totals from streaming chunks.
- [ ] Unit tests pass for all normalization functions.
- [ ] `cargo check -p aisopod-provider` passes.

---
*Created: 2026-02-15*
