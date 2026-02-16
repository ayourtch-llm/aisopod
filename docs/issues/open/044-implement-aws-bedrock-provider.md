# Issue 044: Implement AWS Bedrock Provider

## Summary
Implement the `ModelProvider` trait for the AWS Bedrock Runtime API, supporting streaming completions via the AWS SDK for Rust and the full AWS credential chain (environment variables, profile, IAM role).

## Location
- Crate: `aisopod-provider-bedrock` (new crate, or module at `crates/aisopod-provider/src/providers/bedrock.rs`)
- File: `crates/aisopod-provider/src/providers/bedrock.rs` (if module approach)

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

## Acceptance Criteria
- [ ] `BedrockProvider` implements `ModelProvider`.
- [ ] AWS credentials are resolved via the standard credential chain.
- [ ] Region is configurable.
- [ ] Streaming responses from `InvokeModelWithResponseStream` are parsed into `ChatCompletionChunk` values.
- [ ] AWS-specific errors (expired credentials, throttling) are handled gracefully.
- [ ] `cargo check` passes for the provider crate/module.

---
*Created: 2026-02-15*
