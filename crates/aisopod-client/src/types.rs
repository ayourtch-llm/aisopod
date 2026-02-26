use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Client state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ClientState {
    Disconnected,
    Connected,
    Authenticating,
    Error,
}

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server_url: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub auth_token: String,
    pub client_name: String,
    pub client_version: String,
    pub device_id: Uuid,
    pub protocol_version: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:8080/ws".to_string(),
            auth_token: String::new(),
            client_name: "aisopod-client".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            device_id: Uuid::new_v4(),
            protocol_version: "1.0".to_string(),
        }
    }
}

/// Authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub client_name: String,
    pub client_version: String,
    pub device_id: Uuid,
    pub protocol_version: String,
    pub token: String,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Server event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEvent {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Device capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapability {
    pub service: String,
    pub methods: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: Uuid,
    pub device_name: String,
    pub device_type: String,
    pub device_version: String,
    pub capabilities: Vec<DeviceCapability>,
}

/// Chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

/// Pair request result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairRequestResult {
    pub pair_id: String,
    pub expires_at: i64,
}

/// Pair confirm result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairConfirmResult {
    pub device_token: String,
    pub paired_at: i64,
    pub scopes: Vec<String>,
}

/// Pair revoke result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairRevokeResult {
    pub revoked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Node describe result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDescribeResult {
    pub node_id: String,
    pub capabilities: Vec<DeviceCapability>,
}

/// Node invoke result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInvokeResult {
    pub result: serde_json::Value,
}
