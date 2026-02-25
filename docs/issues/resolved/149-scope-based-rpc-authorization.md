# Issue 149: Implement Scope-Based Authorization for RPC Methods

## Summary
Define scope requirements for each of the 24 RPC method namespaces and implement a scope-checking middleware that rejects unauthorized method calls with proper JSON-RPC error codes.

## Location
- Crate: `aisopod-gateway`
- Files:
  - `crates/aisopod-gateway/src/auth/scopes.rs` (new)
  - `crates/aisopod-gateway/src/rpc/middleware/auth.rs` (new or extend)

## Current Behavior
The gateway authentication system (Issue 031) extracts a role and scopes from authenticated requests, and the RPC router (Issue 030) dispatches methods to handlers. However, there is no enforcement layer that checks whether the caller's scopes permit calling a specific RPC method. All authenticated users can call any method.

## Expected Behavior
After this issue is completed:
- Each RPC method namespace has a defined required scope (e.g., `operator.read`, `operator.write`, `operator.admin`, `operator.approvals`, `operator.pairing`).
- A scope-checking layer runs before method dispatch and rejects calls where the caller lacks the required scope.
- Unauthorized calls return a JSON-RPC error with code `-32603` (or a custom code) and a descriptive message.
- Scope definitions are centralized and easy to update as new methods are added.

## Impact
Without scope enforcement, any authenticated user has full access to all RPC methods, including admin operations. This is a critical authorization gap that would allow privilege escalation.

## Suggested Implementation

1. **Define the scope constants** in `crates/aisopod-gateway/src/auth/scopes.rs`:
   ```rust
   /// Permission scopes for RPC method access control.
   #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum Scope {
       OperatorAdmin,
       OperatorRead,
       OperatorWrite,
       OperatorApprovals,
       OperatorPairing,
   }

   impl Scope {
       pub fn as_str(&self) -> &'static str {
           match self {
               Self::OperatorAdmin => "operator.admin",
               Self::OperatorRead => "operator.read",
               Self::OperatorWrite => "operator.write",
               Self::OperatorApprovals => "operator.approvals",
               Self::OperatorPairing => "operator.pairing",
           }
       }
   }

   impl std::fmt::Display for Scope {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           f.write_str(self.as_str())
       }
   }
   ```

2. **Map RPC method namespaces to required scopes:**
   ```rust
   use std::collections::HashMap;
   use once_cell::sync::Lazy;

   static METHOD_SCOPES: Lazy<HashMap<&'static str, Scope>> = Lazy::new(|| {
       let mut m = HashMap::new();

       // Read-only methods
       m.insert("system.ping", Scope::OperatorRead);
       m.insert("system.info", Scope::OperatorRead);
       m.insert("agent.list", Scope::OperatorRead);
       m.insert("agent.get", Scope::OperatorRead);
       m.insert("session.list", Scope::OperatorRead);
       m.insert("session.get", Scope::OperatorRead);
       m.insert("chat.history", Scope::OperatorRead);
       m.insert("tools.list", Scope::OperatorRead);
       m.insert("models.list", Scope::OperatorRead);
       m.insert("channels.list", Scope::OperatorRead);
       m.insert("config.get", Scope::OperatorRead);
       m.insert("health.check", Scope::OperatorRead);
       m.insert("memory.query", Scope::OperatorRead);

       // Write methods
       m.insert("agent.start", Scope::OperatorWrite);
       m.insert("agent.stop", Scope::OperatorWrite);
       m.insert("chat.send", Scope::OperatorWrite);
       m.insert("session.create", Scope::OperatorWrite);
       m.insert("session.close", Scope::OperatorWrite);
       m.insert("config.update", Scope::OperatorWrite);

       // Approval methods
       m.insert("approval.decide", Scope::OperatorApprovals);
       m.insert("approval.list", Scope::OperatorApprovals);

       // Pairing methods
       m.insert("pairing.initiate", Scope::OperatorPairing);
       m.insert("pairing.confirm", Scope::OperatorPairing);

       // Admin methods
       m.insert("admin.shutdown", Scope::OperatorAdmin);

       m
   });

   /// Get the required scope for a method, if any.
   pub fn required_scope(method: &str) -> Option<&Scope> {
       METHOD_SCOPES.get(method)
   }
   ```

3. **Implement the scope-checking middleware/extractor:**
   ```rust
   use crate::rpc::handler::RequestContext;
   use crate::rpc::jsonrpc::{RpcError, RpcResponse};

   const UNAUTHORIZED_CODE: i64 = -32603;

   pub fn check_scope(
       ctx: &RequestContext,
       method: &str,
   ) -> Result<(), RpcResponse> {
       let Some(required) = required_scope(method) else {
           // No scope requirement defined — allow by default
           // (or deny by default for stricter security)
           return Ok(());
       };

       if ctx.scopes.contains(required) {
           Ok(())
       } else {
           Err(RpcResponse::error(
               ctx.request_id.clone(),
               RpcError {
                   code: UNAUTHORIZED_CODE,
                   message: format!(
                       "Insufficient permissions: method '{}' requires scope '{}'",
                       method, required
                   ),
                   data: None,
               },
           ))
       }
   }
   ```

4. **Integrate into the RPC router** from Issue 030:
   ```rust
   // In MethodRouter::dispatch()
   pub async fn dispatch(
       &self,
       ctx: RequestContext,
       method: &str,
       params: Option<Value>,
   ) -> RpcResponse {
       // Check authorization before dispatch
       if let Err(err_response) = check_scope(&ctx, method) {
           return err_response;
       }

       // Proceed with normal dispatch
       match self.handlers.get(method) {
           Some(handler) => handler.handle(ctx, params).await,
           None => RpcResponse::method_not_found(ctx.request_id),
       }
   }
   ```

5. **Add unit tests:**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       fn ctx_with_scopes(scopes: Vec<Scope>) -> RequestContext {
           RequestContext {
               scopes,
               request_id: Some(serde_json::Value::Number(1.into())),
               ..Default::default()
           }
       }

       #[test]
       fn test_read_scope_allows_list_methods() {
           let ctx = ctx_with_scopes(vec![Scope::OperatorRead]);
           assert!(check_scope(&ctx, "agent.list").is_ok());
           assert!(check_scope(&ctx, "session.list").is_ok());
       }

       #[test]
       fn test_read_scope_denies_write_methods() {
           let ctx = ctx_with_scopes(vec![Scope::OperatorRead]);
           assert!(check_scope(&ctx, "agent.start").is_err());
           assert!(check_scope(&ctx, "config.update").is_err());
       }

       #[test]
       fn test_admin_scope_required_for_shutdown() {
           let ctx = ctx_with_scopes(vec![Scope::OperatorWrite]);
           assert!(check_scope(&ctx, "admin.shutdown").is_err());

           let ctx = ctx_with_scopes(vec![Scope::OperatorAdmin]);
           assert!(check_scope(&ctx, "admin.shutdown").is_ok());
       }

       #[test]
       fn test_approval_scope_required() {
           let ctx = ctx_with_scopes(vec![Scope::OperatorRead]);
           assert!(check_scope(&ctx, "approval.decide").is_err());

           let ctx = ctx_with_scopes(vec![Scope::OperatorApprovals]);
           assert!(check_scope(&ctx, "approval.decide").is_ok());
       }

       #[test]
       fn test_unknown_method_allowed_by_default() {
           let ctx = ctx_with_scopes(vec![]);
           assert!(check_scope(&ctx, "unknown.method").is_ok());
       }
   }
   ```

## Dependencies
- Issue 031 (gateway authentication — role and scope extraction)
- Issue 030 (RPC router — `MethodRouter`, `RequestContext`)

## Acceptance Criteria
- [x] All 24 RPC method namespaces have defined scope requirements
- [x] Scope checking runs before method dispatch in the RPC router
- [x] Unauthorized calls return a JSON-RPC error with a descriptive message
- [x] `OperatorRead` scope grants access to read-only methods but not write/admin
- [x] `OperatorWrite` scope grants access to write methods
- [x] `OperatorAdmin` scope grants access to admin methods
- [x] `OperatorApprovals` scope grants access to approval methods
- [x] `OperatorPairing` scope grants access to pairing methods
- [x] Unit tests verify scope enforcement for each namespace

## Resolution

This issue implemented scope-based authorization for RPC methods. The implementation includes:

### Files Created/Modified:

1. **`crates/aisopod-gateway/src/auth/scopes.rs`** (new)
   - Defined the `Scope` enum with 5 scope types: `OperatorAdmin`, `OperatorRead`, `OperatorWrite`, `OperatorApprovals`, `OperatorPairing`
   - Implemented a static `METHOD_SCOPES` mapping from RPC method names to required scopes
   - Added `required_scope()` and `requires_scope_validation()` functions
   - Included comprehensive unit tests

2. **`crates/aisopod-gateway/src/rpc/middleware/auth.rs`** (new)
   - Implemented `check_scope()` function that validates method access against user scopes
   - Created `has_scope()` helper to check if auth_info has the required scope
   - Used error code `-32603` (JSON-RPC server error) for unauthorized access
   - Added unit tests for all scope combinations

3. **`crates/aisopod-gateway/src/rpc/handler.rs`** (modified)
   - Extended `RequestContext` to include `auth_info` field for scope checking
   - Added `with_auth()` constructor to create context with auth info
   - Modified `dispatch()` to check scope authorization before method dispatch

4. **`crates/aisopod-gateway/src/rpc/mod.rs`** (modified)
   - Added `middleware` submodule
   - Created `jsonrpc` re-export module for JSON-RPC types

5. **`crates/aisopod-gateway/src/auth.rs`** (modified)
   - Exported `scopes` module and re-exported `Scope` and `required_scope`

6. **`crates/aisopod-gateway/src/ws.rs`** (modified)
   - Updated `RequestContext` creation to include AuthInfo for scope checking

### Implementation Details:

- **Scope Hierarchy**: Admin scope grants access to all operations. Read scope allows read-only access. Write/Approvals/Pairing scopes each grant read plus their specific functionality.
- **Error Handling**: Unauthorized requests return JSON-RPC error with code `-32603` and descriptive message
- **Backward Compatibility**: Methods without defined scope requirements remain accessible to all authenticated users

### Testing:
- 101 unit tests pass including new tests for scope-based authorization
- Integration tests verify WebSocket RPC dispatch with scope checking
- Build passes with `RUSTFLAGS=-Awarnings`

---
*Created: 2026-02-15*
*Resolved: 2026-02-25*
