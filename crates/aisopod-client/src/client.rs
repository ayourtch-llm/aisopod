//! WebSocket client for aisopod protocol

use crate::error::{ClientError, Result};
use crate::message::{error_response, parse_response, RpcRequest, RpcResponse};
use crate::types::{AuthRequest, AuthResponse, ClientConfig, ClientState, ServerEvent};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};

/// The main aisopod WebSocket client
#[derive(Debug)]
pub struct AisopodClient {
    /// WebSocket stream for communication
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    /// Pending requests waiting for responses
    pending_requests: HashMap<String, oneshot::Sender<RpcResponse>>,
    /// Channel for receiving server events
    event_receiver: mpsc::Receiver<ServerEvent>,
    /// Client configuration
    config: ClientConfig,
    /// Current client state
    state: ClientState,
}

impl AisopodClient {
    /// Create a new client from a WebSocket stream
    pub fn new(
        ws_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        config: ClientConfig,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::channel(100);
        Self {
            ws_stream,
            pending_requests: HashMap::new(),
            event_receiver: event_rx,
            config,
            state: ClientState::Connected,
        }
    }

    /// Connect to an aisopod server and perform the handshake
    pub async fn connect(config: ClientConfig) -> Result<Self> {
        info!(
            "Connecting to {} as {}",
            config.server_url, config.client_name
        );

        // Parse URL first to validate it
        let parsed_url = config.server_url.parse::<url::Url>().map_err(|e| {
            ClientError::Protocol(format!("Invalid server URL: {}", e))
        })?;

        // Prepare upgrade headers
        let url_str = parsed_url.as_str();
        let mut request = tungstenite::handshake::client::Request::new(());
        *request.uri_mut() = url_str.parse().map_err(|e| {
            ClientError::Protocol(format!("Invalid server URL: {}", e))
        })?;
        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", config.auth_token)
                .parse()
                .map_err(|_| ClientError::Auth("Invalid authorization header".to_string()))?,
        );
        request.headers_mut().insert(
            "X-Aisopod-Client",
            format!("{} {}", config.client_name, config.client_version)
                .parse()
                .map_err(|_| ClientError::Auth("Invalid client header".to_string()))?,
        );
        request.headers_mut().insert(
            "X-Aisopod-Device-Id",
            config.device_id.to_string().parse().unwrap_or_else(|_| "unknown".parse().unwrap()),
        );
        request.headers_mut().insert(
            "X-Aisopod-Protocol-Version",
            config.protocol_version.parse().unwrap_or_else(|_| "1.0".parse().unwrap()),
        );

        // Connect to WebSocket
        let (ws_stream, response) = connect_async(request)
            .await
            .map_err(|e| ClientError::WebSocket(e))?;

        info!("WebSocket connected, status: {}", response.status());

        let mut client = Self::new(ws_stream, config);

        // Wait for welcome message
        client.receive_welcome().await?;

        Ok(client)
    }

    /// Wait for the welcome message from the server
    async fn receive_welcome(&mut self) -> Result<()> {
        let msg = self
            .ws_stream
            .next()
            .await
            .ok_or_else(|| ClientError::Closed)?;

        match msg? {
            Message::Text(text) => {
                debug!("Received welcome message: {}", text);
                // Welcome message is handled by the event loop
                Ok(())
            }
            Message::Close(_) => Err(ClientError::Closed),
            _ => Err(ClientError::Protocol("Expected text message for welcome".to_string())),
        }
    }

    /// Send a JSON-RPC request and await the response
    pub async fn request<P: serde::Serialize, R: for<'de> serde::Deserialize<'de>>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<R> {
        if self.state != ClientState::Connected {
            return Err(ClientError::Protocol(format!(
                "Client not connected (state: {:?})",
                self.state
            )));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let params_value = serde_json::to_value(params).map_err(ClientError::Json)?;

        let request = RpcRequest::new(method, Some(params_value), &id);
        let request_json = serde_json::to_string(&request).map_err(ClientError::Json)?;

        debug!("Sending request: {} (id: {})", method, id);

        // Create channel for response
        let (tx, rx) = oneshot::channel();

        // Store the channel for this request ID
        self.pending_requests.insert(id.clone(), tx);

        // Send the request
        self.ws_stream
            .send(Message::Text(request_json))
            .await
            .map_err(|e| ClientError::WebSocket(e))?;

        // Await the response with a timeout
        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(response)) => {
                if let Some(result) = response.result {
                    serde_json::from_value(result).map_err(ClientError::Json)
                } else if let Some(error) = response.error {
                    Err(ClientError::Protocol(format!(
                        "JSON-RPC error [{}]: {}",
                        error.code, error.message
                    )))
                } else {
                    Err(ClientError::InvalidResponse("Empty response".to_string()))
                }
            }
            Ok(Err(_)) => Err(ClientError::MessageIdNotFound(id)),
            Err(_) => Err(ClientError::Timeout(30)),
        }
    }

    /// Send a chat message to an agent
    pub async fn chat_send(&mut self, agent_id: &str, message: &str) -> Result<crate::types::ChatResponse> {
        let params = serde_json::json!({
            "agent_id": agent_id,
            "message": message
        });
        self.request("chat.send", params).await
    }

    /// Request node pairing
    pub async fn node_pair_request(
        &mut self,
        device_info: crate::types::DeviceInfo,
    ) -> Result<crate::types::PairRequestResult> {
        self.request("node.pair.request", device_info).await
    }

    /// Describe the node capabilities
    pub async fn node_describe(
        &mut self,
        capabilities: Vec<crate::types::DeviceCapability>,
    ) -> Result<crate::types::NodeDescribeResult> {
        let params = serde_json::json!({
            "capabilities": capabilities
        });
        self.request("node.describe", params).await
    }

    /// Invoke a node method
    pub async fn node_invoke(
        &mut self,
        node_id: &str,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<crate::types::NodeInvokeResult> {
        let params = params.map_or_else(
            || serde_json::json!({"node_id": node_id, "method": method}),
            |p| serde_json::json!({"node_id": node_id, "method": method, "params": p}),
        );
        self.request("node.invoke", params).await
    }

    /// Start the background event loop
    pub async fn start_event_loop(&mut self) {
        let mut event_loop = EventLoop {
            ws_stream: std::mem::replace(&mut self.ws_stream, unreachable!()),
            pending_requests: std::mem::take(&mut self.pending_requests),
            event_sender: None,
        };

        // Note: This method would typically spawn the event loop as a background task
        // For now, we just update the client state
        self.state = ClientState::Connected;
    }

    /// Get the current client state
    pub fn state(&self) -> ClientState {
        self.state
    }

    /// Check if the client is connected
    pub fn is_connected(&self) -> bool {
        self.state == ClientState::Connected
    }
}

/// Background event loop that processes incoming messages
#[allow(dead_code)]
struct EventLoop {
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    pending_requests: HashMap<String, oneshot::Sender<RpcResponse>>,
    event_sender: Option<mpsc::Sender<ServerEvent>>,
}

impl EventLoop {
    /// Process incoming WebSocket messages
    async fn run(mut self) {
        loop {
            match self.ws_stream.next().await {
                Some(Ok(msg)) => {
                    if let Err(e) = self.handle_message(msg).await {
                        error!("Error handling message: {}", e);
                    }
                }
                Some(Err(e)) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                None => {
                    info!("WebSocket connection closed");
                    break;
                }
            }
        }
    }

    /// Handle a single incoming message
    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Text(text) => {
                debug!("Received text message: {}", text);
                self.handle_text_message(&text).await?;
            }
            Message::Binary(data) => {
                debug!("Received binary message ({} bytes)", data.len());
            }
            Message::Ping(data) => {
                debug!("Received ping, sending pong");
                self.ws_stream.send(Message::Pong(data)).await?;
            }
            Message::Pong(_) => {
                debug!("Received pong");
            }
            Message::Close(_) => {
                info!("Received close frame");
                return Err(ClientError::Closed);
            }
            Message::Frame(_) => {
                debug!("Received frame message");
            }
        }
        Ok(())
    }

    /// Handle a text message (JSON-RPC response or event)
    async fn handle_text_message(&mut self, text: &str) -> Result<()> {
        // Try to parse as JSON-RPC response
        let response = match parse_response(text) {
            Ok(r) => r,
            Err(_) => {
                // Try to parse as a server event
                if let Ok(event) = serde_json::from_str::<ServerEvent>(text) {
                    // Forward to event channel
                    if let Some(sender) = &self.event_sender {
                        if let Err(e) = sender.send(event).await {
                            warn!("Failed to send event: {}", e);
                        }
                    }
                    return Ok(());
                }
                debug!("Unknown message format: {}", text);
                return Ok(());
            }
        };

        // Match response to pending request
        let response_id = response.id.clone();
        if let Some(tx) = self.pending_requests.remove(&response.id) {
            // Clone before sending to avoid move
            if tx.send(response.clone()).is_err() {
                warn!("Failed to send response for request {}", response_id);
            }
        } else {
            debug!("Response for unknown request: {}", response_id);
        }

        Ok(())
    }
}

/// Build authentication request
pub fn build_auth_request(config: &ClientConfig) -> AuthRequest {
    AuthRequest {
        client_name: config.client_name.clone(),
        client_version: config.client_version.clone(),
        device_id: config.device_id,
        protocol_version: config.protocol_version.clone(),
        token: config.auth_token.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_client_config_defaults() {
        let config = ClientConfig::default();

        assert_eq!(config.server_url, "ws://localhost:8080/ws");
        assert!(config.auth_token.is_empty());
        assert_eq!(config.client_name, "aisopod-client");
        assert_eq!(config.client_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(config.device_id.get_version(), Some(uuid::Version::Random));
        assert_eq!(config.protocol_version, "1.0");
    }

    #[test]
    fn test_build_auth_request() {
        let config = ClientConfig {
            server_url: "ws://test:8080/ws".to_string(),
            auth_token: "test-token".to_string(),
            client_name: "test-client".to_string(),
            client_version: "1.0.0".to_string(),
            device_id: Uuid::new_v4(),
            protocol_version: "1.0".to_string(),
        };

        let auth = build_auth_request(&config);

        assert_eq!(auth.client_name, "test-client");
        assert_eq!(auth.client_version, "1.0.0");
        assert_eq!(auth.token, "test-token");
        assert_eq!(auth.protocol_version, "1.0");
        assert_eq!(auth.device_id, config.device_id);
    }

    #[test]
    fn test_auth_request_serialization() {
        let auth = AuthRequest {
            client_name: "test".to_string(),
            client_version: "1.0".to_string(),
            device_id: Uuid::new_v4(),
            protocol_version: "1.0".to_string(),
            token: "secret".to_string(),
        };

        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("\"client_name\":\"test\""));
        assert!(json.contains("\"token\":\"secret\""));
    }
}
