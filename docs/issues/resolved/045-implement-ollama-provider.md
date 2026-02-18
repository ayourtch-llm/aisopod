# Issue 045: Implement Ollama Provider

## Summary
Implement the `ModelProvider` trait for the Ollama REST API, supporting streaming chat completions, local model discovery, and a configurable endpoint URL for connecting to locally running Ollama instances.

## Location
- Crate: `aisopod-provider`
- File: `crates/aisopod-provider/src/providers/ollama.rs`

## Current Behavior
There was no Ollama provider implementation. The system could not communicate with a local Ollama instance.

## Expected Behavior
After this issue is completed:
- An `OllamaProvider` struct implements `ModelProvider`.
- Chat completions are sent to `/api/chat` with streaming newline-delimited JSON responses parsed into `ChatCompletionChunk` values.
- Available local models are discovered by querying `/api/tags`.
- The base URL is configurable (defaulting to `http://localhost:11434`).

## Impact
Ollama enables fully local, offline AI model usage with no API keys or cloud dependencies. This is important for development, privacy-sensitive deployments, and environments without internet access.

## Suggested Implementation
1. Create the module file at `crates/aisopod-provider/src/providers/ollama.rs`.
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
     a. Convert `ChatCompletionRequest` to Ollama's chat request JSON format.
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

## Resolution

The Ollama provider was implemented with the following components:

### Files Created
- **`crates/aisopod-provider/src/providers/ollama.rs`** - New file containing the OllamaProvider implementation

### Files Modified
- **`crates/aisopod-provider/src/providers/mod.rs`** - Added `ollama` module declaration

### Implementation Details

#### OllamaProvider
- `client: reqwest::Client` - HTTP client for API requests
- `base_url: String` - Base URL for Ollama API (default: `"http://localhost:11434"`)

#### Methods Implemented
- `new(base_url: Option<String>)` - Constructor with optional custom URL
- `convert_message(&self, message: &Message)` - Converts core Message to OllamaMessage
- `convert_tool(&self, tool: &ToolDefinition)` - Converts core ToolDefinition to OllamaFunction
- `build_ollama_request(&self, request: &ChatCompletionRequest)` - Builds OllamaRequest from core request
- `convert_role(role: OllamaRole) -> Role` - Converts Ollama role to core Role type
- `parse_ollama_chunk(line: &str)` - Parses Ollama chunk response into ChatCompletionChunk

#### ModelProvider Trait Implementation
- `id()` - Returns `"ollama"` as the provider ID
- `list_models()` - Queries `/api/tags` endpoint and returns ModelInfo vec
- `chat_completion()` - Sends request to `/api/chat` with streaming response parsing
- `health_check()` - Queries `/api/tags` to verify provider availability

#### API Types (private module)
- `OllamaRole` - Role enum with serde renames for lowercase serialization
- `OllamaMessage` - Message struct for Ollama API
- `OllamaFunction` - Function definition for tool calling
- `OllamaTool` - Tool enum (currently only Function support)
- `OllamaRequest` - Request struct for Ollama API
- `OllamaChatCompletionChunk` - Response chunk from streaming API
- `OllamaTagsResponse` - Response from `/api/tags`
- `OllamaModelInfo` - Model information from tags endpoint

#### Tests Added
- `test_convert_message_text` - Tests basic message conversion
- `test_convert_message_system` - Tests system role conversion
- `test_convert_message_parts` - Tests multi-part content handling
- `test_build_ollama_request` - Tests request building from core request
- `test_parse_ollama_chunk` - Tests chunk parsing for streaming response
- `test_parse_ollama_chunk_done` - Tests chunk parsing with finish reason and usage

### Acceptance Criteria Met

- [x] `OllamaProvider` implements `ModelProvider`
- [x] Streaming responses from `/api/chat` are parsed into `ChatCompletionChunk` values
- [x] `list_models()` queries `/api/tags` and returns discovered models as `ModelInfo`
- [x] Base URL is configurable, defaulting to `http://localhost:11434`
- [x] `cargo check -p aisopod-provider` passes
- [x] `cargo build` passes at top level
- [x] `cargo test -p aisopod-provider` passes (77 tests)
- [x] All tests pass at top level

---
*Created: 2026-02-15*
*Resolved: 2026-02-18*
