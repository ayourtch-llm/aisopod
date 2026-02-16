# Issue 151: Implement Security Hardening Measures

## Summary
Apply a set of security hardening measures across the gateway and sandbox systems: non-root container execution, loopback-only binding, HTTPS enforcement, request size limits, input sanitization, secrets masking in logs, and config file permission checks.

## Location
- Crate: `aisopod-gateway`, `aisopod-tools`
- Files:
  - `crates/aisopod-gateway/src/server.rs` (extend)
  - `crates/aisopod-gateway/src/middleware/security.rs` (new)
  - `crates/aisopod-tools/src/sandbox/executor.rs` (extend)
  - `crates/aisopod-config/src/loader.rs` (extend)

## Current Behavior
The gateway binds to the configured address without restricting it to loopback. Containers run as root by default. There are no request size limits, no input sanitization layer, no secrets masking in logs, and no config file permission warnings.

## Expected Behavior
After this issue is completed:
- Containers run as a non-root user by default (using `--user` flag).
- The gateway binds to `127.0.0.1` by default; binding to `0.0.0.0` requires explicit opt-in.
- An HTTPS enforcement option redirects HTTP traffic or refuses non-TLS connections.
- Request body size is limited to a configurable maximum (default: 1 MB).
- User-provided inputs (method params, chat messages) are sanitized to prevent injection.
- Sensitive values (tokens, passwords, API keys) are masked in log output.
- Config file permissions are checked at startup; a warning is emitted if the file is world-readable.

## Impact
These measures reduce the attack surface of a running Aisopod instance. Without them, the system is vulnerable to container escapes (root), network exposure (0.0.0.0 binding), denial-of-service (no size limits), injection attacks (no sanitization), credential leaks (log exposure), and permission misconfigurations.

## Suggested Implementation

1. **Non-root container execution** — extend `SandboxExecutor::create_container()`:
   ```rust
   // In create_container(), add the --user flag
   cmd.args(["--user", "1000:1000"]);
   ```

2. **Loopback-only binding** — update `GatewayConfig` default and server startup:
   ```rust
   // In GatewayConfig default
   impl Default for GatewayConfig {
       fn default() -> Self {
           Self {
               host: "127.0.0.1".to_string(), // Loopback only
               port: 3000,
               // ...
           }
       }
   }

   // In server.rs, log a warning if binding to 0.0.0.0
   if config.host == "0.0.0.0" {
       tracing::warn!(
           "Gateway is binding to 0.0.0.0 — accessible from all network interfaces. \
            Use 127.0.0.1 for local-only access."
       );
   }
   ```

3. **HTTPS enforcement** — add a config option and redirect middleware:
   ```rust
   // In GatewayConfig
   pub https_only: bool, // default: false

   // Middleware that rejects non-HTTPS requests when enabled
   pub async fn https_enforcement(
       req: axum::extract::Request,
       next: axum::middleware::Next,
   ) -> axum::response::Response {
       if let Some(scheme) = req.uri().scheme_str() {
           if scheme != "https" {
               return (
                   axum::http::StatusCode::MOVED_PERMANENTLY,
                   [("Location", format!("https://{}{}", req.uri().host().unwrap_or("localhost"), req.uri().path()))],
               ).into_response();
           }
       }
       next.run(req).await
   }
   ```

4. **Request size limits** — use Axum's `DefaultBodyLimit`:
   ```rust
   use axum::extract::DefaultBodyLimit;

   // In server setup
   let app = Router::new()
       // ... routes ...
       .layer(DefaultBodyLimit::max(config.max_body_size.unwrap_or(1_048_576))); // 1 MB default
   ```

5. **Input sanitization** — create a sanitization utility:
   ```rust
   // crates/aisopod-gateway/src/middleware/security.rs

   /// Sanitize user-provided string input.
   /// Removes null bytes and trims excessive whitespace.
   pub fn sanitize_input(input: &str) -> String {
       input
           .replace('\0', "")   // Remove null bytes
           .trim()
           .to_string()
   }

   /// Validate that a string doesn't contain dangerous patterns.
   pub fn validate_no_injection(input: &str) -> Result<(), &'static str> {
       // Check for common injection patterns in identifiers
       if input.contains("../") || input.contains("..\\") {
           return Err("Path traversal detected");
       }
       Ok(())
   }
   ```

6. **Secrets masking in logs** — implement a `SecretString` wrapper:
   ```rust
   /// A string that masks its value in Debug/Display output.
   #[derive(Clone, serde::Deserialize)]
   pub struct SecretString(String);

   impl SecretString {
       pub fn new(value: String) -> Self {
           Self(value)
       }

       pub fn expose(&self) -> &str {
           &self.0
       }
   }

   impl std::fmt::Debug for SecretString {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           f.write_str("[REDACTED]")
       }
   }

   impl std::fmt::Display for SecretString {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           f.write_str("[REDACTED]")
       }
   }
   ```

7. **Config file permission checks** — extend config loading:
   ```rust
   // In config loader
   #[cfg(unix)]
   fn check_config_permissions(path: &Path) {
       use std::os::unix::fs::PermissionsExt;
       if let Ok(metadata) = std::fs::metadata(path) {
           let mode = metadata.permissions().mode();
           if mode & 0o004 != 0 {
               tracing::warn!(
                   path = %path.display(),
                   mode = format!("{:o}", mode),
                   "Config file is world-readable. Consider restricting permissions: chmod 600 {}",
                   path.display()
               );
           }
       }
   }
   ```

8. **Add unit tests:**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_sanitize_removes_null_bytes() {
           assert_eq!(sanitize_input("hello\0world"), "helloworld");
       }

       #[test]
       fn test_sanitize_trims_whitespace() {
           assert_eq!(sanitize_input("  hello  "), "hello");
       }

       #[test]
       fn test_validate_rejects_path_traversal() {
           assert!(validate_no_injection("../../etc/passwd").is_err());
           assert!(validate_no_injection("normal-input").is_ok());
       }

       #[test]
       fn test_secret_string_masks_output() {
           let secret = SecretString::new("my-secret-token".to_string());
           assert_eq!(format!("{:?}", secret), "[REDACTED]");
           assert_eq!(format!("{}", secret), "[REDACTED]");
           assert_eq!(secret.expose(), "my-secret-token");
       }
   }
   ```

## Dependencies
- Issue 026 (gateway HTTP server skeleton)
- Issue 016 (core configuration types)

## Acceptance Criteria
- [ ] Containers run as non-root user (UID 1000) by default
- [ ] Gateway binds to `127.0.0.1` by default; a warning is logged when binding to `0.0.0.0`
- [ ] HTTPS enforcement option is available and redirects HTTP to HTTPS when enabled
- [ ] Request body size is limited to a configurable maximum (default 1 MB)
- [ ] Input sanitization removes null bytes and detects path traversal
- [ ] `SecretString` type masks sensitive values in Debug and Display output
- [ ] Config file permission check warns when the file is world-readable (Unix only)
- [ ] All hardening measures are active by default (where applicable)
- [ ] Unit tests cover sanitization, secret masking, and permission checks

---
*Created: 2026-02-15*
