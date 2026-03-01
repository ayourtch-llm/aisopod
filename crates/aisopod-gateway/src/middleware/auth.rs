#![allow(clippy::all)]
//! Authentication middleware for Axum
//!
//! This module provides an Axum middleware that validates incoming requests
//! and attaches AuthInfo to the request extensions on success.

use aisopod_config::types::AuthConfig;
use axum::{
    extract::ConnectInfo,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::{debug, warn};

use crate::audit::{log_auth_failure, log_auth_success};
use crate::auth::{build_password_map, build_token_map, validate_basic, validate_token, AuthInfo};
use crate::auth::{hash_password, verify_password, TokenStore};
use aisopod_config::sensitive::Sensitive;

/// Request extension key for AuthInfo
pub const AUTH_INFO_KEY: &str = "aisopod.auth.info";

/// Request extension key for client IP address
const CLIENT_IP_KEY: &str = "aisopod.client_ip";

/// Extract client IP from request extensions or use localhost as fallback
fn get_client_ip(req: &axum::extract::Request) -> String {
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.to_string())
        .unwrap_or_else(|| "127.0.0.1:0".to_string())
}

/// Configuration for the auth middleware
#[derive(Debug, Clone)]
pub struct AuthConfigData {
    /// The auth configuration
    config: AuthConfig,
    /// Pre-computed token lookup map
    token_map: HashMap<String, AuthInfo>,
    /// Token store for rotation support
    token_store: Option<TokenStore>,
    /// Pre-computed password lookup map (username -> password -> auth_info)
    password_map: HashMap<String, HashMap<String, AuthInfo>>,
    /// Flag indicating if passwords are hashed
    passwords_hashed: bool,
}

impl AuthConfigData {
    /// Create a new auth config data from the auth config
    pub fn new(config: AuthConfig) -> Self {
        let token_map = build_token_map(&config);
        let password_map = build_password_map(&config);

        // Check if any password looks like a hash (starts with $argon2)
        let passwords_hashed = config
            .passwords
            .iter()
            .any(|cred| cred.password.expose().starts_with("$argon2"));

        // Create token store if using token auth
        let token_store = if config.gateway_mode == aisopod_config::types::AuthMode::Token {
            config
                .tokens
                .first()
                .map(|cred| TokenStore::new(cred.token.clone()))
        } else {
            None
        };

        Self {
            config,
            token_map,
            token_store,
            password_map,
            passwords_hashed,
        }
    }

    /// Get the auth mode
    pub fn mode(&self) -> &aisopod_config::types::AuthMode {
        &self.config.gateway_mode
    }

    /// Validate a token and return AuthInfo if valid
    pub fn validate_token(&self, token: &str) -> Option<AuthInfo> {
        if self.config.gateway_mode != aisopod_config::types::AuthMode::Token {
            return None;
        }

        // Check token store first (for rotation support)
        if let Some(ref store) = self.token_store {
            if store.validate(token) {
                // Find the token credential to get role and scopes
                return self.config.tokens.iter().find_map(|cred| {
                    if cred.token == token {
                        Some(AuthInfo {
                            role: cred.role.clone(),
                            scopes: cred.scopes.clone(),
                        })
                    } else {
                        None
                    }
                });
            }
        }

        // Fallback to token map for direct lookup
        self.token_map.get(token).cloned()
    }

    /// Validate basic auth credentials and return AuthInfo if valid
    pub fn validate_basic(&self, username: &str, password: &str) -> Option<AuthInfo> {
        if self.config.gateway_mode != aisopod_config::types::AuthMode::Password {
            return None;
        }

        if self.passwords_hashed {
            // Use hashed password verification
            self.config.passwords.iter().find_map(|cred| {
                if cred.username == username {
                    match verify_password(password, &cred.password.expose()) {
                        Ok(true) => Some(AuthInfo {
                            role: cred.role.clone(),
                            scopes: cred.scopes.clone(),
                        }),
                        Ok(false) | Err(_) => None,
                    }
                } else {
                    None
                }
            })
        } else {
            // Use plain password lookup
            if let Some(passwords) = self.password_map.get(username) {
                passwords.get(password).cloned()
            } else {
                None
            }
        }
    }
}

/// Extract Authorization header value
fn extract_authorization(header_map: &HeaderMap) -> Option<String> {
    header_map
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Parse Bearer token from Authorization header
fn parse_bearer_token(auth_value: &str) -> Option<String> {
    let parts: Vec<&str> = auth_value.splitn(2, ' ').collect();
    if parts.len() == 2 && parts[0].eq_ignore_ascii_case("bearer") {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Parse Basic auth credentials from Authorization header
fn parse_basic_credentials(auth_value: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = auth_value.splitn(2, ' ').collect();
    if parts.len() == 2 && parts[0].eq_ignore_ascii_case("basic") {
        // Base64 decode the credentials
        if let Ok(decoded) = base64::decode(parts[1]) {
            if let Ok(decoded_str) = String::from_utf8(decoded) {
                let mut parts = decoded_str.splitn(2, ':');
                if let (Some(username), Some(password)) = (parts.next(), parts.next()) {
                    return Some((username.to_string(), password.to_string()));
                }
            }
        }
    }
    None
}

/// Error response for unauthorized requests
fn unauthorized_response(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(serde_json::json!({
            "error": "unauthorized",
            "message": message
        })),
    )
        .into_response()
}

/// Error response for unauthorized requests in JSON-RPC format
fn unauthorized_rpc_response(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32603,
                "message": message
            },
            "id": None::<serde_json::Value>
        })),
    )
        .into_response()
}

/// Check if the request is for an RPC endpoint
fn is_rpc_request(request: &axum::extract::Request) -> bool {
    request.uri().path() == "/rpc"
}

/// Authentication middleware
///
/// This middleware validates incoming requests based on the configured auth mode:
/// - **token**: Validates `Authorization: Bearer <token>`
/// - **password**: Validates `Authorization: Basic <base64(username:password)>`
/// - **none**: Allows all requests through without validation
///
/// On successful authentication, the AuthInfo is stored in request extensions.
/// The /health endpoint is always accessible without authentication.
pub async fn auth_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    eprintln!("=== AUTH MIDDLEWARE CALLED ===");

    let mut request = request;

    // Always allow health checks
    let path = request.uri().path();
    if path == "/health" {
        eprintln!("Auth: /health endpoint, allowing through");
        return next.run(request).await;
    }

    let config_data = request.extensions().get::<AuthConfigData>().cloned();

    eprintln!("AuthConfigData in extensions: {:?}", config_data.is_some());

    // If no config is available, allow the request through
    // (this might happen in tests or unusual configurations)
    let config_data = match config_data {
        Some(config) => config,
        None => {
            eprintln!("No auth config found, allowing request through");
            return next.run(request).await;
        }
    };

    // Get the auth mode
    let mode = config_data.mode();
    eprintln!("Auth mode: {:?}", mode);

    // Match on auth mode
    match mode {
        aisopod_config::types::AuthMode::None => {
            // No auth required, just allow through
            eprintln!("Auth mode is none, allowing request");
            return next.run(request).await;
        }

        aisopod_config::types::AuthMode::Token => {
            eprintln!("Token auth mode - checking authorization");
            // Extract and validate Bearer token
            let header_map = request.headers();
            let auth_value = match extract_authorization(header_map) {
                Some(v) => v,
                None => {
                    let client_ip = get_client_ip(&request);
                    log_auth_failure(&client_ip, "token", "missing authorization header");
                    eprintln!("Missing Authorization header");
                    return if is_rpc_request(&request) {
                        unauthorized_rpc_response("Missing Authorization header")
                    } else {
                        unauthorized_response("Missing Authorization header")
                    };
                }
            };
            eprintln!("Auth value: {:?}", auth_value);

            let token = match parse_bearer_token(&auth_value) {
                Some(t) => t,
                None => {
                    let client_ip = get_client_ip(&request);
                    log_auth_failure(&client_ip, "token", "invalid authorization header format");
                    eprintln!("Invalid Authorization header format");
                    return if is_rpc_request(&request) {
                        unauthorized_rpc_response("Invalid Authorization header format")
                    } else {
                        unauthorized_response("Invalid Authorization header format")
                    };
                }
            };
            eprintln!("Token: {:?}", token);

            match config_data.validate_token(&token) {
                Some(auth_info) => {
                    let client_ip = get_client_ip(&request);
                    log_auth_success(&client_ip, "token", &auth_info.role);
                    eprintln!("Token validation successful for role: {}", auth_info.role);
                    request.extensions_mut().insert(auth_info);
                    next.run(request).await
                }
                None => {
                    let client_ip = get_client_ip(&request);
                    log_auth_failure(&client_ip, "token", "invalid token");
                    eprintln!("Invalid token provided, returning 401");
                    if is_rpc_request(&request) {
                        unauthorized_rpc_response("Invalid token")
                    } else {
                        unauthorized_response("Invalid token")
                    }
                }
            }
        }

        aisopod_config::types::AuthMode::Password => {
            // Extract and validate Basic auth credentials
            let header_map = request.headers();
            let auth_value = match extract_authorization(header_map) {
                Some(v) => v,
                None => {
                    let client_ip = get_client_ip(&request);
                    warn!("Missing Authorization header for password auth");
                    log_auth_failure(&client_ip, "password", "missing authorization header");
                    return if is_rpc_request(&request) {
                        unauthorized_rpc_response("Missing Authorization header")
                    } else {
                        unauthorized_response("Missing Authorization header")
                    };
                }
            };

            let (username, password) = match parse_basic_credentials(&auth_value) {
                Some((u, p)) => (u, p),
                None => {
                    let client_ip = get_client_ip(&request);
                    warn!("Invalid Authorization header format for Basic auth");
                    log_auth_failure(
                        &client_ip,
                        "password",
                        "invalid authorization header format",
                    );
                    return if is_rpc_request(&request) {
                        unauthorized_rpc_response("Invalid Authorization header format")
                    } else {
                        unauthorized_response("Invalid Authorization header format")
                    };
                }
            };

            match config_data.validate_basic(&username, &password) {
                Some(auth_info) => {
                    let client_ip = get_client_ip(&request);
                    debug!(
                        "Basic auth validation successful for user: {}, role: {}",
                        username, auth_info.role
                    );
                    log_auth_success(&client_ip, "password", &auth_info.role);
                    request.extensions_mut().insert(auth_info);
                    next.run(request).await
                }
                None => {
                    let client_ip = get_client_ip(&request);
                    warn!("Invalid credentials provided for user: {}", username);
                    log_auth_failure(
                        &client_ip,
                        "password",
                        &format!("invalid credentials for user {}", username),
                    );
                    if is_rpc_request(&request) {
                        unauthorized_rpc_response("Invalid credentials")
                    } else {
                        unauthorized_response("Invalid credentials")
                    }
                }
            }
        }
    }
}

/// Extension trait for extracting AuthInfo from request
pub trait ExtractAuthInfo {
    /// Extract AuthInfo from request extensions, or return None if not authenticated
    fn extract_auth_info(&self) -> Option<AuthInfo>;
}

impl ExtractAuthInfo for axum::extract::Request {
    fn extract_auth_info(&self) -> Option<AuthInfo> {
        self.extensions().get::<AuthInfo>().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request as AxumRequest;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    async fn echo_auth_info(request: AxumRequest<Body>) -> impl IntoResponse {
        let auth_info = request.extensions().get::<AuthInfo>().cloned();
        axum::Json(serde_json::json!({
            "authenticated": auth_info.is_some(),
            "role": auth_info.as_ref().map(|a| a.role.clone()),
            "scopes": auth_info.as_ref().map(|a| a.scopes.clone())
        }))
    }

    fn create_test_router_with_middleware(config: AuthConfig) -> Router {
        let config_data = AuthConfigData::new(config.clone());

        Router::new()
            .route("/test", get(echo_auth_info))
            .layer(axum::middleware::from_fn(auth_middleware))
            .layer(axum::middleware::from_fn(
                move |mut req: AxumRequest<Body>, next: axum::middleware::Next| {
                    let config_data = config_data.clone();
                    async move {
                        // Inject config_data into request AFTER auth_middleware so it's available
                        req.extensions_mut().insert(config_data);
                        next.run(req).await
                    }
                },
            ))
    }

    #[tokio::test]
    async fn test_auth_middleware_token_success() {
        let config = AuthConfig {
            gateway_mode: aisopod_config::types::AuthMode::Token,
            tokens: vec![aisopod_config::types::TokenCredential {
                token: "test-token".to_string(),
                role: "operator".to_string(),
                scopes: vec!["chat:write".to_string()],
            }],
            ..Default::default()
        };

        let router = create_test_router_with_middleware(config);

        let request = AxumRequest::builder()
            .uri("/test")
            .header(axum::http::header::AUTHORIZATION, "Bearer test-token")
            .body(Body::empty())
            .expect("test should pass");

        let response = router.oneshot(request).await.expect("test should pass");
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("test should pass");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("test should pass");
        assert_eq!(json["authenticated"], true);
        assert_eq!(json["role"], "operator");
        assert_eq!(
            json["scopes"],
            serde_json::json!(vec!["chat:write".to_string()])
        );
    }

    #[tokio::test]
    async fn test_auth_middleware_token_missing() {
        let config = AuthConfig {
            gateway_mode: aisopod_config::types::AuthMode::Token,
            tokens: vec![aisopod_config::types::TokenCredential {
                token: "test-token".to_string(),
                role: "operator".to_string(),
                scopes: vec!["chat:write".to_string()],
            }],
            ..Default::default()
        };

        let router = create_test_router_with_middleware(config);

        let request = AxumRequest::builder()
            .uri("/test")
            .body(Body::empty())
            .expect("test should pass");

        let response = router.oneshot(request).await.expect("test should pass");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_token_invalid() {
        let config = AuthConfig {
            gateway_mode: aisopod_config::types::AuthMode::Token,
            tokens: vec![aisopod_config::types::TokenCredential {
                token: "test-token".to_string(),
                role: "operator".to_string(),
                scopes: vec!["chat:write".to_string()],
            }],
            ..Default::default()
        };

        let router = create_test_router_with_middleware(config);

        let request = AxumRequest::builder()
            .uri("/test")
            .header(axum::http::header::AUTHORIZATION, "Bearer invalid-token")
            .body(Body::empty())
            .expect("test should pass");

        let response = router.oneshot(request).await.expect("test should pass");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_password_success() {
        let config = AuthConfig {
            gateway_mode: aisopod_config::types::AuthMode::Password,
            passwords: vec![aisopod_config::types::PasswordCredential {
                username: "admin".to_string(),
                password: Sensitive::new("password123".to_string()),
                role: "operator".to_string(),
                scopes: vec!["chat:write".to_string()],
            }],
            ..Default::default()
        };

        let router = create_test_router_with_middleware(config);

        let creds = base64::encode("admin:password123");
        let request = AxumRequest::builder()
            .uri("/test")
            .header(
                axum::http::header::AUTHORIZATION,
                format!("Basic {}", creds),
            )
            .body(Body::empty())
            .expect("test should pass");

        let response = router.oneshot(request).await.expect("test should pass");
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("test should pass");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("test should pass");
        assert_eq!(json["authenticated"], true);
        assert_eq!(json["role"], "operator");
    }

    #[tokio::test]
    async fn test_auth_middleware_password_invalid() {
        let config = AuthConfig {
            gateway_mode: aisopod_config::types::AuthMode::Password,
            passwords: vec![aisopod_config::types::PasswordCredential {
                username: "admin".to_string(),
                password: Sensitive::new("password123".to_string()),
                role: "operator".to_string(),
                scopes: vec!["chat:write".to_string()],
            }],
            ..Default::default()
        };

        let router = create_test_router_with_middleware(config);

        let creds = base64::encode("admin:wrongpassword");
        let request = AxumRequest::builder()
            .uri("/test")
            .header(
                axum::http::header::AUTHORIZATION,
                format!("Basic {}", creds),
            )
            .body(Body::empty())
            .expect("test should pass");

        let response = router.oneshot(request).await.expect("test should pass");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_none() {
        let config = AuthConfig {
            gateway_mode: aisopod_config::types::AuthMode::None,
            ..Default::default()
        };

        let router = create_test_router_with_middleware(config);

        let request = AxumRequest::builder()
            .uri("/test")
            .body(Body::empty())
            .expect("test should pass");

        let response = router.oneshot(request).await.expect("test should pass");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_endpoint_always_allowed() {
        let config = AuthConfig {
            gateway_mode: aisopod_config::types::AuthMode::Token,
            tokens: vec![aisopod_config::types::TokenCredential {
                token: "test-token".to_string(),
                role: "operator".to_string(),
                scopes: vec!["chat:write".to_string()],
            }],
            ..Default::default()
        };

        let router = create_test_router_with_middleware(config);

        let request = AxumRequest::builder()
            .uri("/health")
            .body(Body::empty())
            .expect("test should pass");

        let response = router.oneshot(request).await.expect("test should pass");
        // The health endpoint should not be accessible through this test router
        // but the middleware should allow it through
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
