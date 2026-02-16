# Issue 186: Add Tier 3 Channel Integration Tests

## Summary
Create integration tests for all Tier 3 channel implementations (Matrix, IRC, Mattermost, Nextcloud Talk, Twitch, Nostr, LINE, Lark/Feishu, Zalo). Tests cover basic connectivity, message send/receive, error handling, and diagnostics for each channel using mock services.

## Location
- Crate: `aisopod-channel-tests`
- File: `crates/aisopod-channel-tests/tests/tier3_integration.rs`

## Current Behavior
No integration tests exist for Tier 3 channels. Individual channel crates may have unit tests, but there are no integration tests verifying end-to-end behavior against the channel abstraction layer.

## Expected Behavior
After implementation:
- Integration tests verify each Tier 3 channel can connect, send, and receive messages.
- Mock services simulate each platform's API/protocol.
- Error handling and diagnostics tests verify clear failure messages.
- Tests run in CI without external service dependencies.

## Impact
Ensures all nine Tier 3 channel implementations are reliable, regression-free, and provide clear diagnostics when issues occur. This is especially important for community-maintained channels where automated testing prevents quality degradation.

## Suggested Implementation

1. **Create test structure:**
   ```
   crates/aisopod-channel-tests/tests/
   ├── tier3_integration.rs
   ├── mocks/
   │   ├── matrix_mock.rs
   │   ├── irc_mock.rs
   │   ├── mattermost_mock.rs
   │   ├── nextcloud_mock.rs
   │   ├── twitch_mock.rs
   │   ├── nostr_mock.rs
   │   ├── line_mock.rs
   │   ├── lark_mock.rs
   │   └── zalo_mock.rs
   └── fixtures/
       ├── matrix_sync.json
       ├── irc_messages.txt
       ├── mattermost_events.json
       ├── twitch_irc.txt
       ├── nostr_events.json
       ├── line_webhooks.json
       ├── lark_events.json
       └── zalo_webhooks.json
   ```

2. **Mock Matrix server** in `mocks/matrix_mock.rs`:
   ```rust
   use axum::Router;

   pub async fn start_mock_homeserver() -> (String, tokio::task::JoinHandle<()>) {
       let app = Router::new()
           .route("/_matrix/client/v3/login", axum::routing::post(mock_login))
           .route("/_matrix/client/v3/sync", axum::routing::get(mock_sync))
           .route("/_matrix/client/v3/rooms/:room_id/send/:event_type/:txn_id",
               axum::routing::put(mock_send_event));

       let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
       let addr = listener.local_addr().unwrap();
       let handle = tokio::spawn(async move {
           axum::serve(listener, app).await.unwrap();
       });
       (format!("http://{}", addr), handle)
   }
   ```

3. **Mock IRC server** in `mocks/irc_mock.rs`:
   ```rust
   use tokio::net::TcpListener;

   pub async fn start_mock_irc_server() -> (String, tokio::task::JoinHandle<()>) {
       let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
       let addr = listener.local_addr().unwrap();
       let handle = tokio::spawn(async move {
           loop {
               let (socket, _) = listener.accept().await.unwrap();
               tokio::spawn(handle_irc_client(socket));
           }
       });
       (format!("127.0.0.1:{}", addr.port()), handle)
   }

   async fn handle_irc_client(socket: tokio::net::TcpStream) {
       // Respond to NICK, USER, JOIN commands
       // Send PRIVMSG test messages
       // Respond to PING with PONG
       todo!()
   }
   ```

4. **Matrix integration tests:**
   ```rust
   #[tokio::test]
   async fn test_matrix_connect_and_send() {
       let (mock_url, _handle) = mocks::matrix_mock::start_mock_homeserver().await;
       let config = MatrixConfig {
           homeserver_url: mock_url,
           auth: MatrixAuth::Password {
               username: "testbot".into(),
               password: "testpass".into(),
           },
           enable_e2ee: false,
           rooms: vec!["!test:localhost".into()],
           state_store_path: None,
       };

       let mut channel = MatrixChannel::new(config);
       assert!(channel.connect().await.is_ok());
       assert!(channel.send(test_message("Hello Matrix")).await.is_ok());
   }

   #[tokio::test]
   async fn test_matrix_receive_message() {
       // Test receiving messages from mock sync response
       todo!()
   }
   ```

5. **IRC integration tests:**
   ```rust
   #[tokio::test]
   async fn test_irc_connect_join_send() {
       let (addr, _handle) = mocks::irc_mock::start_mock_irc_server().await;
       let config = IrcConfig {
           servers: vec![IrcServerConfig {
               server: addr.split(':').next().unwrap().into(),
               port: addr.split(':').last().unwrap().parse().unwrap(),
               use_tls: false,
               nickname: "testbot".into(),
               nickserv_password: None,
               channels: vec!["#test".into()],
               server_password: None,
           }],
       };

       let mut channel = IrcChannel::new(config);
       assert!(channel.connect().await.is_ok());
       assert!(channel.send(test_channel_message("#test", "Hello IRC")).await.is_ok());
   }

   #[tokio::test]
   async fn test_irc_nickserv_auth() {
       // Test NickServ authentication flow
       todo!()
   }
   ```

6. **Mattermost integration tests:**
   ```rust
   #[tokio::test]
   async fn test_mattermost_connect_and_send() {
       let (mock_url, _handle) = mocks::mattermost_mock::start_mock_server().await;
       // Test REST API posting and WebSocket event reception
       todo!()
   }
   ```

7. **Twitch TMI integration tests:**
   ```rust
   #[tokio::test]
   async fn test_twitch_connect_and_chat() {
       // Mock TMI WebSocket server
       // Test PRIVMSG send/receive
       // Test badge parsing (mod, subscriber)
       todo!()
   }

   #[tokio::test]
   async fn test_twitch_whisper() {
       // Test whisper DM functionality
       todo!()
   }
   ```

8. **Nostr relay integration tests:**
   ```rust
   #[tokio::test]
   async fn test_nostr_connect_publish_subscribe() {
       let (relay_url, _handle) = mocks::nostr_mock::start_mock_relay().await;
       // Test relay connection, event publishing, subscription
       todo!()
   }

   #[tokio::test]
   async fn test_nostr_encrypted_dm() {
       // Test NIP-04 encrypted DM send/receive
       todo!()
   }
   ```

9. **LINE, Lark, Zalo webhook tests:**
   ```rust
   #[tokio::test]
   async fn test_line_webhook_and_reply() {
       // Mock LINE API, send webhook event, verify reply
       todo!()
   }

   #[tokio::test]
   async fn test_lark_event_subscription() {
       // Mock Lark API, test event callback handling
       todo!()
   }

   #[tokio::test]
   async fn test_zalo_webhook_and_send() {
       // Mock Zalo API, test webhook event processing
       todo!()
   }
   ```

10. **Error handling and diagnostics tests:**
    ```rust
    #[tokio::test]
    async fn test_matrix_invalid_homeserver() {
        let config = MatrixConfig {
            homeserver_url: "http://localhost:1".into(), // unreachable
            // ...
        };
        let mut channel = MatrixChannel::new(config);
        let result = channel.connect().await;
        assert!(result.is_err());
        // Verify error message is descriptive
        let err = result.unwrap_err();
        assert!(err.to_string().contains("connect") || err.to_string().contains("connection"));
    }

    #[tokio::test]
    async fn test_irc_invalid_server() {
        // Similar error handling test for IRC
        todo!()
    }

    // ... similar tests for each Tier 3 channel
    ```

11. **Shared test infrastructure:**
    ```rust
    fn test_message(text: &str) -> OutboundMessage {
        OutboundMessage {
            text: text.into(),
            recipient: "test-recipient".into(),
            ..Default::default()
        }
    }

    fn test_channel_message(channel: &str, text: &str) -> OutboundMessage {
        OutboundMessage {
            text: text.into(),
            recipient: channel.into(),
            ..Default::default()
        }
    }
    ```

## Dependencies
- Issue 174: Matrix channel implementation
- Issue 175: IRC channel implementation
- Issue 176: Mattermost channel implementation
- Issue 177: Nextcloud Talk channel implementation
- Issue 178: Twitch channel implementation
- Issue 179: Nostr channel implementation
- Issue 180: LINE channel implementation
- Issue 181: Lark/Feishu channel implementation
- Issue 182: Zalo channel implementation

## Acceptance Criteria
- [ ] Matrix connectivity, messaging, and auth tests pass
- [ ] IRC connectivity, channel join, PRIVMSG, and NickServ tests pass
- [ ] Mattermost REST API and WebSocket tests pass
- [ ] Nextcloud Talk room messaging tests pass
- [ ] Twitch TMI chat and whisper tests pass
- [ ] Nostr relay connection, publishing, and NIP-04 DM tests pass
- [ ] LINE webhook and messaging tests pass
- [ ] Lark event subscription and messaging tests pass
- [ ] Zalo webhook and messaging tests pass
- [ ] Error handling tests verify descriptive error messages for each channel
- [ ] All tests run without external service dependencies (mocks only)
- [ ] Tests are included in CI pipeline
- [ ] Test fixtures cover representative message formats per platform

---
*Created: 2026-02-15*
