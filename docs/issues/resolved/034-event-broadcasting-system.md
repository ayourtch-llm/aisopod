# Issue 034: Implement Event Broadcasting System

## Summary
Build a publish-subscribe event system that allows the gateway to broadcast real-time events to all connected WebSocket clients or to a filtered subset based on per-client subscriptions.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/broadcast.rs`

## Current Behavior
There is no mechanism to push events from the server to connected clients. Each WebSocket connection operates in a request-response pattern only.

## Expected Behavior
- A `tokio::broadcast` channel carries gateway events.
- Event types include: presence changes, health updates, agent lifecycle events, and chat events.
- Each connected client subscribes to the broadcast channel on connect.
- Clients can filter which event types they wish to receive.
- Events are serialized as JSON-RPC notification messages (no `id` field) before being sent over the WebSocket.

## Impact
Broadcasting is essential for real-time features such as live presence indicators, agent status updates in the web UI, and chat message streaming. Without it, clients must poll for updates.

## Suggested Implementation
1. Create `crates/aisopod-gateway/src/broadcast.rs`:
   - Define an event enum:
     ```rust
     #[derive(Clone, Serialize)]
     #[serde(tag = "type")]
     pub enum GatewayEvent {
         Presence { conn_id: String, status: String },
         Health { snapshot: HealthSnapshot },
         Agent { agent_id: String, event: String },
         Chat { room_id: String, message: Value },
     }
     ```
   - Create a `Broadcaster` struct wrapping `tokio::sync::broadcast::Sender<GatewayEvent>`.
   - Implement `pub fn publish(&self, event: GatewayEvent)`.
   - Implement `pub fn subscribe(&self) -> broadcast::Receiver<GatewayEvent>`.
2. In each client's WebSocket task:
   - Call `broadcaster.subscribe()` on connect.
   - Spawn a forwarding loop that reads from the receiver and writes to the WebSocket sender.
   - Before sending, check the client's subscription filter; skip events the client has not opted into.
3. Define a `Subscription` struct on `GatewayClient` with a set of event types the client cares about. Default to all types.
4. Add an RPC method (e.g., `gateway.subscribe`) that lets clients update their subscription filter at runtime.
5. Write unit tests: publish an event, verify all subscribers receive it; verify filtered subscribers do not.

## Dependencies
- Issue 033 (Client connection management)

## Acceptance Criteria
- [x] `GatewayEvent` enum covers presence, health, agent, and chat events.
- [x] `Broadcaster` can publish events to all subscribers.
- [x] Each WebSocket client receives broadcast events as JSON-RPC notifications.
- [x] Per-client subscription filtering works correctly.
- [x] Disconnected clients stop receiving events without errors.
- [x] Unit tests verify broadcast delivery and filtering.

## Resolution
The event broadcasting system was implemented in `crates/aisopod-gateway/src/broadcast.rs`:

### Subscription
```rust
pub struct Subscription {
    pub event_types: HashSet<String>,
}
```
- Default subscription includes: "presence", "health", "agent", "chat"
- `includes(event_type)` method to check if event type is subscribed

### GatewayEvent
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum GatewayEvent {
    Presence { conn_id: String, status: String },
    Health { snapshot: HealthSnapshot },
    Agent { agent_id: String, event: String },
    Chat { room_id: String, message: Value },
}
```
- Event types with type tag for serialization
- Implements Clone, Serialize, Deserialize

### Broadcaster
```rust
pub struct Broadcaster {
    sender: broadcast::Sender<GatewayEvent>,
}
```
Methods:
- `new() - Create new broadcaster with 16-message capacity
- `publish(event)` - Send event to all subscribers
- `subscribe()` - Create new receiver for events

### Integration into ws.rs
The WebSocket handler was updated to:
1. Extract Broadcaster from request extensions
2. Subscribe to broadcast channel on connect
3. Spawn broadcast forwarding loop that checks subscription filter
4. Serialize events as JSON-RPC notifications (no id field)
5. Stop receiving events on disconnect

### RPC Method: gateway.subscribe
Added handler in rpc/handler.rs that:
- Accepts subscription parameters from request params
- Validates parameters
- Updates client subscription via request extensions
- Returns success response with subscribed event types

### Unit Tests (in broadcast.rs)
- `test_subscription_default` - Default subscription includes all types
- `test_subscription_with_events` - Custom subscription creation
- `test_subscription_includes` - Event type checking
- `test_gateway_event_serialization` - JSON serialization
- `test_broadcaster_publish_subscribe` - Basic pub/sub
- `test_broadcaster_capacity` - Messages not dropped immediately
- `test_broadcaster_multiple_subscribers` - Multiple subscribers

### Extension Keys
Added `BROADCASTER_KEY` extension key for broadcasting in ws.rs

---
*Created: 2026-02-15*
*Resolved: 2026-02-17*
