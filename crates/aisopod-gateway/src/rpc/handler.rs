use std::collections::HashMap;
use std::net::SocketAddr;

use crate::rpc::types;

/// Request context for RPC method handlers
#[derive(Clone)]
pub struct RequestContext {
    pub conn_id: String,
    pub remote_addr: SocketAddr,
    pub role: Option<String>,
    pub scopes: Vec<String>,
}

impl RequestContext {
    /// Create a new request context with the given connection ID and remote address
    pub fn new(conn_id: String, remote_addr: SocketAddr) -> Self {
        Self {
            conn_id,
            remote_addr,
            role: None,
            scopes: Vec::new(),
        }
    }
}

/// RPC method trait - all handler implementations must implement this
pub trait RpcMethod {
    /// Handle an RPC request
    /// Returns an RpcResponse that will be sent back to the client
    fn handle(&self, ctx: &RequestContext, params: Option<serde_json::Value>) -> types::RpcResponse;
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
    fn handle(&self, _ctx: &RequestContext, _params: Option<serde_json::Value>) -> types::RpcResponse {
        types::RpcResponse::error(
            Some(serde_json::Value::Number(1.into())),
            types::error_codes::METHOD_NOT_FOUND,
            format!("Method {} is not implemented", self.method_name),
        )
    }
}

/// Method router that dispatches requests to the appropriate handler
pub struct MethodRouter {
    methods: HashMap<String, Box<dyn RpcMethod>>,
}

impl MethodRouter {
    /// Create a new empty method router
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    /// Register a handler for a specific method name
    pub fn register(&mut self, name: &str, handler: impl RpcMethod + 'static) {
        self.methods.insert(name.to_string(), Box::new(handler));
    }

    /// Dispatch an RPC request to the appropriate handler
    /// Returns a response with -32601 if the method is not found
    pub fn dispatch(&self, ctx: RequestContext, req: types::RpcRequest) -> types::RpcResponse {
        let method_name = &req.method;

        if let Some(handler) = self.methods.get(method_name) {
            handler.handle(&ctx, req.params)
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
        self.methods.len()
    }
}

impl Default for MethodRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default router with placeholder handlers for all 24 method namespaces
pub fn default_router() -> MethodRouter {
    let mut router = MethodRouter::new();
    
    // Define all 24 method namespaces (grouped by category)
    let namespaces = vec![
        // Agent methods (4)
        "agent.create",
        "agent.update",
        "agent.delete",
        "agent.list",
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
        // Config methods (4)
        "config.get",
        "config.set",
        "config.validate",
        "config.reload",
    ];
    
    for namespace in namespaces {
        router.register(namespace, PlaceholderHandler::new(namespace));
    }
    
    router
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
        let mut router = MethodRouter::new();
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
        assert!(response.error.as_ref().unwrap().message.contains("not found"));
    }

    #[test]
    fn test_default_router_method_count() {
        let router = default_router();
        
        // Should have 24 methods registered
        assert_eq!(router.method_count(), 24);
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
}
