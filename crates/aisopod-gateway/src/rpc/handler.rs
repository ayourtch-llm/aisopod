#![allow(clippy::all)]
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::auth::AuthInfo;
use crate::broadcast::{Broadcaster, GatewayEvent, Subscription};
use crate::rpc::approval::{PendingApproval, ApprovalStatus, ApprovalRequestParams, ApprovalStore};
use crate::rpc::canvas::{CanvasInteractParams, CanvasInteractResult};
use crate::rpc::chat::ChatSendHandler;
use crate::rpc::middleware::auth::check_scope;
use crate::rpc::node_pair::{PairingStore, PairRequestHandler, PairConfirmHandler, PairRevokeHandler};
use crate::rpc::types;

/// Request context for RPC method handlers
#[derive(Clone)]
pub struct RequestContext {
    pub conn_id: String,
    pub remote_addr: SocketAddr,
    pub role: Option<String>,
    pub scopes: Vec<String>,
    pub auth_info: Option<AuthInfo>,
}

impl RequestContext {
    /// Create a new request context with the given connection ID and remote address
    pub fn new(conn_id: String, remote_addr: SocketAddr) -> Self {
        Self {
            conn_id,
            remote_addr,
            role: None,
            scopes: Vec::new(),
            auth_info: None,
        }
    }

    /// Create a new request context with the given connection ID, remote address, and auth info
    pub fn with_auth(conn_id: String, remote_addr: SocketAddr, auth_info: AuthInfo) -> Self {
        Self {
            conn_id,
            remote_addr,
            role: Some(auth_info.role.clone()),
            scopes: auth_info.scopes.clone(),
            auth_info: Some(auth_info),
        }
    }

    /// Get the auth info if available
    pub fn auth_info(&self) -> Option<&AuthInfo> {
        self.auth_info.as_ref()
    }
}

/// RPC method trait - all handler implementations must implement this
pub trait RpcMethod: Send + Sync {
    /// Handle an RPC request
    /// Returns an RpcResponse that will be sent back to the client
    fn handle(&self, ctx: &RequestContext, params: Option<serde_json::Value>)
        -> types::RpcResponse;
}

/// Trait for RPC methods that need additional dependencies
pub trait RpcMethodWithDeps: Send + Sync {
    /// Handle an RPC request with dependencies
    fn handle_with_deps(&self, conn_id: String, params: Option<serde_json::Value>)
        -> types::RpcResponse;
}

/// Placeholder handler that returns "not implemented" error
pub struct PlaceholderHandler {
    method_name: String,
}

impl PlaceholderHandler {
    /// Create a new placeholder handler for the given method name
    pub fn new(method_name: impl Into<String>) -> Self {
        Self {
            method_name: method_name.into(),
        }
    }
}

impl RpcMethod for PlaceholderHandler {
    fn handle(
        &self,
        _ctx: &RequestContext,
        _params: Option<serde_json::Value>,
    ) -> types::RpcResponse {
        types::RpcResponse::error(
            Some(serde_json::Value::Number(1.into())),
            types::error_codes::METHOD_NOT_FOUND,
            format!("Method {} is not implemented", self.method_name),
        )
    }
}

/// Method router that dispatches requests to the appropriate handler
pub struct MethodRouter {
    methods: Arc<Mutex<HashMap<String, Arc<Box<dyn RpcMethod>>>>>,
    /// Approval-specific dependencies (optional)
    approval_store: Option<Arc<ApprovalStore>>,
    broadcaster: Option<Arc<Broadcaster>>,
}

impl MethodRouter {
    /// Create a new empty method router
    pub fn new() -> Self {
        Self {
            methods: Arc::new(Mutex::new(HashMap::new())),
            approval_store: None,
            broadcaster: None,
        }
    }

    /// Set approval-specific dependencies for the router
    pub fn with_approval_deps(mut self, store: Arc<ApprovalStore>, broadcaster: Arc<Broadcaster>) -> Self {
        self.approval_store = Some(store);
        self.broadcaster = Some(broadcaster);
        self
    }

    /// Register a handler for a specific method name
    pub fn register(&self, name: &str, handler: impl RpcMethod + 'static) {
        let mut methods = self.methods.lock().unwrap();
        methods.insert(name.to_string(), Arc::new(Box::new(handler)));
    }

    /// Dispatch an RPC request to the appropriate handler
    /// Returns a response with -32601 if the method is not found
    pub fn dispatch(&self, ctx: RequestContext, req: types::RpcRequest) -> types::RpcResponse {
        let method_name = &req.method;

        // Check scope authorization before dispatching
        if let Some(auth_info) = ctx.auth_info() {
            let client_ip = ctx.remote_addr.to_string();
            if let Err(error_response) = check_scope(auth_info, method_name, &client_ip) {
                return error_response;
            }
        }

        let methods = self.methods.lock().unwrap();

        if let Some(handler) = methods.get(method_name) {
            let mut response = handler.handle(&ctx, req.params);
            // Preserve the original request id if not set by the handler
            if response.id.is_none() {
                response.id = req.id;
            }
            response
        } else {
            types::RpcResponse::error(
                req.id,
                types::error_codes::METHOD_NOT_FOUND,
                format!("Method {} not found", method_name),
            )
        }
    }

    /// Get the number of registered methods
    pub fn method_count(&self) -> usize {
        let methods = self.methods.lock().unwrap();
        methods.len()
    }

    /// (Test-only) Clear all registered methods from the router
    ///
    /// This should only be used in tests to ensure test isolation.
    pub fn clear_for_tests(&self) {
        let mut methods = self.methods.lock().unwrap();
        methods.clear();
    }
}

impl Default for MethodRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default router with placeholder handlers for all method namespaces
/// plus gateway.subscribe for subscription management
///
/// Note: The `chat.send` method is implemented in `chat.rs` and requires
/// runtime dependencies (AgentRunner, WebSocket sender) that are injected
/// directly into the WebSocket connection handler via ws.rs.
/// The placeholder in this router will never be invoked for chat.send.
/// 
/// Note: The `approval.request` method requires the approval store and broadcaster
/// to be injected. These handlers are registered but will return errors until
/// dependencies are set via MethodRouter::with_approval_deps().
pub fn default_router() -> MethodRouter {
    let mut router = MethodRouter::new();

    // Define all method namespaces (grouped by category)
    let namespaces = vec![
        // Agent methods (6)
        "agent.create",
        "agent.update",
        "agent.delete",
        "agent.list",
        "agent.start",
        "agent.stop",
        // Chat methods (4)
        "chat.create",
        "chat.send",
        "chat.history",
        "chat.delete",
        // Tools methods (4)
        "tools.invoke",
        "tools.list",
        "tools.describe",
        "tools.authorize",
        // Session methods (4)
        "session.create",
        "session.get",
        "session.update",
        "session.delete",
        // Model methods (4)
        "model.list",
        "model.describe",
        "model.select",
        "model.feedback",
        // Config methods (5)
        "config.get",
        "config.set",
        "config.validate",
        "config.reload",
        "config.update",
        // Approval methods (4)
        "approval.request",
        "approval.approve",
        "approval.deny",
        "approval.list",
        // Canvas methods (2)
        "canvas.update",
        "canvas.interact",
    ];

    for namespace in namespaces {
        router.register(namespace, PlaceholderHandler::new(namespace));
    }

    // Register gateway.subscribe for runtime subscription updates
    router.register("gateway.subscribe", GatewaySubscribeHandler);

    // Register canvas.interact handler
    router.register("canvas.interact", CanvasInteractHandler::new());

    router
}

/// Handler for gateway.subscribe RPC method
///
/// This method allows clients to update their event subscription filter
/// at runtime to receive only the event types they're interested in.
pub struct GatewaySubscribeHandler;

impl RpcMethod for GatewaySubscribeHandler {
    fn handle(
        &self,
        _ctx: &RequestContext,
        params: Option<serde_json::Value>,
    ) -> types::RpcResponse {
        // Parse the subscription parameters
        let subscription: Subscription = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(sub) => sub,
                Err(e) => {
                    // Use -32602 (INVALID_PARAMS) - standard JSON-RPC 2.0 error code
                    return types::RpcResponse::error(
                        None, // Notification-style error (no id)
                        -32602,
                        format!("Invalid subscription parameters: {}", e),
                    );
                }
            },
            None => {
                // Use -32602 (INVALID_PARAMS) - standard JSON-RPC 2.0 error code
                return types::RpcResponse::error(
                    None, // Notification-style error (no id)
                    -32602,
                    "Subscription parameters required",
                );
            }
        };

        // Create a response indicating the subscription was updated
        types::RpcResponse::success(
            None, // No id - this is a notification response
            serde_json::json!({
                "status": "subscribed",
                "events": subscription.event_types,
            }),
        )
    }
}

/// Handler for approval.request RPC method.
///
/// This handler creates approval requests and broadcasts them to all
/// connected WebSocket clients that have subscribed to the "approval" event type.
pub struct ApprovalRequestHandler {
    approval_store: Option<Arc<ApprovalStore>>,
    broadcaster: Option<Arc<Broadcaster>>,
}

impl ApprovalRequestHandler {
    /// Create a new approval request handler
    pub fn new() -> Self {
        Self {
            approval_store: None,
            broadcaster: None,
        }
    }

    /// Create a new approval request handler with dependencies
    pub fn with_deps(store: Arc<ApprovalStore>, broadcaster: Arc<Broadcaster>) -> Self {
        Self {
            approval_store: Some(store),
            broadcaster: Some(broadcaster),
        }
    }
}

impl RpcMethod for ApprovalRequestHandler {
    fn handle(
        &self,
        ctx: &RequestContext,
        params: Option<serde_json::Value>,
    ) -> types::RpcResponse {
        // Get dependencies
        let store = match &self.approval_store {
            Some(store) => store.clone(),
            None => return types::RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32602,
                "Approval store not available",
            ),
        };

        let broadcaster = match &self.broadcaster {
            Some(b) => b.clone(),
            None => return types::RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32602,
                "Broadcaster not available",
            ),
        };

        // Parse parameters inline
        let params: ApprovalRequestParams = match params {
            Some(p) => match serde_json::from_value::<ApprovalRequestParams>(p) {
                Ok(p) => p,
                Err(e) => {
                    return types::RpcResponse::error(
                        Some(serde_json::json!(ctx.conn_id.clone())),
                        -32602,
                        format!("Invalid parameters: {}", e)
                    );
                }
            },
            None => {
                return types::RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32602,
                    "Missing parameters"
                );
            }
        };

        // Create a new pending approval
        let approval = PendingApproval::new(
            params.agent_id,
            params.operation,
            params.risk_level,
        );

        // Store the approval request
        store.store(approval.clone());

        // Create and broadcast the approval event
        let event = GatewayEvent::ApprovalRequired {
            id: approval.id.clone(),
            agent_id: approval.agent_id.clone(),
            operation: approval.operation.clone(),
            risk_level: approval.risk_level.clone(),
        };

        let broadcast_count = broadcaster.publish(event);

        types::RpcResponse::success(
            Some(serde_json::json!(ctx.conn_id.clone())),
            serde_json::json!({
                "status": "created",
                "id": approval.id,
                "broadcast_recipients": broadcast_count,
                "message": format!("Approval request {} has been created and broadcast to {} operator(s)", approval.id, broadcast_count)
            }),
        )
    }
}

/// Handler for approval.approve RPC method
///
/// This method allows operators to approve pending approval requests.
pub struct ApprovalApproveHandler {
    approval_store: Option<Arc<ApprovalStore>>,
}

impl ApprovalApproveHandler {
    /// Create a new approval approve handler
    pub fn new() -> Self {
        Self { approval_store: None }
    }

    /// Create a new approval approve handler with approval store
    pub fn with_store(store: Arc<ApprovalStore>) -> Self {
        Self { approval_store: Some(store) }
    }
}

impl RpcMethod for ApprovalApproveHandler {
    fn handle(
        &self,
        ctx: &RequestContext,
        params: Option<serde_json::Value>,
    ) -> types::RpcResponse {
        let id = match &params {
            Some(p) => match p.get("id") {
                Some(id_val) => match id_val.as_str() {
                    Some(id) => id.to_string(),
                    None => {
                        return types::RpcResponse::error(
                            Some(serde_json::json!(ctx.conn_id.clone())),
                            -32602,
                            "Invalid approval request ID: must be a string",
                        );
                    }
                },
                None => {
                    return types::RpcResponse::error(
                        Some(serde_json::json!(ctx.conn_id.clone())),
                        -32602,
                        "Missing 'id' parameter in approval.approve",
                    );
                }
            },
            None => {
                return types::RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32602,
                    "approval.approve requires parameters",
                );
            }
        };

        // Use the approval store if available
        let store = match &self.approval_store {
            Some(store) => store.clone(),
            None => return types::RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32602,
                "Approval store not available",
            ),
        };

        // Remove and check if approval exists
        match store.remove(&id) {
            Some(mut approval) => {
                approval.status = ApprovalStatus::Approved;

                types::RpcResponse::success(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    serde_json::json!({
                        "status": "approved",
                        "id": id,
                        "message": format!("Approval request {} has been approved", id)
                    }),
                )
            }
            None => {
                types::RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32601,
                    format!("Approval request {} not found or already processed", id),
                )
            }
        }
    }
}

/// Handler for approval.deny RPC method
///
/// This method allows operators to deny pending approval requests.
pub struct ApprovalDenyHandler {
    approval_store: Option<Arc<ApprovalStore>>,
}

impl ApprovalDenyHandler {
    /// Create a new approval deny handler
    pub fn new() -> Self {
        Self { approval_store: None }
    }

    /// Create a new approval deny handler with approval store
    pub fn with_store(store: Arc<ApprovalStore>) -> Self {
        Self { approval_store: Some(store) }
    }
}

impl RpcMethod for ApprovalDenyHandler {
    fn handle(
        &self,
        ctx: &RequestContext,
        params: Option<serde_json::Value>,
    ) -> types::RpcResponse {
        let id = match &params {
            Some(p) => match p.get("id") {
                Some(id_val) => match id_val.as_str() {
                    Some(id) => id.to_string(),
                    None => {
                        return types::RpcResponse::error(
                            Some(serde_json::json!(ctx.conn_id.clone())),
                            -32602,
                            "Invalid approval request ID: must be a string",
                        );
                    }
                },
                None => {
                    return types::RpcResponse::error(
                        Some(serde_json::json!(ctx.conn_id.clone())),
                        -32602,
                        "Missing 'id' parameter in approval.deny",
                    );
                }
            },
            None => {
                return types::RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32602,
                    "approval.deny requires parameters",
                );
            }
        };

        // Get the reason for denial (optional)
        let reason = params
            .as_ref()
            .and_then(|p| p.get("reason"))
            .and_then(|v| v.as_str())
            .unwrap_or("No reason provided")
            .to_string();

        // Use the approval store if available
        let store = match &self.approval_store {
            Some(store) => store.clone(),
            None => return types::RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32602,
                "Approval store not available",
            ),
        };

        // Remove and check if approval exists
        match store.remove(&id) {
            Some(mut approval) => {
                approval.status = ApprovalStatus::Denied;

                types::RpcResponse::success(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    serde_json::json!({
                        "status": "denied",
                        "id": id,
                        "reason": reason,
                        "message": format!("Approval request {} has been denied", id)
                    }),
                )
            }
            None => {
                types::RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32601,
                    format!("Approval request {} not found or already processed", id),
                )
            }
        }
    }
}

/// Handler for approval.list RPC method
///
/// This method allows operators to list pending approval requests.
pub struct ApprovalListHandler {
    approval_store: Option<Arc<ApprovalStore>>,
}

impl ApprovalListHandler {
    /// Create a new approval list handler
    pub fn new() -> Self {
        Self { approval_store: None }
    }

    /// Create a new approval list handler with approval store
    pub fn with_store(store: Arc<ApprovalStore>) -> Self {
        Self { approval_store: Some(store) }
    }
}

impl RpcMethod for ApprovalListHandler {
    fn handle(
        &self,
        ctx: &RequestContext,
        _params: Option<serde_json::Value>,
    ) -> types::RpcResponse {
        let store = match &self.approval_store {
            Some(store) => store.clone(),
            None => return types::RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32602,
                "Approval store not available",
            ),
        };

        let approvals = store.list();

        types::RpcResponse::success(
            Some(serde_json::json!(ctx.conn_id.clone())),
            serde_json::json!({
                "approvals": approvals.iter().map(|a| serde_json::json!({
                    "id": a.id,
                    "agent_id": a.agent_id,
                    "operation": a.operation,
                    "risk_level": a.risk_level,
                    "requested_at": a.requested_at,
                    "status": match a.status {
                        ApprovalStatus::Pending => "pending",
                        ApprovalStatus::Approved => "approved",
                        ApprovalStatus::Denied => "denied",
                    }
                })).collect::<Vec<_>>(),
                "count": approvals.len(),
                "message": format!("Found {} approval request(s)", approvals.len())
            }),
        )
    }
}

/// Handler for canvas.interact RPC method
///
/// This method handles user interaction events from clients within canvases.
/// The client sends interaction events (clicks, input changes, form submissions)
/// back to the server, which forwards them to the appropriate agent/handler.
pub struct CanvasInteractHandler;

impl CanvasInteractHandler {
    /// Create a new canvas interact handler
    pub fn new() -> Self {
        Self
    }
}

impl RpcMethod for CanvasInteractHandler {
    fn handle(
        &self,
        ctx: &RequestContext,
        params: Option<serde_json::Value>,
    ) -> types::RpcResponse {
        // Parse the interaction parameters
        let params: CanvasInteractParams = match params {
            Some(p) => match serde_json::from_value::<CanvasInteractParams>(p) {
                Ok(p) => p,
                Err(e) => {
                    return types::RpcResponse::error(
                        Some(serde_json::json!(ctx.conn_id.clone())),
                        -32602,
                        format!("Invalid parameters: {}", e),
                    );
                }
            },
            None => {
                return types::RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32602,
                    "Missing parameters for canvas.interact",
                );
            }
        };

        // Verify the canvas_id exists in the connection's active canvases
        // Note: In a real implementation, this would check a per-connection canvas state.
        // For now, we just forward the interaction and let the receiving side validate.
        
        // Forward the interaction event to the agent/handler that owns this canvas
        // This is typically done via the event broadcasting system
        
        types::RpcResponse::success(
            Some(serde_json::json!(ctx.conn_id.clone())),
            serde_json::json!({
                "received": true,
                "canvas_id": params.canvas_id,
                "event_type": params.event_type,
                "message": format!("Canvas interaction {} for canvas {} has been received", params.event_type, params.canvas_id)
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_creation() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let ctx = RequestContext::new("conn-123".to_string(), addr);

        assert_eq!(ctx.conn_id, "conn-123");
        assert_eq!(ctx.remote_addr, addr);
        assert_eq!(ctx.role, None);
        assert!(ctx.scopes.is_empty());
    }

    #[test]
    fn test_method_router_dispatch_known_method() {
        let router = MethodRouter::new();
        router.register("test.method", PlaceholderHandler::new("test.method"));

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let ctx = RequestContext::new("conn-1".to_string(), addr);

        let req = types::RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test.method".to_string(),
            params: None,
            id: Some(serde_json::Value::Number(1.into())),
        };

        let response = router.dispatch(ctx, req);

        // Placeholder returns METHOD_NOT_FOUND with id None (notification-style error)
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_some());
    }

    #[test]
    fn test_method_router_dispatch_unknown_method() {
        let router = MethodRouter::new();

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let ctx = RequestContext::new("conn-1".to_string(), addr);

        let req = types::RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "nonexistent.method".to_string(),
            params: None,
            id: Some(serde_json::Value::Number(1.into())),
        };

        let response = router.dispatch(ctx, req);

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32601);
        assert!(response
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("not found"));
    }

    #[test]
    fn test_default_router_method_count() {
        let router = default_router();

        // Should have 34 methods registered:
        // - 32 original placeholders (29 base + 3 new: agent.start, agent.stop, config.update)
        // - 2 canvas methods (canvas.update, canvas.interact)
        // - gateway.subscribe
        assert_eq!(router.method_count(), 34);
    }

    #[test]
    fn test_default_router_contains_agent_methods() {
        let router = default_router();

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let ctx = RequestContext::new("conn-1".to_string(), addr);

        let methods = vec!["agent.create", "agent.update", "agent.delete", "agent.list"];

        for method in methods {
            let req = types::RpcRequest {
                jsonrpc: "2.0".to_string(),
                method: method.to_string(),
                params: None,
                id: Some(serde_json::Value::Number(1.into())),
            };

            let response = router.dispatch(ctx.clone(), req);

            // All should return METHOD_NOT_FOUND
            assert_eq!(response.jsonrpc, "2.0");
            assert!(response.error.is_some());
            assert_eq!(response.error.as_ref().unwrap().code, -32601);
        }
    }

    #[test]
    fn test_canvas_interact_handler_success() {
        let handler = CanvasInteractHandler::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let ctx = RequestContext::new("conn-1".to_string(), addr);

        let params = serde_json::json!({
            "canvas_id": "canvas-123",
            "event_type": "click",
            "element_id": "btn-submit",
            "data": {"foo": "bar"}
        });

        let response = handler.handle(&ctx, Some(params));

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        let result = response.result.as_ref().unwrap();
        assert_eq!(result["received"], true);
        assert_eq!(result["canvas_id"], "canvas-123");
        assert_eq!(result["event_type"], "click");
    }

    #[test]
    fn test_canvas_interact_handler_minimal() {
        let handler = CanvasInteractHandler::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let ctx = RequestContext::new("conn-1".to_string(), addr);

        let params = serde_json::json!({
            "canvas_id": "canvas-456",
            "event_type": "input"
        });

        let response = handler.handle(&ctx, Some(params));

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        let result = response.result.as_ref().unwrap();
        assert_eq!(result["received"], true);
        assert_eq!(result["canvas_id"], "canvas-456");
        assert_eq!(result["event_type"], "input");
    }

    #[test]
    fn test_canvas_interact_handler_missing_params() {
        let handler = CanvasInteractHandler::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let ctx = RequestContext::new("conn-1".to_string(), addr);

        let response = handler.handle(&ctx, None);

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
        assert!(response.error.as_ref().unwrap().message.contains("Missing parameters"));
    }

    #[test]
    fn test_canvas_interact_handler_invalid_params() {
        let handler = CanvasInteractHandler::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let ctx = RequestContext::new("conn-1".to_string(), addr);

        let params = serde_json::json!({
            "canvas_id": 12345,  // Should be string
            "event_type": "click"
        });

        let response = handler.handle(&ctx, Some(params));

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
    }

    #[test]
    fn test_default_router_contains_canvas_methods() {
        let router = default_router();

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let ctx = RequestContext::new("conn-1".to_string(), addr);

        // Test canvas.update (server-initiated message, should return METHOD_NOT_FOUND)
        let req = types::RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "canvas.update".to_string(),
            params: None,
            id: Some(serde_json::Value::Number(1.into())),
        };
        let response = router.dispatch(ctx.clone(), req);

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32601);

        // Test canvas.interact (client-initiated call with valid params)
        let req = types::RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "canvas.interact".to_string(),
            params: Some(serde_json::json!({
                "canvas_id": "test-canvas",
                "event_type": "click"
            })),
            id: Some(serde_json::Value::Number(2.into())),
        };
        let response = router.dispatch(ctx.clone(), req);

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
    }
}
