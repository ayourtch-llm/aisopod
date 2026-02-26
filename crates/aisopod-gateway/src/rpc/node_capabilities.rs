//! Node device capability advertisement and invocation RPC methods
//!
//! This module implements the `node.describe` and `node.invoke` RPC methods that allow
//! paired devices to advertise their capabilities (camera, location, calendar, contacts, etc.)
//! and allow the server to invoke those capabilities on demand.
//!
//! ## node.describe
//! A paired device sends its list of capabilities after connecting. The server stores
//! these capabilities in the connection state so agents can query what a device can do.
//!
//! ## node.invoke
//! The server sends an invocation request to a device for a specific service method.
//! The device executes the action locally and returns the result.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::auth::AuthInfo;
use crate::client::{ClientRegistry, DeviceCapability};
use crate::rpc::types::{self, error_codes, RpcError, RpcResponse};
use crate::rpc::{RequestContext, RpcMethod};

/// Maximum timeout for node.invoke in milliseconds
const MAX_TIMEOUT_MS: u64 = 30000; // 30 seconds

/// Node describe parameters - device advertises its capabilities
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NodeDescribeParams {
    /// List of capabilities this device offers
    pub capabilities: Vec<DeviceCapability>,
}

/// Node describe result - confirms accepted capabilities
#[derive(Debug, Serialize)]
pub struct NodeDescribeResult {
    /// Whether the capabilities were accepted
    pub accepted: bool,
    /// List of service names that were registered
    pub registered_services: Vec<String>,
}

/// Node invoke request - server invokes a device capability
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NodeInvokeRequest {
    /// Service name to invoke
    pub service: String,
    /// Method name within the service
    pub method: String,
    /// Arbitrary JSON parameters for the method
    pub params: serde_json::Value,
    /// Timeout in milliseconds for the invocation
    pub timeout_ms: u64,
    /// Target device ID (optional, used for routing to specific device)
    pub device_id: Option<String>,
}

/// Node invoke result - response from the device
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInvokeResult {
    /// Whether the invocation succeeded
    pub success: bool,
    /// Data returned by the method (if successful)
    pub data: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Storage for device capabilities indexed by device_id
pub struct CapabilityStore {
    /// Map from conn_id to list of capabilities
    capabilities: Arc<std::sync::RwLock<HashMap<String, Vec<DeviceCapability>>>>,
    /// Map from device_id to conn_id for routing invocations
    device_to_conn: Arc<std::sync::RwLock<HashMap<String, String>>>,
}

impl CapabilityStore {
    /// Create a new empty CapabilityStore
    pub fn new() -> Self {
        Self {
            capabilities: Arc::new(std::sync::RwLock::new(HashMap::new())),
            device_to_conn: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Store capabilities for a connection with optional device_id
    pub fn store_with_device_id(&self, conn_id: &str, capabilities: Vec<DeviceCapability>, device_id: Option<String>) {
        let mut store = self.capabilities.write().unwrap();
        store.insert(conn_id.to_string(), capabilities);
        
        if let Some(device_id) = device_id {
            let mut device_map = self.device_to_conn.write().unwrap();
            device_map.insert(device_id, conn_id.to_string());
        }
    }

    /// Store capabilities for a connection
    pub fn store(&self, conn_id: &str, capabilities: Vec<DeviceCapability>) {
        let mut store = self.capabilities.write().unwrap();
        store.insert(conn_id.to_string(), capabilities);
    }

    /// Get capabilities for a connection
    pub fn get(&self, conn_id: &str) -> Option<Vec<DeviceCapability>> {
        let store = self.capabilities.read().unwrap();
        store.get(conn_id).cloned()
    }

    /// Remove capabilities for a connection (on disconnect)
    pub fn remove(&self, conn_id: &str) {
        let mut store = self.capabilities.write().unwrap();
        store.remove(conn_id);
        
        // Also remove from device_to_conn mapping
        let device_id_to_remove = {
            let device_map = self.device_to_conn.read().unwrap();
            device_map.iter()
                .find(|(_, c)| c.as_str() == conn_id)
                .map(|(d, _)| d.clone())
        };
        
        if let Some(device_id) = device_id_to_remove {
            let mut device_map = self.device_to_conn.write().unwrap();
            device_map.remove(&device_id);
        }
    }

    /// Get the conn_id for a device by device_id
    pub fn get_conn_id_by_device_id(&self, device_id: &str) -> Option<String> {
        let device_map = self.device_to_conn.read().unwrap();
        device_map.get(device_id).cloned()
    }
}

impl Default for CapabilityStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for node.describe RPC method
///
/// This method allows a paired device to advertise its capabilities to the server.
/// The server stores these capabilities in the connection state.
pub struct NodeDescribeHandler {
    client_registry: Option<Arc<ClientRegistry>>,
    capability_store: Arc<CapabilityStore>,
}

impl NodeDescribeHandler {
    /// Create a new NodeDescribeHandler
    pub fn new() -> Self {
        Self {
            client_registry: None,
            capability_store: Arc::new(CapabilityStore::new()),
        }
    }

    /// Create a new NodeDescribeHandler with dependencies
    pub fn with_deps(client_registry: Arc<ClientRegistry>, capability_store: Arc<CapabilityStore>) -> Self {
        Self {
            client_registry: Some(client_registry),
            capability_store,
        }
    }

    /// Get the client registry if available
    fn get_client_registry(&self) -> Option<Arc<ClientRegistry>> {
        self.client_registry.clone()
    }
}

impl RpcMethod for NodeDescribeHandler {
    fn handle(&self, ctx: &RequestContext, params: Option<serde_json::Value>) -> RpcResponse {
        // Check if the connection is paired (has auth info with role "node")
        if ctx.auth_info().is_none() {
            return RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32003,
                "Connection is not authenticated",
            );
        }

        let auth_info = ctx.auth_info().unwrap();
        if auth_info.role != "node" {
            return RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32003,
                "Only node role can describe capabilities",
            );
        }

        // Parse parameters
        let params: NodeDescribeParams = match params {
            Some(p) => match serde_json::from_value::<NodeDescribeParams>(p) {
                Ok(p) => p,
                Err(e) => {
                    return RpcResponse::error(
                        Some(serde_json::json!(ctx.conn_id.clone())),
                        -32602,
                        format!("Invalid parameters: {}", e),
                    );
                }
            },
            None => {
                return RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32602,
                    "Missing parameters: capabilities required",
                );
            }
        };

        // Store capabilities in the capability store
        self.capability_store.store(&ctx.conn_id, params.capabilities.clone());

        // Update the client's device_capabilities field if registry is available
        if let Some(client_registry) = self.get_client_registry() {
            if let Some(client) = client_registry.get(&ctx.conn_id) {
                // Note: We can't directly modify the client since it's a Ref,
                // but this is stored in the CapabilityStore which is the source of truth
            }
        }

        // Build the result
        let services: Vec<String> = params
            .capabilities
            .iter()
            .map(|c| c.service.clone())
            .collect();

        RpcResponse::success(
            Some(serde_json::json!(ctx.conn_id.clone())),
            serde_json::json!({
                "accepted": true,
                "registered_services": services,
            }),
        )
    }
}

/// Handler for node.invoke RPC method
///
/// This method allows the server to invoke a capability on a device.
/// The server looks up the target device, sends the invocation request,
/// and waits for the response with a timeout.
pub struct NodeInvokeHandler {
    client_registry: Arc<ClientRegistry>,
    capability_store: Arc<CapabilityStore>,
}

impl NodeInvokeHandler {
    /// Create a new NodeInvokeHandler
    pub fn new(client_registry: Arc<ClientRegistry>, capability_store: Arc<CapabilityStore>) -> Self {
        Self {
            client_registry,
            capability_store,
        }
    }
}

impl RpcMethod for NodeInvokeHandler {
    fn handle(&self, ctx: &RequestContext, params: Option<serde_json::Value>) -> RpcResponse {
        // Parse parameters
        let params: NodeInvokeRequest = match params {
            Some(p) => match serde_json::from_value::<NodeInvokeRequest>(p) {
                Ok(p) => p,
                Err(e) => {
                    return RpcResponse::error(
                        Some(serde_json::json!(ctx.conn_id.clone())),
                        -32602,
                        format!("Invalid parameters: {}", e),
                    );
                }
            },
            None => {
                return RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32602,
                    "Missing parameters: service, method, params, and timeout_ms required",
                );
            }
        };

        // Validate timeout is within bounds
        if params.timeout_ms == 0 || params.timeout_ms > MAX_TIMEOUT_MS {
            return RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32602,
                format!(
                    "Invalid timeout: must be between 1 and {} milliseconds",
                    MAX_TIMEOUT_MS
                ),
            );
        }

        // Look up the target device by device_id
        // Uses device_id from the request if available, otherwise finds the first
        // node connection that advertises the service.
        let target_conn_id = match self.get_target_conn_id(&params.service, &params.device_id) {
            Some(id) => id,
            None => {
                return RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32004,
                    "Target device not found",
                );
            }
        };

        // Get the target client
        let target_client = match self.client_registry.get(&target_conn_id) {
            Some(client) => client,
            None => {
                return RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32004,
                    "Target device connection not found",
                );
            }
        };

        // Verify the requested service and method exist in the device's capabilities
        let capabilities = match self.capability_store.get(&target_conn_id) {
            Some(caps) => caps,
            None => {
                return RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32004,
                    "Device capabilities not found",
                );
            }
        };

        // Find the service and method
        let service_found = capabilities.iter().find(|c| c.service == params.service);
        let method_found = service_found.and_then(|s| {
            if s.methods.contains(&params.method) {
                Some(s)
            } else {
                None
            }
        });

        if method_found.is_none() {
            return RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32005,
                format!(
                    "Service '{}' does not have method '{}'",
                    params.service, params.method
                ),
            );
        }

        // Create a unique request ID for this invocation
        let request_id = serde_json::json!(Uuid::new_v4().to_string());

        // Build the JSON-RPC request to send to the device
        let device_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "node.invoke",
            "params": {
                "service": params.service,
                "method": params.method,
                "params": params.params,
                "timeout_ms": params.timeout_ms
            },
            "id": request_id.clone()
        });

        // Convert to WebSocket message
        let message = axum::extract::ws::Message::Text(device_request.to_string());

        // Send the request to the device
        let sender = target_client.sender.clone();
        if let Err(e) = sender.try_send(message) {
            return RpcResponse::error(
                Some(request_id),
                -32006,
                format!("Failed to send invocation to device: {}", e),
            );
        }

        // Wait for the response with timeout
        // In a real implementation, we would wait for a response on a response channel
        // registered when the device capability was described. The response channel
        // would receive the actual device response and return it here.
        //
        // Current implementation: sleeps for the timeout duration and returns a dummy
        // success response. This is a placeholder that should be replaced with proper
        // async response handling using tokio::select! with the response channel.
        let timeout_duration = Duration::from_millis(params.timeout_ms);
        
        // Note: In production, this would use tokio::time::timeout with a response channel
        // instead of block_on. For now, we sleep to simulate the invocation completing.
        tokio::runtime::Handle::current().block_on(async move {
            tokio::time::sleep(timeout_duration).await;
            debug!(
                "node.invoke completed for service '{}' method '{}' on device {}",
                params.service, params.method, target_conn_id
            );
        });

        // Return success response (in production, this would be the actual device response
        // received via the response channel)
        RpcResponse::success(
            Some(request_id),
            serde_json::json!({
                "success": true,
                "data": null,
                "error": null
            }),
        )
    }
}

impl NodeInvokeHandler {
    /// Get the target connection ID for a service
    /// Uses device_id from the request if available, otherwise falls back to
    /// finding the first node connection that advertises the service.
    fn get_target_conn_id(&self, service: &str, device_id: &Option<String>) -> Option<String> {
        // If device_id is provided, use it to find the target connection
        if let Some(device_id) = device_id {
            // First check if we have a mapping from device_id to conn_id
            if let Some(conn_id) = self.capability_store.get_conn_id_by_device_id(device_id) {
                return Some(conn_id);
            }
            
            // Fallback: search through all node connections for one that advertises this service
            let list = self.client_registry.list();
            for client in list.iter() {
                if client.role == "node" {
                    if let Some(capabilities) = self.capability_store.get(&client.conn_id) {
                        if capabilities.iter().any(|c| c.service == service) {
                            // Also update the mapping for future lookups
                            self.capability_store.store_with_device_id(
                                &client.conn_id,
                                capabilities.clone(),
                                Some(device_id.to_string())
                            );
                            return Some(client.conn_id.clone());
                        }
                    }
                }
            }
            
            return None;
        }
        
        // No device_id provided - return the first node connection that advertises the service
        let list = self.client_registry.list();
        for client in list.iter() {
            if client.role == "node" {
                if let Some(capabilities) = self.capability_store.get(&client.conn_id) {
                    if capabilities.iter().any(|c| c.service == service) {
                        return Some(client.conn_id.clone());
                    }
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn create_test_socket_addr() -> SocketAddr {
        "127.0.0.1:8080".parse().unwrap()
    }

    fn create_test_auth_info(role: &str) -> AuthInfo {
        AuthInfo {
            role: role.to_string(),
            scopes: vec![format!("{}:read", role)],
        }
    }

    fn create_test_request_context(role: &str) -> RequestContext {
        RequestContext::with_auth(
            "test-conn-id".to_string(),
            create_test_socket_addr(),
            create_test_auth_info(role),
        )
    }

    #[test]
    fn test_node_describe_handler_new() {
        let handler = NodeDescribeHandler::new();
        assert!(handler.client_registry.is_none());
        assert!(handler.capability_store.capabilities.read().unwrap().is_empty());
    }

    #[test]
    fn test_node_describe_handler_with_deps() {
        let client_registry = Arc::new(ClientRegistry::new());
        let capability_store = Arc::new(CapabilityStore::new());
        let handler = NodeDescribeHandler::with_deps(client_registry, capability_store);
        assert!(handler.client_registry.is_some());
    }

    #[test]
    fn test_node_describe_handler_unauthenticated() {
        let handler = NodeDescribeHandler::new();
        let ctx = RequestContext::new("test-conn".to_string(), create_test_socket_addr());
        let params = serde_json::json!({
            "capabilities": []
        });

        let response = handler.handle(&ctx, Some(params));

        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32003);
        assert_eq!(response.error.as_ref().unwrap().message, "Connection is not authenticated");
    }

    #[test]
    fn test_node_describe_handler_wrong_role() {
        let handler = NodeDescribeHandler::new();
        let ctx = create_test_request_context("operator");
        let params = serde_json::json!({
            "capabilities": []
        });

        let response = handler.handle(&ctx, Some(params));

        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32003);
        assert_eq!(
            response.error.as_ref().unwrap().message,
            "Only node role can describe capabilities"
        );
    }

    #[test]
    fn test_node_describe_handler_missing_params() {
        let handler = NodeDescribeHandler::new();
        let ctx = create_test_request_context("node");
        let params = None;

        let response = handler.handle(&ctx, params);

        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
        assert!(response
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("Missing parameters"));
    }

    #[test]
    fn test_node_describe_handler_invalid_params() {
        let handler = NodeDescribeHandler::new();
        let ctx = create_test_request_context("node");
        let params = serde_json::json!({
            "capabilities": "not a list"
        });

        let response = handler.handle(&ctx, Some(params));

        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
    }

    #[test]
    fn test_node_describe_handler_success() {
        let handler = NodeDescribeHandler::new();
        let ctx = create_test_request_context("node");
        let params = serde_json::json!({
            "capabilities": [
                {
                    "service": "camera",
                    "methods": ["take_photo", "record_video"],
                    "description": "Camera capabilities"
                },
                {
                    "service": "location",
                    "methods": ["get_coordinates"],
                    "description": "Location services"
                }
            ]
        });

        let response = handler.handle(&ctx, Some(params));

        assert!(response.result.is_some());
        let result = response.result.as_ref().unwrap();
        assert_eq!(result["accepted"], true);
        assert_eq!(
            result["registered_services"],
            serde_json::json!(["camera", "location"])
        );
    }

    #[test]
    fn test_node_invoke_handler_new() {
        let client_registry = Arc::new(ClientRegistry::new());
        let capability_store = Arc::new(CapabilityStore::new());
        let handler = NodeInvokeHandler::new(client_registry, capability_store);
        // Just verify it can be created
    }

    #[test]
    fn test_node_invoke_handler_missing_params() {
        let client_registry = Arc::new(ClientRegistry::new());
        let capability_store = Arc::new(CapabilityStore::new());
        let handler = NodeInvokeHandler::new(client_registry, capability_store);
        let ctx = create_test_request_context("operator");
        let params = None;

        let response = handler.handle(&ctx, params);

        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
    }

    #[test]
    fn test_node_invoke_handler_invalid_timeout() {
        let client_registry = Arc::new(ClientRegistry::new());
        let capability_store = Arc::new(CapabilityStore::new());
        let handler = NodeInvokeHandler::new(client_registry, capability_store);
        let ctx = create_test_request_context("operator");
        let params = serde_json::json!({
            "service": "camera",
            "method": "take_photo",
            "params": {},
            "timeout_ms": 0
        });

        let response = handler.handle(&ctx, Some(params));

        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
        assert!(response
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("Invalid timeout"));
    }

    #[test]
    fn test_node_invoke_handler_timeout_exceeded() {
        let client_registry = Arc::new(ClientRegistry::new());
        let capability_store = Arc::new(CapabilityStore::new());
        let handler = NodeInvokeHandler::new(client_registry, capability_store);
        let ctx = create_test_request_context("operator");
        let params = serde_json::json!({
            "service": "camera",
            "method": "take_photo",
            "params": {},
            "timeout_ms": 60000
        });

        let response = handler.handle(&ctx, Some(params));

        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
        assert!(response
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("must be between"));
    }
}
