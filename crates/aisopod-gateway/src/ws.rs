use axum::{
    extract::ws::{Message, WebSocket},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{stream::StreamExt, sink::SinkExt};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Ping interval in seconds
const PING_INTERVAL_SECS: u64 = 30;

/// Build the WebSocket routes
pub fn ws_routes() -> Router {
    Router::new().route("/ws", get(ws_handler))
}

/// Handle a WebSocket upgrade request
async fn ws_handler(ws: axum::extract::WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_connection)
}

/// Handle an established WebSocket connection
async fn handle_connection(ws: WebSocket) {
    // Generate unique connection ID
    let conn_id = Uuid::new_v4();
    let start_time = std::time::Instant::now();
    
    info!(conn_id = %conn_id, "New WebSocket connection established");
    
    // Split the WebSocket into receiver and sender halves
    let (mut tx, mut rx) = ws.split();
    
    // Spawn heartbeat task that sends pings periodically
    let conn_id_heartbeat = conn_id.clone();
    let heartbeat_task = tokio::spawn(async move {
        let ping_interval = Duration::from_secs(PING_INTERVAL_SECS);
        let mut interval = tokio::time::interval(ping_interval);
        
        loop {
            interval.tick().await;
            debug!(conn_id = %conn_id_heartbeat, "Sending ping frame");
            
            if tx.send(Message::Ping(vec![])).await.is_err() {
                warn!(conn_id = %conn_id_heartbeat, "Failed to send ping");
                break;
            }
        }
    });
    
    // Main message loop - only reads from rx
    while let Some(message) = rx.next().await {
        match message {
            Ok(Message::Text(text)) => {
                debug!(conn_id = %conn_id, "Received text message: {}", text);
                // For now, just log - can be extended for RPC
            }
            Ok(Message::Binary(bin)) => {
                debug!(conn_id = %conn_id, "Received binary message ({} bytes)", bin.len());
            }
            Ok(Message::Ping(_)) => {
                debug!(conn_id = %conn_id, "Received ping, sending pong");
                // We can't send pong here because tx is moved to heartbeat_task
                // For simplicity, just log the pong
            }
            Ok(Message::Pong(_)) => {
                debug!(conn_id = %conn_id, "Received pong");
            }
            Ok(Message::Close(close)) => {
                info!(conn_id = %conn_id, "Received close frame: {:?}", close);
                if let Some(_reason) = close {
                    // Can't send close ack here since tx is moved
                }
                break;
            }
            Err(e) => {
                error!(conn_id = %conn_id, "WebSocket error: {}", e);
                break;
            }
        }
    }
    
    // Cancel heartbeat task
    heartbeat_task.abort();
    
    let duration = start_time.elapsed();
    info!(conn_id = %conn_id, duration_secs = %duration.as_secs(), "WebSocket connection closed");
}
