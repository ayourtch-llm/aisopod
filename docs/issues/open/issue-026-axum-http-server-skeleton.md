# Issue 026: Set up Axum HTTP Server Skeleton

## Summary
Create the foundational HTTP server for the `aisopod-gateway` crate using the Axum web framework. This server will bind to a configurable address and port, expose a health-check endpoint, and shut down gracefully on system signals.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/server.rs`

## Current Behavior
The `aisopod-gateway` crate exists but contains no HTTP server implementation. There is no way to accept incoming HTTP connections or serve any endpoints.

## Expected Behavior
After this issue is completed the gateway will:
- Start an Axum HTTP server bound to the address and port specified in `GatewayConfig`.
- Expose a `GET /health` endpoint that returns HTTP 200 with a JSON body such as `{"status":"ok"}`.
- Shut down gracefully when it receives a SIGINT or SIGTERM signal, draining in-flight connections before exiting.

## Impact
Every other gateway feature (REST endpoints, WebSocket handling, authentication, static-file serving) depends on this server skeleton. It is the entry point for all network traffic into the pod.

## Suggested Implementation
1. Add dependencies to `crates/aisopod-gateway/Cargo.toml`:
   - `axum` (latest stable, with `ws` feature enabled for future use)
   - `tokio` (with `full` feature)
   - `serde` and `serde_json` for JSON responses
   - `tracing` for structured logging
2. Create `crates/aisopod-gateway/src/server.rs`:
   - Define an async `run(config: &GatewayConfig) -> Result<()>` function.
   - Build an `axum::Router` with a single route: `GET /health`.
   - Bind the router to `config.listen_addr` using `axum::serve`.
   - Register a `tokio::signal` handler that listens for SIGINT and SIGTERM.
   - Pass the signal future to `axum::serve(...).with_graceful_shutdown(signal)`.
3. Implement the `/health` handler:
   ```rust
   async fn health() -> impl IntoResponse {
       Json(serde_json::json!({"status": "ok"}))
   }
   ```
4. Re-export the `run` function from `crates/aisopod-gateway/src/lib.rs`.
5. Verify by running `cargo run -p aisopod-gateway` and curling `http://localhost:<port>/health`.

## Dependencies
- Issue 011 (GatewayConfig definition)
- Issue 016 (Configuration loading)

## Acceptance Criteria
- [ ] `axum` and `tokio` are listed in `aisopod-gateway` dependencies.
- [ ] Server starts and binds to the address/port from `GatewayConfig`.
- [ ] `GET /health` returns HTTP 200 with JSON body `{"status":"ok"}`.
- [ ] Server shuts down cleanly on SIGINT or SIGTERM without error logs.
- [ ] Unit or smoke test confirms the health endpoint responds correctly.

---
*Created: 2026-02-15*
