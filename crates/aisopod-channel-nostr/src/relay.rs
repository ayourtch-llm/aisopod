//! Nostr relay connection management.
//!
//! This module provides relay connection management for connecting to
//! Nostr relays via WebSocket, publishing events, and subscribing to events.

use crate::events::NostrEvent;
use anyhow::{anyhow, Result};
use futures::{stream::StreamExt, TryStreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};
use tracing::{debug, error, info, warn};

/// A relay connection with its WebSocket.
pub struct RelayConnection {
    url: String,
    ws: Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>,
}

impl RelayConnection {
    /// Create a new relay connection.
    pub fn new(url: String) -> Self {
        Self { url, ws: None }
    }

    /// Check if the relay is connected.
    pub fn is_connected(&self) -> bool {
        self.ws.is_some()
    }

    /// Get the relay URL.
    pub fn url(&self) -> &str {
        &self.url
    }
}

/// A pool of relay connections.
///
/// This struct manages multiple relay connections and provides methods
/// for publishing events to all relays and receiving events from any relay.
pub struct RelayPool {
    relays: Vec<RelayConnection>,
    url_to_index: HashMap<String, usize>,
}

impl RelayPool {
    /// Create a new relay pool.
    pub fn new() -> Self {
        Self {
            relays: Vec::new(),
            url_to_index: HashMap::new(),
        }
    }

    /// Connect to multiple relays.
    ///
    /// # Arguments
    /// * `urls` - List of relay URLs to connect to
    ///
    /// # Returns
    /// * `Ok(RelayPool)` - The pool with connected relays
    /// * `Err(anyhow::Error)` - An error if any connection fails
    pub async fn connect(urls: &[String]) -> Result<Self> {
        let mut pool = Self::new();
        
        for url in urls {
            let (ws, _) = connect_async(url)
                .await
                .map_err(|e| anyhow!("Failed to connect to relay {}: {}", url, e))?;
            
            let connection = RelayConnection {
                url: url.clone(),
                ws: Some(ws),  // WebSocketStream<MaybeTlsStream<TcpStream>>
            };
            
            let index = pool.relays.len();
            pool.url_to_index.insert(url.clone(), index);
            pool.relays.push(connection);
            
            info!("Connected to relay: {}", url);
        }
        
        Ok(pool)
    }

    /// Get the number of connected relays.
    pub fn len(&self) -> usize {
        self.relays.len()
    }

    /// Check if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.relays.is_empty()
    }

    /// Publish an event to all relays.
    ///
    /// # Arguments
    /// * `event` - The event to publish
    ///
    /// # Returns
    /// * `Ok(())` - Successfully sent to at least one relay
    /// * `Err(anyhow::Error)` - An error if sending fails
    pub async fn publish(&mut self, event: &NostrEvent) -> Result<()> {
        let event_json = event.to_json_value();
        let msg = serde_json::json!(["EVENT", event_json]);
        let msg_str = serde_json::to_string(&msg)
            .map_err(|e| anyhow!("Failed to serialize event: {}", e))?;
        
        let mut success = false;
        let msg = Message::text(msg_str);
        
        for relay in &mut self.relays {
            if let Some(ref mut ws) = relay.ws {
                if ws.send(msg.clone()).await.is_ok() {
                    success = true;
                    debug!("Published event to relay: {}", relay.url);
                } else {
                    warn!("Failed to publish to relay: {}", relay.url);
                }
            }
        }
        
        if success {
            Ok(())
        } else {
            Err(anyhow!("Failed to publish to any relay"))
        }
    }

    /// Subscribe to events from all relays.
    ///
    /// # Arguments
    /// * `filters` - List of filter objects to subscribe to
    ///
    /// # Returns
    /// * `Ok(())` - Successfully subscribed to all relays
    /// * `Err(anyhow::Error)` - An error if subscription fails
    pub async fn subscribe(&mut self, filters: Vec<serde_json::Value>) -> Result<()> {
        let sub_id = format!("sub_{}", chrono::Utc::now().timestamp());
        let msg = serde_json::json!(["REQ", sub_id, filters]);
        let msg_str = serde_json::to_string(&msg)
            .map_err(|e| anyhow!("Failed to serialize subscription: {}", e))?;
        
        let msg = Message::text(msg_str);
        
        for relay in &mut self.relays {
            if let Some(ref mut ws) = relay.ws {
                if ws.send(msg.clone()).await.is_err() {
                    warn!("Failed to subscribe on relay: {}", relay.url);
                } else {
                    debug!("Subscribed to relay: {}", relay.url);
                }
            }
        }
        
        Ok(())
    }

    /// Wait for the next event from any relay.
    ///
    /// # Returns
    /// * `Ok(NostrEvent)` - The next event received
    /// * `Err(anyhow::Error)` - An error if receiving fails
    pub async fn next_event(&mut self) -> Result<Option<NostrEvent>> {
        for relay in &mut self.relays {
            if let Some(ref mut ws) = relay.ws {
                match tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    ws.next()
                ).await {
                    Ok(Some(Ok(msg))) => {
                        if let Message::Text(text) = msg {
                            if let Ok(json) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                                if json[0] == "EVENT" && json.len() >= 3 {
                                    // ["EVENT", sub_id, event]
                                    if let Ok(event) = NostrEvent::from_json_value(json[2].clone()) {
                                        debug!("Received event from relay: {}", relay.url);
                                        return Ok(Some(event));
                                    }
                                }
                            }
                        }
                    }
                    Ok(Some(Err(e))) => {
                        debug!("WebSocket error from relay {}: {}", relay.url, e);
                    }
                    Ok(None) => {
                        debug!("WebSocket stream ended for relay: {}", relay.url);
                    }
                    Err(_) => {
                        // Timeout, no message available
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// Disconnect from all relays.
    pub async fn disconnect(&mut self) {
        for relay in &mut self.relays {
            if let Some(ref mut ws) = relay.ws {
                if let Err(e) = ws.close(None).await {
                    warn!("Failed to close relay connection {}: {}", relay.url, e);
                }
            }
        }
        info!("Disconnected from all relays");
    }
}

impl Drop for RelayPool {
    fn drop(&mut self) {
        // Try to disconnect on drop (best effort)
        if !self.relays.is_empty() {
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    self.disconnect().await;
                });
            })) {
                eprintln!("Warning: Failed to disconnect relays on drop: {:?}", e);
            }
        }
    }
}
