use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{stream::StreamExt, sink::SinkExt};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;

/// Default handshake timeout in seconds
pub const DEFAULT_HANDSHAKE_TIMEOUT_SECS: u64 = 5;

/// Ping interval in seconds
const PING_INTERVAL_SECS: u64 = 30;

/// Build the WebSocket routes with configurable timeout
pub fn ws_routes(handshake_timeout: Option<u64>) -> Router {
    let timeout = handshake_timeout;
    Router::new().route("/ws", get(move |ws: WebSocketUpgrade| {
        ws_handler(ws, timeout)
    }))
}

/// Handle a WebSocket upgrade request with configurable timeout
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    handshake_timeout: Option<u64>,
) -> impl IntoResponse {
    let handshake_timeout_duration = Duration::from_secs(handshake_timeout.unwrap_or(DEFAULT_HANDSHAKE_TIMEOUT_SECS));
    
    // Create a future that wraps the on_upgrade call
    // We need to run the upgrade in a separate task with timeout
    let ws_handler = async move {
        tokio::time::timeout(handshake_timeout_duration, async move {
            // The on_upgrade method returns a Response, not a Future
            // We need to wrap it in a future that completes when the connection is closed
            ws.on_upgrade(handle_connection)
        })
        .await
    };
    
    match ws_handler.await {
        Ok(result) => result,
        Err(_) => {
            warn!(timeout_secs = handshake_timeout_duration.as_secs(), "WebSocket handshake timed out");
            axum::response::Response::builder()
                .status(axum::http::StatusCode::REQUEST_TIMEOUT)
                .body(axum::body::Body::from("WebSocket handshake timeout"))
                .expect("Failed to build timeout response")
        }
    }
}

/// Handle an established WebSocket connection
async fn handle_connection(ws: WebSocket) {
    // Generate unique connection ID
    let conn_id = Uuid::new_v4();
    let start_time = std::time::Instant::now();
    
    info!(conn_id = %conn_id, "New WebSocket connection established");
    
    // Split the WebSocket into sink and stream halves
    // Axum's split() returns (SplitSink, SplitStream) - first is for sending, second is for receiving
    let (ws_tx, ws_rx) = ws.split();
    
    // Wrap ws_tx in Arc<Mutex<>> so multiple tasks can send
    let ws_tx = Arc::new(Mutex::new(ws_tx));
    
    // Create a broadcast channel to forward messages from ws_rx to both tasks
    // Broadcast allows multiple subscribers to receive the same messages
    let (tx, mut rx_heartbeat) = tokio::sync::broadcast::channel::<Message>(16);
    let mut rx_main = rx_heartbeat.resubscribe();
    
    // Spawn one task that reads from ws_rx and sends to the channel
    let forwarder_task = tokio::spawn(async move {
        let mut stream = ws_rx;
        loop {
            match stream.next().await {
                Some(Ok(msg)) => {
                    // Try to send to the channel; if it fails, the receiver is dropped
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
                Some(Err(e)) => {
                    error!(conn_id = %conn_id, "WebSocket error in forwarder: {}", e);
                    break;
                }
                None => {
                    debug!(conn_id = %conn_id, "WebSocket receiver closed in forwarder");
                    break;
                }
            }
        }
    });
    
    // Spawn heartbeat task that sends pings periodically and handles pongs
    let ws_tx_heartbeat = Arc::clone(&ws_tx);
    let conn_id_heartbeat = conn_id.clone();
    let heartbeat_task = tokio::spawn(async move {
        let ping_interval = Duration::from_secs(PING_INTERVAL_SECS);
        let mut interval = tokio::time::interval(ping_interval);
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    debug!(conn_id = %conn_id_heartbeat, "Sending ping frame");
                    
                    // ws_tx is SplitSink wrapped in Arc<Mutex<>>, use lock() to send
                    let mut ws_tx_guard = ws_tx_heartbeat.lock().await;
                    if let Err(e) = ws_tx_guard.send(Message::Ping(vec![])).await {
                        warn!(conn_id = %conn_id_heartbeat, "Failed to send ping: {}", e);
                        drop(ws_tx_guard);
                        break;
                    }
                }
                // Receive from the channel instead of ws_rx directly
                msg = rx_heartbeat.recv() => {
                    match msg {
                        Ok(Message::Ping(_)) => {
                            debug!(conn_id = %conn_id_heartbeat, "Received ping, sending pong");
                            let mut ws_tx_guard = ws_tx_heartbeat.lock().await;
                            if let Err(e) = ws_tx_guard.send(Message::Pong(vec![])).await {
                                warn!(conn_id = %conn_id_heartbeat, "Failed to send pong: {}", e);
                                drop(ws_tx_guard);
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            debug!(conn_id = %conn_id_heartbeat, "Received close during heartbeat");
                            break;
                        }
                        Ok(_) => {
                            // Ignore other message types in heartbeat
                        }
                        Err(_) => {
                            debug!(conn_id = %conn_id_heartbeat, "Channel closed during heartbeat");
                            break;
                        }
                    }
                }
            }
        }
    });
    
    // Main message loop - handle incoming messages from the client
    let ws_tx_main = Arc::clone(&ws_tx);
    let conn_id_main = conn_id.clone();
    let main_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = rx_main.recv() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            debug!(conn_id = %conn_id_main, "Received text message");
                            // Process the text message (for now just log)
                            debug!(conn_id = %conn_id_main, text = %text, "Text message content");
                        }
                        Ok(Message::Binary(data)) => {
                            debug!(conn_id = %conn_id_main, "Received binary message");
                            // Process the binary message
                            debug!(conn_id = %conn_id_main, len = data.len(), "Binary message received");
                        }
                        Ok(Message::Pong(_)) => {
                            debug!(conn_id = %conn_id_main, "Received pong");
                        }
                        Ok(Message::Close(_)) => {
                            debug!(conn_id = %conn_id_main, "Received close frame");
                            break;
                        }
                        Ok(_) => {
                            // Ignore other message types in main loop
                        }
                        Err(_) => {
                            debug!(conn_id = %conn_id_main, "Channel closed");
                            break;
                        }
                    }
                }
            }
        }
    });
    
    // Wait for both tasks to complete
    let _ = heartbeat_task.await;
    let _ = main_task.await;
    
    // Cleanup on disconnect with logging
    let duration = start_time.elapsed();
    info!(conn_id = %conn_id, duration_secs = %duration.as_secs(), "WebSocket connection closed");
}
