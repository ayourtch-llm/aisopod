//! Client connection management for the gateway
//!
//! This module provides the GatewayClient struct and ClientRegistry
//! for tracking all active WebSocket connections.

use std::net::SocketAddr;
use std::time::Instant;

use dashmap::DashMap;
use uuid::Uuid;

use crate::auth::AuthInfo;
use crate::broadcast::Subscription;

/// A single gateway client connection
#[derive(Debug, Clone)]
pub struct GatewayClient {
    /// Unique connection identifier
    pub conn_id: String,
    /// Sender channel for WebSocket messages
    pub sender: std::sync::Arc<tokio::sync::mpsc::Sender<axum::extract::ws::Message>>,
    /// Presence key for the client
    pub presence_key: String,
    /// Remote address of the client
    pub remote_addr: SocketAddr,
    /// Authenticated role (e.g., "operator", "node")
    pub role: String,
    /// Permission scopes granted to this client
    pub scopes: Vec<String>,
    /// Connection establishment time
    pub connected_at: Instant,
    /// Event subscription filter for broadcast events
    pub subscription: Subscription,
}

impl GatewayClient {
    /// Create a new GatewayClient from auth info
    pub fn new(
        conn_id: String,
        sender: std::sync::Arc<tokio::sync::mpsc::Sender<axum::extract::ws::Message>>,
        presence_key: String,
        remote_addr: SocketAddr,
        role: String,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            conn_id,
            sender,
            presence_key,
            remote_addr,
            role,
            scopes,
            connected_at: Instant::now(),
            subscription: Subscription::default(),
        }
    }

    /// Create a GatewayClient from AuthInfo
    pub fn from_auth_info(
        conn_id: String,
        sender: std::sync::Arc<tokio::sync::mpsc::Sender<axum::extract::ws::Message>>,
        remote_addr: SocketAddr,
        auth_info: AuthInfo,
    ) -> Self {
        Self {
            conn_id,
            sender,
            presence_key: format!("{}:{}", auth_info.role, Uuid::new_v4()),
            remote_addr,
            role: auth_info.role,
            scopes: auth_info.scopes,
            connected_at: Instant::now(),
            subscription: Subscription::default(),
        }
    }
}

/// Health snapshot of the gateway
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct HealthSnapshot {
    /// Total number of connected clients
    pub total_connections: usize,
    /// Number of operator clients
    pub operators: usize,
    /// Number of node clients
    pub nodes: usize,
}

/// Registry for managing all gateway client connections
#[derive(Debug, Default)]
pub struct ClientRegistry {
    /// Map of connection ID to GatewayClient
    clients: DashMap<String, GatewayClient>,
}

impl ClientRegistry {
    /// Create a new empty ClientRegistry
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
        }
    }

    /// Handle a new client connection
    ///
    /// Inserts the client into the registry and logs the event.
    pub fn on_connect(&self, client: GatewayClient) {
        let conn_id = client.conn_id.clone();
        let presence_key = client.presence_key.clone();
        let role = client.role.clone();
        self.clients.insert(client.conn_id.clone(), client);
        
        tracing::info!(
            conn_id = %conn_id,
            presence_key = %presence_key,
            role = %role,
            total_connections = self.clients.len(),
            "New client connected"
        );
    }

    /// Handle a client disconnection
    ///
    /// Removes the client from the registry and logs the event.
    pub fn on_disconnect(&self, conn_id: &str) {
        if let Some((_, client)) = self.clients.remove(conn_id) {
            let presence_key = client.presence_key.clone();
            let role = client.role.clone();
            
            tracing::info!(
                conn_id = %conn_id,
                presence_key = %presence_key,
                role = %role,
                total_connections = self.clients.len(),
                "Client disconnected"
            );
        } else {
            tracing::debug!(conn_id = %conn_id, "Disconnect for unknown connection");
        }
    }

    /// Get a client by connection ID
    pub fn get(&self, conn_id: &str) -> Option<dashmap::mapref::one::Ref<'_, String, GatewayClient>> {
        self.clients.get(conn_id)
    }

    /// Get all connected clients
    pub fn list(&self) -> Vec<GatewayClient> {
        self.clients.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Get the current health snapshot
    ///
    /// Returns counts for total connections, operators, and nodes.
    pub fn health_snapshot(&self) -> HealthSnapshot {
        let mut operators = 0;
        let mut nodes = 0;

        for client in self.clients.iter() {
            match client.value().role.as_str() {
                "operator" => operators += 1,
                "node" => nodes += 1,
                _ => {} // Count other roles in total only
            }
        }

        HealthSnapshot {
            total_connections: self.clients.len(),
            operators,
            nodes,
        }
    }

    /// Get the number of connected clients
    pub fn len(&self) -> usize {
        self.clients.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn create_test_sender() -> std::sync::Arc<tokio::sync::mpsc::Sender<axum::extract::ws::Message>> {
        // Create a channel with a small buffer for testing
        let (tx, _rx) = tokio::sync::mpsc::channel(16);
        std::sync::Arc::new(tx)
    }

    fn create_test_auth_info() -> AuthInfo {
        AuthInfo {
            role: "operator".to_string(),
            scopes: vec!["chat:write".to_string(), "agent:read".to_string()],
        }
    }

    fn create_test_socket_addr() -> SocketAddr {
        "127.0.0.1:8080".parse().unwrap()
    }

    #[test]
    fn test_gateway_client_creation_from_auth_info() {
        let sender = create_test_sender();
        let addr = create_test_socket_addr();
        let auth_info = create_test_auth_info();

        let client = GatewayClient::from_auth_info(
            "conn-123".to_string(),
            sender,
            addr,
            auth_info,
        );

        assert_eq!(client.conn_id, "conn-123");
        assert_eq!(client.role, "operator");
        assert_eq!(client.scopes, vec!["chat:write", "agent:read"]);
        assert_eq!(client.remote_addr, addr);
        assert!(client.presence_key.contains("operator"));
        assert!(client.connected_at <= Instant::now());
    }

    #[test]
    fn test_gateway_client_creation_with_auth() {
        let sender = create_test_sender();
        let addr = create_test_socket_addr();

        let client = GatewayClient::new(
            "conn-456".to_string(),
            sender,
            "presence-key-789".to_string(),
            addr,
            "node".to_string(),
            vec!["agent:admin".to_string()],
        );

        assert_eq!(client.conn_id, "conn-456");
        assert_eq!(client.presence_key, "presence-key-789");
        assert_eq!(client.role, "node");
        assert_eq!(client.scopes, vec!["agent:admin"]);
        assert_eq!(client.remote_addr, addr);
    }

    #[test]
    fn test_client_registry_new_is_empty() {
        let registry = ClientRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_client_registry_on_connect() {
        let registry = ClientRegistry::new();
        let sender = create_test_sender();
        let addr = create_test_socket_addr();
        let auth_info = create_test_auth_info();

        let client = GatewayClient::from_auth_info(
            "conn-001".to_string(),
            sender,
            addr,
            auth_info,
        );

        registry.on_connect(client);

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        // Verify we can retrieve the client
        let retrieved = registry.get("conn-001");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().conn_id, "conn-001");
    }

    #[test]
    fn test_client_registry_on_disconnect() {
        let registry = ClientRegistry::new();
        let sender = create_test_sender();
        let addr = create_test_socket_addr();
        let auth_info = create_test_auth_info();

        let client = GatewayClient::from_auth_info(
            "conn-002".to_string(),
            sender,
            addr,
            auth_info,
        );

        registry.on_connect(client);
        assert_eq!(registry.len(), 1);

        registry.on_disconnect("conn-002");
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        // Verify client is removed
        let retrieved = registry.get("conn-002");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_client_registry_on_disconnect_unknown() {
        let registry = ClientRegistry::new();

        // Should not panic and should log debug message
        registry.on_disconnect("nonexistent-conn");
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_client_registry_list() {
        let registry = ClientRegistry::new();
        let addr = create_test_socket_addr();

        for i in 0..3 {
            let sender = create_test_sender();
            let auth_info = create_test_auth_info();
            let client = GatewayClient::from_auth_info(
                format!("conn-{:03}", i),
                sender,
                addr,
                auth_info,
            );
            registry.on_connect(client);
        }

        let clients = registry.list();
        assert_eq!(clients.len(), 3);

        let conn_ids: Vec<&str> = clients.iter().map(|c| c.conn_id.as_str()).collect();
        assert!(conn_ids.contains(&"conn-000"));
        assert!(conn_ids.contains(&"conn-001"));
        assert!(conn_ids.contains(&"conn-002"));
    }

    #[test]
    fn test_client_registry_health_snapshot() {
        let registry = ClientRegistry::new();
        let addr = create_test_socket_addr();

        // Add 2 operators
        for i in 0..2 {
            let sender = create_test_sender();
            let auth_info = AuthInfo {
                role: "operator".to_string(),
                scopes: vec!["chat:write".to_string()],
            };
            let client = GatewayClient::from_auth_info(
                format!("operator-{:03}", i),
                sender,
                addr,
                auth_info,
            );
            registry.on_connect(client);
        }

        // Add 1 node
        {
            let sender = create_test_sender();
            let auth_info = AuthInfo {
                role: "node".to_string(),
                scopes: vec!["agent:read".to_string()],
            };
            let client = GatewayClient::from_auth_info(
                "node-000".to_string(),
                sender,
                addr,
                auth_info,
            );
            registry.on_connect(client);
        }

        // Add 1 client with unknown role
        {
            let sender = create_test_sender();
            let auth_info = AuthInfo {
                role: "admin".to_string(),
                scopes: vec!["admin:all".to_string()],
            };
            let client = GatewayClient::from_auth_info(
                "admin-000".to_string(),
                sender,
                addr,
                auth_info,
            );
            registry.on_connect(client);
        }

        let snapshot = registry.health_snapshot();
        assert_eq!(snapshot.total_connections, 4);
        assert_eq!(snapshot.operators, 2);
        assert_eq!(snapshot.nodes, 1);
    }

    #[tokio::test]
    async fn test_client_registry_concurrent_access() {
        let registry = std::sync::Arc::new(ClientRegistry::new());
        let addr = create_test_socket_addr();

        // Spawn multiple tasks to simulate concurrent connections
        let mut handles = vec![];
        for i in 0..10 {
            let registry = registry.clone();
            let sender = create_test_sender();
            let auth_info = create_test_auth_info();
            let addr = addr.clone();

            let handle = tokio::spawn(async move {
                let client = GatewayClient::from_auth_info(
                    format!("conn-{:03}", i),
                    sender,
                    addr,
                    auth_info,
                );
                registry.on_connect(client);
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(registry.len(), 10);
    }

    #[test]
    fn test_health_snapshot_empty() {
        let registry = ClientRegistry::new();
        let snapshot = registry.health_snapshot();

        assert_eq!(snapshot.total_connections, 0);
        assert_eq!(snapshot.operators, 0);
        assert_eq!(snapshot.nodes, 0);
    }
}
