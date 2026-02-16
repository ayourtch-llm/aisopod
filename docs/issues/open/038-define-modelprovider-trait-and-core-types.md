# Issue 038: Define ModelProvider Trait and Core Types

## Summary
Define the `ModelProvider` trait and all supporting request/response types that form the foundation of the AI model provider abstraction layer. Every concrete provider (Anthropic, OpenAI, Gemini, Bedrock, Ollama) will implement this trait.

## Location
- Crate: `aisopod-provider`
- File: `crates/aisopod-provider/src/trait.rs` (trait), `crates/aisopod-provider/src/types.rs` (types)

## Current Behavior
The `aisopod-provider` crate exists as a skeleton with no provider trait or message types defined. There is no abstraction for communicating with AI model APIs.

## Expected Behavior
After this issue is completed the crate will expose:
- A `ModelProvider` async trait with methods `id()`, `list_models()`, `chat_completion()` (returning a streaming response), and `health_check()`.
- Supporting types: `ChatCompletionRequest`, `ChatCompletionChunk`, `Message`, `MessageDelta`, `FinishReason`, `TokenUsage`, `ToolCall`, `ToolDefinition`, `Role`, `MessageContent`, `ModelInfo`, and `ProviderHealth`.
- Streaming chat completions use `Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>`.

## Impact
This trait is the single integration point for every AI provider. All downstream features — agent orchestration, session management, tool calling — depend on these types being stable and well-documented.

## Suggested Implementation
1. Add dependencies to `crates/aisopod-provider/Cargo.toml`:
   - `async-trait` for the async trait definition.
   - `futures-core` for the `Stream` trait.
   - `pin-project-lite` for ergonomic pin projections if needed.
   - `serde` and `serde_json` with the `derive` feature for serialization.
2. Create `crates/aisopod-provider/src/types.rs`:
   - Define `Role` as an enum: `System`, `User`, `Assistant`, `Tool`.
   - Define `MessageContent` as an enum supporting `Text(String)` and `Parts(Vec<ContentPart>)` (for multi-modal).
   - Define `ContentPart` enum with `Text { text: String }` and `Image { media_type: String, data: String }` variants.
   - Define `Message` struct with fields `role`, `content`, `tool_calls: Option<Vec<ToolCall>>`, `tool_call_id: Option<String>`.
   - Define `ToolDefinition` struct with `name`, `description`, and `parameters` (as `serde_json::Value`).
   - Define `ToolCall` struct with `id`, `name`, and `arguments` (as `String`).
   - Define `ChatCompletionRequest` with `model`, `messages`, `tools`, `temperature`, `max_tokens`, `stop`, and `stream` fields.
   - Define `ChatCompletionChunk` with `id`, `delta`, `finish_reason`, and `usage` fields.
   - Define `MessageDelta` with `role: Option<Role>`, `content: Option<String>`, `tool_calls: Option<Vec<ToolCall>>`.
   - Define `FinishReason` enum: `Stop`, `Length`, `ToolCall`, `ContentFilter`, `Error`.
   - Define `TokenUsage` with `prompt_tokens`, `completion_tokens`, `total_tokens` (all `u32`).
   - Define `ModelInfo` with `id`, `name`, `provider`, `context_window: u32`, `supports_vision: bool`, `supports_tools: bool`.
   - Define `ProviderHealth` with `available: bool` and `latency_ms: Option<u64>`.
   - Derive `Debug`, `Clone`, `Serialize`, `Deserialize` on all types where appropriate.
3. Create `crates/aisopod-provider/src/trait.rs`:
   ```rust
   #[async_trait]
   pub trait ModelProvider: Send + Sync {
       fn id(&self) -> &str;
       async fn list_models(&self) -> Result<Vec<ModelInfo>>;
       async fn chat_completion(
           &self,
           request: ChatCompletionRequest,
       ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>>;
       async fn health_check(&self) -> Result<ProviderHealth>;
   }
   ```
4. Re-export the trait and all types from `crates/aisopod-provider/src/lib.rs`.
5. Add doc-comments (`///`) to every public type, field, variant, and method.
6. Run `cargo check -p aisopod-provider` to confirm everything compiles.

## Dependencies
- Issue 004 (Create `aisopod-provider` crate)
- Issue 016 (Core configuration types)

## Acceptance Criteria
- [ ] `ModelProvider` trait is defined with `id()`, `list_models()`, `chat_completion()`, and `health_check()`.
- [ ] `chat_completion()` returns `Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>`.
- [ ] All supporting types (`ChatCompletionRequest`, `ChatCompletionChunk`, `Message`, `MessageDelta`, `FinishReason`, `TokenUsage`, `ToolCall`, `ToolDefinition`, `Role`, `MessageContent`) are defined.
- [ ] `async-trait`, `futures-core`, and `pin-project-lite` are listed in dependencies.
- [ ] Every public item has a doc-comment.
- [ ] `cargo check -p aisopod-provider` passes with no errors.

---
*Created: 2026-02-15*
