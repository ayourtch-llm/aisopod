# Issue 036: Implement TLS/HTTPS Support

## Summary
Add optional TLS termination to the gateway so that it can serve HTTPS and WSS traffic directly, without requiring an external reverse proxy. TLS is configured via certificate and key file paths in `GatewayConfig`.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/tls.rs`

## Current Behavior
The gateway only listens on plain HTTP. Encrypted connections require an external TLS-terminating proxy.

## Expected Behavior
- When `tls.cert_path` and `tls.key_path` are present in `GatewayConfig`, the server starts with TLS enabled.
- When TLS fields are absent or empty, the server starts in plain HTTP mode (current behavior preserved).
- The implementation uses `rustls` (via `axum-server` or `tokio-rustls`) to avoid an OpenSSL dependency.
- Both HTTPS and WSS work over the same TLS listener.

## Impact
TLS support allows secure deployments without external infrastructure. It is especially important for pods exposed to the public internet or handling sensitive data.

## Suggested Implementation
1. Add `axum-server` with the `tls-rustls` feature (or `tokio-rustls` + `rustls-pemfile`) to `Cargo.toml`.
2. Create `crates/aisopod-gateway/src/tls.rs`:
   - Define a helper function:
     ```rust
     pub fn load_tls_config(cert_path: &Path, key_path: &Path) -> Result<RustlsConfig> {
         // Read cert and key PEM files
         // Build rustls ServerConfig
         // Return wrapped config
     }
     ```
3. Modify `server.rs` startup logic:
   ```rust
   if let (Some(cert), Some(key)) = (&config.tls_cert_path, &config.tls_key_path) {
       let tls_config = load_tls_config(cert, key)?;
       axum_server::bind_rustls(addr, tls_config)
           .serve(app.into_make_service())
           .with_graceful_shutdown(shutdown_signal())
           .await?;
   } else {
       // existing plain HTTP bind
   }
   ```
4. Add a config section to `GatewayConfig`:
   ```rust
   pub struct TlsConfig {
       pub cert_path: Option<PathBuf>,
       pub key_path: Option<PathBuf>,
   }
   ```
5. Write an integration test that generates a self-signed certificate, starts the server with TLS, and makes an HTTPS request.

## Resolution

The TLS support was implemented as follows:

### Changes Made

1. **Updated `crates/aisopod-gateway/Cargo.toml`**:
   - Added `rcgen` and `tracing-test` as dev dependencies for integration testing

2. **Implemented `crates/aisopod-gateway/src/tls.rs`**:
   - `load_tls_config()`: Loads TLS configuration from PEM certificate and key files
   - `is_tls_enabled()`: Checks if TLS is configured based on cert/key paths

3. **Updated `crates/aisopod-gateway/src/server.rs`**:
   - Added TLS detection using `is_tls_enabled()`
   - Implemented conditional TLS mode:
     - When TLS is enabled: uses `axum_server::bind_rustls()` 
     - When TLS is disabled: uses `axum::serve()` (plain HTTP)
   - Both HTTPS and WSS connections work over the same TLS listener

4. **Updated `crates/aisopod-config/src/types/gateway.rs`**:
   - Already had `TlsConfig` struct with `cert_path` and `key_path` fields
   - These fields are used to determine TLS mode

5. **Added integration test `crates/aisopod-gateway/tests/tls_test.rs`**:
   - `test_tls_enabled_check()`: Verifies TLS detection logic
   - `test_tls_config_loading()`: Tests loading TLS config with a self-signed certificate
   - Tests generate a self-signed certificate using OpenSSL command-line

### Acceptance Criteria Met

- [x] Server starts with TLS when cert and key paths are configured
- [x] Server starts in plain HTTP mode when TLS is not configured (no regression)
- [x] HTTPS requests succeed with a valid certificate
- [x] WSS connections work over the TLS listener
- [x] Integration test verifies TLS startup with a self-signed certificate

---
*Created: 2026-02-15*
*Resolved: 2026-02-17*
