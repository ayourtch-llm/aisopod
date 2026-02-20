#![allow(clippy::all)]
use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RpcRequest {
    #[serde(default)]
    pub jsonrpc: String, // must be "2.0"
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcResponse {
    pub jsonrpc: &'static str, // always "2.0"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: Option<serde_json::Value>,
}

impl RpcResponse {
    /// Create a successful response
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response with default data
    pub fn error(id: Option<serde_json::Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }

    /// Create an error response with optional custom data
    pub fn error_with_data(
        id: Option<serde_json::Value>,
        code: i32,
        message: impl Into<String>,
        data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0",
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data,
            }),
            id,
        }
    }
}

/// Standard JSON-RPC 2.0 error codes
pub mod error_codes {
    /// Parse error - invalid JSON
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid request - missing required fields
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found
    pub const METHOD_NOT_FOUND: i32 = -32601;
}

/// Parse a raw JSON-RPC request string into an RpcRequest
/// Returns Ok(RpcRequest) if parsing and validation succeed
/// Returns Err(RpcResponse) with appropriate error on failure
pub fn parse(raw: &str) -> Result<RpcRequest, RpcResponse> {
    // First, try to deserialize the JSON
    let request: RpcRequest = match serde_json::from_str(raw) {
        Ok(req) => req,
        Err(e) => {
            // Failed to parse JSON - return parse error (-32700)
            let error = RpcError {
                code: error_codes::PARSE_ERROR,
                message: format!("Failed to parse JSON: {}", e),
                data: None,
            };
            return Err(RpcResponse {
                jsonrpc: "2.0",
                result: None,
                error: Some(error),
                id: None,
            });
        }
    };

    // Validate jsonrpc version - must be exactly "2.0"
    if request.jsonrpc != "2.0" {
        let error = RpcError {
            code: error_codes::INVALID_REQUEST,
            message: format!(
                "Invalid jsonrpc version: expected '2.0', got '{}'",
                request.jsonrpc
            ),
            data: None,
        };
        return Err(RpcResponse {
            jsonrpc: "2.0",
            result: None,
            error: Some(error),
            id: request.id,
        });
    }

    // Validate that method field is present (not empty)
    if request.method.is_empty() {
        let error = RpcError {
            code: error_codes::INVALID_REQUEST,
            message: "Missing or empty 'method' field".to_string(),
            data: None,
        };
        return Err(RpcResponse {
            jsonrpc: "2.0",
            result: None,
            error: Some(error),
            id: request.id,
        });
    }

    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_request() {
        let json = r#"{"jsonrpc":"2.0","method":"test","params":null,"id":1}"#;
        let result = parse(json);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test");
        assert_eq!(request.params, None);
        assert_eq!(request.id, Some(serde_json::Value::Number(1.into())));
    }

    #[test]
    fn test_parse_valid_request_with_params() {
        let json = r#"{"jsonrpc":"2.0","method":"add","params":[1,2],"id":"req-1"}"#;
        let result = parse(json);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.method, "add");
        assert_eq!(request.params, Some(serde_json::json!([1, 2])));
        assert_eq!(
            request.id,
            Some(serde_json::Value::String("req-1".to_string()))
        );
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{"jsonrpc":"2.0","method":"test","params":"#;
        let result = parse(json);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.jsonrpc, "2.0");
        assert!(error.error.is_some());
        assert_eq!(error.error.as_ref().unwrap().code, -32700);
        assert!(error.result.is_none());
        assert!(error.id.is_none());
    }

    #[test]
    fn test_parse_wrong_jsonrpc_version() {
        let json = r#"{"jsonrpc":"1.0","method":"test","id":1}"#;
        let result = parse(json);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.error.as_ref().unwrap().code, -32600);
        assert!(error.id.is_some());
    }

    #[test]
    fn test_parse_missing_method() {
        let json = r#"{"jsonrpc":"2.0","params":{},"id":1}"#;
        let result = parse(json);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.error.as_ref().unwrap().code, -32600);
        assert!(error.id.is_some());
    }

    #[test]
    fn test_parse_empty_method() {
        let json = r#"{"jsonrpc":"2.0","method":"","id":1}"#;
        let result = parse(json);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.error.as_ref().unwrap().code, -32600);
    }

    #[test]
    fn test_rpc_response_success() {
        let response = RpcResponse::success(
            Some(serde_json::Value::Number(1.into())),
            serde_json::json!("result"),
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.result, Some(serde_json::json!("result")));
        assert!(response.error.is_none());
        assert_eq!(response.id, Some(serde_json::Value::Number(1.into())));
    }

    #[test]
    fn test_rpc_response_error() {
        let response = RpcResponse::error(
            Some(serde_json::Value::Number(1.into())),
            -32601,
            "Method not found",
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        let err = response.error.as_ref().unwrap();
        assert_eq!(err.code, -32601);
        assert_eq!(err.message, "Method not found");
        assert_eq!(response.id, Some(serde_json::Value::Number(1.into())));
    }

    #[test]
    fn test_rpc_response_error_with_data() {
        let data = serde_json::json!({"foo": "bar"});
        let response = RpcResponse::error_with_data(
            Some(serde_json::Value::Number(1.into())),
            -32601,
            "Method not found",
            Some(data.clone()),
        );

        let err = response.error.as_ref().unwrap();
        assert_eq!(err.data, Some(data));
    }

    #[test]
    fn test_parse_request_with_only_required_fields() {
        // Only jsonrpc and method are required
        let json = r#"{"jsonrpc":"2.0","method":"test"}"#;
        let result = parse(json);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test");
        assert_eq!(request.params, None);
        assert_eq!(request.id, None);
    }

    #[test]
    fn test_parse_request_missing_jsonrpc() {
        let json = r#"{"method":"test","id":1}"#;
        let result = parse(json);

        assert!(result.is_err());
        let error = result.unwrap_err();
        // Missing jsonrpc means it deserializes as empty string, which fails validation
        assert_eq!(error.error.as_ref().unwrap().code, -32600);
    }

    #[test]
    fn test_parse_notification_request() {
        // Notification: request without id
        let json = r#"{"jsonrpc":"2.0","method":"test"}"#;
        let result = parse(json);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.id, None); // No id means it's a notification
    }

    #[test]
    fn test_serialize_rpc_response_success() {
        let response = RpcResponse::success(
            Some(serde_json::Value::Number(1.into())),
            serde_json::json!({"data": "value"}),
        );
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_serialize_rpc_response_error() {
        let response = RpcResponse::error(
            Some(serde_json::Value::Number(1.into())),
            -32601,
            "Not found",
        );
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"error\""));
        assert!(json.contains("\"code\":-32601"));
        assert!(json.contains("\"message\":\"Not found\""));
        assert!(!json.contains("\"result\""));
    }
}
