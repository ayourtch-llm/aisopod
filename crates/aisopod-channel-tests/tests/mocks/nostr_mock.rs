//! Mock Nostr relay server for integration testing.
//!
//! This module provides a mock implementation of a Nostr relay server
//! that implements the basic relay protocol, allowing integration
//! tests to verify Nostr channel behavior without requiring a real relay.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};
use futures_util::stream::StreamExt;
use serde::{Serialize, Deserialize};

/// State for the Nostr mock server
#[derive(Clone, Default)]
pub struct MockNostrState {
    /// List of events stored by the relay
    pub events: Arc<Mutex<Vec<NostrEvent>>>,
    /// Number of subscription requests received
    pub subscription_count: Arc<Mutex<usize>>,
    /// Number of publish requests received
    pub publish_count: Arc<Mutex<usize>>,
    /// Event kinds stored (for filtering)
    pub event_kinds: Arc<Mutex<HashSet<u64>>>,
}

impl MockNostrState {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            subscription_count: Arc::new(Mutex::new(0)),
            publish_count: Arc::new(Mutex::new(0)),
            event_kinds: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

/// A Nostr event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NostrEvent {
    pub id: String,
    pub pubkey: String,
    pub created_at: i64,
    pub kind: u64,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

/// A mock Nostr relay WebSocket server
pub struct MockNostrServer {
    /// The server address in format "host:port"
    pub addr: String,
    /// The state of the mock server
    pub state: MockNostrState,
    /// The join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
    /// Channel to signal server shutdown
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockNostrServer {
    /// Starts a mock Nostr relay server on a random port
    ///
    /// # Returns
    ///
    /// A tuple of (server address, MockNostrServer instance)
    pub async fn start() -> (String, Self) {
        let state = MockNostrState::new();
        let state_clone = state.clone();

        // Create a channel to signal shutdown
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local addr");
        let addr_str = format!("127.0.0.1:{}", addr.port());

        info!("Starting mock Nostr relay server at {}", addr_str);

        let handle = tokio::spawn(async move {
            let mut shutdown_rx = Some(shutdown_rx);

            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((socket, _addr)) => {
                                info!("Nostr client connected from {}", _addr);
                                let state = state_clone.clone();
                                tokio::spawn(handle_nostr_client(socket, state));
                            }
                            Err(e) => {
                                warn!("Failed to accept Nostr connection: {}", e);
                                continue;
                            }
                        }
                    }
                    _ = shutdown_rx.as_mut().expect("Shutdown channel should not be None") => {
                        info!("Mock Nostr server shutdown signal received");
                        break;
                    }
                }
            }
        });

        let server = Self {
            addr: addr_str,
            state,
            _handle: handle,
            _shutdown_tx: Some(shutdown_tx),
        };

        let addr = server.addr.clone();
        (addr, server)
    }

    /// Get all events stored by the relay
    pub async fn get_events(&self) -> Vec<NostrEvent> {
        self.state.events.lock().unwrap().clone()
    }

    /// Get events of a specific kind
    pub async fn get_events_by_kind(&self, kind: u64) -> Vec<NostrEvent> {
        let all_events = self.state.events.lock().unwrap();
        all_events.iter().filter(|e| e.kind == kind).cloned().collect()
    }

    /// Get the count of subscription requests
    pub async fn get_subscription_count(&self) -> usize {
        *self.state.subscription_count.lock().unwrap()
    }

    /// Get the count of publish requests
    pub async fn get_publish_count(&self) -> usize {
        *self.state.publish_count.lock().unwrap()
    }
}

/// Handle a Nostr client connection
async fn handle_nostr_client(
    socket: tokio::net::TcpStream,
    state: MockNostrState,
) {
    let ws_stream = accept_async(socket)
        .await
        .expect("Failed to accept WebSocket");

    // Use futures_util::stream::StreamExt for split
    use futures_util::stream::StreamExt;
    let (ws_sender, ws_receiver) = ws_stream.split();

    // Spawn reader task
    let reader_state = state.clone();
    let reader_handle = tokio::spawn(async move {
        use futures_util::stream::StreamExt;
        let mut receiver: futures_util::stream::SplitStream<WebSocketStream<tokio::net::TcpStream>> = ws_receiver;

        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(msg) => {
                    if let Ok(text) = msg.to_text() {
                        handle_nostr_message(text, &reader_state).await;
                    }
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Spawn writer task to send periodic NOSTR messages
    let writer_state = state.clone();
    let writer_handle = tokio::spawn(async move {
        use tokio_tungstenite::tungstenite::Message;
        use futures_util::sink::SinkExt;
        use futures_util::stream::StreamExt;
        let mut ws_sender = ws_sender;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            // Send a minimal OK message to keep connection alive
            let _ = ws_sender.send(Message::text(r#"["OK","event-id",true,""]"#)).await;
        }
    });

    // Wait for either task to complete
    let _ = tokio::join!(reader_handle, writer_handle);
}

/// Handle an incoming Nostr message
async fn handle_nostr_message(message: &str, state: &MockNostrState) {
    info!("Received Nostr message: {}", message);

    // Try to parse as JSON array
    if let Ok(json) = serde_json::from_str::<Vec<serde_json::Value>>(message) {
        if let Some(cmd) = json.get(0).and_then(|v| v.as_str()) {
            match cmd {
                "EVENT" => {
                    // Handle EVENT message (publish)
                    if json.len() >= 2 {
                        if let Some(event_json) = json.get(1) {
                            if let Ok(event) = serde_json::from_value::<NostrEvent>(event_json.clone()) {
                                let mut events = state.events.lock().unwrap();
                                events.push(event.clone());
                                
                                let mut kinds = state.event_kinds.lock().unwrap();
                                kinds.insert(event.kind);
                                
                                let mut count = state.publish_count.lock().unwrap();
                                *count += 1;
                                
                                info!("Received EVENT (kind {}): {}", event.kind, event.content);
                            }
                        }
                    }
                }
                "REQ" => {
                    // Handle REQ message (subscribe)
                    let mut count = state.subscription_count.lock().unwrap();
                    *count += 1;
                    
                    info!("Received REQ subscription");
                }
                "CLOSE" => {
                    // Handle CLOSE message
                    info!("Received CLOSE subscription");
                }
                "AUTH" => {
                    // Handle AUTH message
                    info!("Received AUTH message");
                }
                _ => {
                    info!("Unknown Nostr command: {}", cmd);
                }
            }
        }
    }
}

impl Default for MockNostrServer {
    fn default() -> Self {
        panic!("MockNostrServer must be started with MockNostrServer::start()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_nostr_server_starts() {
        let (_addr, server) = MockNostrServer::start().await;
        assert!(server.addr.len() > 0);
        assert!(server.state.subscription_count.lock().unwrap().eq(&0));
    }

    #[tokio::test]
    async fn test_mock_nostr_publish_event() {
        let (_addr, server) = MockNostrServer::start().await;

        // Simulate an EVENT message
        let event = NostrEvent {
            id: "event-id-123".to_string(),
            pubkey: "test-pubkey".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            kind: 1,
            tags: vec![],
            content: "Hello from Nostr".to_string(),
            sig: "test-sig".to_string(),
        };
        
        let event_json = serde_json::to_value(&event).unwrap();
        let cmd = format!(r#"["EVENT",{}]"#, event_json);
        handle_nostr_message(&cmd, &server.state).await;

        let events = server.state.events.lock().unwrap();
        assert!(!events.is_empty());
        assert_eq!(events[0].content, "Hello from Nostr");
    }
}
