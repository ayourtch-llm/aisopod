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

/// Default heartbeat pong timeout in seconds (after sending ping)
const HEARTBEAT_PONG_TIMEOUT_SECS: u64 = 10;

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
    
    ws.on_upgrade(move |socket| async move {
        // Use tokio::time::timeout to enforce the handshake timeout
        match tokio::time::timeout(handshake_timeout_duration, handle_connection(socket)).await {
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

/// Handle an established WebSocket connection
async fn handle_connection(ws: WebSocket) {
    // Generate unique connection ID
    let conn_id = Uuid::new_v4();
    let start_time = std::time::Instant::now();
    
    info!(conn_id = %conn_id, "New WebSocket connection established");
    
    // Split the WebSocket into sink and stream halves
    let (mut ws_tx, mut ws_rx) = ws.split();
    
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
                    return;
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
                        debug!(conn_id = %conn_id, "Received text message");
                        // Process the text message (for now just log)
                        debug!(conn_id = %conn_id, text = %text, "Text message content");
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
        }
    }
    
    // Cleanup on disconnect with logging
    let duration = start_time.elapsed();
    info!(conn_id = %conn_id, duration_secs = %duration.as_secs(), "WebSocket connection closed");
}
