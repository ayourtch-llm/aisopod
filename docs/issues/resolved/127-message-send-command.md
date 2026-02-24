# Issue 127: Implement Message Send Command

## Summary
Implement the `aisopod message` command that sends a message to an active or specified agent via the running gateway's WebSocket interface and streams the response to the terminal.

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/commands/message.rs`

## Current Behavior
The message subcommand is a stub that panics with `todo!`. There is no way to send messages to agents from the CLI.

## Expected Behavior
Users can send a text message from the command line. The CLI connects to the running gateway via WebSocket, sends a `chat.send` JSON-RPC request, and streams the agent's response back to the terminal in real time.

## Impact
This is the primary user-facing interaction command. It enables quick one-off messages to agents directly from the terminal without needing a separate chat interface.

## Suggested Implementation

1. Define the message subcommand arguments:

```rust
use clap::Args;

#[derive(Args)]
pub struct MessageArgs {
    /// Message text to send
    pub text: String,

    /// Target channel
    #[arg(long)]
    pub channel: Option<String>,

    /// Target agent ID (uses default if not specified)
    #[arg(long)]
    pub agent: Option<String>,
}
```

2. Implement the async command handler:

```rust
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;

pub async fn run(args: MessageArgs, config_path: Option<String>) -> anyhow::Result<()> {
    let config = load_config(config_path)?;
    let gateway_url = config.gateway_ws_url();

    // Connect to the running gateway via WebSocket
    let (mut ws_stream, _) = connect_async(&gateway_url).await?;

    // Build the JSON-RPC request
    let request = json!({
        "jsonrpc": "2.0",
        "method": "chat.send",
        "params": {
            "text": args.text,
            "channel": args.channel,
            "agent": args.agent,
        },
        "id": 1
    });

    // Send the request
    ws_stream.send(request.to_string().into()).await?;

    // Stream response chunks to terminal
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if msg.is_text() {
            let response: serde_json::Value = serde_json::from_str(msg.to_text()?)?;
            if let Some(chunk) = response.get("result").and_then(|r| r.get("text")) {
                print!("{}", chunk.as_str().unwrap_or(""));
            }
            if response.get("result").and_then(|r| r.get("done")).and_then(|d| d.as_bool()) == Some(true) {
                println!();
                break;
            }
        }
    }

    ws_stream.close(None).await?;
    Ok(())
}
```

3. Update the `Commands` enum and dispatch:

```rust
// In cli.rs
Message(MessageArgs),

// In main.rs
Commands::Message(args) => {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(commands::message::run(args, cli.config))?;
}
```

## Dependencies
- Issue 124 (clap CLI framework)
- Issue 029 (JSON-RPC protocol)
- Issue 066 (agent execution pipeline)

## Acceptance Criteria
- [x] `aisopod message "Hello"` sends a message to the default agent
- [x] `aisopod message --channel telegram "Hello"` sends to a specific channel
- [x] `aisopod message --agent myagent "Hello"` sends to a specific agent
- [x] Response is streamed to the terminal in real time
- [x] Graceful error when gateway is not running
- [x] WebSocket connection is cleanly closed after response completes

## Resolution

The `aisopod message` command is now fully implemented with end-to-end functionality:

### Changes Made:

1. **Gateway RPC Handler** (`crates/aisopod-gateway/src/rpc/chat.rs`):
   - Created `ChatSendHandler` with `handle_with_deps()` method
   - Added `SendMessageParams` struct with `text`, `channel`, and `agent` fields
   - Implemented `run_agent_and_stream()` function that:
     - Receives messages via WebSocket
     - Executes agents through `AgentRunner`
     - Streams responses with incremental `text` fields
     - Sends final `done: true` marker

2. **Gateway Integration** (`crates/aisopod-gateway/src/ws.rs`):
   - Added `AGENT_RUNNER_KEY` extension constant
   - Created `create_agent_runner()` function to instantiate `AgentRunner`
   - Modified `handle_connection()` to:
     - Clone WebSocket sender for chat handler
     - Handle `chat.send` requests directly with full dependencies
     - Stream responses via WebSocket with incremental updates

3. **Dependencies** (`crates/aisopod-gateway/Cargo.toml`):
   - Added `aisopod-agent`
   - Added `aisopod-provider`
   - Added `aisopod-tools`
   - Added `aisopod-session`

4. **Module Exports** (`crates/aisopod-gateway/src/rpc/mod.rs`):
   - Added `pub mod chat;`

### Implementation Notes:
- The CLI implementation in `crates/aisopod/src/commands/message.rs` was already complete
- The fix focused on implementing the missing backend `chat.send` RPC handler
- Streaming uses incremental JSON-RPC responses with text chunks and final `done: true` marker
- Error handling returns proper JSON-RPC error responses

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
