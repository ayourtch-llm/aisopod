# Issue 044: Implement AWS Bedrock Provider

## Summary
Implement the `ModelProvider` trait for the AWS Bedrock Runtime API, supporting streaming completions via the AWS SDK for Rust and the full AWS credential chain (environment variables, profile, IAM role).

## Location
- Crate: `aisopod-provider`
- Module: `crates/aisopod-provider/src/providers/bedrock.rs`
- API Types: `crates/aisopod-provider/src/providers/bedrock/api_types.rs`

## Current Behavior
There is no AWS Bedrock provider implementation. The system cannot invoke AI models hosted on AWS Bedrock.

## Expected Behavior
After this issue is completed:
- A `BedrockProvider` struct implements `ModelProvider`.
- Chat completions are sent via the Bedrock Runtime `InvokeModelWithResponseStream` API.
- AWS credentials are resolved using the standard credential chain (environment variables, `~/.aws/credentials` profile, IAM role).
- The AWS region is configurable.
- Streaming response chunks are parsed into `ChatCompletionChunk` values.

## Impact
AWS Bedrock provides access to Anthropic Claude and other models within an AWS-managed environment. This integration is essential for enterprise deployments that require AWS-native credential management and network isolation.

## Suggested Implementation
1. If creating a new crate, run `cargo new --lib crates/aisopod-provider-bedrock` and add it to the workspace `Cargo.toml`. Otherwise, create the module file.
2. Add dependencies:
   - `aws-sdk-bedrockruntime` for the Bedrock Runtime client.
   - `aws-config` for credential and region resolution.
   - `aws-types` for shared AWS types.
   - `tokio`, `serde`, `serde_json`, `futures-core`, `async-trait`, `tracing`.
   - The `aisopod-provider` crate.
3. Define `BedrockProvider`:
   ```rust
   pub struct BedrockProvider {
       client: aws_sdk_bedrockruntime::Client,
       region: String,
   }
   ```
4. Implement a constructor:
   - Load AWS configuration with `aws_config::load_defaults(BehaviorVersion::latest()).await`.
   - Optionally override the region from the provider configuration.
   - Build the Bedrock Runtime client from the config.
5. Implement `ModelProvider` for `BedrockProvider`:
   - `id()` → `"bedrock"`.
   - `list_models()` → return a curated list of Bedrock-supported models with capability metadata (or call the Bedrock `ListFoundationModels` API if available).
   - `chat_completion()`:
     a. Convert `ChatCompletionRequest` to the Bedrock Converse API format or the model-specific input format.
     b. Call `invoke_model_with_response_stream` with the model ID and serialized body.
     c. Consume the `EventStream` from the SDK, mapping each `PayloadPart` to a `ChatCompletionChunk`.
     d. Return the chunk stream as `Pin<Box<dyn Stream<…>>>`.
   - `health_check()` → attempt a lightweight API call (e.g., list models) to verify credentials and connectivity.
6. Handle AWS-specific errors (expired credentials, throttling) and map them to standard provider error types.
7. Run `cargo check` to confirm compilation.

## Dependencies
- Issue 038 (ModelProvider trait and core types)
- Issue 039 (Provider registry)

## Resolution
The AWS Bedrock provider has been implemented as a module under `aisopod-provider` crate with the following files:

1. **`crates/aisopod-provider/src/providers/bedrock.rs`**: Main implementation with `BedrockProvider` struct that implements `ModelProvider` trait. Features:
   - Async constructor with AWS credential chain resolution via `aws-config`
   - Region configuration support
   - Streaming chat completions via `InvokeModelWithResponseStream` API
   - AWS-specific error handling (expired credentials, throttling, access denied, model timeout)
   - Health check implementation
   - Model listing with curated list of Bedrock-supported models

2. **`crates/aisopod-provider/src/providers/bedrock/api_types.rs`**: Private submodule with AWS Bedrock-specific types:
   - Request types: `BedrockRequest`, `BedrockMessage`, `BedrockContentBlock`, `BedrockTool`, etc.
   - Response types: `BedrockStreamEvent`, `BedrockStreamEvent` variants
   - All types implement `Serialize` and `Deserialize`

3. **Updated `crates/aisopod-provider/src/providers/mod.rs`**: Added `bedrock` module export

4. **Updated `crates/aisopod-provider/Cargo.toml`**: Added `async-stream = "0.3"` dependency for stream construction

Key implementation details:
- Uses `aws_config::load_defaults(BehaviorVersion::v2024_03_28())` for credential chain resolution
- Region is configurable via `Region::new(Cow::Owned(String))` pattern to satisfy 'static lifetime requirement
- Streaming uses `async-stream::stream!` macro to convert `EventReceiver` to `Stream`
- Error mapping handles AWS-specific errors: ExpiredToken, Throttling, AccessDenied, ModelTimeout
- All tests pass including provider creation, serialization tests, and integration tests

## Acceptance Criteria
- [x] `BedrockProvider` implements `ModelProvider`.
- [x] AWS credentials are resolved via the standard credential chain.
- [x] Region is configurable.
- [x] Streaming responses from `InvokeModelWithResponseStream` are parsed into `ChatCompletionChunk` values.
- [x] AWS-specific errors (expired credentials, throttling) are handled gracefully.
- [x] `cargo check` passes for the provider crate/module.
- [x] `cargo build` passes at top level.
- [x] `cargo test` passes at top level.

---
*Created: 2026-02-15*
*Resolved: 2026-02-18*
