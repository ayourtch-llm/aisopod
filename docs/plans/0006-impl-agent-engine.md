# 0006 — Agent Execution Engine

**Master Plan Reference:** Section 3.4 — Agent Execution Engine  
**Phase:** 3 (Agent Engine)  
**Dependencies:** 0002 (Configuration), 0004 (Model Providers), 0005 (Tool System)

---

## Objective

Port the AI agent execution engine, managing agent lifecycle, streaming execution,
model failover, session compaction, and subagent spawning.

---

## Deliverables

### 1. Agent Runner (`aisopod-agent`)

Core execution entry point:

```rust
pub struct AgentRunner {
    config: Arc<AisopodConfig>,
    providers: Arc<ProviderRegistry>,
    tools: Arc<ToolRegistry>,
    sessions: Arc<SessionStore>,
}

impl AgentRunner {
    /// Run an agent with the given messages, returning a streaming result
    pub async fn run(
        &self,
        params: AgentRunParams,
    ) -> Result<AgentRunStream> { ... }

    /// Subscribe to agent events for a session
    pub fn subscribe(
        &self,
        session_key: &str,
    ) -> broadcast::Receiver<AgentEvent> { ... }

    /// Abort a running agent
    pub async fn abort(&self, session_key: &str) -> Result<()> { ... }
}
```

### 2. Agent Resolution

Port the agent scope resolution system:

- `resolve_session_agent_id()` — Map session key to agent via bindings
- `resolve_agent_config()` — Get per-agent configuration from agents list
- `resolve_agent_model()` — Resolve primary model with fallback chain
- `list_agent_ids()` — Enumerate all configured agents

**Agent binding evaluation:**
```rust
pub struct AgentBinding {
    pub agent_id: String,
    pub match_rule: BindingMatch,
}

pub struct BindingMatch {
    pub channel: Option<String>,
    pub account_id: Option<String>,
    pub peer: Option<PeerMatch>,
    pub guild_id: Option<String>,
}
```

### 3. Execution Pipeline

```
AgentRunParams (messages, session_key, agent_id)
  → Resolve agent config
  → Resolve model (primary + fallbacks)
  → Prepare tool set (filtered by policy)
  → Build system prompt
  → Repair message transcript (provider-specific turn rules)
  → Attempt loop:
    ├── Call AI model with streaming
    ├── Handle tool calls (execute → return result)
    ├── Check for failures (auth, rate limit, context overflow)
    ├── Failover to next model on error
    └── Compact history if context too large
  → Stream events to subscribers
  → Save usage, session metadata
  → Return final result
```

### 4. Streaming Execution

- Use `tokio::mpsc` channels for event streaming
- Event types:
  - `TextDelta` — Incremental text from model
  - `ToolCallStart` — Tool invocation beginning
  - `ToolCallResult` — Tool execution result
  - `ModelSwitch` — Failover to different model
  - `Error` — Execution error
  - `Complete` — Agent run finished
  - `Usage` — Token usage report

### 5. Model Failover

Port the multi-model failover system:

```
Primary model attempt
  → Success? Return result
  → Auth error? Try next auth profile, then failover
  → Rate limit? Wait/failover
  → Context overflow? Compact, retry, or failover
  → Timeout? Failover
  → All models exhausted? Return error to user
```

**Failover state:**
```rust
pub struct FailoverState {
    pub attempted_models: Vec<ModelAttempt>,
    pub current_model_index: usize,
    pub max_attempts: usize,
}
```

### 6. Session Compaction

Port adaptive history management:

**Strategies:**
- **Adaptive chunking** — Split history into summarizable chunks
- **Summary** — Summarize older messages, keep recent
- **Hard clear** — Drop oldest messages beyond threshold
- **Oversized tool result truncation** — Trim large tool outputs

**Context window guard:**
```rust
pub struct ContextWindowGuard {
    pub warn_threshold: f64,      // e.g., 0.8 (80% of max)
    pub hard_limit: usize,        // Absolute token limit
    pub min_available: usize,     // Minimum tokens for response
}
```

### 7. System Prompt Construction

- Base system prompt from agent config
- Dynamic context injection (date/time, workspace info, channel context)
- Tool descriptions appended
- Skill instructions merged
- Memory/context retrieval results injected

### 8. Message Transcript Repair

Handle provider-specific message ordering requirements:
- Anthropic: alternating user/assistant turns, no consecutive same-role messages
- Google: specific turn ordering rules
- OpenAI: more flexible but still has constraints
- Insert synthetic messages to repair invalid sequences

### 9. Subagent Support

- Spawn child agents within parent session
- Depth limit enforcement (prevent infinite recursion)
- Allowlist control for subagent models
- Thread ID propagation for context sharing
- Resource budget inheritance

### 10. Usage Tracking

- Per-request token usage (input + output)
- Per-session cumulative usage
- Per-agent usage aggregation
- Usage reporting via events and API

---

## Acceptance Criteria

- [ ] Agent runs with streaming text output
- [ ] Tool calls are executed and results returned to model
- [ ] Model failover switches to fallback on errors
- [ ] Session compaction manages context window size
- [ ] System prompt is correctly constructed with all context
- [ ] Message transcript repair handles all provider formats
- [ ] Subagent spawning respects depth limits
- [ ] Agent abort stops execution promptly
- [ ] Usage tracking reports accurate token counts
- [ ] Integration tests cover full agent execution cycle
- [ ] Stress tests verify concurrent agent execution
