# Issue 045: Implement Ollama Provider

## Summary
Implement the `ModelProvider` trait for the Ollama REST API, supporting streaming chat completions, local model discovery, and a configurable endpoint URL for connecting to locally running Ollama instances.

## Location
- Crate: `aisopod-provider-ollama` (new crate, or module at `crates/aisopod-provider/src/providers/ollama.rs`)
- File: `crates/aisopod-provider/src/providers/ollama.rs` (if module approach)

## Current Behavior
There is no Ollama provider implementation. The system cannot communicate with a local Ollama instance.

## Expected Behavior
After this issue is completed:
- An `OllamaProvider` struct implements `ModelProvider`.
- Chat completions are sent to `/api/chat` with streaming newline-delimited JSON responses parsed into `ChatCompletionChunk` values.
- Available local models are discovered by querying `/api/tags`.
- The base URL is configurable (defaulting to `http://localhost:11434`).

## Impact
Ollama enables fully local, offline AI model usage with no API keys or cloud dependencies. This is important for development, privacy-sensitive deployments, and environments without internet access.

## Suggested Implementation
1. If creating a new crate, run `cargo new --lib crates/aisopod-provider-ollama` and add it to the workspace `Cargo.toml`. Otherwise, create the module file.
2. Add dependencies: `reqwest` (with `stream` and `json` features), `tokio`, `serde`, `serde_json`, `futures-core`, `async-trait`, `tracing`, and the `aisopod-provider` crate.
3. Define `OllamaProvider`:
   ```rust
   pub struct OllamaProvider {
       client: reqwest::Client,
       base_url: String, // default "http://localhost:11434"
   }
   ```
4. Implement a constructor:
   - `new(base_url: Option<String>) -> Self` — use the provided URL or default to `http://localhost:11434`.
5. Implement `ModelProvider` for `OllamaProvider`:
   - `id()` → `"ollama"`.
   - `list_models()`:
     a. GET `{base_url}/api/tags`.
     b. Parse the JSON response to extract model names and metadata.
     c. Map each model to a `ModelInfo` struct (context window and capabilities may need to be estimated or fetched from model metadata).
   - `chat_completion()`:
     a. Convert `ChatCompletionRequest` to Ollama's chat request JSON format:
        ```json
        {
          "model": "llama3",
          "messages": [{"role": "user", "content": "Hello"}],
          "stream": true
        }
        ```
     b. POST to `{base_url}/api/chat`.
     c. Parse the newline-delimited JSON stream. Each line is a JSON object with fields like `message.content`, `done`, and optionally token usage.
     d. Map each JSON object to a `ChatCompletionChunk`, setting `finish_reason` to `Stop` when `done` is `true`.
     e. Return the chunk stream as `Pin<Box<dyn Stream<…>>>`.
   - `health_check()` → GET `{base_url}/api/tags` and check for a successful response.
6. Write Ollama-specific serde types in a private submodule.
7. Run `cargo check` to confirm compilation.

## Dependencies
- Issue 038 (ModelProvider trait and core types)
- Issue 039 (Provider registry)

## Acceptance Criteria
- [ ] `OllamaProvider` implements `ModelProvider`.
- [ ] Streaming responses from `/api/chat` are parsed into `ChatCompletionChunk` values.
- [ ] `list_models()` queries `/api/tags` and returns discovered models as `ModelInfo`.
- [ ] Base URL is configurable, defaulting to `http://localhost:11434`.
- [ ] `cargo check` passes for the provider crate/module.

---
*Created: 2026-02-15*
