# Issue 042: Implement OpenAI Provider

## Summary
Implement the `ModelProvider` trait for the OpenAI Chat Completions API, supporting streaming SSE responses, function calling / tool use, vision support, JSON mode, and organization header support.

## Location
- Crate: `aisopod-provider-openai` (new crate, or module at `crates/aisopod-provider/src/providers/openai.rs`)
- File: `crates/aisopod-provider/src/providers/openai.rs` (if module approach)

## Current Behavior
There is no OpenAI provider implementation. The system cannot communicate with the OpenAI Chat Completions API.

## Expected Behavior
After this issue is completed:
- An `OpenAIProvider` struct implements `ModelProvider`.
- Chat completions are sent to `/v1/chat/completions` with streaming SSE responses parsed into `ChatCompletionChunk` values.
- Function calling / tool use is supported via OpenAI's `tools` parameter.
- Vision support allows image content parts to be sent as OpenAI image URLs or base64 data.
- JSON mode can be enabled via `response_format`.
- An optional `OpenAI-Organization` header is included when configured.

## Impact
OpenAI is one of the most widely used model providers. This integration enables GPT-4, GPT-4o, o1, and o3 model access for all agent interactions.

## Suggested Implementation
1. If creating a new crate, run `cargo new --lib crates/aisopod-provider-openai` and add it to the workspace `Cargo.toml`. Otherwise, create `crates/aisopod-provider/src/providers/openai.rs`.
2. Add dependencies: `reqwest` (with `stream` and `json` features), `tokio`, `serde`, `serde_json`, `futures-core`, `async-trait`, `tracing`, and the `aisopod-provider` crate.
3. Define `OpenAIProvider`:
   ```rust
   pub struct OpenAIProvider {
       client: reqwest::Client,
       api_key: String,
       base_url: String,        // default "https://api.openai.com"
       organization: Option<String>,
   }
   ```
4. Implement `ModelProvider` for `OpenAIProvider`:
   - `id()` → `"openai"`.
   - `list_models()` → call `GET /v1/models` and map results to `ModelInfo`.
   - `chat_completion()`:
     a. Convert `ChatCompletionRequest` into the OpenAI request JSON.
     b. Map `ToolDefinition` to OpenAI's `tools` array with `type: "function"`.
     c. Map `MessageContent` variants to OpenAI's content format (string or array of content parts).
     d. POST to `{base_url}/v1/chat/completions` with headers `Authorization: Bearer {api_key}`, `Content-Type: application/json`, and optionally `OpenAI-Organization`.
     e. Parse the SSE stream (`data: {...}` lines), deserializing each chunk into `ChatCompletionChunk`.
     f. Handle the `[DONE]` sentinel to close the stream.
     g. Return the chunk stream as `Pin<Box<dyn Stream<…>>>`.
   - `health_check()` → call `GET /v1/models` and check for a successful response.
5. Support JSON mode by setting `response_format: { "type": "json_object" }` when requested.
6. Write OpenAI-specific serde types in a private submodule.
7. Run `cargo check` to confirm compilation.

## Dependencies
- Issue 038 (ModelProvider trait and core types)
- Issue 039 (Provider registry)
- Issue 040 (Auth profile management)

## Acceptance Criteria
- [ ] `OpenAIProvider` implements `ModelProvider`.
- [ ] Streaming SSE responses from `/v1/chat/completions` are parsed into `ChatCompletionChunk` values.
- [ ] The `[DONE]` sentinel correctly terminates the stream.
- [ ] Function calling / tool use is supported in requests and responses.
- [ ] Vision content parts are formatted correctly for OpenAI.
- [ ] JSON mode is supported via `response_format`.
- [ ] `OpenAI-Organization` header is included when configured.
- [ ] `cargo check` passes for the provider crate/module.

---
*Created: 2026-02-15*
