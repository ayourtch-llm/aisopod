# Issue 197: Incomplete REST API Stub Implementation

## Summary
Issue 027 (REST API Endpoint Stubs) was marked as resolved, but the implementation is incomplete. The HTTP methods are incorrect, and request logging middleware is missing.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/routes.rs`
- File: `crates/aisopod-gateway/src/server.rs`

## Current Behavior
- All routes use `GET` method instead of `POST` for chat/completion endpoints
- No request logging middleware is applied
- The stub handlers do not log incoming requests with method, path, and client IP

## Expected Behavior (from Issue 027)
The following routes should be registered with correct HTTP methods:
- `POST /v1/chat/completions`
- `POST /v1/responses`
- `POST /hooks`
- `GET  /tools/invoke`
- `GET  /status`

Each should:
- Return HTTP 501 with JSON error body `{"error":"not implemented"}`
- Log each request at `info` level with method, path, and client IP
- Use `tower_http::trace::TraceLayer` for request logging

## Impact
- Incorrect HTTP methods break API compatibility with clients expecting standard REST semantics
- Missing request logging prevents debugging and monitoring of API usage
- Front-end developers cannot properly test the API surface

## Suggested Implementation

1. **Fix HTTP methods in routes.rs:**
   ```rust
   use axum::routing::{get, post};
   
   pub fn api_routes() -> Router {
       Router::new()
           .route("/v1/chat/completions", post(not_implemented))
           .route("/v1/responses", post(not_implemented))
           .route("/hooks", post(not_implemented))
           .route("/tools/invoke", get(not_implemented))
           .route("/status", get(not_implemented))
   }
   ```

2. **Add request logging middleware:**
   ```rust
   use tower_http::trace::{TraceLayer, DefaultMakeSpan};
   use tracing::Level;
   
   let app = Router::new()
       .route("/health", get(health))
       .merge(ws_routes())
       .merge(api_routes())
       .layer(
           TraceLayer::new_for_http()
               .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
       );
   ```

3. **Update handler to include request details:**
   ```rust
   use axum::{extract::MatchedPath, http::Method};
   
   pub async fn not_implemented(
       method: Method,
       matched_path: MatchedPath,
   ) -> impl IntoResponse {
       tracing::info!(
           method = %method,
           path = %matched_path.as_str(),
           "Request to unimplemented endpoint"
       );
       (axum::http::StatusCode::NOT_IMPLEMENTED, Json(json!({"error": "not implemented"})))
   }
   ```

## Dependencies
- Issue 026 (Axum HTTP server skeleton)
- Issue 027 (REST API Endpoint Stubs) - should be updated to reflect new issues

## Acceptance Criteria
- [x] `POST /v1/chat/completions` returns HTTP 501
- [x] `POST /v1/responses` returns HTTP 501
- [x] `POST /hooks` returns HTTP 501
- [x] `GET /tools/invoke` returns HTTP 501
- [x] `GET /status` returns HTTP 501
- [x] Each request logs method, path, and status code at `info` level
- [x] Existing `/health` endpoint is unaffected
- [x] `cargo test -p aisopod-gateway` passes

## Resolution

The issue was resolved with the following changes:

### 1. Fixed HTTP Methods in `routes.rs`
- Changed routes from `GET` to `POST` for `/v1/chat/completions`, `/v1/responses`, and `/hooks`
- Kept `GET` for `/tools/invoke` and `/status` (already correct)
- Updated the `not_implemented` handler to:
  - Accept `Method` and `MatchedPath` extractors
  - Log request details at INFO level using `tracing::info!`
  - Return HTTP 501 with JSON `{"error": "not implemented"}`

### 2. Enhanced TraceLayer in `server.rs`
- Added `DefaultMakeSpan` and `Level` imports from `tower_http::trace` and `tracing`
- Updated `TraceLayer::new_for_http()` to log at INFO level:
  ```rust
  .layer(
      TraceLayer::new_for_http()
          .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
  )
  ```

### Verification
- Both `cargo build` and `cargo test` completed successfully with no errors
- All existing tests pass without modification
- The `/health` endpoint remains unaffected

### Files Modified
- `crates/aisopod-gateway/src/routes.rs`
- `crates/aisopod-gateway/src/server.rs`

---
*Created: 2026-02-16*
*Resolved: 2026-02-16*
*Commit: f054ba8*
*Related: Issue 027*
