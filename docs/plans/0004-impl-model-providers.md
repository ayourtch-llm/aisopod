# 0004 — AI Model Provider Integrations

**Master Plan Reference:** Section 3.5 — AI Model Provider Integrations  
**Phase:** 2 (Core Runtime)  
**Dependencies:** 0001 (Project Structure), 0002 (Configuration System)

---

## Objective

Implement the provider abstraction layer and concrete integrations for AI model
providers, supporting streaming chat completions, model discovery, auth management,
and failover.

---

## Deliverables

### 1. Provider Trait (`aisopod-provider`)

Define the core abstraction:

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Provider identifier (e.g., "anthropic", "openai")
    fn id(&self) -> &str;

    /// List available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// Send a chat completion request (streaming)
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>>;

    /// Check if the provider is healthy/available
    async fn health_check(&self) -> Result<ProviderHealth>;
}
```

**Supporting types:**
```rust
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
}

pub struct ChatCompletionChunk {
    pub id: String,
    pub delta: MessageDelta,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<TokenUsage>,
}

pub struct Message {
    pub role: Role,
    pub content: MessageContent,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}
```

### 2. Anthropic Claude Provider (`aisopod-provider-anthropic`)

- Messages API (`/v1/messages`) with streaming
- Support for Claude models (Opus, Sonnet, Haiku)
- Tool use / function calling
- System prompt handling
- Vision (image) support
- Token counting and usage reporting
- API key authentication
- Extended thinking / chain-of-thought support

### 3. OpenAI Provider (`aisopod-provider-openai`)

- Chat completions API (`/v1/chat/completions`) with streaming
- Support for GPT-4, GPT-4o, o1, o3 models
- Function calling / tool use
- Vision support
- JSON mode
- API key authentication
- Organization header support

### 4. Google Gemini Provider (`aisopod-provider-gemini`)

- Gemini API with streaming
- Support for Gemini Pro, Ultra models
- Function calling
- Multi-modal (text + image)
- OAuth authentication flow
- API key authentication

### 5. AWS Bedrock Provider (`aisopod-provider-bedrock`)

- Bedrock Runtime API with streaming
- Support for Anthropic models via Bedrock
- AWS credential chain (env, profile, IAM role)
- Region configuration

### 6. Ollama Provider (`aisopod-provider-ollama`)

- Ollama REST API (`/api/chat`) with streaming
- Local model discovery (`/api/tags`)
- Model pulling support
- Configurable endpoint URL

### 7. Auth Profile Management

Port the auth profile rotation system:
- Multiple API keys per provider
- Round-robin selection with cooldown tracking
- Mark good/failed profiles
- Automatic rotation on rate limit or auth errors
- Profile state persistence

### 8. Model Discovery

- Provider-specific model listing
- Model capability metadata (context window, vision, tools)
- Cached model catalog with refresh
- Model alias resolution

### 9. Request/Response Normalization

- Normalize between provider-specific formats and internal format
- Handle provider-specific quirks (message ordering for Anthropic, etc.)
- Map error codes to standard error types
- Token usage aggregation across providers

---

## Acceptance Criteria

- [ ] Provider trait is well-defined with full async streaming support
- [ ] Anthropic provider handles streaming chat completions with tools
- [ ] OpenAI provider handles streaming chat completions with tools
- [ ] Gemini provider handles streaming chat completions
- [ ] Bedrock provider handles streaming via AWS SDK
- [ ] Ollama provider works with local models
- [ ] Auth profile rotation works across multiple API keys
- [ ] Model discovery returns accurate capability metadata
- [ ] Error handling maps provider errors to standard types
- [ ] Integration tests verify streaming behavior for each provider
- [ ] Token usage is tracked and reported accurately
