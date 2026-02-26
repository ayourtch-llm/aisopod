# Issue 168: Build Reference WebSocket Client Library

## Summary
Create a Rust reference client library (`aisopod-client`) that implements the aisopod WebSocket protocol for use in integration tests, CLI tools, and as a reference for third-party client implementations.

## Location
- Crate: `aisopod-client` (new library crate)
- File: `crates/aisopod-client/src/lib.rs`

## Current Behavior
No client library exists. Testing the WebSocket protocol requires manually constructing raw WebSocket frames and JSON-RPC messages.

## Expected Behavior
A reusable async Rust library that:

1. Connects to an aisopod server via WebSocket with proper handshake headers.
2. Authenticates using bearer tokens or device tokens.
3. Sends JSON-RPC requests and awaits responses.
4. Receives and dispatches server-initiated events and broadcasts.
5. Provides typed helper methods for common operations.

## Impact
This client library will be used by the conformance test suite (issue 169), integration tests, and any Rust-based tooling that needs to communicate with an aisopod server. It also serves as a living reference implementation of the protocol.

## Suggested Implementation
1. Create the new crate:
   ```bash
   cargo new crates/aisopod-client --lib
   ```
2. Add dependencies to `crates/aisopod-client/Cargo.toml`:
   ```toml
   [dependencies]
   tokio = { version = "1", features = ["full"] }
   tokio-tungstenite = "0.21"
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   uuid = { version = "1", features = ["v4"] }
   thiserror = "1"
   tracing = "0.1"
   ```
3. Define the client struct in `src/lib.rs`:
   ```rust
   pub mod client;
   pub mod error;
   pub mod message;
   pub mod types;
   ```
4. Implement the core client in `src/client.rs`:
   ```rust
   use tokio_tungstenite::tungstenite::Message;
   use uuid::Uuid;

   pub struct AisopodClient {
       ws_stream: WebSocketStream,
       pending_requests: HashMap<String, oneshot::Sender<RpcResponse>>,
       event_receiver: mpsc::Receiver<ServerEvent>,
   }

   pub struct ClientConfig {
       pub server_url: String,
       pub auth_token: String,
       pub client_name: String,
       pub client_version: String,
       pub device_id: Uuid,
       pub protocol_version: String,
   }

   impl AisopodClient {
       /// Connect to an aisopod server and perform the handshake.
       pub async fn connect(config: ClientConfig) -> Result<Self, ClientError> {
           let request = build_upgrade_request(&config)?;
           let (ws_stream, _response) = tokio_tungstenite::connect_async(request).await?;

           let mut client = Self::new(ws_stream);

           // Wait for welcome message
           let welcome = client.receive_welcome().await?;
           tracing::info!("Connected to server v{}", welcome.server_version);

           Ok(client)
       }

       /// Send an RPC request and await the response.
       pub async fn request<P: Serialize, R: DeserializeOwned>(
           &mut self,
           method: &str,
           params: P,
       ) -> Result<R, ClientError> {
           let id = Uuid::new_v4().to_string();
           let msg = RpcRequest { jsonrpc: "2.0", id: &id, method, params };
           self.send_raw(serde_json::to_string(&msg)?).await?;
           let response = self.await_response(&id).await?;
           Ok(serde_json::from_value(response.result)?)
       }
   }
   ```
5. Add typed helper methods in `src/client.rs`:
   ```rust
   impl AisopodClient {
       pub async fn chat_send(
           &mut self, agent_id: &str, message: &str,
       ) -> Result<ChatResponse, ClientError> {
           self.request("chat.send", ChatSendParams { agent_id, message }).await
       }

       pub async fn node_pair_request(
           &mut self, device_info: DeviceInfo,
       ) -> Result<PairRequestResult, ClientError> {
           self.request("node.pair.request", device_info).await
       }

       pub async fn node_describe(
           &mut self, capabilities: Vec<DeviceCapability>,
       ) -> Result<NodeDescribeResult, ClientError> {
           self.request("node.describe", NodeDescribeParams { capabilities }).await
       }
   }
   ```
6. Implement the background event loop that reads from the WebSocket, routes responses to pending request channels, and forwards events to the event receiver.
7. Add the new crate to the workspace `Cargo.toml`.

## Dependencies
- Issue 028 (WebSocket connection lifecycle)
- Issue 029 (JSON-RPC message format)
- Issue 162 (protocol specification for message schemas)

## Acceptance Criteria
- [x] `crates/aisopod-client` crate exists and compiles
- [x] Client connects to a running aisopod server via WebSocket
- [x] Handshake sends correct upgrade headers (Authorization, X-Aisopod-Client, etc.)
- [x] Client receives and parses the welcome message
- [x] `request()` sends JSON-RPC and correctly matches response by ID
- [x] Server events are received and dispatched to the event channel
- [x] Helper methods exist for chat, node pairing, node describe, and node invoke
- [x] Basic integration test demonstrates a connect → authenticate → request → disconnect flow

## Resolution
Created new `aisopod-client` crate (`crates/aisopod-client`) with the following implementation:

**Dependencies added to `crates/aisopod-client/Cargo.toml`:**
- `tokio` with `full` features for async runtime
- `tokio-tungstenite` for WebSocket support
- `serde` with `derive` for serialization
- `serde_json` for JSON handling
- `uuid` with `v4` feature for request IDs
- `thiserror` for error handling
- `tracing` for logging

**Core client implementation in `src/client.rs`:**
- `AisopodClient` struct with `ws_stream`, `pending_requests`, and `event_receiver`
- `ClientConfig` struct with `server_url`, `auth_token`, `client_name`, `client_version`, `device_id`, and `protocol_version`
- `connect()` method - WebSocket connection with proper handshake headers
- `request()` method - JSON-RPC request/response handling
- Helper methods: `chat_send`, `node_pair_request`, `node_describe`, `node_invoke`
- Background event loop for reading WebSocket messages

**Workspace updates:**
- Updated `uuid` dependency at workspace level to include `std` feature

**Testing:**
- All 14 tests passing (3 unit + 11 integration)

**Changes committed** to the repository.

---
*Created: 2026-02-15*
*Resolved: 2026-02-25*
