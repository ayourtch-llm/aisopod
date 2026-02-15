# Issue 152: Implement Audit Logging and Add Security Tests

## Summary
Implement structured audit logging for all security-relevant events and add integration tests that verify sandbox isolation, authentication, and authorization. This is the capstone issue for the Sandbox & Security plan.

## Location
- Crate: `aisopod-gateway`, `aisopod-tools`
- Files:
  - `crates/aisopod-gateway/src/audit.rs` (new)
  - `crates/aisopod-gateway/tests/security_integration.rs` (new)
  - `crates/aisopod-tools/tests/sandbox_integration.rs` (new)

## Current Behavior
Security-relevant events (authentication attempts, authorization decisions, tool executions, approval workflows) are not systematically logged. There are no integration tests that verify the sandbox isolation boundary or the end-to-end auth/authorization flow.

## Expected Behavior
After this issue is completed:
- All security-relevant events are logged using structured `tracing` spans and events.
- Authentication attempts (success and failure) include the client IP address.
- Authorization decisions log the method, required scope, and outcome.
- Tool executions log the tool name, agent, and whether it ran in a sandbox.
- Approval workflow events log the request, decision, and timing.
- Config changes are logged with before/after values (secrets redacted).
- Integration tests verify that sandbox containers cannot access the host filesystem outside the workspace.
- Security tests verify that unauthenticated requests are rejected and scope enforcement works end-to-end.

## Impact
Audit logging is essential for incident response, compliance, and debugging. Without it, security events are invisible. Without security integration tests, regressions in sandbox isolation or auth enforcement could go undetected.

## Suggested Implementation

1. **Create the audit logging module** (`crates/aisopod-gateway/src/audit.rs`):
   ```rust
   use tracing::{info, warn};

   /// Log a successful authentication.
   pub fn log_auth_success(client_ip: &str, auth_mode: &str, role: &str) {
       info!(
           target: "audit",
           event = "auth_success",
           client_ip = client_ip,
           auth_mode = auth_mode,
           role = role,
           "Authentication successful"
       );
   }

   /// Log a failed authentication attempt.
   pub fn log_auth_failure(client_ip: &str, auth_mode: &str, reason: &str) {
       warn!(
           target: "audit",
           event = "auth_failure",
           client_ip = client_ip,
           auth_mode = auth_mode,
           reason = reason,
           "Authentication failed"
       );
   }

   /// Log an authorization decision.
   pub fn log_authz_decision(
       method: &str,
       required_scope: &str,
       granted: bool,
       client_ip: &str,
   ) {
       if granted {
           info!(
               target: "audit",
               event = "authz_granted",
               method = method,
               required_scope = required_scope,
               client_ip = client_ip,
               "Authorization granted"
           );
       } else {
           warn!(
               target: "audit",
               event = "authz_denied",
               method = method,
               required_scope = required_scope,
               client_ip = client_ip,
               "Authorization denied"
           );
       }
   }

   /// Log a tool execution.
   pub fn log_tool_execution(
       tool_name: &str,
       agent_id: &str,
       sandboxed: bool,
       session_key: &str,
   ) {
       info!(
           target: "audit",
           event = "tool_execution",
           tool_name = tool_name,
           agent_id = agent_id,
           sandboxed = sandboxed,
           session_key = session_key,
           "Tool executed"
       );
   }

   /// Log an approval workflow event.
   pub fn log_approval_event(
       request_id: &str,
       agent_id: &str,
       operation: &str,
       decision: &str,
       duration_ms: u64,
   ) {
       info!(
           target: "audit",
           event = "approval_decision",
           request_id = request_id,
           agent_id = agent_id,
           operation = operation,
           decision = decision,
           duration_ms = duration_ms,
           "Approval workflow completed"
       );
   }

   /// Log a configuration change (with secrets redacted).
   pub fn log_config_change(field: &str, old_value: &str, new_value: &str) {
       info!(
           target: "audit",
           event = "config_change",
           field = field,
           old_value = old_value,
           new_value = new_value,
           "Configuration changed"
       );
   }
   ```

2. **Integrate audit calls** into existing middleware and handlers:
   ```rust
   // In auth middleware (crates/aisopod-gateway/src/middleware/auth.rs)
   match authenticate(&request, &config) {
       Ok(identity) => {
           audit::log_auth_success(&client_ip, &config.mode_name(), &identity.role);
           // ... proceed
       }
       Err(e) => {
           audit::log_auth_failure(&client_ip, &config.mode_name(), &e.to_string());
           // ... reject
       }
   }

   // In scope check (crates/aisopod-gateway/src/auth/scopes.rs)
   let granted = ctx.scopes.contains(required);
   audit::log_authz_decision(method, required.as_str(), granted, &ctx.client_ip);
   ```

3. **Add sandbox integration tests** (`crates/aisopod-tools/tests/sandbox_integration.rs`):
   ```rust
   use aisopod_tools::sandbox::{SandboxConfig, SandboxExecutor, SandboxRuntime, WorkspaceAccess};
   use std::path::Path;
   use std::time::Duration;
   use tempfile::TempDir;

   #[tokio::test]
   #[ignore] // Requires Docker
   async fn test_sandbox_cannot_access_host_filesystem() {
       let executor = SandboxExecutor::new(SandboxRuntime::Docker);
       let workspace = TempDir::new().unwrap();

       let config = SandboxConfig {
           enabled: true,
           image: "alpine:latest".to_string(),
           workspace_access: WorkspaceAccess::ReadOnly,
           timeout: Duration::from_secs(10),
           ..Default::default()
       };

       // Try to read /etc/hostname from inside the container
       // (should be the container's hostname, not the host's)
       let result = executor
           .run_one_shot(&config, "cat /etc/hostname", workspace.path())
           .await
           .unwrap();

       assert_eq!(result.exit_code, 0);
       // The hostname should be the container ID, not the host
       assert!(!result.stdout.is_empty());
   }

   #[tokio::test]
   #[ignore] // Requires Docker
   async fn test_readonly_workspace_prevents_writes() {
       let executor = SandboxExecutor::new(SandboxRuntime::Docker);
       let workspace = TempDir::new().unwrap();

       let config = SandboxConfig {
           enabled: true,
           image: "alpine:latest".to_string(),
           workspace_access: WorkspaceAccess::ReadOnly,
           timeout: Duration::from_secs(10),
           ..Default::default()
       };

       let result = executor
           .run_one_shot(&config, "touch /workspace/test.txt", workspace.path())
           .await
           .unwrap();

       // Should fail because workspace is mounted read-only
       assert_ne!(result.exit_code, 0);
   }

   #[tokio::test]
   #[ignore] // Requires Docker
   async fn test_no_workspace_mount() {
       let executor = SandboxExecutor::new(SandboxRuntime::Docker);
       let workspace = TempDir::new().unwrap();

       let config = SandboxConfig {
           enabled: true,
           image: "alpine:latest".to_string(),
           workspace_access: WorkspaceAccess::None,
           timeout: Duration::from_secs(10),
           ..Default::default()
       };

       let result = executor
           .run_one_shot(&config, "ls /workspace", workspace.path())
           .await
           .unwrap();

       // /workspace should not exist when access is None
       assert_ne!(result.exit_code, 0);
   }

   #[tokio::test]
   #[ignore] // Requires Docker
   async fn test_resource_limits_enforced() {
       let executor = SandboxExecutor::new(SandboxRuntime::Docker);
       let workspace = TempDir::new().unwrap();

       let config = SandboxConfig {
           enabled: true,
           image: "alpine:latest".to_string(),
           memory_limit: Some("32m".to_string()),
           cpu_limit: Some(0.5),
           timeout: Duration::from_secs(10),
           ..Default::default()
       };

       // This should succeed but be constrained
       let result = executor
           .run_one_shot(&config, "echo 'constrained'", workspace.path())
           .await
           .unwrap();

       assert_eq!(result.exit_code, 0);
       assert_eq!(result.stdout.trim(), "constrained");
   }
   ```

4. **Add auth/authorization security tests** (`crates/aisopod-gateway/tests/security_integration.rs`):
   ```rust
   use axum_test::TestServer;
   use serde_json::json;

   #[tokio::test]
   async fn test_unauthenticated_request_rejected() {
       let app = build_app_with_token_auth("test-token").await;
       let server = TestServer::new(app).unwrap();

       let response = server
           .post("/rpc")
           .json(&json!({
               "jsonrpc": "2.0",
               "method": "agent.list",
               "id": 1
           }))
           .await;

       assert_eq!(response.status(), 401);
   }

   #[tokio::test]
   async fn test_authenticated_request_accepted() {
       let app = build_app_with_token_auth("test-token").await;
       let server = TestServer::new(app).unwrap();

       let response = server
           .post("/rpc")
           .header("Authorization", "Bearer test-token")
           .json(&json!({
               "jsonrpc": "2.0",
               "method": "agent.list",
               "id": 1
           }))
           .await;

       assert_eq!(response.status(), 200);
   }

   #[tokio::test]
   async fn test_insufficient_scope_rejected() {
       let app = build_app_with_scoped_auth(vec!["operator.read"]).await;
       let server = TestServer::new(app).unwrap();

       let response = server
           .post("/rpc")
           .header("Authorization", "Bearer test-token")
           .json(&json!({
               "jsonrpc": "2.0",
               "method": "admin.shutdown",
               "id": 1
           }))
           .await;

       let body: serde_json::Value = response.json();
       assert!(body["error"].is_object());
       assert!(body["error"]["message"]
           .as_str()
           .unwrap()
           .contains("Insufficient permissions"));
   }
   ```

5. **Set up a `tracing` subscriber** that captures audit events:
   ```rust
   // In main.rs or gateway setup
   use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

   tracing_subscriber::registry()
       .with(EnvFilter::new("audit=info,aisopod=info"))
       .with(tracing_subscriber::fmt::layer().json()) // JSON output for structured logs
       .init();
   ```

## Dependencies
- Issue 144 (sandbox configuration types)
- Issue 145 (container execution)
- Issue 146 (workspace access controls)
- Issue 147 (execution approval workflow)
- Issue 148 (API token generation and password hashing)
- Issue 149 (scope-based authorization)
- Issue 150 (device token management)
- Issue 151 (security hardening measures)

## Acceptance Criteria
- [ ] Authentication successes and failures are logged with client IP and auth mode
- [ ] Authorization decisions are logged with method name, scope, and outcome
- [ ] Tool executions are logged with tool name, agent ID, and sandbox status
- [ ] Approval workflow events are logged with request ID, decision, and duration
- [ ] Config changes are logged with field name and redacted values
- [ ] All audit events use the `audit` tracing target for easy filtering
- [ ] Integration tests verify sandbox containers cannot escape the workspace
- [ ] Integration tests verify read-only workspace prevents writes
- [ ] Security tests verify unauthenticated requests are rejected
- [ ] Security tests verify insufficient scopes are rejected
- [ ] All tests pass with `cargo test` (sandbox tests marked `#[ignore]` for CI without Docker)

---
*Created: 2026-02-15*
