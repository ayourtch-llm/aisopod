# Issue 032: Implement Rate Limiting

## Summary
Add per-IP sliding-window rate limiting to the gateway so that abusive or misconfigured clients cannot overwhelm the server. Limits are configurable and enforced as Axum middleware.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/middleware/rate_limit.rs`

## Current Behavior
The gateway accepts an unlimited number of requests from any IP address. There is no throttling mechanism.

## Expected Behavior
- Each client IP is tracked in a sliding time window.
- When the request count exceeds the configured limit within the window, the server returns HTTP 429 Too Many Requests with a `Retry-After` header indicating how many seconds until the window resets.
- Rate-limit state is stored in memory using a concurrent map (`DashMap`).
- A background task periodically evicts expired entries to prevent unbounded memory growth.
- Limits are read from `GatewayConfig`.

## Impact
Rate limiting is a basic operational safeguard that prevents denial-of-service conditions. It must be in place before the gateway is exposed to any untrusted network.

## Suggested Implementation
1. Add `dashmap` to `aisopod-gateway/Cargo.toml`.
2. Create `crates/aisopod-gateway/src/middleware/rate_limit.rs`:
   - Define a `RateLimiter` struct:
     ```rust
     pub struct RateLimiter {
         state: DashMap<IpAddr, Vec<Instant>>,
         max_requests: u64,
         window: Duration,
     }
     ```
   - Implement `pub fn check(&self, ip: IpAddr) -> Result<(), Duration>`:
     - Look up or insert the IP entry.
     - Remove timestamps older than `window`.
     - If the remaining count ≥ `max_requests`, return `Err(remaining_duration)`.
     - Otherwise, push the current timestamp and return `Ok(())`.
   - Implement a `pub async fn cleanup_loop(&self)` method that runs every `window` and removes stale entries.
3. Create an Axum middleware function:
   ```rust
   async fn rate_limit_middleware(
       State(limiter): State<Arc<RateLimiter>>,
       ConnectInfo(addr): ConnectInfo<SocketAddr>,
       request: Request,
       next: Next,
   ) -> Response { ... }
   ```
   - On `Err(retry_after)`, return 429 with `Retry-After` header.
4. Add the middleware to the router in `server.rs`, after authentication.
5. Spawn the cleanup task in the server startup function.
6. Write unit tests for under-limit, at-limit, and over-limit scenarios.

## Dependencies
- Issue 026 (Axum HTTP server skeleton)
- Issue 031 (Gateway authentication — rate limiting runs after auth)

## Acceptance Criteria
- [x] Requests within the configured limit succeed normally.
- [x] Requests exceeding the limit receive HTTP 429 with a `Retry-After` header.
- [x] Rate-limit state is per-IP.
- [x] Expired entries are cleaned up periodically (no memory leak).
- [x] Limits are configurable via `GatewayConfig`.
- [x] Unit tests cover under-limit, at-limit, and over-limit cases.

## Resolution
The rate limiting middleware was implemented in `crates/aisopod-gateway/src/middleware/rate_limit.rs`:

### RateLimitConfig
```rust
pub struct RateLimitConfig {
    pub max_requests: u64,
    pub window: Duration,
}
```
- Configurable rate limit settings
- Default: 100 requests per 60 seconds

### RateLimiter
```rust
pub struct RateLimiter {
    state: DashMap<IpAddr, Vec<Instant>>,
    max_requests: u64,
    window: Duration,
}
```
- Thread-safe per-IP tracking using `dashmap::DashMap`
- `check(ip)` implements sliding-window rate limiting
- Returns `Ok(())` if allowed, `Err(retry_after)` if over limit

### Rate Limiting Algorithm
1. Look up or create entry for the IP address
2. Remove timestamps older than the window
3. If count >= max_requests, return `Err(retry_after)`
4. Otherwise, add current timestamp and return `Ok(())`
5. `retry_after` is the time until the oldest request expires

### Axum Middleware (`rate_limit_middleware`)
- Gets `RateLimiter` from request extensions (not State)
- Gets client IP from `ConnectInfo<SocketAddr>` extension
- Returns HTTP 429 with `Retry-After` header when limit exceeded
- Allows request to proceed if under limit

### Cleanup Loop
- Background task runs every window duration
- Removes expired timestamps from each IP's entry
- Removes empty entries to prevent memory leaks

### Integration in server.rs
```rust
// Inject rate limiter into request extensions
.layer(axum::middleware::from_fn(move |mut req, next| {
    let rate_limiter = rate_limiter.clone();
    async move {
        req.extensions_mut().insert(rate_limiter);
        next.run(req).await
    }
}))
// Rate limiting middleware runs after auth
.layer(axum::middleware::from_fn(rate_limit_middleware))
```

### Unit Tests
- `test_under_limit`: 5 requests allowed with limit of 5
- `test_at_limit`: 3 requests succeed at limit of 3, 4th rejected
- `test_over_limit`: 2 requests allowed at limit of 2, 3rd+ rejected
- `test_sliding_window_expiry`: Window expiry after 100ms
- `test_different_ips_independent`: Each IP tracked separately

### Dependencies Added
Added `dashmap` to `aisopod-gateway/Cargo.toml`:
```toml
dashmap = "5"
```

---
*Created: 2026-02-15*
*Resolved: 2026-02-17*
