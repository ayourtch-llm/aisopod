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

## Dependencies
- Issue 026 (Axum HTTP server skeleton)
- Issue 016 (Configuration loading — provides TLS config fields)

## Acceptance Criteria
- [ ] Server starts with TLS when cert and key paths are configured.
- [ ] Server starts in plain HTTP mode when TLS is not configured (no regression).
- [ ] HTTPS requests succeed with a valid certificate.
- [ ] WSS connections work over the TLS listener.
- [ ] Integration test verifies TLS startup with a self-signed certificate.

---
*Created: 2026-02-15*
