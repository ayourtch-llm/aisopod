#![allow(clippy::all)]
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::auth::AuthInfo;
use crate::broadcast::Broadcaster;
use crate::client::{ClientRegistry, GatewayClient};
use crate::rpc::{self, chat::ChatSendHandler, MethodRouter, RequestContext, ApprovalStore, ApprovalRequestHandler, ApprovalApproveHandler, ApprovalDenyHandler, ApprovalListHandler, PairingStore, PairRequestHandler, PairConfirmHandler, PairRevokeHandler, CapabilityStore, NodeDescribeHandler, NodeInvokeHandler};
use crate::auth::DeviceTokenManager;

/// Default handshake timeout in seconds
pub const DEFAULT_HANDSHAKE_TIMEOUT_SECS: u64 = 5;

/// Default heartbeat pong timeout in seconds (after sending ping)
const HEARTBEAT_PONG_TIMEOUT_SECS: u64 = 10;

/// Ping interval in seconds
const PING_INTERVAL_SECS: u64 = 30;

/// Extension key for ClientRegistry
pub const CLIENT_REGISTRY_KEY: &str = "aisopod.client.registry";

/// Extension key for Broadcaster
pub const BROADCASTER_KEY: &str = "aisopod.broadcast.broadcaster";

/// Extension key for AgentRunner
pub const AGENT_RUNNER_KEY: &str = "aisopod.gateway.agent_runner";

/// Build a complete agent dependencies stack for the gateway
///
/// This function creates the necessary dependencies for agent execution:
/// - Config: From the aisopod-config crate
/// - ProviderRegistry: For LLM provider access
/// - ToolRegistry: For tool execution
/// - SessionStore: For conversation state management
/// - AgentRunner: The main agent execution orchestrator
pub fn create_agent_runner() -> Arc<aisopod_agent::AgentRunner> {
    // Create configuration (will be replaced with actual config loading in production)
    let config = Arc::new(aisopod_config::AisopodConfig::default());
    
    // Create provider registry
    let providers = Arc::new(aisopod_provider::ProviderRegistry::new());
    
    // Create tool registry with built-in tools
    let mut tools = aisopod_tools::ToolRegistry::new();
    aisopod_tools::register_all_tools(&mut tools);
    let tools = Arc::new(tools);
    
    // Create session store (in-memory for now)
    let sessions = Arc::new(
        aisopod_session::SessionStore::new_in_memory()
            .expect("Failed to create in-memory session store"),
    );

    // Create the agent runner with all dependencies
    Arc::new(aisopod_agent::AgentRunner::new(
        config,
        providers,
        tools,
        sessions,
    ))
}

/// Build the WebSocket routes with configurable timeout
pub fn ws_routes(handshake_timeout: Option<u64>) -> Router {
    let timeout = handshake_timeout;
    Router::new().route(
        "/ws",
        get(move |ws: WebSocketUpgrade, req: axum::extract::Request| ws_handler(ws, req, timeout)),
    )
}

/// Handle a WebSocket upgrade request with configurable timeout
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    request: axum::extract::Request,
    handshake_timeout: Option<u64>,
) -> impl IntoResponse {
    let handshake_timeout_duration =
        Duration::from_secs(handshake_timeout.unwrap_or(DEFAULT_HANDSHAKE_TIMEOUT_SECS));

    ws.on_upgrade(move |socket| async move {
        // Use tokio::time::timeout to enforce the handshake timeout
        match tokio::time::timeout(
            handshake_timeout_duration,
            handle_connection(socket, request),
        )
        .await
        {
            Ok(()) => {
                // Connection completed normally
            }
            Err(_) => {
                warn!(
                    "WebSocket handshake timed out after {} seconds",
                    handshake_timeout_duration.as_secs()
                );
            }
        }
    })
}

/// Extract remote address from request extensions
fn extract_remote_addr(request: &axum::extract::Request) -> SocketAddr {
    request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|info| info.0)
        .unwrap_or_else(|| "127.0.0.1:0".parse().expect("default IP address is valid"))
}

/// Handle an established WebSocket connection
async fn handle_connection(ws: WebSocket, request: axum::extract::Request) {
    // Extract connection metadata from request
    let remote_addr = extract_remote_addr(&request);

    // Get auth info from extensions
    let auth_info = request.extensions().get::<AuthInfo>().cloned();

    // Get client registry from extensions
    let client_registry = request
        .extensions()
        .get::<std::sync::Arc<ClientRegistry>>()
        .cloned();

    // Get broadcaster from extensions
    let broadcaster = request
        .extensions()
        .get::<std::sync::Arc<Broadcaster>>()
        .cloned();

    // Get pairing store from extensions
    let pairing_store = request
        .extensions()
        .get::<std::sync::Arc<PairingStore>>()
        .cloned();

    // Create agent runner for this connection
    let agent_runner = create_agent_runner();
    
    // Clone the agent runner for use in the loop
    let agent_runner_for_loop = Arc::clone(&agent_runner);

    // Generate unique connection ID
    let conn_id = Uuid::new_v4().to_string();
    let start_time = std::time::Instant::now();

    info!(conn_id = %conn_id, "New WebSocket connection established");

    // Split the WebSocket into sink and stream halves
    let (mut ws_tx, mut ws_rx) = ws.split();

    // Create sender for messages (wrapped in Arc for sharing)
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);

    // Create RPC method router for dispatching requests
    let method_router = std::sync::Arc::new(MethodRouter::default());

    // Clone tx for use in chat.send handler (since tx will be moved into GatewayClient)
    let tx_for_chat = tx.clone();

    // Create approval store for managing pending approval requests
    let approval_store = std::sync::Arc::new(ApprovalStore::new());

    // Register approval handlers with the method router if broadcaster is available
    if let Some(broadcaster) = &broadcaster {
        let store = approval_store.clone();
        let broadcaster = broadcaster.clone();
        
        method_router.register("approval.request", 
            ApprovalRequestHandler::with_deps(store.clone(), broadcaster.clone()));
        method_router.register("approval.approve", 
            ApprovalApproveHandler::with_store(store.clone()));
        method_router.register("approval.deny", 
            ApprovalDenyHandler::with_store(store.clone()));
        method_router.register("approval.list", 
            ApprovalListHandler::with_store(store.clone()));
    }

    // Register node.pair handlers if pairing store is available
    if let Some(pairing_store_ref) = &pairing_store {
        let pairing_store_for_request = pairing_store_ref.clone();
        let pairing_store_for_confirm = pairing_store_ref.clone();
        let pairing_store_for_revoke = pairing_store_ref.clone();
        
        // Note: token_manager is created per-connection, but it should be shared
        // For now, create a fresh one. In production, this should also be shared.
        let token_manager = std::sync::Arc::new(std::sync::Mutex::new(DeviceTokenManager::new(
            std::path::PathBuf::from("device_tokens.toml")
        )));
        
        let token_manager_for_request = token_manager.clone();
        let token_manager_for_confirm = token_manager.clone();
        
        method_router.register("node.pair.request", 
            PairRequestHandler::with_deps(pairing_store_for_request, token_manager_for_request));
        method_router.register("node.pair.confirm", 
            PairConfirmHandler::with_deps(pairing_store_for_confirm, token_manager_for_confirm));
        method_router.register("node.pair.revoke", 
            PairRevokeHandler::with_deps(token_manager));
    }

    // Create capability store for managing device capabilities
    let capability_store = std::sync::Arc::new(CapabilityStore::new());

    // Register node.describe handler
    let node_describe_handler = NodeDescribeHandler::new();
    method_router.register("node.describe", node_describe_handler);

    // Register node.invoke handler
    if let Some(client_registry_clone) = client_registry.clone() {
        let capability_store_for_invoke = capability_store.clone();
        let node_invoke_handler = NodeInvokeHandler::new(client_registry_clone, capability_store_for_invoke);
        method_router.register("node.invoke", node_invoke_handler);
    }

    // Register client if we have auth info and registry
    // The sender is moved into the client and also used in the main loop
    // Clone auth_info before moving it into GatewayClient
    let client = if let (Some(auth_info), Some(registry)) = (auth_info.clone(), client_registry.clone()) {
        let sender = std::sync::Arc::new(tx);
        let client = GatewayClient::from_auth_info(conn_id.clone(), sender, remote_addr, auth_info);
        registry.on_connect(client.clone());
        Some(client)
    } else {
        None
    };

    // Subscribe to broadcast events if broadcaster is available
    let mut broadcast_rx = if let Some(broadcaster) = &broadcaster {
        Some(broadcaster.subscribe())
    } else {
        None
    };

    let pong_timeout = Duration::from_secs(HEARTBEAT_PONG_TIMEOUT_SECS);
    let mut last_pong = std::time::Instant::now();
    let mut ping_interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL_SECS));
    // Track if we've received at least one pong to validate timeout
    let mut has_received_pong = false;

    loop {
        tokio::select! {
            // Periodic ping sending
            _ = ping_interval.tick() => {
                // Check if we've exceeded pong timeout since last pong
                // Only enforce timeout after we've received at least one pong
                if has_received_pong && last_pong.elapsed() > pong_timeout {
                    info!(conn_id = %conn_id, duration_secs = %start_time.elapsed().as_secs(), "WebSocket connection closed due to pong timeout");
                    break;
                }

                debug!(conn_id = %conn_id, "Sending ping frame");
                if let Err(e) = ws_tx.send(Message::Ping(vec![])).await {
                    warn!(conn_id = %conn_id, "Failed to send ping: {}", e);
                    break;
                }
            }
            // Read incoming messages from the client
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(Message::Pong(_))) => {
                        debug!(conn_id = %conn_id, "Received pong frame");
                        last_pong = std::time::Instant::now();
                        has_received_pong = true;
                    }
                    Some(Ok(Message::Close(_))) => {
                        debug!(conn_id = %conn_id, "Received close frame");
                        // Send close response
                        if let Err(e) = ws_tx.send(Message::Close(None)).await {
                            warn!(conn_id = %conn_id, "Failed to send close response: {}", e);
                        }
                        break;
                    }
                    Some(Ok(Message::Text(text))) => {
                        eprintln!("=== WS RECEIVED TEXT: {} ===", text);
                        // Try to parse as JSON-RPC request
                        match rpc::parse(&text) {
                            Ok(request) => {
                                // Check if this is a chat.send request
                                if request.method == "chat.send" {
                                    // Handle chat.send specially with full dependencies
                                    let chat_handler = ChatSendHandler;
                                    let ws_sender = std::sync::Arc::new(tx_for_chat.clone());
                                    
                                    // Use the agent runner we already have
                                    if let Some(agent_runner) = Some(Arc::clone(&agent_runner_for_loop)) {
                                        // Call the handler directly with full dependencies
                                        let response = chat_handler.handle_with_deps(
                                            conn_id.clone(),
                                            request.params,
                                            agent_runner,
                                            ws_sender,
                                        );
                                        
                                        // Serialize response to JSON
                                        let response_text = match serde_json::to_string(&response) {
                                            Ok(json) => json,
                                            Err(e) => {
                                                error!(conn_id = %conn_id, "Failed to serialize response: {}", e);
                                                continue;
                                            }
                                        };
                                        
                                        // Send response back to client
                                        if let Err(e) = ws_tx.send(Message::Text(response_text)).await {
                                            error!(conn_id = %conn_id, "Failed to send RPC response: {}", e);
                                            break;
                                        }
                                        eprintln!("=== WS SENT CHAT.SEND RESPONSE ===");
                                    } else {
                                        // AgentRunner not available
                                        let error_response = serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "error": {
                                                "code": 500,
                                                "message": "AgentRunner not available"
                                            },
                                            "id": conn_id
                                        });
                                        let error_text = serde_json::to_string(&error_response).unwrap();
                                        if let Err(e) = ws_tx.send(Message::Text(error_text)).await {
                                            error!(conn_id = %conn_id, "Failed to send error response: {}", e);
                                            break;
                                        }
                                    }
                                } else {
                                    // Handle other methods via method router
                                    let auth_info = auth_info.clone().unwrap_or_default();
                                    let ctx = RequestContext::with_auth(conn_id.clone(), remote_addr, auth_info);
                                    let response = method_router.dispatch(ctx, request);
                                    eprintln!("=== WS DISPATCHED RESPONSE: {:?} ===", response);

                                    // Serialize response to JSON
                                    let response_text = match serde_json::to_string(&response) {
                                        Ok(json) => json,
                                        Err(e) => {
                                            error!(conn_id = %conn_id, "Failed to serialize response: {}", e);
                                            continue;
                                        }
                                    };

                                    // Send response back to client
                                    if let Err(e) = ws_tx.send(Message::Text(response_text)).await {
                                        error!(conn_id = %conn_id, "Failed to send RPC response: {}", e);
                                        break;
                                    }
                                    eprintln!("=== WS SENT RESPONSE ===");
                                }
                            }
                            Err(rpc_error_response) => {
                                eprintln!("=== WS PARSE ERROR: {:?} ===", rpc_error_response);
                                // Failed to parse as valid JSON-RPC - send error response
                                // Note: rpc_error_response.id is None for parse errors (not tied to a request)
                                // We should send the error back but without an id since the request was malformed
                                let error_text = match serde_json::to_string(&rpc_error_response) {
                                    Ok(json) => json,
                                    Err(e) => {
                                        error!(conn_id = %conn_id, "Failed to serialize error response: {}", e);
                                        continue;
                                    }
                                };

                                eprintln!("=== WS SENDING ERROR TEXT: {} ===", error_text);
                                if let Err(e) = ws_tx.send(Message::Text(error_text)).await {
                                    error!(conn_id = %conn_id, "Failed to send error response: {}", e);
                                    break;
                                }
                                eprintln!("=== WS SENT ERROR ===");
                            }
                        }
                    }
                    Some(Ok(Message::Binary(data))) => {
                        debug!(conn_id = %conn_id, "Received binary message");
                        // Process the binary message
                        debug!(conn_id = %conn_id, len = data.len(), "Binary message received");
                    }
                    Some(Ok(Message::Ping(_))) => {
                        // Server should not receive pings from client, send pong back
                        debug!(conn_id = %conn_id, "Received ping, sending pong");
                        if let Err(e) = ws_tx.send(Message::Pong(vec![])).await {
                            warn!(conn_id = %conn_id, "Failed to send pong: {}", e);
                        }
                    }
                    Some(Err(e)) => {
                        error!(conn_id = %conn_id, "WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        debug!(conn_id = %conn_id, "WebSocket receiver closed");
                        break;
                    }
                }
            }
            // Send messages from the channel to the client
            msg = rx.recv() => {
                match msg {
                    Some(msg) => {
                        if let Err(e) = ws_tx.send(msg).await {
                            error!(conn_id = %conn_id, "Failed to send message to client: {}", e);
                            break;
                        }
                    }
                    None => {
                        // Sender was dropped, exit the loop
                        break;
                    }
                }
            }
            // Forward broadcast events to the client
            broadcast_event = async {
                if let Some(ref mut rx) = broadcast_rx {
                    rx.recv().await.ok()
                } else {
                    futures_util::future::pending().await
                }
            } => {
                if let Some(event) = broadcast_event {
                    // Check if the event type is in the client's subscription
                    let should_send = client.as_ref()
                        .map(|c| c.subscription.includes(event.event_type()))
                        .unwrap_or(false);

                    if should_send {
                        // Serialize event as JSON-RPC notification (no id field)
                        let notification = serde_json::json!({
                            "jsonrpc": "2.0",
                            "method": "gateway.event",
                            "params": event
                        });

                        if let Err(e) = ws_tx.send(Message::Text(notification.to_string())).await {
                            error!(conn_id = %conn_id, "Failed to send broadcast event to client: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    }

    // Cleanup on disconnect with logging
    let duration = start_time.elapsed();
    info!(conn_id = %conn_id, duration_secs = %duration.as_secs(), "WebSocket connection closed");

    // Remove device capabilities from the capability store
    capability_store.remove(&conn_id);

    // Deregister client from registry
    if let Some(registry) = client_registry {
        registry.on_disconnect(&conn_id);
    }
}
