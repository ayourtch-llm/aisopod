# Issue 048: Add Provider Unit and Integration Tests

## Summary
Create a comprehensive test suite for the provider subsystem, including mock-based unit tests for each provider, integration tests with configurable real API endpoints, and tests for streaming behavior, error handling, and auth rotation.

## Location
- Crate: `aisopod-provider` (and each provider crate/module)
- Files: `crates/aisopod-provider/tests/`, per-provider test modules

## Current Behavior
There are no tests for the provider subsystem. Provider implementations, the registry, auth management, normalization, and model discovery are untested.

## Expected Behavior
After this issue is completed:
- Each provider (Anthropic, OpenAI, Gemini, Bedrock, Ollama) has unit tests using mock HTTP responses.
- Integration tests can be run against real API endpoints when API keys are provided via environment variables.
- Streaming behavior is tested: chunks arrive in order, the stream terminates correctly, and partial failures are handled.
- Error handling is tested: auth errors, rate limits, invalid requests, and network failures produce the correct `ProviderError` variants.
- Auth rotation is tested: the `AuthProfileManager` correctly rotates keys and respects cooldowns under simulated failures.

## Impact
Tests are essential for confidence in the provider layer. They catch regressions when provider APIs change, ensure consistent behavior across providers, and validate that error handling and auth rotation work correctly.

## Suggested Implementation
1. **Mock infrastructure** — Create a shared test helper in `crates/aisopod-provider/tests/helpers/mod.rs`:
   - Use `wiremock` or a simple `axum`-based mock server to simulate provider API responses.
   - Define helper functions to create mock SSE streams, error responses, and auth failures.
   - Create a `MockProvider` struct implementing `ModelProvider` for testing the registry, catalog, and normalization without real HTTP calls.

2. **Unit tests per provider** — In each provider's module or crate:
   - Test request serialization: verify that `ChatCompletionRequest` is correctly converted to the provider's JSON format.
   - Test response deserialization: feed mock SSE/JSON data into the response parser and verify `ChatCompletionChunk` values.
   - Test tool call handling: verify that tool definitions and tool call responses round-trip correctly.
   - Test error responses: simulate 401, 429, 400, 500 responses and verify the correct `ProviderError` is returned.

3. **Streaming tests**:
   - Test that chunks arrive in order and `finish_reason` is set on the final chunk.
   - Test that a stream that closes mid-response produces an appropriate error.
   - Test that an empty stream is handled gracefully.

4. **Registry tests** — In `crates/aisopod-provider/tests/registry.rs`:
   - Test provider registration, lookup, and listing.
   - Test alias registration and resolution.
   - Test `resolve_model()` with valid aliases and unknown names.

5. **Auth rotation tests** — In `crates/aisopod-provider/tests/auth.rs`:
   - Test round-robin rotation across multiple profiles.
   - Test that failed profiles are skipped during cooldown.
   - Test cooldown expiration and profile recovery.
   - Test behavior when all profiles are in cooldown (should return `None`).

6. **Integration tests** — In `crates/aisopod-provider/tests/integration/`:
   - Gate tests behind environment variables (e.g., `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`).
   - Use `#[ignore]` attribute so they don't run in CI by default.
   - Test a simple streaming chat completion against each real API.
   - Verify that `list_models()` returns non-empty results.
   - Verify that `health_check()` returns a healthy status.

7. Run all tests: `cargo test -p aisopod-provider` (and per-provider crates if separate).

## Dependencies
- Issue 038 (ModelProvider trait and core types)
- Issue 039 (Provider registry)
- Issue 040 (Auth profile management)
- Issue 041 (Anthropic provider)
- Issue 042 (OpenAI provider)
- Issue 043 (Gemini provider)
- Issue 044 (Bedrock provider)
- Issue 045 (Ollama provider)
- Issue 046 (Normalization layer)
- Issue 047 (Model discovery)

## Acceptance Criteria
- [ ] Each provider has unit tests covering request serialization, response deserialization, and error handling.
- [ ] Streaming tests verify chunk ordering, stream termination, and partial failure handling.
- [ ] Registry tests cover registration, lookup, listing, and alias resolution.
- [ ] Auth rotation tests cover round-robin, cooldown, and recovery.
- [ ] Integration tests exist for each provider, gated behind environment variables.
- [ ] All unit tests pass: `cargo test -p aisopod-provider`.
- [ ] Test coverage includes both success and error paths.

---
*Created: 2026-02-15*
