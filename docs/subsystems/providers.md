# AI Model Provider Subsystem

**Crates:** `aisopod-provider`, `aisopod-provider-anthropic`, `aisopod-provider-openai`,
`aisopod-provider-gemini`, `aisopod-provider-bedrock`, `aisopod-provider-ollama`

## Overview

The provider subsystem abstracts AI model APIs behind a unified trait, enabling
streaming chat completions, model discovery, and transparent failover across providers.
Each concrete provider lives in its own crate to keep compile times and dependency
trees isolated.

## Key Types

- **`ModelProvider` trait** — Core abstraction: `id()`, `list_models()`,
  `chat_completion(request) -> Stream<ChatCompletionChunk>`, `health_check()`.
- **`ChatCompletionRequest`** — Normalized request: model, messages, tools,
  temperature, max_tokens, stop sequences, stream flag.
- **`ChatCompletionChunk`** — Streaming delta: id, message delta, finish reason,
  token usage.
- **`Message`** — Role + content + optional tool calls/tool_call_id.
- **`ProviderRegistry`** — Manages all registered providers with lookup by id.
- **`AuthProfileManager`** — Round-robin key rotation with cooldown tracking.

## Supported Providers

| Provider   | API                        | Key Features                      |
|------------|----------------------------|-----------------------------------|
| Anthropic  | `/v1/messages`             | Claude models, extended thinking  |
| OpenAI     | `/v1/chat/completions`     | GPT-4o, o1/o3, JSON mode         |
| Gemini     | Gemini API                 | Multi-modal, OAuth + API key auth |
| Bedrock    | AWS Bedrock Runtime        | AWS credential chain, regions     |
| Ollama     | `/api/chat`, `/api/tags`   | Local models, model pulling       |

## Auth Profile Rotation

Multiple API keys per provider rotate via round-robin with cooldown. Profiles are
marked good/failed; rotation triggers automatically on rate-limit or auth errors.
State persists across restarts.

## Dependencies

- **aisopod-config** — `ModelsConfig`, `AuthConfig` (API keys, auth profiles).
- **aisopod-shared** — Usage aggregation types.

## Design Decisions

- **One crate per provider:** Keeps `reqwest`/AWS SDK/provider-specific deps out of
  the core binary when unused; activated via Cargo feature flags.
- **`Pin<Box<dyn Stream>>` for streaming:** Provides a uniform async streaming
  interface while letting each provider manage its own SSE/chunked response parsing.
- **Request/response normalization in provider crates:** Each provider maps its
  wire format to/from the internal `Message`/`ChatCompletionChunk` types, isolating
  provider quirks (e.g., Anthropic's alternating-turn requirement).
