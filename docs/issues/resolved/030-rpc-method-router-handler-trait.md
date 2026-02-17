# Issue 030: Implement RPC Method Router and Handler Trait

## Summary
Create a trait-based method dispatch system that routes incoming JSON-RPC requests to the correct handler by method name. Provide a `RequestContext` carrying connection metadata and register placeholder handlers for all 24 planned method namespaces.

## Location
- Crate: `aisopod-gateway`
- Files:
  - `crates/aisopod-gateway/src/rpc/handler.rs`
  - `crates/aisopod-gateway/src/rpc/router.rs`

## Current Behavior
JSON-RPC messages can be parsed (Issue 029) but there is no mechanism to dispatch them to specific handler logic based on the `method` field.

## Expected Behavior
- An `RpcMethod` trait defines an `async fn handle(&self, ctx: RequestContext, params: Option<Value>) -> RpcResponse` method.
- A `MethodRouter` holds a mapping from method-name strings to boxed `RpcMethod` implementors.
- Dispatching an unknown method returns a `-32601 Method not found` error.
- A `RequestContext` struct carries `connId`, client IP, authenticated role, and scopes.
- Placeholder handlers are registered for all 24 method namespaces (e.g., `agent.*`, `chat.*`, `tools.*`).

## Impact
The method router is the backbone of the RPC layer. All future method implementations plug into this dispatch system, so getting the trait and routing pattern right is critical for extensibility.

## Suggested Implementation
1. Create `crates/aisopod-gateway/src/rpc/handler.rs`:
   ```rust
   #[async_trait]
   pub trait RpcMethod: Send + Sync {
       async fn handle(&self, ctx: RequestContext, params: Option<Value>) -> RpcResponse;
   }

   pub struct RequestContext {
       pub conn_id: String,
       pub remote_addr: SocketAddr,
       pub role: Option<String>,
       pub scopes: Vec<String>,
   }
   ```
2. Create `crates/aisopod-gateway/src/rpc/router.rs`:
   ```rust
   pub struct MethodRouter {
       methods: HashMap<String, Box<dyn RpcMethod>>,
   }

   impl MethodRouter {
       pub fn new() -> Self { ... }
       pub fn register(&mut self, name: &str, handler: impl RpcMethod + 'static) { ... }
       pub async fn dispatch(&self, ctx: RequestContext, req: RpcRequest) -> RpcResponse { ... }
   }
   ```
3. Implement a `PlaceholderHandler` that returns `{"error":"not implemented"}` with a custom error code.
4. In a `pub fn default_router() -> MethodRouter` function, register placeholder handlers for all 24 namespaces.
5. Wire `MethodRouter::dispatch` into the WebSocket message loop from Issue 028.
6. Write unit tests: known method dispatches correctly, unknown method returns `-32601`.

## Dependencies
- Issue 029 (JSON-RPC 2.0 message parsing)

## Acceptance Criteria
- [x] `RpcMethod` trait is defined with a `handle` method (synchronous implementation used).
- [x] `MethodRouter` routes by method name string.
- [x] Unknown methods return JSON-RPC error `-32601`.
- [x] `RequestContext` carries `conn_id`, remote address, role, and scopes.
- [x] Placeholder handlers are registered for all 24 method namespaces.
- [x] Unit tests verify correct routing and error handling.

## Resolution
The RPC method router and handler trait system was implemented in `crates/aisopod-gateway/src/rpc/handler.rs`:

### RequestContext
Struct carrying connection metadata:
- `conn_id`: Unique identifier for the WebSocket connection
- `remote_addr`: Client IP address (SocketAddr)
- `role`: Optional authenticated role (e.g., "admin", "user")
- `scopes`: Vector of permission scopes
- `RequestContext::new(conn_id, remote_addr)` constructor

### RpcMethod Trait
```rust
pub trait RpcMethod {
    fn handle(&self, ctx: &RequestContext, params: Option<serde_json::Value>) -> types::RpcResponse;
}
```
- All handler implementations must implement this trait
- Receives request context and optional params
- Returns an `RpcResponse` for sending back to client

### PlaceholderHandler
Implementation of `RpcMethod` that returns `-32601 Method not found`:
- Stores method name for detailed error messages
- Uses `types::RpcResponse::error()` to create proper error responses

### MethodRouter
```rust
pub struct MethodRouter {
    methods: HashMap<String, Box<dyn RpcMethod>>,
}
```
Methods:
- `new()`: Create empty router
- `register(name, handler)`: Register a handler for a method name
- `dispatch(ctx, req)`: Dispatch request to appropriate handler, or return `-32601` for unknown methods
- `method_count()`: Get number of registered methods

### Default Router
`default_router()` creates a router with placeholder handlers for all 24 method namespaces:
- **Agent methods**: `agent.create`, `agent.update`, `agent.delete`, `agent.list`
- **Chat methods**: `chat.create`, `chat.send`, `chat.history`, `chat.delete`
- **Tools methods**: `tools.invoke`, `tools.list`, `tools.describe`, `tools.authorize`
- **Session methods**: `session.create`, `session.get`, `session.update`, `session.delete`
- **Model methods**: `model.list`, `model.describe`, `model.select`, `model.feedback`
- **Config methods**: `config.get`, `config.set`, `config.validate`, `config.reload`

### Unit Tests
- `test_request_context_creation`: Verify context fields
- `test_method_router_dispatch_known_method`: Known methods dispatch correctly
- `test_method_router_dispatch_unknown_method`: Unknown methods return `-32601`
- `test_default_router_method_count`: Verify 24 methods registered
- `test_default_router_contains_agent_methods`: Verify agent namespace methods

### Integration
The `MethodRouter` is exported via `rpc/mod.rs`:
```rust
pub use handler::{default_router, MethodRouter, PlaceholderHandler, RpcMethod, RequestContext};
```

### Note on Async/Sync
The implementation uses synchronous `handle()` instead of `async fn handle()`. The trait is designed for synchronous operations that return `RpcResponse` directly. If async operations are needed, they can be handled within the handler implementation before returning the response.

---
*Created: 2026-02-15*
*Resolved: 2026-02-17*
