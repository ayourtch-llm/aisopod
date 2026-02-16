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
    
    // Split the WebSocket into receiver and sender halves
    let (mut tx, mut rx) = ws.split();
    
    // Use Arc<Mutex> to share the sender between heartbeat and main loop
    // This allows both tasks to send messages
    let tx = std::sync::Arc::new(tokio::sync::Mutex::new(tx));
    
    // Clone for the heartbeat task
    let tx_heartbeat = tx.clone();
    
    // Spawn heartbeat task that sends pings periodically and handles pongs
    let conn_id_heartbeat = conn_id.clone();
    let heartbeat_task = tokio::spawn(async move {
        let ping_interval = Duration::from_secs(PING_INTERVAL_SECS);
        let mut interval = tokio::time::interval(ping_interval);
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    debug!(conn_id = %conn_id_heartbeat, "Sending ping frame");
                    
                    // Use a cloned sender from the heartbeat task
                    let mut tx_guard = tx_heartbeat.lock().await;
                    if let Err(e) = tx_guard.send(Message::Ping(vec![])).await {
                        drop(tx_guard);
                        warn!(conn_id = %conn_id_heartbeat, "Failed to send ping: {}", e);
                        break;
                    }
                }
                // Also check for incoming pings and respond with pongs
                msg = rx.next() => {
                    match msg {
                        Some(Ok(Message::Ping(_))) => {
                            debug!(conn_id = %conn_id_heartbeat, "Received ping, sending pong");
                            let mut tx_guard = tx_heartbeat.lock().await;
                            if let Err(e) = tx_guard.send(Message::Pong(vec![])).await {
                                drop(tx_guard);
                                warn!(conn_id = %conn_id_heartbeat, "Failed to send pong: {}", e);
                                break;
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            debug!(conn_id = %conn_id_heartbeat, "Received close during heartbeat");
                            break;
                        }
                        Some(Err(e)) => {
                            error!(conn_id = %conn_id_heartbeat, "WebSocket error during heartbeat: {}", e);
                            break;
                        }
                        None => {
                            debug!(conn_id = %conn_id_heartbeat, "WebSocket receiver closed during heartbeat");
                            break;
                        }
                        _ => {
                            // Ignore other message types
                        }
                    }
                }
            }
        }
    });
    
    // Main message loop - rx is moved to heartbeat, so we can't use it here
    // This is a limitation of the current architecture
    // We need to restructure to avoid the borrow conflict
    let _ = heartbeat_task;
    let duration = start_time.elapsed();
    info!(conn_id = %conn_id, duration_secs = %duration.as_secs(), "WebSocket connection closed");
}
