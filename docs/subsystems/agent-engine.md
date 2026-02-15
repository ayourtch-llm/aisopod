# Agent Execution Engine

**Crate:** `aisopod-agent`

## Overview

The agent engine orchestrates AI agent runs — resolving configuration, selecting
models, executing tool calls, streaming results, and managing the conversation
lifecycle including compaction and failover. It is the central runtime that ties
providers, tools, and sessions together.

## Key Types

- **`AgentRunner`** — Main entry point. Holds `Arc` references to config, provider
  registry, tool registry, and session store. Methods: `run()`, `subscribe()`, `abort()`.
- **`AgentRunParams`** — Input to a run: messages, session key, agent ID.
- **`AgentRunStream`** — Async stream of `AgentEvent` values.
- **`AgentEvent`** — Enum: `TextDelta`, `ToolCallStart`, `ToolCallResult`,
  `ModelSwitch`, `Error`, `Complete`, `Usage`.
- **`FailoverState`** — Tracks attempted models and current index for multi-model
  failover.
- **`ContextWindowGuard`** — Configurable warn threshold, hard limit, and minimum
  available tokens for response generation.

## Execution Pipeline

1. Resolve agent config and model (primary + fallback chain) via bindings
2. Prepare filtered tool set per agent policy
3. Build system prompt (base + dynamic context + tool descriptions + memory)
4. Repair message transcript for provider-specific turn rules
5. **Attempt loop:**
   - Stream chat completion from provider
   - Execute any tool calls, return results to model
   - On auth/rate-limit/context-overflow error → failover to next model
   - Compact history if context window exceeded
6. Stream events to subscribers via `tokio::mpsc`
7. Persist usage and session metadata

## Model Failover

Primary → fallback 1 → fallback 2 → fallback 3. Auth profile rotation is attempted
before switching models. Context overflow triggers compaction then retry before
failover.

## Session Compaction

Strategies (configurable per agent):
- **Summary** — LLM-generated summary replaces older messages.
- **Hard clear** — Drop oldest messages beyond a token threshold.
- **Oversized tool result truncation** — Trim large tool outputs inline.

## Message Transcript Repair

Inserts synthetic messages to satisfy provider constraints (e.g., Anthropic requires
strictly alternating user/assistant turns; Gemini has its own ordering rules).

## Subagent Spawning

Child agents run within the parent session with enforced depth limits and an
allowlist controlling which models subagents may use. Thread IDs propagate for
context sharing.

## Dependencies

- **aisopod-config** — Agent definitions, binding rules, compaction settings.
- **aisopod-provider** — `ModelProvider` trait and `ProviderRegistry`.
- **aisopod-tools** — `ToolRegistry` for tool execution.
- **aisopod-session** — Session history storage and retrieval.
- **aisopod-memory** — Pre-query relevant memories for context injection.

## Design Decisions

- **`tokio::mpsc` for event streaming:** Backpressure-aware channels let multiple
  subscribers (WebSocket clients, logging) consume agent events independently.
- **Failover as a state machine:** `FailoverState` makes the retry logic testable
  and deterministic, avoiding ad-hoc retry loops.
- **Transcript repair at the engine level:** Centralizing provider-specific message
  fixups in the agent engine keeps provider crates focused on wire-format concerns.
