# Issue 169: Create Protocol Conformance Test Suite

## Summary
Build a comprehensive test suite that validates an aisopod server's compliance with the WebSocket protocol specification. The suite tests every aspect of the protocol from connection handshake to specific RPC method behaviors.

## Location
- Crate: `aisopod-client` (tests directory)
- File: `crates/aisopod-client/tests/conformance/`

## Current Behavior
No automated way to verify that the server correctly implements the protocol specification. Changes to the server could silently break protocol compliance.

## Expected Behavior
A test suite organized by protocol area that can be run against any aisopod server instance. Each test category validates a specific part of the protocol.

## Impact
The conformance test suite is the safety net for all protocol changes. It ensures that server updates do not break existing clients and that the implementation matches the specification document.

## Suggested Implementation
1. Create the test directory structure:
   ```
   crates/aisopod-client/tests/conformance/
   ├── mod.rs
   ├── handshake.rs
   ├── rpc_methods.rs
   ├── error_handling.rs
   ├── device_pairing.rs
   ├── canvas.rs
   └── version_negotiation.rs
   ```
2. Create a shared test harness in `mod.rs`:
   ```rust
   use aisopod_client::{AisopodClient, ClientConfig};

   pub async fn connect_test_client() -> AisopodClient {
       let config = ClientConfig {
           server_url: std::env::var("AISOPOD_TEST_URL")
               .unwrap_or_else(|_| "ws://127.0.0.1:8080/ws".to_string()),
           auth_token: std::env::var("AISOPOD_TEST_TOKEN")
               .unwrap_or_else(|_| "test-token".to_string()),
           client_name: "conformance-test".to_string(),
           client_version: "0.1.0".to_string(),
           device_id: uuid::Uuid::new_v4(),
           protocol_version: "1.0".to_string(),
       };
       AisopodClient::connect(config).await.expect("Failed to connect")
   }
   ```
3. Implement handshake tests in `handshake.rs`:
   ```rust
   #[tokio::test]
   async fn test_successful_handshake() {
       let client = connect_test_client().await;
       // Verify welcome message was received
       assert!(client.is_connected());
   }

   #[tokio::test]
   async fn test_handshake_without_auth_header() {
       // Connect without Authorization header, expect rejection
       let result = connect_without_auth().await;
       assert!(result.is_err());
   }

   #[tokio::test]
   async fn test_welcome_message_fields() {
       let client = connect_test_client().await;
       let welcome = client.welcome_message();
       assert!(!welcome.server_version.is_empty());
       assert!(!welcome.protocol_version.is_empty());
       assert!(!welcome.session_id.is_empty());
   }
   ```
4. Implement RPC method tests in `rpc_methods.rs`:
   ```rust
   #[tokio::test]
   async fn test_unknown_method_returns_method_not_found() {
       let mut client = connect_test_client().await;
       let result: Result<serde_json::Value, _> =
           client.request("nonexistent.method", serde_json::json!({})).await;
       assert!(result.is_err());
       // Verify error code is -32601 (Method not found)
   }

   #[tokio::test]
   async fn test_malformed_json_rpc_returns_error() {
       let mut client = connect_test_client().await;
       client.send_raw(r#"{"not": "valid jsonrpc"}"#).await.unwrap();
       let error = client.receive_error().await.unwrap();
       assert_eq!(error.code, -32600); // Invalid request
   }
   ```
5. Implement device pairing tests in `device_pairing.rs`:
   ```rust
   #[tokio::test]
   async fn test_pair_request_returns_code() {
       let mut client = connect_test_client().await;
       let result = client.node_pair_request(test_device_info()).await.unwrap();
       assert_eq!(result.pairing_code.len(), 6);
       assert!(result.expires_at > chrono::Utc::now());
   }

   #[tokio::test]
   async fn test_pair_confirm_with_invalid_code() {
       let mut client = connect_test_client().await;
       let result = client.node_pair_confirm("000000").await;
       assert!(result.is_err());
   }
   ```
6. Implement canvas tests in `canvas.rs`:
   ```rust
   #[tokio::test]
   async fn test_canvas_interact_unknown_canvas() {
       let mut client = connect_test_client().await;
       let result = client.canvas_interact("nonexistent", "click", None).await;
       assert!(result.is_err());
   }
   ```
7. Implement version negotiation tests in `version_negotiation.rs`:
   ```rust
   #[tokio::test]
   async fn test_compatible_version() {
       let client = connect_with_version("1.0").await;
       assert!(client.is_ok());
   }

   #[tokio::test]
   async fn test_incompatible_major_version() {
       let result = connect_with_version("99.0").await;
       assert!(result.is_err());
   }

   #[tokio::test]
   async fn test_missing_version_defaults_to_1_0() {
       let client = connect_without_version_header().await;
       assert!(client.is_ok());
   }
   ```
8. Add a CI configuration note: these tests require a running server instance, so they should be gated behind a feature flag or environment variable.

## Dependencies
- Issue 162 (protocol specification — defines what to test)
- Issue 163 (device pairing — tested by pairing tests)
- Issue 164 (node describe/invoke — tested by RPC method tests)
- Issue 165 (canvas protocol — tested by canvas tests)
- Issue 166 (version negotiation — tested by version tests)
- Issue 167 (migration doc — informs naming expectations)
- Issue 168 (client library — used to run the tests)

## Acceptance Criteria
- [ ] Conformance test directory exists with organized test modules
- [ ] Handshake tests validate connection flow and welcome message
- [ ] RPC method tests validate request/response schemas and error codes
- [ ] Error handling tests cover malformed messages, unauthorized access, and rate limiting
- [ ] Device pairing tests cover the full pair/confirm/revoke flow
- [ ] Canvas tests cover update delivery and interaction reporting
- [ ] Version negotiation tests cover compatible, incompatible, and missing versions
- [ ] All tests pass against a correctly configured server
- [ ] Tests can be run in CI with a server started as a test fixture

## Resolution

Created a comprehensive protocol conformance test suite for the aisopod-client crate:

- Created conformance test directory at `crates/aisopod-client/tests/conformance/`
- Implemented all 6 test modules: `handshake.rs`, `rpc_methods.rs`, `error_handling.rs`, `device_pairing.rs`, `canvas.rs`, `version_negotiation.rs`
- Each module contains the required tests as specified in the issue
- Environment gating via `AISOPOD_TEST_URL` and `RUN_CONFORMANCE_TESTS` environment variables implemented
- Added missing client methods: `node_pair_confirm()`, `node_pair_revoke()`, `canvas_interact()`
- Added `PairConfirmResult` and `PairRevokeResult` types
- Added `test_rate_limiting()` test
- All 39 tests passing (3 unit + 25 conformance + 11 integration)
- All changes committed in commit 4313866

---
*Created: 2026-02-15*
*Resolved: 2026-02-25*
