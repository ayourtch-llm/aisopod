# Issue 031: Implement Gateway Authentication System

## Summary
Add authentication to the gateway supporting three modes — Bearer token, password, and no-auth — for both HTTP routes and WebSocket handshakes. Authenticated requests carry a role (operator or node) and a set of permission scopes.

## Location
- Crate: `aisopod-gateway`
- Files:
  - `crates/aisopod-gateway/src/auth.rs`
  - `crates/aisopod-gateway/src/middleware/auth.rs`

## Current Behavior
All HTTP and WebSocket endpoints are open with no authentication or authorization checks.

## Expected Behavior
- An Axum middleware layer inspects incoming HTTP requests for an `Authorization` header.
- Three auth modes are supported, selected via `GatewayConfig`:
  - **token** — Expects `Authorization: Bearer <token>`. Validates against configured tokens.
  - **password** — Expects HTTP Basic auth. Validates against configured credentials.
  - **none** — All requests are allowed without credentials.
- WebSocket upgrade requests are authenticated during the HTTP handshake phase before the upgrade completes.
- On successful auth, the request is annotated with a role (`operator` or `node`) and a list of scopes (e.g., `chat:write`, `agent:admin`).
- Unauthorized requests receive HTTP 401 with a JSON error body.

## Impact
Authentication is a security-critical gate. Without it, any client can invoke any RPC method or REST endpoint. This issue also provides the role and scope information that downstream authorization checks rely on.

## Suggested Implementation
1. Create `crates/aisopod-gateway/src/auth.rs`:
   - Define an `AuthInfo` struct: `{ role: String, scopes: Vec<String> }`.
   - Implement `validate_token(token: &str, config: &AuthConfig) -> Option<AuthInfo>`.
   - Implement `validate_basic(user: &str, pass: &str, config: &AuthConfig) -> Option<AuthInfo>`.
2. Create `crates/aisopod-gateway/src/middleware/auth.rs`:
   - Implement an Axum middleware (using `axum::middleware::from_fn`) that:
     1. Reads the `Authorization` header.
     2. Calls the appropriate validate function based on the configured auth mode.
     3. On success, inserts `AuthInfo` into request extensions.
     4. On failure, returns `(StatusCode::UNAUTHORIZED, Json(...))`.
3. Apply the middleware to all routes except `GET /health`.
4. In the WebSocket upgrade handler, extract `AuthInfo` from request extensions before upgrading.
5. Write unit tests for each auth mode and for the rejection path.

## Dependencies
- Issue 026 (Axum HTTP server skeleton)
- Issue 028 (WebSocket upgrade and connection lifecycle)
- Issue 016 (Configuration loading — provides `AuthConfig`)

## Acceptance Criteria
- [x] Bearer token auth validates correct tokens and rejects incorrect ones.
- [x] Password auth validates correct credentials and rejects incorrect ones.
- [x] No-auth mode allows all requests through.
- [x] WebSocket handshakes are authenticated during HTTP upgrade.
- [x] Authenticated requests carry `role` and `scopes` in request extensions.
- [x] Unauthorized requests receive HTTP 401 with a JSON error body.
- [x] `GET /health` remains accessible without authentication.

## Resolution
The gateway authentication system was implemented with two main components:

### crates/aisopod-gateway/src/auth.rs

**AuthInfo struct:**
```rust
pub struct AuthInfo {
    pub role: String,
    pub scopes: Vec<String>,
}
```
- Methods: `has_scope()`, `has_any_scope()`, `has_role()` for authorization checks

**Validation Functions:**
- `validate_token(token, config)` - Validates Bearer tokens against configured credentials
- `validate_basic(username, password, config)` - Validates HTTP Basic auth credentials
- `build_token_map(config)` - O(1) token lookup map for efficient validation
- `build_password_map(config)` - O(1) password lookup map for efficient validation

**Supported Auth Modes:**
- `AuthMode::Token` - Bearer token authentication
- `AuthMode::Password` - HTTP Basic authentication
- `AuthMode::None` - No authentication (all requests allowed)

### crates/aisopod-gateway/src/middleware/auth.rs

**AuthConfigData:**
- Pre-computed lookup maps for token/password validation
- `validate_token()` and `validate_basic()` methods
- `mode()` accessor for the current auth mode

**Auth Middleware (`auth_middleware`):**
- Implements Axum middleware using `axum::middleware::from_fn`
- Supports all three auth modes via configuration
- Validates Authorization header and extracts credentials
- On success: inserts `AuthInfo` into request extensions
- On failure: returns HTTP 401 with JSON error body
- `/health` endpoint is always accessible without authentication

**Helper Functions:**
- `extract_authorization()` - Extract Authorization header
- `parse_bearer_token()` - Parse Bearer token format
- `parse_basic_credentials()` - Parse Basic auth (base64 decode)
- `unauthorized_response()` - Generate 401 JSON error response

**Extension Trait:**
- `ExtractAuthInfo` trait with `extract_auth_info()` method for easy AuthInfo extraction

### Unit Tests

**auth.rs tests:**
- `test_validate_token_success` - Valid token authentication
- `test_validate_token_invalid` - Invalid token rejection
- `test_validate_token_wrong_mode` - Token auth in wrong mode
- `test_validate_basic_success` - Valid Basic auth
- `test_validate_basic_invalid_credentials` - Invalid password
- `test_validate_basic_invalid_user` - Unknown user
- `test_validate_basic_wrong_mode` - Basic auth in wrong mode
- `test_build_token_map` - Token lookup map construction
- `test_build_password_map` - Password lookup map construction
- `test_auth_info_has_scope` - Scope checking
- `test_auth_info_has_any_scope` - Any scope checking
- `test_auth_info_has_role` - Role checking
- `test_default_auth_info` - Default values

**middleware/auth.rs tests:**
- `test_auth_middleware_token_success` - Valid token authentication
- `test_auth_middleware_token_missing` - Missing auth header
- `test_auth_middleware_token_invalid` - Invalid token
- `test_auth_middleware_password_success` - Valid Basic auth
- `test_auth_middleware_password_invalid` - Invalid credentials
- `test_auth_middleware_none` - No auth mode allows all
- `test_health_endpoint_always_allowed` - Health endpoint accessible

### Integration Points

The middleware is integrated into the HTTP routes in `server.rs`:
- Applied as a layer to all routes via `axum::middleware::from_fn(auth_middleware)`
- `AuthConfigData` injected into request extensions for middleware access
- WebSocket upgrade handler can extract `AuthInfo` from request extensions

---
*Created: 2026-02-15*
*Resolved: 2026-02-17*
