//! Device Pairing Protocol Implementation
//!
//! This module implements the `node.pair.*` RPC methods that allow mobile/desktop
//! devices to securely pair with an aisopod server instance, receive a persistent
//! device token, and manage paired device lifecycle.
//!
//! The pairing protocol consists of three methods:
//! 1. `node.pair.request` - Initiates pairing by generating a 6-digit pairing code
//! 2. `node.pair.confirm` - Confirms pairing and issues a device token
//! 3. `node.pair.revoke` - Revokes a previously paired device

use crate::auth::{DeviceTokenManager, Scope};
use crate::rpc::handler::RpcMethod;
use crate::rpc::types;
use crate::rpc::RequestContext;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Maximum duration for a pairing code to be valid (5 minutes)
const PAIRING_CODE_EXPIRY: Duration = Duration::from_secs(5 * 60);

/// Pairing request parameters
#[derive(Debug, Clone, Deserialize)]
pub struct PairRequestParams {
    pub device_name: String,
    pub device_type: String,       // "ios", "android", "desktop"
    pub client_version: String,
    pub device_id: String,         // Device identifier (UUID string)
}

/// Pairing request result
#[derive(Debug, Serialize)]
pub struct PairRequestResult {
    pub pairing_code: String,      // 6-digit code
    pub expires_at: DateTime<Utc>,
    pub expires_in: u64,           // Seconds until expiration
}

/// Pairing confirmation parameters
#[derive(Debug, Deserialize)]
pub struct PairConfirmParams {
    pub pairing_code: String,
    pub device_id: String,         // Device identifier (UUID string)
}

/// Pairing confirmation result
#[derive(Debug, Serialize)]
pub struct PairConfirmResult {
    pub device_token: String,
    pub paired_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

/// Pairing revoke parameters
#[derive(Debug, Deserialize)]
pub struct PairRevokeParams {
    pub device_id: String,         // Device identifier (UUID string)
}

/// Pairing revoke result
#[derive(Debug, Serialize)]
pub struct PairRevokeResult {
    pub revoked: bool,
}

/// Pending pairing request stored during the pairing flow
#[derive(Debug, Clone)]
pub struct PendingPairing {
    pub params: PairRequestParams,
    pub pairing_code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Storage for pending pairing requests
pub struct PairingStore {
    /// Map from pairing_code to PendingPairing
    pending_pairings: Mutex<HashMap<String, PendingPairing>>,
    /// Map from device_id to pairing_code (for lookup by device_id)
    device_to_code: Mutex<HashMap<String, String>>,
}

impl PairingStore {
    /// Create a new empty PairingStore
    pub fn new() -> Self {
        Self {
            pending_pairings: Mutex::new(HashMap::new()),
            device_to_code: Mutex::new(HashMap::new()),
        }
    }

    /// Store a pending pairing request
    pub fn store(&self, pairing: PendingPairing) {
        let code = pairing.pairing_code.clone();
        let device_id = pairing.params.device_id.clone();
        
        {
            let mut pending = self.pending_pairings.lock().unwrap();
            pending.insert(code.clone(), pairing);
        }
        {
            let mut device_map = self.device_to_code.lock().unwrap();
            device_map.insert(device_id, code);
        }
    }

    /// Get a pending pairing by code
    pub fn get_by_code(&self, code: &str) -> Option<PendingPairing> {
        let pending = self.pending_pairings.lock().unwrap();
        pending.get(code).cloned()
    }

    /// Get a pending pairing by device_id
    pub fn get_by_device_id(&self, device_id: &str) -> Option<PendingPairing> {
        let device_map = self.device_to_code.lock().unwrap();
        device_map.get(device_id).and_then(|code| {
            let pending = self.pending_pairings.lock().unwrap();
            pending.get(code).cloned()
        })
    }

    /// Remove a pending pairing by code
    pub fn remove_by_code(&self, code: &str) -> Option<PendingPairing> {
        let pending_device_id = {
            let mut pending = self.pending_pairings.lock().unwrap();
            pending.remove(code).map(|p| p.params.device_id)
        };
        
        // Remove from device_to_code map
        if let Some(device_id) = pending_device_id {
            let mut device_map = self.device_to_code.lock().unwrap();
            device_map.remove(&device_id);
        }
        
        // We can't return the PendingPairing since we already removed it
        None
    }

    /// Remove a pending pairing by device_id
    pub fn remove_by_device_id(&self, device_id: &str) -> Option<PendingPairing> {
        let code = {
            let mut device_map = self.device_to_code.lock().unwrap();
            device_map.remove(device_id)
        };
        
        // Remove from pending_pairings map
        if let Some(code) = code {
            let mut pending = self.pending_pairings.lock().unwrap();
            return pending.remove(&code);
        }
        
        None
    }

    /// Clean up expired pairings
    pub fn cleanup_expired(&self) -> Vec<String> {
        let now = Utc::now();
        let expired_codes: Vec<String> = {
            let pending = self.pending_pairings.lock().unwrap();
            pending
                .iter()
                .filter(|(_, p)| p.expires_at < now)
                .map(|(code, _)| code.clone())
                .collect()
        };

        // Remove expired pairings
        let mut removed_codes = vec![];
        for code in &expired_codes {
            let mut pending = self.pending_pairings.lock().unwrap();
            let device_id = pending.remove(code).map(|p| p.params.device_id);
            
            if let Some(device_id) = device_id {
                let mut device_map = self.device_to_code.lock().unwrap();
                device_map.remove(&device_id);
                removed_codes.push(code.clone());
            }
        }
        
        removed_codes
    }

    /// Get the count of pending pairings
    pub fn pending_count(&self) -> usize {
        let pending = self.pending_pairings.lock().unwrap();
        pending.len()
    }

    /// Check if a pairing code exists
    pub fn code_exists(&self, code: &str) -> bool {
        let pending = self.pending_pairings.lock().unwrap();
        pending.contains_key(code)
    }

    /// Get the device_id associated with a pairing code
    pub fn get_device_id_by_code(&self, code: &str) -> Option<String> {
        let pending = self.pending_pairings.lock().unwrap();
        pending.get(code).map(|p| p.params.device_id.clone())
    }
}

impl Default for PairingStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random 6-digit numeric pairing code
pub fn generate_pairing_code() -> String {
    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(0..1_000_000);
    format!("{:06}", code)
}

/// Handler for node.pair.request RPC method
///
/// This method initiates the pairing process by generating a pairing code
/// that the user must confirm on the server.
pub struct PairRequestHandler {
    pairing_store: Arc<PairingStore>,
    token_manager: Arc<Mutex<DeviceTokenManager>>,
}

impl PairRequestHandler {
    /// Create a new PairRequestHandler
    pub fn new() -> Self {
        Self {
            pairing_store: Arc::new(PairingStore::new()),
            token_manager: Arc::new(Mutex::new(DeviceTokenManager::new(
                std::path::PathBuf::from("device_tokens.toml")
            ))),
        }
    }

    /// Create a new PairRequestHandler with custom dependencies
    pub fn with_deps(pairing_store: Arc<PairingStore>, token_manager: Arc<Mutex<DeviceTokenManager>>) -> Self {
        Self {
            pairing_store,
            token_manager,
        }
    }

    /// Validate device type is supported
    fn is_valid_device_type(device_type: &str) -> bool {
        matches!(device_type.to_lowercase().as_str(), "ios" | "android" | "desktop")
    }
}

impl RpcMethod for PairRequestHandler {
    fn handle(&self, ctx: &RequestContext, params: Option<serde_json::Value>) -> types::RpcResponse {
        // Parse parameters
        let params: PairRequestParams = match params {
            Some(p) => match serde_json::from_value(p) {
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

        // Validate device type
        if !Self::is_valid_device_type(&params.device_type) {
            return types::RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32602,
                format!("Invalid device_type: '{}'. Must be 'ios', 'android', or 'desktop'", params.device_type)
            );
        }

        // Validate device_id is a valid UUID
        if Uuid::parse_str(&params.device_id).is_err() {
            return types::RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32602,
                format!("Invalid device_id: '{}'. Must be a valid UUID string", params.device_id)
            );
        }

        // Generate pairing code
        let pairing_code = generate_pairing_code();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::from_std(PAIRING_CODE_EXPIRY).unwrap_or_default();

        // Store pending pairing
        let pairing = PendingPairing {
            params,
            pairing_code: pairing_code.clone(),
            created_at: now,
            expires_at,
        };
        self.pairing_store.store(pairing);

        types::RpcResponse::success(
            Some(serde_json::json!(ctx.conn_id.clone())),
            serde_json::json!({
                "pairing_code": pairing_code,
                "expires_at": expires_at.to_rfc3339(),
                "expires_in": PAIRING_CODE_EXPIRY.as_secs()
            }),
        )
    }
}

/// Handler for node.pair.confirm RPC method
///
/// This method confirms a pending pairing request and issues a device token.
pub struct PairConfirmHandler {
    pairing_store: Arc<PairingStore>,
    token_manager: Arc<Mutex<DeviceTokenManager>>,
}

impl PairConfirmHandler {
    /// Create a new PairConfirmHandler
    pub fn new() -> Self {
        Self {
            pairing_store: Arc::new(PairingStore::new()),
            token_manager: Arc::new(Mutex::new(DeviceTokenManager::new(
                std::path::PathBuf::from("device_tokens.toml")
            ))),
        }
    }

    /// Create a new PairConfirmHandler with custom dependencies
    pub fn with_deps(pairing_store: Arc<PairingStore>, token_manager: Arc<Mutex<DeviceTokenManager>>) -> Self {
        Self {
            pairing_store,
            token_manager,
        }
    }
}

impl RpcMethod for PairConfirmHandler {
    fn handle(&self, ctx: &RequestContext, params: Option<serde_json::Value>) -> types::RpcResponse {
        // Parse parameters
        let params: PairConfirmParams = match params {
            Some(p) => match serde_json::from_value(p) {
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

        // Look up the pending pairing
        let pairing = match self.pairing_store.get_by_code(&params.pairing_code) {
            Some(p) => p,
            None => {
                return types::RpcResponse::error(
                    Some(serde_json::json!(ctx.conn_id.clone())),
                    -32003,
                    "Invalid or expired pairing code"
                );
            }
        };

        // Check if expired
        let now = Utc::now();
        if now > pairing.expires_at {
            // Clean up expired pairing
            self.pairing_store.remove_by_code(&params.pairing_code);
            return types::RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32003,
                "Pairing code has expired"
            );
        }

        // Validate device_id matches
        if params.device_id != pairing.params.device_id {
            return types::RpcResponse::error(
                Some(serde_json::json!(ctx.conn_id.clone())),
                -32003,
                "Device ID does not match pairing request"
            );
        }

        // Issue device token
        let scopes = vec![Scope::OperatorRead.as_str().to_string()];
        let device_id = pairing.params.device_id.clone();
        
        let token = {
            let mut tm = self.token_manager.lock().unwrap();
            match tm.issue(
                pairing.params.device_name.clone(),
                device_id.clone(),
                vec![Scope::OperatorRead],
            ) {
                Ok(token) => token,
                Err(e) => {
                    return types::RpcResponse::error(
                        Some(serde_json::json!(ctx.conn_id.clone())),
                        -32003,
                        format!("Failed to issue device token: {}", e)
                    );
                }
            }
        };

        // Clean up the pairing
        self.pairing_store.remove_by_code(&params.pairing_code);

        types::RpcResponse::success(
            Some(serde_json::json!(ctx.conn_id.clone())),
            serde_json::json!({
                "device_token": token,
                "paired_at": now.to_rfc3339(),
                "scopes": scopes
            }),
        )
    }
}

/// Handler for node.pair.revoke RPC method
///
/// This method revokes a previously paired device, invalidating its token.
pub struct PairRevokeHandler {
    token_manager: Arc<Mutex<DeviceTokenManager>>,
}

impl PairRevokeHandler {
    /// Create a new PairRevokeHandler
    pub fn new() -> Self {
        Self {
            token_manager: Arc::new(Mutex::new(DeviceTokenManager::new(
                std::path::PathBuf::from("device_tokens.toml")
            ))),
        }
    }

    /// Create a new PairRevokeHandler with custom dependencies
    pub fn with_deps(token_manager: Arc<Mutex<DeviceTokenManager>>) -> Self {
        Self {
            token_manager,
        }
    }
}

impl RpcMethod for PairRevokeHandler {
    fn handle(&self, ctx: &RequestContext, params: Option<serde_json::Value>) -> types::RpcResponse {
        // Parse parameters
        let params: PairRevokeParams = match params {
            Some(p) => match serde_json::from_value(p) {
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

        // Revoke the device token
        let revoked = {
            let mut tm = self.token_manager.lock().unwrap();
            match tm.revoke(&params.device_id) {
                Ok(revoked) => revoked,
                Err(e) => {
                    return types::RpcResponse::error(
                        Some(serde_json::json!(ctx.conn_id.clone())),
                        -32003,
                        format!("Failed to revoke device token: {}", e)
                    );
                }
            }
        };

        types::RpcResponse::success(
            Some(serde_json::json!(ctx.conn_id.clone())),
            serde_json::json!({
                "revoked": revoked
            }),
        )
    }
}

/// Background cleanup task for expired pairing codes
///
/// This function should be spawned as a background task to periodically
/// clean up expired pairing codes from the store.
pub async fn run_pairing_cleanup_task(pairing_store: Arc<PairingStore>, interval: Duration) {
    loop {
        tokio::time::sleep(interval).await;
        pairing_store.cleanup_expired();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper to create a new handler with test dependencies
    fn create_handlers() -> (PairRequestHandler, PairConfirmHandler, PairRevokeHandler) {
        let pairing_store = Arc::new(PairingStore::new());
        let token_manager = Arc::new(Mutex::new(DeviceTokenManager::new(
            std::path::PathBuf::from("/tmp/test_device_tokens.toml")
        )));

        let request_handler = PairRequestHandler::with_deps(pairing_store.clone(), token_manager.clone());
        let confirm_handler = PairConfirmHandler::with_deps(pairing_store.clone(), token_manager.clone());
        let revoke_handler = PairRevokeHandler::with_deps(token_manager);

        (request_handler, confirm_handler, revoke_handler)
    }

    #[test]
    fn test_generate_pairing_code_format() {
        let code = generate_pairing_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_pairing_code_unique() {
        let code1 = generate_pairing_code();
        let code2 = generate_pairing_code();
        assert_ne!(code1, code2);
    }

    #[test]
    fn test_is_valid_device_type() {
        assert!(PairRequestHandler::is_valid_device_type("ios"));
        assert!(PairRequestHandler::is_valid_device_type("android"));
        assert!(PairRequestHandler::is_valid_device_type("desktop"));
        assert!(PairRequestHandler::is_valid_device_type("IOS"));
        assert!(PairRequestHandler::is_valid_device_type("Android"));
        assert!(PairRequestHandler::is_valid_device_type("DESKTOP"));
        assert!(!PairRequestHandler::is_valid_device_type("web"));
        assert!(!PairRequestHandler::is_valid_device_type("unknown"));
    }

    #[test]
    fn test_pair_request_success() {
        let (handler, _, _) = create_handlers();
        let ctx = RequestContext::new("test-conn".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        let params = serde_json::json!({
            "device_name": "Test Phone",
            "device_type": "ios",
            "client_version": "1.0.0",
            "device_id": "123e4567-e89b-12d3-a456-426614174000"
        });

        let response = handler.handle(&ctx, Some(params));
        
        assert!(response.result.is_some());
        let result = response.result.as_ref().unwrap();
        assert_eq!(result["pairing_code"].as_str().unwrap().len(), 6);
        assert!(result["expires_in"].as_u64().unwrap() > 0);
    }

    #[test]
    fn test_pair_request_invalid_device_type() {
        let (handler, _, _) = create_handlers();
        let ctx = RequestContext::new("test-conn".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        let params = serde_json::json!({
            "device_name": "Test Phone",
            "device_type": "web",
            "client_version": "1.0.0",
            "device_id": "123e4567-e89b-12d3-a456-426614174000"
        });

        let response = handler.handle(&ctx, Some(params));
        
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
    }

    #[test]
    fn test_pair_request_invalid_device_id() {
        let (handler, _, _) = create_handlers();
        let ctx = RequestContext::new("test-conn".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        let params = serde_json::json!({
            "device_name": "Test Phone",
            "device_type": "ios",
            "client_version": "1.0.0",
            "device_id": "not-a-uuid"
        });

        let response = handler.handle(&ctx, Some(params));
        
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
    }

    #[test]
    fn test_pair_request_missing_params() {
        let (handler, _, _) = create_handlers();
        let ctx = RequestContext::new("test-conn".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        let response = handler.handle(&ctx, None);
        
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
    }

    #[test]
    fn test_pair_confirm_success() {
        let (request_handler, confirm_handler, _) = create_handlers();
        let ctx = RequestContext::new("test-conn".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        // First, create a pairing
        let request_params = serde_json::json!({
            "device_name": "Test Phone",
            "device_type": "ios",
            "client_version": "1.0.0",
            "device_id": "123e4567-e89b-12d3-a456-426614174000"
        });
        
        let request_response = request_handler.handle(&ctx, Some(request_params));
        assert!(request_response.result.is_some());
        
        let pairing_code = request_response.result.as_ref().unwrap()["pairing_code"].as_str().unwrap().to_string();
        
        // Now confirm the pairing
        let confirm_params = serde_json::json!({
            "pairing_code": pairing_code,
            "device_id": "123e4567-e89b-12d3-a456-426614174000"
        });
        
        let confirm_response = confirm_handler.handle(&ctx, Some(confirm_params));
        
        assert!(confirm_response.result.is_some());
        let result = confirm_response.result.as_ref().unwrap();
        assert!(result["device_token"].is_string());
        assert!(result["paired_at"].is_string());
    }

    #[test]
    fn test_pair_confirm_invalid_code() {
        let (_, confirm_handler, _) = create_handlers();
        let ctx = RequestContext::new("test-conn".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        let params = serde_json::json!({
            "pairing_code": "000000",
            "device_id": "123e4567-e89b-12d3-a456-426614174000"
        });
        
        let response = confirm_handler.handle(&ctx, Some(params));
        
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32003);
        assert!(response.error.as_ref().unwrap().message.contains("Invalid or expired"));
    }

    #[test]
    fn test_pair_confirm_device_id_mismatch() {
        let (request_handler, confirm_handler, _) = create_handlers();
        let ctx = RequestContext::new("test-conn".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        // Create pairing with one device_id
        let request_params = serde_json::json!({
            "device_name": "Test Phone",
            "device_type": "ios",
            "client_version": "1.0.0",
            "device_id": "123e4567-e89b-12d3-a456-426614174000"
        });
        
        let request_response = request_handler.handle(&ctx, Some(request_params));
        assert!(request_response.result.is_some());
        
        let pairing_code = request_response.result.as_ref().unwrap()["pairing_code"].as_str().unwrap().to_string();
        
        // Try to confirm with different device_id
        let confirm_params = serde_json::json!({
            "pairing_code": pairing_code,
            "device_id": "00000000-0000-0000-0000-000000000000"
        });
        
        let response = confirm_handler.handle(&ctx, Some(confirm_params));
        
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32003);
        assert!(response.error.as_ref().unwrap().message.contains("Device ID does not match"));
    }

    #[test]
    fn test_pair_revoke_success() {
        let (_, _, revoke_handler) = create_handlers();
        let ctx = RequestContext::new("test-conn".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        // First, issue a token
        let token = {
            let mut tm = revoke_handler.token_manager.lock().unwrap();
            let token = tm.issue("Test Device".to_string(), "test-device-1".to_string(), vec![Scope::OperatorRead]).unwrap();
            
            // Verify token is valid
            assert!(tm.validate(&token).is_some());
            token
        };
        
        // Revoke the device
        let params = serde_json::json!({
            "device_id": "test-device-1"
        });
        
        let response = revoke_handler.handle(&ctx, Some(params));
        
        assert!(response.result.is_some());
        let result = response.result.as_ref().unwrap();
        assert_eq!(result["revoked"].as_bool().unwrap(), true);
        
        // Verify token is no longer valid
        {
            let mut tm = revoke_handler.token_manager.lock().unwrap();
            assert!(tm.validate(&token).is_none());
        }
    }

    #[test]
    fn test_pair_revoke_nonexistent_device() {
        let (_, _, revoke_handler) = create_handlers();
        let ctx = RequestContext::new("test-conn".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        let params = serde_json::json!({
            "device_id": "nonexistent-device"
        });
        
        let response = revoke_handler.handle(&ctx, Some(params));
        
        assert!(response.result.is_some());
        let result = response.result.as_ref().unwrap();
        assert_eq!(result["revoked"].as_bool().unwrap(), false);
    }

    #[test]
    fn test_pairing_store_cleanup_expired() {
        let store = Arc::new(PairingStore::new());
        
        // Store a pairing
        let pairing = PendingPairing {
            params: PairRequestParams {
                device_name: "Test".to_string(),
                device_type: "ios".to_string(),
                client_version: "1.0.0".to_string(),
                device_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            },
            pairing_code: "123456".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() - chrono::Duration::minutes(1), // Already expired
        };
        store.store(pairing);
        
        // Cleanup should remove it
        let removed = store.cleanup_expired();
        assert_eq!(removed.len(), 1);
        assert!(store.get_by_code("123456").is_none());
    }

    #[test]
    fn test_full_pairing_flow() {
        let (request_handler, confirm_handler, revoke_handler) = create_handlers();
        let ctx = RequestContext::new("test-conn".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        // Step 1: Request pairing
        let request_params = serde_json::json!({
            "device_name": "Test Phone",
            "device_type": "ios",
            "client_version": "1.0.0",
            "device_id": "123e4567-e89b-12d3-a456-426614174000"
        });
        
        let request_response = request_handler.handle(&ctx, Some(request_params));
        assert!(request_response.result.is_some());
        
        let pairing_code = request_response.result.as_ref().unwrap()["pairing_code"].as_str().unwrap().to_string();
        
        // Step 2: Confirm pairing
        let confirm_params = serde_json::json!({
            "pairing_code": pairing_code,
            "device_id": "123e4567-e89b-12d3-a456-426614174000"
        });
        
        let confirm_response = confirm_handler.handle(&ctx, Some(confirm_params));
        assert!(confirm_response.result.is_some());
        
        let device_token = confirm_response.result.as_ref().unwrap()["device_token"].as_str().unwrap().to_string();
        
        // Step 3: Verify token is valid
        {
            let mut tm = revoke_handler.token_manager.lock().unwrap();
            let validated_token = tm.validate(&device_token);
            assert!(validated_token.is_some());
            assert_eq!(validated_token.unwrap().device_name, "Test Phone");
        }
        
        // Step 4: Revoke device
        let params = serde_json::json!({
            "device_id": "123e4567-e89b-12d3-a456-426614174000"
        });
        
        let revoke_response = revoke_handler.handle(&ctx, Some(params));
        assert!(revoke_response.result.is_some());
        assert!(revoke_response.result.as_ref().unwrap()["revoked"].as_bool().unwrap());
        
        // Step 5: Verify token is no longer valid
        {
            let mut tm = revoke_handler.token_manager.lock().unwrap();
            assert!(tm.validate(&device_token).is_none());
        }
    }
}
