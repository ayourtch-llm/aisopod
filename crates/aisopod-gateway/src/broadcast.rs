//! Event broadcasting system for the gateway
//!
//! This module provides a publish-subscribe event system that allows
//! the gateway to broadcast real-time events to all connected WebSocket
//! clients or to a filtered subset based on per-client subscriptions.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Subscription filter for a client's event preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Set of event types the client wishes to receive
    #[serde(rename = "events")]
    pub event_types: HashSet<String>,
}

impl Default for Subscription {
    fn default() -> Self {
        Self {
            event_types: HashSet::from([
                "presence".to_string(),
                "health".to_string(),
                "agent".to_string(),
                "chat".to_string(),
            ]),
        }
    }
}

impl Subscription {
    /// Create a new subscription with specific event types
    pub fn with_events(event_types: HashSet<String>) -> Self {
        Self { event_types }
    }

    /// Check if the subscription includes a specific event type
    pub fn includes(&self, event_type: &str) -> bool {
        self.event_types.contains(event_type)
    }
}

/// Gateway event types that can be broadcast to clients
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum GatewayEvent {
    /// Presence status change for a client
    #[serde(rename = "presence")]
    Presence {
        /// Connection ID of the client
        conn_id: String,
        /// New presence status (e.g., "online", "idle", "offline")
        status: String,
    },
    /// Health snapshot update
    #[serde(rename = "health")]
    Health {
        /// Current health snapshot of the gateway
        snapshot: crate::client::HealthSnapshot,
    },
    /// Agent lifecycle event
    #[serde(rename = "agent")]
    Agent {
        /// ID of the agent
        agent_id: String,
        /// Type of event (e.g., "created", "deleted", "status_changed")
        event: String,
    },
    /// Chat message event
    #[serde(rename = "chat")]
    Chat {
        /// Room ID for the chat
        room_id: String,
        /// Message content as JSON value
        message: Value,
    },
}

impl GatewayEvent {
    /// Get the event type name for filtering
    pub fn event_type(&self) -> &'static str {
        match self {
            GatewayEvent::Presence { .. } => "presence",
            GatewayEvent::Health { .. } => "health",
            GatewayEvent::Agent { .. } => "agent",
            GatewayEvent::Chat { .. } => "chat",
        }
    }
}

/// Broadcast channel for gateway events
#[derive(Debug, Clone)]
pub struct Broadcaster {
    /// Underlying broadcast sender
    sender: broadcast::Sender<GatewayEvent>,
}

impl Broadcaster {
    /// Create a new broadcaster with the specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all subscribers
    ///
    /// Returns the number of subscribers that received the event
    pub fn publish(&self, event: GatewayEvent) -> usize {
        self.sender.send(event).map_or(0, |count| count)
    }

    /// Create a new subscriber
    pub fn subscribe(&self) -> broadcast::Receiver<GatewayEvent> {
        self.sender.subscribe()
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new(128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_subscription_default_includes_all_events() {
        let sub = Subscription::default();
        assert!(sub.includes("presence"));
        assert!(sub.includes("health"));
        assert!(sub.includes("agent"));
        assert!(sub.includes("chat"));
    }

    #[test]
    fn test_subscription_with_events() {
        let events = HashSet::from(["presence".to_string(), "chat".to_string()]);
        let sub = Subscription::with_events(events);
        
        assert!(sub.includes("presence"));
        assert!(sub.includes("chat"));
        assert!(!sub.includes("health"));
        assert!(!sub.includes("agent"));
    }

    #[test]
    fn test_subscription_includes() {
        let sub = Subscription::default();
        assert!(sub.includes("presence"));
        assert!(sub.includes("health"));
        assert!(sub.includes("agent"));
        assert!(sub.includes("chat"));
    }

    #[test]
    fn test_gateway_event_presencetype() {
        let event = GatewayEvent::Presence {
            conn_id: "conn-123".to_string(),
            status: "online".to_string(),
        };
        assert_eq!(event.event_type(), "presence");
    }

    #[test]
    fn test_gateway_event_health_type() {
        let snapshot = crate::client::HealthSnapshot {
            total_connections: 10,
            operators: 5,
            nodes: 5,
        };
        let event = GatewayEvent::Health { snapshot };
        assert_eq!(event.event_type(), "health");
    }

    #[test]
    fn test_gateway_event_agent_type() {
        let event = GatewayEvent::Agent {
            agent_id: "agent-456".to_string(),
            event: "created".to_string(),
        };
        assert_eq!(event.event_type(), "agent");
    }

    #[test]
    fn test_gateway_event_chat_type() {
        let event = GatewayEvent::Chat {
            room_id: "room-789".to_string(),
            message: json!("hello"),
        };
        assert_eq!(event.event_type(), "chat");
    }

    #[test]
    fn test_broadcaster_publish_and_subscribe() {
        let broadcaster = Broadcaster::new(16);
        
        // Create a subscriber
        let mut rx = broadcaster.subscribe();
        
        // Publish an event
        let event = GatewayEvent::Presence {
            conn_id: "conn-123".to_string(),
            status: "online".to_string(),
        };
        let count = broadcaster.publish(event.clone());
        
        // Verify event was published
        assert_eq!(count, 1);
        
        // Receive the event
        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap(), event);
    }

    #[test]
    fn test_broadcaster_multiple_subscribers() {
        let broadcaster = Broadcaster::new(16);
        
        // Create multiple subscribers
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();
        let mut rx3 = broadcaster.subscribe();
        
        // Publish an event
        let event = GatewayEvent::Health {
            snapshot: crate::client::HealthSnapshot {
                total_connections: 5,
                operators: 2,
                nodes: 3,
            },
        };
        let count = broadcaster.publish(event.clone());
        
        // Verify all three subscribers received it
        assert_eq!(count, 3);
        
        // Each receiver should get the event
        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
        assert!(rx3.try_recv().is_ok());
    }

    #[test]
    fn test_broadcaster_event_serialization() {
        let event = GatewayEvent::Presence {
            conn_id: "conn-123".to_string(),
            status: "online".to_string(),
        };
        
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "presence");
        assert_eq!(json["conn_id"], "conn-123");
        assert_eq!(json["status"], "online");
    }
}
