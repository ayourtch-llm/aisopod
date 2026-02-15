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
- [ ] `RpcMethod` trait is defined with an async `handle` method.
- [ ] `MethodRouter` routes by method name string.
- [ ] Unknown methods return JSON-RPC error `-32601`.
- [ ] `RequestContext` carries `conn_id`, remote address, role, and scopes.
- [ ] Placeholder handlers are registered for all 24 method namespaces.
- [ ] Unit tests verify correct routing and error handling.

---
*Created: 2026-02-15*
