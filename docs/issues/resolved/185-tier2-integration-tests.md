# Issue 185: Add Tier 2 Channel Integration Tests

## Summary
Create integration tests for all Tier 2 channel implementations (Signal, iMessage, Google Chat, Microsoft Teams). Tests cover connectivity, message send/receive, and error handling for each channel using mock services and test fixtures.

## Location
- Crate: `aisopod-channel-tests`
- File: `crates/aisopod-channel-tests/tests/tier2_integration.rs`

## Current Behavior
No integration tests exist for Tier 2 channels. Each channel may have its own unit tests, but there are no cross-channel integration tests that verify end-to-end behavior against the channel abstraction layer.

## Expected Behavior
After implementation:
- Integration tests verify each Tier 2 channel can connect, send, and receive messages.
- Mock services simulate each platform's API without requiring real credentials.
- Error handling tests verify graceful failure modes.
- Tests run in CI without external service dependencies.

## Impact
Ensures Tier 2 channel implementations are reliable, catch regressions early, and maintain compatibility with the channel abstraction layer as it evolves.

## Suggested Implementation

1. **Create test structure:**
   ```
   crates/aisopod-channel-tests/
   ├── Cargo.toml
   └── tests/
       ├── tier2_integration.rs
       ├── mocks/
       │   ├── mod.rs
       │   ├── signal_mock.rs
       │   ├── imessage_mock.rs
       │   ├── googlechat_mock.rs
       │   └── msteams_mock.rs
       └── fixtures/
           ├── signal_messages.json
           ├── googlechat_events.json
           └── teams_activities.json
   ```

2. **Mock Signal CLI** in `mocks/signal_mock.rs`:
   ```rust
   use tokio::process::Command;
   use std::path::PathBuf;

   /// Mock signal-cli that responds with predefined JSON messages.
   pub struct MockSignalCli {
       script_path: PathBuf,
   }

   impl MockSignalCli {
       pub fn new() -> Self {
           // Create a temporary script that mimics signal-cli output
           todo!()
       }

       pub fn path(&self) -> &str {
           self.script_path.to_str().unwrap()
       }
   }
   ```

3. **Signal integration test:**
   ```rust
   #[tokio::test]
   async fn test_signal_connect_and_send() {
       let mock = mocks::signal_mock::MockSignalCli::new();
       let config = SignalConfig {
           signal_cli_path: mock.path().to_string(),
           phone_number: "+1234567890".into(),
           groups: vec![],
           poll_interval_secs: 1,
       };

       let mut channel = SignalChannel::new(config);
       assert!(channel.connect().await.is_ok());
       assert!(channel.send(test_message("Hello")).await.is_ok());
       assert!(channel.disconnect().await.is_ok());
   }

   #[tokio::test]
   async fn test_signal_receive_dm() {
       let mock = mocks::signal_mock::MockSignalCli::new();
       // Configure mock to emit a DM
       let mut channel = create_signal_channel(&mock);
       channel.connect().await.unwrap();
       let msg = channel.receive().await.unwrap();
       assert_eq!(msg.text, "Hello from Signal");
       assert!(!msg.is_group);
   }

   #[tokio::test]
   async fn test_signal_receive_group_message() {
       // Similar test for group messages
       todo!()
   }

   #[tokio::test]
   async fn test_signal_cli_unavailable() {
       let config = SignalConfig {
           signal_cli_path: "/nonexistent/path".into(),
           phone_number: "+1234567890".into(),
           groups: vec![],
           poll_interval_secs: 1,
       };
       let mut channel = SignalChannel::new(config);
       let result = channel.connect().await;
       assert!(result.is_err());
   }
   ```

4. **Mock Google Chat API** in `mocks/googlechat_mock.rs`:
   ```rust
   use axum::{Router, Json};

   /// Start a mock Google Chat API server.
   pub async fn start_mock_server() -> (String, tokio::task::JoinHandle<()>) {
       let app = Router::new()
           .route("/v1/spaces/:space/messages", axum::routing::post(mock_send_message))
           .route("/oauth2/v4/token", axum::routing::post(mock_token));

       let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
       let addr = listener.local_addr().unwrap();
       let handle = tokio::spawn(async move {
           axum::serve(listener, app).await.unwrap();
       });

       (format!("http://{}", addr), handle)
   }

   async fn mock_send_message(Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
       Json(serde_json::json!({ "name": "spaces/test/messages/123" }))
   }

   async fn mock_token() -> Json<serde_json::Value> {
       Json(serde_json::json!({
           "access_token": "mock-token",
           "expires_in": 3600,
           "token_type": "Bearer"
       }))
   }
   ```

5. **Google Chat integration test:**
   ```rust
   #[tokio::test]
   async fn test_googlechat_connect_and_send() {
       let (mock_url, _handle) = mocks::googlechat_mock::start_mock_server().await;
       let config = GoogleChatConfig {
           auth: GoogleChatAuth::ServiceAccount { key_file: "test_key.json".into() },
           project_id: "test-project".into(),
           webhook_port: 0,
           spaces: vec!["spaces/test".into()],
       };

       // Override base URL to point to mock
       let mut channel = GoogleChatChannel::new_with_base_url(config, &mock_url);
       assert!(channel.connect().await.is_ok());
       assert!(channel.send(test_message("Hello Google Chat")).await.is_ok());
   }
   ```

6. **Mock Teams Bot Framework** — similar pattern with mock HTTP server for Bot Framework activities.

7. **iMessage platform test:**
   ```rust
   #[tokio::test]
   async fn test_imessage_non_macos_error() {
       #[cfg(not(target_os = "macos"))]
       {
           let config = IMessageConfig {
               backend: IMessageBackend::AppleScript,
               bluebubbles_url: None,
               bluebubbles_password: None,
               poll_interval_secs: 1,
           };
           let mut channel = IMessageChannel::new(config);
           let result = channel.connect().await;
           assert!(result.is_err());
           assert!(result.unwrap_err().to_string().contains("macOS"));
       }
   }
   ```

8. **Shared test helpers:**
   ```rust
   fn test_message(text: &str) -> OutboundMessage {
       OutboundMessage {
           text: text.into(),
           recipient: "test-recipient".into(),
           ..Default::default()
       }
   }
   ```

## Dependencies
- Issue 170: Signal channel implementation
- Issue 171: iMessage channel implementation
- Issue 172: Google Chat channel implementation
- Issue 173: Microsoft Teams channel implementation

## Acceptance Criteria
- [ ] Signal connectivity and message send/receive tests pass
- [ ] Signal error handling tests pass (missing CLI, invalid phone, etc.)
- [ ] iMessage platform detection test passes (error on non-macOS)
- [ ] Google Chat API connectivity and messaging tests pass
- [ ] Google Chat OAuth and service account auth tests pass
- [ ] Microsoft Teams Bot Framework connectivity tests pass
- [ ] Teams Adaptive Card rendering tests pass
- [ ] All tests run without external service dependencies (mocks only)
- [ ] Tests are included in CI pipeline
- [ ] Test fixtures cover representative message formats per platform

---
*Created: 2026-02-15*
