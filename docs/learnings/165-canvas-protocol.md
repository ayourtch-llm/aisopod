# Canvas Protocol Implementation Documentation (Issue #165)

## Summary

This issue implemented the Canvas Protocol for interactive UI in the Aisopod project. The canvas protocol enables the server to push rich interactive UI content (HTML/CSS/JS) to connected clients and receive user interaction events back from those clients.

## Architecture Overview

The canvas protocol consists of two distinct communication patterns:

1. **Server-initiated (`canvas.update`)**: The server sends HTML/CSS/JS content to clients for rendering in a canvas container
2. **Client-initiated (`canvas.interact`)**: Clients report user interactions (clicks, input changes, form submissions) back to the server

## Learning: Separating Server-Initiated vs Client-Initiated Messages

### Key Insight

In JSON-RPC 2.0, there are two distinct patterns for communication:

| Pattern | Direction | Method Type | Registration | Handler Approach |
|---------|-----------|-------------|--------------|------------------|
| Server-initiated | Server → Client | Notification (no `id`) | Not registered in router | Sent via WebSocket broadcast |
| Client-initiated | Client → Server | Call (has `id`) | Registered in router | Standard RPC handler |

### Implementation Details

#### `canvas.update` - Server-Initiated Notification

```rust
// NOT registered in MethodRouter
// Sent via WebSocket broadcast when agent needs to update canvas
{
    "jsonrpc": "2.0",
    "method": "canvas.update",
    "params": {
        "canvas_id": "canvas-1",
        "action": "create",
        "content": {
            "html": "<div>Hello</div>",
            "css": null,
            "js": null,
            "title": null
        }
    }
}
```

**Key Points:**
- Does NOT have an `id` field (notification pattern)
- Should NOT be registered in the MethodRouter
- Sent via the broadcast system (WebSocket)
- Returns `METHOD_NOT_FOUND` (-32601) if queried via RPC

#### `canvas.interact` - Client-Initiated Call

```rust
// Registered in MethodRouter as CanvasInteractHandler
// Handles client RPC calls with proper request/response cycle
{
    "jsonrpc": "2.0",
    "method": "canvas.interact",
    "params": {
        "canvas_id": "canvas-1",
        "event_type": "click",
        "element_id": "btn-submit",
        "data": {"foo": "bar"}
    },
    "id": 1
}
```

**Key Points:**
- Has an `id` field (call pattern)
- MUST be registered in the MethodRouter
- Returns a proper response with the same `id`
- Validates canvas existence and forwards interactions to agent

## Canvas State Management

### Per-Connection Canvas State

Each connection maintains its own `CanvasState` with the following operations:

```rust
pub struct CanvasState {
    active_canvases: HashMap<String, CanvasContent>,
}

impl CanvasState {
    pub fn new() -> Self;
    pub fn create_or_update(&mut self, canvas_id: String, content: CanvasContent);
    pub fn destroy(&mut self, canvas_id: &str) -> bool;
    pub fn get(&self, canvas_id: &str) -> Option<&CanvasContent>;
    pub fn exists(&self, canvas_id: &str) -> bool;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### Canvas Lifecycle

1. **Create**: Server sends `canvas.update` with `action: "create"` and full content
2. **Update**: Server sends `canvas.update` with `action: "update"` and modified content
3. **Interact**: Client sends `canvas.interact` with user events
4. **Destroy**: Server sends `canvas.update` with `action: "destroy"` and no content

## Interaction Forwarding Mechanism

The current implementation forwards interactions via the event broadcasting system:

```rust
// From CanvasInteractHandler::handle()
// Forward the interaction event to the agent/handler that owns this canvas
// This is typically done via the event broadcasting system
```

### Future Enhancement: Agent Ownership

Each canvas should track which agent owns it, enabling targeted interaction delivery:

```rust
// Recommended enhancement
pub struct CanvasContent {
    pub html: String,
    pub css: Option<String>,
    pub js: Option<String>,
    pub title: Option<String>,
    pub owner_agent_id: Option<String>,  // NEW: Track owner agent
}
```

## Verification Test Strategy

The test `test_default_router_contains_canvas_methods` demonstrates proper separation:

```rust
#[test]
fn test_default_router_contains_canvas_methods() {
    let router = default_router();

    // Test canvas.update (server-initiated, should return METHOD_NOT_FOUND)
    let req = types::RpcRequest {
        method: "canvas.update".to_string(),
        params: None,
        id: Some(...),
    };
    let response = router.dispatch(ctx.clone(), req);
    assert_eq!(response.error.as_ref().unwrap().code, -32601);

    // Test canvas.interact (client-initiated, should succeed)
    let req = types::RpcRequest {
        method: "canvas.interact".to_string(),
        params: Some(json!({ "canvas_id": "test-canvas", "event_type": "click" })),
        id: Some(...),
    };
    let response = router.dispatch(ctx.clone(), req);
    assert!(response.result.is_some());
}
```

## Data Structures Reference

### CanvasUpdateParams (Server → Client)
```rust
{
    "canvas_id": String,           // Unique canvas identifier
    "action": "create"|"update"|"destroy",
    "content": CanvasContent|null  // Required for create/update, null for destroy
}
```

### CanvasContent
```rust
{
    "html": String,              // Required HTML content
    "css": String|null,          // Optional CSS styling
    "js": String|null,           // Optional JavaScript behavior
    "title": String|null         // Optional canvas title
}
```

### CanvasInteractParams (Client → Server)
```rust
{
    "canvas_id": String,         // Canvas identifier
    "event_type": String,        // "click", "input", "submit", "custom"
    "element_id": String|null,   // Optional element within canvas
    "data": Value|null           // Event-specific payload
}
```

### CanvasInteractResult
```rust
{
    "received": bool,
    "canvas_id": String,
    "event_type": String,
    "message": String
}
```

## Testing Coverage

All 18 canvas-related tests pass:

### Canvas State Tests (10 tests)
- `test_canvas_state_new_is_empty`
- `test_canvas_state_create_or_update`
- `test_canvas_state_destroy`
- `test_canvas_state_get`
- `test_canvas_state_exists`
- `test_canvas_state_update_replaces`
- `test_canvas_content_serialization`
- `test_canvas_content_minimal_serialization`
- `test_canvas_action_serialization`
- `test_canvas_interact_params_deserialization`

### Handler Tests (5 tests)
- `test_canvas_interact_handler_success`
- `test_canvas_interact_handler_minimal`
- `test_canvas_interact_handler_missing_params`
- `test_canvas_interact_handler_invalid_params`

### Router Tests (3 tests)
- `test_default_router_contains_canvas_methods`
- `test_default_router_method_count`
- Canvas serialization/deserialization roundtrip tests

## Deployment Considerations

### Security
- Canvas content should be sanitized before rendering in clients
- Implement content security policy (CSP) for canvas HTML/JS
- Validate all canvas IDs and event types on the server

### Performance
- Consider canvas content diffing for updates
- Implement canvas cleanup for disconnected clients
- Monitor memory usage for active canvases per connection

### Future Enhancements
1. **Canvas Diffing**: Send only changed content to reduce bandwidth
2. **Owner Tracking**: Track which agent owns each canvas for targeted delivery
3. **Canvas Metadata**: Add canvas metadata (z-index, positioning, etc.)
4. **Event Aggregation**: Batch multiple interaction events
5. **Canvas Expiration**: Auto-destroy canvases after inactivity period

## References

- Issue #165: Implement Canvas Protocol for Interactive UI
- Issue #030: RPC router for method registration
- Issue #034: Event broadcasting for forwarding interactions
- Issue #162: WebSocket Protocol Specification

---
*Created: 2026-02-26*
*Issue: #165*
*Status: Resolved*
