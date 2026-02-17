# Issue 033: Implement Client Connection Management

## Summary
Create a `GatewayClient` struct and a connection registry that tracks all active WebSocket clients, their identity, presence state, and health. Connections are cleaned up automatically on disconnect.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/client.rs`

## Current Behavior
WebSocket connections are handled in isolated tasks with no central tracking. The gateway has no way to enumerate connected clients or query their status.

## Expected Behavior
- A `GatewayClient` struct holds: `conn_id`, a handle to the WebSocket sender, `presence_key`, client IP, authenticated role, and scopes.
- A `ClientRegistry` backed by `DashMap<String, GatewayClient>` tracks all active clients.
- Connection lifecycle hooks:
  - `on_connect` — inserts the client into the registry and logs the event.
  - `on_disconnect` — removes the client, logs the event, and performs any cleanup.
- The registry exposes methods to list clients, look up by `conn_id`, and produce a health snapshot (count of connected clients, per-role counts).

## Impact
Client connection management is the foundation for broadcasting events, enforcing per-client policies, and providing operational visibility into the gateway. Issues 034 (broadcasting) and the `/status` endpoint both depend on this registry.

## Suggested Implementation
1. Add `dashmap` to dependencies (if not already present from Issue 032).
2. Create `crates/aisopod-gateway/src/client.rs`:
   ```rust
   pub struct GatewayClient {
       pub conn_id: String,
       pub sender: mpsc::Sender<Message>,
       pub presence_key: String,
       pub remote_addr: SocketAddr,
       pub role: String,
       pub scopes: Vec<String>,
       pub connected_at: Instant,
   }

   pub struct ClientRegistry {
       clients: DashMap<String, GatewayClient>,
   }

   impl ClientRegistry {
       pub fn new() -> Self { ... }
       pub fn on_connect(&self, client: GatewayClient) { ... }
       pub fn on_disconnect(&self, conn_id: &str) { ... }
       pub fn get(&self, conn_id: &str) -> Option<...> { ... }
       pub fn list(&self) -> Vec<...> { ... }
       pub fn health_snapshot(&self) -> HealthSnapshot { ... }
   }
   ```
3. Define `HealthSnapshot`:
   ```rust
   pub struct HealthSnapshot {
       pub total_connections: usize,
       pub operators: usize,
       pub nodes: usize,
   }
   ```
4. Integrate `ClientRegistry` into the WebSocket handler:
   - Call `on_connect` after successful upgrade and auth.
   - Call `on_disconnect` when the read loop exits.
5. Write unit tests for insert, remove, list, and health-snapshot operations.

## Dependencies
- Issue 028 (WebSocket upgrade and connection lifecycle)
- Issue 031 (Gateway authentication — provides role and scopes)

## Acceptance Criteria
- [x] `GatewayClient` struct contains all specified fields.
- [x] `ClientRegistry` correctly tracks connect and disconnect events.
- [x] `list()` returns all currently connected clients.
- [x] `health_snapshot()` returns accurate counts.
- [x] Disconnected clients are fully removed from the registry.
- [x] Unit tests cover connect, disconnect, and concurrent access scenarios.

## Resolution
The client connection management system was implemented in `crates/aisopod-gateway/src/client.rs`:

### GatewayClient
```rust
pub struct GatewayClient {
    pub conn_id: String,
    pub sender: Arc<mpsc::Sender<Message>>,
    pub presence_key: String,
    pub remote_addr: SocketAddr,
    pub role: String,
    pub scopes: Vec<String>,
    pub connected_at: Instant,
}
```
Methods:
- `new(...)` - Create client with all fields
- `from_auth_info(...)` - Create from AuthInfo, generates presence_key

### ClientRegistry
```rust
pub struct ClientRegistry {
    clients: DashMap<String, GatewayClient>,
}
```
Methods:
- `new()` - Create empty registry
- `on_connect(client)` - Insert client and log event
- `on_disconnect(conn_id)` - Remove client and log event
- `get(conn_id)` - Retrieve client by ID
- `list()` - Get all connected clients
- `health_snapshot()` - Get counts of total, operators, nodes
- `len()` - Get client count
- `is_empty()` - Check if registry is empty

### HealthSnapshot
```rust
pub struct HealthSnapshot {
    pub total_connections: usize,
    pub operators: usize,
    pub nodes: usize,
}
```

### WebSocket Integration
The ClientRegistry is integrated into the WebSocket handler in ws.rs:
- `on_connect` called after successful upgrade and authentication
- `on_disconnect` called when the read loop exits

### Unit Tests
- `test_gateway_client_creation_from_auth_info` - Client creation from AuthInfo
- `test_gateway_client_creation_with_auth` - Client creation with explicit values
- `test_client_registry_new_is_empty` - Empty registry
- `test_client_registry_on_connect` - Insert and retrieve
- `test_client_registry_on_disconnect` - Remove client
- `test_client_registry_on_disconnect_unknown` - Handle unknown client
- `test_client_registry_list` - List all clients
- `test_client_registry_health_snapshot` - Accurate counts
- `test_client_registry_concurrent_access` - Thread safety
- `test_health_snapshot_empty` - Empty snapshot

### Dependencies
dashmap was already added in Issue 032 (version 5.5)

---
*Created: 2026-02-15*
*Resolved: 2026-02-17*
