# Learning: Implementing the Message Send Command (Issue #127)

## Overview

This document captures key learnings from implementing the `aisopod message` command for sending messages to agents via the gateway's WebSocket interface.

---

## Key Challenges

### 1. Gateway Architecture and Agent Dependencies

**Challenge**: The gateway's RPC system is designed as a lightweight router without agent execution dependencies.

**Current State**:
- Gateway uses `default_router()` which creates `PlaceholderHandler` instances
- Handlers have no access to `AgentRunner`, `ProviderRegistry`, `ToolRegistry`, etc.
- WebSocket connection handling is separate from agent execution

**Solution Path**:
```rust
// Need to pass dependencies through request context
struct RequestContext {
    pub conn_id: String,
    pub remote_addr: SocketAddr,
    pub agent_runner: Arc<AgentRunner>,  // ← New dependency
}
```

**Learnings**:
1. Gateway should either:
   - Have agent execution capabilities (more complex)
   - Or forward requests to an agent execution service (cleaner separation)

2. Consider using dependency injection for RPC handlers:
```rust
struct ChatSendHandler {
    runner: Arc<AgentRunner>,
    config: Arc<AisopodConfig>,
}

impl RpcMethod for ChatSendHandler {
    fn handle(&self, ctx: &RequestContext, params: Option<Value>) -> RpcResponse {
        // Has access to all needed dependencies
    }
}
```

### 2. Streaming Architecture

**Challenge**: How to stream agent events through JSON-RPC responses.

**Agent Events** (from `aisopod-agent/src/types.rs`):
```rust
pub enum AgentEvent {
    TextDelta { text: String, index: Option<usize> },
    ToolCallStart { tool_name: String, call_id: String },
    ToolCallResult { call_id: String, result: String, is_error: bool },
    ModelSwitch { from: String, to: String, reason: String },
    Error { message: String },
    Complete { result: AgentRunResult },
    Usage { usage: UsageReport },
}
```

**Required JSON-RPC Response Format**:
```json
{
  "jsonrpc": "2.0",
  "method": "chat.send",
  "params": {
    "result": {
      "text": "Hello ",
      "done": false
    }
  }
}
```

**Conversion Strategy**:
```rust
fn agent_event_to_streaming_response(event: AgentEvent) -> Value {
    match event {
        AgentEvent::TextDelta { text, .. } => {
            json!({
                "result": {
                    "text": text,
                    "done": false
                }
            })
        }
        AgentEvent::Complete { result } => {
            json!({
                "result": {
                    "text": result.response,
                    "done": true,
                    "usage": result.usage
                }
            })
        }
        // Handle other events as needed
        _ => Value::Null,
    }
}
```

**Learnings**:
1. Agent events need to be transformed for JSON-RPC streaming
2. Consider using `result` field for streaming chunks
3. Use `done: true` to signal completion
4. Include `usage` stats in final response

### 3. Session Management

**Challenge**: Each `chat.send` request needs a session for conversation history.

**Options**:
1. **New session per request**: Simple but loses context
2. **Session from params**: User provides session_key
3. **Auto-generated session**: Gateway creates and returns session_key

**Recommended Approach**:
```rust
// In MessageArgs, add optional session_id
#[derive(Args)]
pub struct MessageArgs {
    pub text: String,
    #[arg(long)]
    pub channel: Option<String>,
    #[arg(long)]
    pub agent: Option<String>,
    #[arg(long)]
    pub session_id: Option<String>,  // ← New field
}
```

**Learnings**:
1. Session ID should be optional for one-off messages
2. Gateway should return session ID if not provided
3. Store conversation in SessionStore

### 4. WebSocket Message Flow

**Current Flow**:
```
CLI → WebSocket → ws_handler → MethodRouter.dispatch → Handler.handle → Response
```

**Issue**: Each request/response is a separate WebSocket message, but streaming requires multiple messages.

**Solution**:
```rust
// Handler spawns task and sends multiple messages
tokio::spawn(async move {
    let stream = agent_runner.run(params).await?;
    
    // Send initial response acknowledging request
    ws_tx.send(Message::Text(json!({
        "jsonrpc": "2.0",
        "result": { "status": "processing" }
    }))).await?;
    
    // Stream events
    while let Some(event) = stream.recv().await {
        let response = agent_event_to_jsonrpc(event);
        ws_tx.send(Message::Text(response.to_string())).await?;
    }
});
```

**Learnings**:
1. Handlers can spawn background tasks to send multiple messages
2. Use `Arc<WebSocketSender>` for sharing across tasks
3. Consider rate limiting for streaming

---

## Code Patterns

### 1. RPC Handler Implementation

```rust
pub struct ChatSendHandler {
    agent_runner: Arc<AgentRunner>,
    config: Arc<AisopodConfig>,
}

impl ChatSendHandler {
    pub fn new(agent_runner: Arc<AgentRunner>, config: Arc<AisopodConfig>) -> Self {
        Self {
            agent_runner,
            config,
        }
    }
}

impl RpcMethod for ChatSendHandler {
    fn handle(&self, ctx: &RequestContext, params: Option<Value>) -> RpcResponse {
        // 1. Parse parameters
        let params = params.ok_or_else(|| RpcError {
            code: -32602,
            message: "Parameters required".to_string(),
            data: None,
        })?;
        
        // 2. Extract text, channel, agent from params
        let text = params.get("text")
            .and_then(|t| t.as_str())
            .ok_or_else(|| RpcError {
                code: -32602,
                message: "Missing 'text' parameter".to_string(),
                data: None,
            })?;
        
        let channel = params.get("channel").and_then(|c| c.as_str()).map(|s| s.to_string());
        let agent = params.get("agent").and_then(|a| a.as_str()).map(|s| s.to_string());
        
        // 3. Run agent and stream results
        // (Implementation details...)
        
        // 4. Return success response
        RpcResponse::success(
            params.get("id").cloned(),
            json!({"status": "processing"}),
        )
    }
}
```

### 2. Dependency Injection in Gateway

```rust
// In server.rs, modify run_with_config()
let agent_runner = create_agent_runner(&config);
let handler = ChatSendHandler::new(Arc::clone(&agent_runner), Arc::clone(&config));

// Register with dependencies
let router = MethodRouter::new();
router.register("chat.send", handler);
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use aisopod_agent::AgentRunner;
    use aisopod_config::AisopodConfig;
    use std::sync::Arc;
    
    #[test]
    fn test_chat_send_handler_initialization() {
        let config = Arc::new(AisopodConfig::default());
        let runner = Arc::new(AgentRunner::new(
            config.clone(),
            Arc::new(ProviderRegistry::new()),
            Arc::new(ToolRegistry::new()),
            Arc::new(SessionStore::new()),
        ));
        
        let handler = ChatSendHandler::new(runner, config);
        // Test handler initialization
    }
    
    #[tokio::test]
    async fn test_chat_send_handler_with_params() {
        let handler = /* create handler */;
        let ctx = RequestContext::new("test".to_string(), "127.0.0.1:8080".parse().unwrap());
        let params = json!({
            "text": "Hello",
            "channel": "telegram",
            "agent": "myagent",
            "id": 1
        });
        
        let response = handler.handle(&ctx, Some(params));
        // Test response
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_message_command_with_gateway() {
    // 1. Start gateway with agent
    let config = create_test_config();
    let addr = start_test_server(config).await;
    
    // 2. Connect via WebSocket
    let (ws_stream, _) = connect_async(format!("ws://{}/ws", addr)).await.unwrap();
    
    // 3. Send message
    let request = json!({
        "jsonrpc": "2.0",
        "method": "chat.send",
        "params": {
            "text": "Hello"
        },
        "id": 1
    });
    
    ws_stream.send(request.to_string().into()).await.unwrap();
    
    // 4. Receive streaming responses
    let mut response_text = String::new();
    while let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if msg.is_text() {
            let response: Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
            if let Some(result) = response.get("result") {
                if result.get("done").and_then(|d| d.as_bool()) == Some(true) {
                    break;
                }
                if let Some(text) = result.get("text").and_then(|t| t.as_str()) {
                    response_text.push_str(text);
                }
            }
        }
    }
    
    // 5. Verify response
    assert!(!response_text.is_empty());
}
```

---

## Future Improvements

### 1. Connection Pooling

For high-throughput scenarios, consider connection pooling:

```rust
struct ConnectionPool {
    connections: DashMap<String, WebSocketSender>,
}

impl ConnectionPool {
    fn get(&self, session_id: &str) -> Option<WebSocketSender> {
        self.connections.get(session_id).cloned()
    }
    
    fn add(&self, session_id: String, sender: WebSocketSender) {
        self.connections.insert(session_id, sender);
    }
}
```

### 2. Rate Limiting per Session

```rust
fn check_rate_limit(session_id: &str) -> bool {
    // Check if session has exceeded rate limit
    // Return true if allowed, false if rate limited
}
```

### 3. Conversation History API

```rust
// Add chat.history RPC method
pub struct ChatHistoryHandler {
    session_store: Arc<SessionStore>,
}

impl RpcMethod for ChatHistoryHandler {
    fn handle(&self, ctx: &RequestContext, params: Option<Value>) -> RpcResponse {
        let session_id = params.get("session_id").and_then(|s| s.as_str())?;
        let messages = self.session_store.get_messages(session_id)?;
        RpcResponse::success(params.get("id").cloned(), json!({ "messages": messages }))
    }
}
```

### 4. Agent Selection Priority

```rust
// Priority: explicit agent > channel-specific agent > default agent
fn select_agent(params: &Value, config: &AisopodConfig) -> Result<String> {
    if let Some(agent) = params.get("agent").and_then(|a| a.as_str()) {
        return Ok(agent.to_string());
    }
    
    if let Some(channel) = params.get("channel").and_then(|c| c.as_str()) {
        if let Some(agent) = find_channel_agent(channel, config) {
            return Ok(agent);
        }
    }
    
    Ok(config.agents.agents[0].id.clone()) // Default
}
```

---

## Debugging Tips

### 1. Enable WebSocket Logging

```rust
eprintln!("=== WS RECEIVED: {} ===", text);
eprintln!("=== WS SENT: {} ===", response_text);
```

### 2. Check Gateway Status

```bash
curl http://localhost:8080/status
```

### 3. Verify Agent Configuration

```bash
aisopod agent list
```

### 4. Test with Simple Echo Agent

Create a test agent that echoes messages to verify the flow works:

```json5
{
  "agents": [
    {
      "id": "echo",
      "name": "Echo Agent",
      "model": "gpt-4",
      "system_prompt": "You are an echo agent. Repeat the user's message exactly."
    }
  ]
}
```

---

## Summary

**What Worked**:
- CLI argument parsing with clap
- WebSocket connection and JSON-RPC request formatting
- Error handling and graceful degradation

**What Was Tricky**:
- Passing agent execution dependencies through the gateway
- Streaming agent events through JSON-RPC responses
- Session management for conversation history

**Key Learnings**:
1. Gateway architecture needs to support agent execution or delegate to a service
2. Streaming requires background tasks and multiple WebSocket messages
3. Dependencies must be passed through handler configuration or request context
4. Tests should cover both happy path and error scenarios

**Recommendations**:
1. Consider microservice architecture for gateway vs. agent execution
2. Implement chat.send with basic functionality first, add streaming later
3. Add comprehensive logging for debugging
4. Create integration tests that start a real gateway

---

*Documentation created: 2026-02-24*
*Related Issue: #127*
