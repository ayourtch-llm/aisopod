use serde::{Deserialize, Serialize};
use thiserror::Error;

/// JSON-RPC Request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: String,
}

impl RpcRequest {
    pub fn new(method: &str, params: Option<serde_json::Value>, id: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: id.to_string(),
        }
    }
}

/// JSON-RPC Error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: String,
}

impl RpcResponse {
    pub fn is_success(&self) -> bool {
        self.result.is_some() && self.error.is_none()
    }

    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn get_result(&self) -> Option<&serde_json::Value> {
        self.result.as_ref()
    }

    pub fn get_error(&self) -> Option<&RpcError> {
        self.error.as_ref()
    }
}

/// Parse a JSON-RPC response from a string
pub fn parse_response(json_str: &str) -> std::result::Result<RpcResponse, ParseResponseError> {
    serde_json::from_str(json_str).map_err(|e| ParseResponseError::ParseError(e.to_string()))
}

/// Parse response error
#[derive(Error, Debug)]
pub enum ParseResponseError {
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Error codes for JSON-RPC
pub mod error_codes {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32006;
    pub const AUTH_ERROR: i64 = -32003;
    pub const NOT_FOUND: i64 = -32004;
    pub const METHOD_NOT_ALLOWED: i64 = -32005;
}

/// Create an error response
pub fn error_response(code: i64, message: &str, id: &str) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(RpcError {
            code,
            message: message.to_string(),
            data: None,
        }),
        id: id.to_string(),
    }
}
