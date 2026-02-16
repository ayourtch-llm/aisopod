# Issue 068: Implement Session Compaction Strategies

## Summary
Implement adaptive history compaction strategies to manage context window size: adaptive chunking, summary-based compaction, hard clear, and oversized tool result truncation. Include a `ContextWindowGuard` that monitors usage against configurable thresholds.

## Location
- Crate: `aisopod-agent`
- Files: `crates/aisopod-agent/src/compaction.rs`, `crates/aisopod-agent/src/context_guard.rs`

## Current Behavior
No compaction logic exists. Long conversations will exceed the model's context window, causing API errors or degraded performance.

## Expected Behavior
After this issue is completed:
- **Adaptive chunking** splits history into summarizable chunks based on topic boundaries or message count.
- **Summary strategy** summarizes older messages while keeping recent ones intact.
- **Hard clear** drops the oldest messages beyond a configurable threshold.
- **Oversized tool result truncation** trims tool outputs that exceed a size limit.
- `ContextWindowGuard` monitors token usage with a `warn_threshold` (e.g., 80%) and a `hard_limit`, triggering compaction automatically.
- The compaction system preserves the most semantically important content.

## Impact
Without compaction, agents cannot handle long conversations or sessions with large tool outputs. Compaction ensures the agent can continue operating within the model's context window while retaining the most relevant information.

## Suggested Implementation
1. **Create `crates/aisopod-agent/src/compaction.rs`:**
   - Define a `CompactionStrategy` enum:
     ```rust
     pub enum CompactionStrategy {
         AdaptiveChunking,
         Summary,
         HardClear { keep_recent: usize },
         ToolResultTruncation { max_chars: usize },
     }
     ```
   - Implement `compact_messages(messages: &[Message], strategy: CompactionStrategy) -> Vec<Message>`:
     - **AdaptiveChunking:** Group older messages into chunks. Replace each chunk with a summary message (the actual summarization call can be stubbed or delegated to the model).
     - **Summary:** Keep the last N messages as-is. Replace everything before them with a single summary message.
     - **HardClear:** Keep only the most recent `keep_recent` messages. Drop the rest.
     - **ToolResultTruncation:** Walk through messages. For any tool result content exceeding `max_chars`, truncate it and append a `[truncated]` marker.
   - Implement a `select_strategy(guard: &ContextWindowGuard, token_count: usize) -> CompactionStrategy`:
     - If token count exceeds `hard_limit`, use `HardClear`.
     - If token count exceeds `warn_threshold`, use `Summary`.
     - If individual tool results are oversized, use `ToolResultTruncation`.
     - Otherwise, use `AdaptiveChunking` for gentle compaction.

2. **Create `crates/aisopod-agent/src/context_guard.rs`:**
   - Define `ContextWindowGuard`:
     ```rust
     pub struct ContextWindowGuard {
         pub warn_threshold: f64,    // e.g., 0.8 (80% of max)
         pub hard_limit: usize,      // Absolute token limit
         pub min_available: usize,   // Minimum tokens reserved for response
     }
     ```
   - Implement `ContextWindowGuard::from_config(config: &AgentConfig) -> Self`.
   - Implement `needs_compaction(current_tokens: usize) -> bool`.
   - Implement `severity(current_tokens: usize) -> CompactionSeverity` returning `None`, `Warn`, or `Critical`.

3. **Integration point:**
   - The execution pipeline (Issue 066) calls `ContextWindowGuard::needs_compaction()` before each model call.
   - If compaction is needed, select a strategy and apply it to the message history.
   - On context overflow errors during failover (Issue 067), trigger compaction and retry.

4. **Update `crates/aisopod-agent/src/lib.rs`:**
   - Add `pub mod compaction;` and `pub mod context_guard;`.

5. **Add unit tests:**
   - Test `HardClear` with `keep_recent: 5` on a 20-message list → 5 messages remain.
   - Test `ToolResultTruncation` with a 10,000-char tool result and `max_chars: 1000` → truncated with marker.
   - Test `Summary` keeps recent messages and replaces older ones with a summary placeholder.
   - Test `ContextWindowGuard::needs_compaction()` returns true when over threshold.
   - Test `select_strategy()` returns appropriate strategy based on severity.

6. **Verify** — Run `cargo test -p aisopod-agent`.

## Dependencies
- Issue 066 (Streaming agent execution pipeline — integration point)
- Issue 016 (Core configuration types — provides context window settings)

## Acceptance Criteria
- [ ] All four compaction strategies are implemented: adaptive chunking, summary, hard clear, tool result truncation.
- [ ] `ContextWindowGuard` monitors token usage against configurable thresholds.
- [ ] Compaction preserves the most recent and semantically important messages.
- [ ] Tool result truncation adds a `[truncated]` marker.
- [ ] Unit tests verify each strategy and the context guard logic.
- [ ] `cargo check -p aisopod-agent` succeeds without errors.

---
*Created: 2026-02-15*
