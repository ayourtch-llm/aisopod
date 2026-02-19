# Issue 054: Implement Message Sending Tool

## Summary
Implement a built-in message sending tool that allows agents to send messages to channels via the channel system, targeting a specific channel, account, or peer, with support for text content.

## Location
- Crate: `aisopod-tools`
- File: `crates/aisopod-tools/src/builtins/message.rs`

## Current Behavior
No message sending tool exists in the crate. Agents have no way to proactively send messages to channels.

## Expected Behavior
After this issue is completed:
- The `MessageTool` struct implements the `Tool` trait.
- It accepts parameters: `channel` (string, required), `content` (string, required), optional `account` (string), and optional `peer` (string).
- The tool sends the message through the channel system (the actual channel dispatch will be wired up when the channel subsystem is integrated; for now, define the interface and use a pluggable sender).
- The tool returns a `ToolResult` confirming the message was sent or reporting an error.

## Impact
Message sending is how agents communicate back to users or other systems through channels like Telegram, Discord, or Slack. This tool is essential for any agent that needs to proactively reach out.

## Suggested Implementation
1. **Create `message.rs`** — Add `crates/aisopod-tools/src/builtins/message.rs`.

2. **Define a `MessageSender` trait** (or use an existing channel abstraction):
   ```rust
   #[async_trait]
   pub trait MessageSender: Send + Sync {
       async fn send_message(
           &self,
           channel: &str,
           account: Option<&str>,
           peer: Option<&str>,
           content: &str,
       ) -> Result<()>;
   }
   ```
   This trait will be implemented by the channel subsystem later.

3. **Define `MessageTool`**:
   ```rust
   pub struct MessageTool {
       sender: Arc<dyn MessageSender>,
   }
   ```

4. **Implement `Tool` for `MessageTool`**:
   - `name()` → `"message"`
   - `description()` → `"Send a message to a channel"`
   - `parameters_schema()` → JSON Schema with `channel`, `content`, `account`, and `peer` properties.
   - `execute()`:
     1. Parse the required `channel` and `content` parameters.
     2. Parse optional `account` and `peer` parameters.
     3. Call `self.sender.send_message(...)`.
     4. Return a `ToolResult` with a confirmation message or error.

5. **Create a no-op `MessageSender` implementation** for testing and initial development.

6. **Register the tool** — Ensure the tool can be registered with the `ToolRegistry`.

7. **Verify** — Run `cargo check -p aisopod-tools`.

## Dependencies
- Issue 049 (Tool trait and core types)
- Issue 050 (Tool registry)

## Acceptance Criteria
- [x] `MessageTool` implements the `Tool` trait.
- [x] Messages can be sent to a specified channel with text content.
- [x] Optional `account` and `peer` targeting is supported.
- [x] `parameters_schema()` returns a valid JSON Schema.
- [x] A no-op sender exists for testing.
- [x] `cargo check -p aisopod-tools` compiles without errors.
- [x] `cargo build` passes at top level.
- [x] `cargo test` passes at top level.

## Resolution
This issue has been completed. The message sending tool was implemented with the following changes:

### Changes Made
1. **Created `crates/aisopod-tools/src/builtins/message.rs`** with:
   - `MessageSender` trait defining the interface for message sending backends
   - `MessageTool` struct implementing the `Tool` trait
   - `NoOpMessageSender` for testing scenarios

2. **Tool Implementation**:
   - `name()` returns `"message"`
   - `description()` returns `"Send a message to a channel"`
   - `parameters_schema()` returns JSON Schema with `channel`, `content` (required), `account` (optional), and `peer` (optional)
   - `execute()` parses parameters, validates required fields, and calls the sender

3. **Tool Registration**:
   - Added to `builtins/mod.rs` exports
   - Registered in `register_all_tools()` function in `lib.rs`
   - Exposed in the crate's public API

4. **Testing**:
   - Unit tests for tool name, description, schema
   - Tests for successful message sending
   - Tests for missing parameter validation
   - Tests for no-op sender

5. **Verification**:
   - `cargo build` passes without errors (using `RUSTFLAGS=-Awarnings`)
   - `cargo test` passes all tests including message-specific tests
   - All integration tests pass

---
*Created: 2026-02-15*
*Resolved: 2026-02-19*
