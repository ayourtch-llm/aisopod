# Issue 043: Implement Google Gemini Provider

## Summary
Implement the `ModelProvider` trait for the Google Gemini API, supporting streaming responses, function calling, multi-modal (text and image) input, and both API key and OAuth authentication.

## Location
- Crate: `aisopod-provider-gemini` (new crate, or module at `crates/aisopod-provider/src/providers/gemini.rs`)
- File: `crates/aisopod-provider/src/providers/gemini.rs` (if module approach)

## Current Behavior
There is no Google Gemini provider implementation. The system cannot communicate with the Gemini API.

## Expected Behavior
After this issue is completed:
- A `GeminiProvider` struct implements `ModelProvider`.
- Chat completions are sent to the Gemini `generateContent` (or `streamGenerateContent`) endpoint with streaming responses parsed into `ChatCompletionChunk` values.
- Function calling is supported via Gemini's `tools` and `function_declarations` format.
- Multi-modal input (text + images) is converted to Gemini's `parts` format.
- Both API key authentication (`?key=`) and OAuth bearer token authentication are supported.

## Impact
Google Gemini provides access to Gemini Pro and Ultra models. This integration broadens the set of available AI backends and enables multi-modal use cases.

## Suggested Implementation
1. If creating a new crate, run `cargo new --lib crates/aisopod-provider-gemini` and add it to the workspace `Cargo.toml`. Otherwise, create `crates/aisopod-provider/src/providers/gemini.rs`.
2. Add dependencies: `reqwest` (with `stream` and `json` features), `tokio`, `serde`, `serde_json`, `futures-core`, `async-trait`, `tracing`, and the `aisopod-provider` crate.
3. Define `GeminiProvider`:
   ```rust
   pub struct GeminiProvider {
       client: reqwest::Client,
       api_key: Option<String>,
       oauth_token: Option<String>,
       base_url: String, // default "https://generativelanguage.googleapis.com"
   }
   ```
4. Implement `ModelProvider` for `GeminiProvider`:
   - `id()` → `"gemini"`.
   - `list_models()` → call `GET /v1beta/models` and map results to `ModelInfo`.
   - `chat_completion()`:
     a. Convert `ChatCompletionRequest` messages to Gemini's `contents` format with `parts`.
     b. Map `Role::User` → `"user"`, `Role::Assistant` → `"model"`, handle system instructions via `system_instruction` field.
     c. Convert `ToolDefinition` entries to Gemini's `function_declarations` format.
     d. Convert `ContentPart::Image` to Gemini's `inline_data` part with `mime_type` and base64 `data`.
     e. POST to `{base_url}/v1beta/models/{model}:streamGenerateContent` with the appropriate auth (API key as query param or OAuth bearer header).
     f. Parse the streaming JSON response, extracting `candidates[0].content.parts` deltas and mapping them to `ChatCompletionChunk`.
     g. Return the chunk stream as `Pin<Box<dyn Stream<…>>>`.
   - `health_check()` → call `GET /v1beta/models` and verify a successful response.
5. Handle Gemini's unique streaming format (newline-delimited JSON array) by parsing each JSON object as it arrives.
6. Write Gemini-specific serde types in a private submodule.
7. Run `cargo check` to confirm compilation.

## Dependencies
- Issue 038 (ModelProvider trait and core types)
- Issue 039 (Provider registry)
- Issue 040 (Auth profile management)

## Acceptance Criteria
- [x] `GeminiProvider` implements `ModelProvider`.
- [x] Streaming responses from `streamGenerateContent` are parsed into `ChatCompletionChunk` values.
- [x] Function calling is supported via Gemini's `function_declarations` format.
- [x] Multi-modal input (text + images) is correctly formatted as Gemini `parts`.
- [x] API key authentication works via query parameter.
- [x] OAuth bearer token authentication works via `Authorization` header.
- [x] `cargo check` passes for the provider crate/module.

## Resolution

The Google Gemini provider was implemented as a module under `aisopod-provider` crate:

**Files Created:**
- `crates/aisopod-provider/src/providers/gemini.rs` - Main provider implementation
- `crates/aisopod-provider/src/providers/gemini/api_types.rs` - Private submodule for Gemini API types
- `crates/aisopod-provider/src/providers/mod.rs` - Module declaration

**Features Implemented:**
- `GeminiProvider` struct implementing `ModelProvider` trait
- Streaming responses from `/v1beta/models/{model}:streamGenerateContent`
- Function calling via `function_declarations` format
- Multi-modal input (text + images) via `parts` format
- API key authentication via query parameter (`?key=`)
- OAuth bearer token authentication via `Authorization` header
- Streaming JSON parsing into `ChatCompletionChunk` values

**Tests Implemented:**
- `test_gemini_part_serialization`
- `test_gemini_function_declaration_serialization`
- `test_gemini_content_serialization`
- `test_gemini_model_info_serialization`
- `test_gemini_request_serialization`
- `test_gemini_role_serialization`

**Verification:**
- `cargo check`: ✅ Passed
- `cargo build`: ✅ Passed
- `cargo test -p aisopod-provider`: ✅ 64 tests passed (6 Gemini-specific)
- `cargo test` (all crates): ✅ 272 tests passed

**Rust Version Update:**
The project's Rust version was updated from 1.90.0 to 1.93.1 via `rust-toolchain.toml` to support newer dependencies.

---
*Created: 2026-02-15*
*Resolved: 2026-02-18*
