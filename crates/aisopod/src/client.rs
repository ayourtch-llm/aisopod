//! Gateway client for CLI-to-gateway WebSocket communication
//!
//! This module provides a WebSocket client for sending JSON-RPC requests to the gateway
//! and handling responses, including streaming responses. It also manages authentication
//! tokens with secure file storage.

use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::path::PathBuf;

/// Gateway client for WebSocket communication with the gateway server
pub struct GatewayClient {
    url: String,
    auth_token: Option<String>,
    request_id: AtomicU64,
}

impl GatewayClient {
    /// Create a new gateway client with the given URL and optional auth token
    pub fn new(url: &str, auth_token: Option<String>) -> Self {
        Self {
            url: url.to_string(),
            auth_token,
            request_id: AtomicU64::new(1),
        }
    }

    /// Create a gateway client from configuration
    pub fn from_config(config: &aisopod_config::AisopodConfig) -> anyhow::Result<Self> {
        let url = Self::gateway_ws_url(config);
        let token = Self::load_auth_token()?;
        Ok(Self::new(&url, token))
    }

    /// Get the WebSocket URL from the gateway configuration
    pub fn gateway_ws_url(config: &aisopod_config::AisopodConfig) -> String {
        let gateway = &config.gateway;
        let bind_addr = &gateway.bind.address;
        let port = gateway.server.port;
        
        // Determine protocol based on TLS config
        let protocol = if gateway.tls.enabled { "wss" } else { "ws" };
        
        format!("{}://{}:{}", protocol, bind_addr, port)
    }

    /// Load the stored auth token from the user's home directory
    pub fn load_auth_token() -> anyhow::Result<Option<String>> {
        let path = token_path()?;
        if path.exists() {
            let token = std::fs::read_to_string(&path)?
                .trim()
                .to_string();
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    /// Send a JSON-RPC request and return the response
    pub async fn call(&self, method: &str, params: Value) -> anyhow::Result<Value> {
        let (mut ws_stream, _) = connect_async(&self.url).await?;

        // Send auth token if available
        if let Some(ref token) = self.auth_token {
            let auth_msg = json!({
                "jsonrpc": "2.0",
                "method": "auth.authenticate",
                "params": { "token": token },
                "id": 0
            });
            ws_stream.send(Message::Text(auth_msg.to_string())).await?;
            
            // Wait for and validate auth response
            if let Some(Ok(Message::Text(resp))) = ws_stream.next().await {
                let resp_str = resp.to_string();
                let resp: Value = serde_json::from_str(&resp_str)?;
                if let Some(error) = resp.get("error") {
                    let message = error.get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Authentication failed");
                    anyhow::bail!("{}", message);
                }
            }
        }

        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        });

        ws_stream.send(Message::Text(request.to_string())).await?;

        // Wait for response
        while let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let text_str = text.to_string();
                let response: Value = serde_json::from_str(&text_str)?;
                if response.get("id") == Some(&json!(id)) {
                    ws_stream.close(None).await?;
                    return Ok(response);
                }
            }
        }

        anyhow::bail!("Connection closed before receiving response")
    }

    /// Send a JSON-RPC request and stream response chunks via callback
    pub async fn call_streaming<F>(
        &self,
        method: &str,
        params: Value,
        mut on_chunk: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(Value) -> bool,
    {
        let (mut ws_stream, _) = connect_async(&self.url).await?;

        // Auth
        if let Some(ref token) = self.auth_token {
            let auth_msg = json!({
                "jsonrpc": "2.0",
                "method": "auth.authenticate",
                "params": { "token": token },
                "id": 0
            });
            ws_stream.send(Message::Text(auth_msg.to_string())).await?;
            
            // Consume auth response (we don't validate here for streaming)
            let _ = ws_stream.next().await;
        }

        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        });

        ws_stream.send(Message::Text(request.to_string())).await?;

        while let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let text_str = text.to_string();
                let value: Value = serde_json::from_str(&text_str)?;
                let should_continue = on_chunk(value);
                if !should_continue {
                    break;
                }
            }
        }

        ws_stream.close(None).await?;
        Ok(())
    }
}

// ============================================================================
// Auth Token Management
// ============================================================================

const TOKEN_FILE: &str = ".aisopod_token";

/// Store an auth token with restrictive file permissions
///
/// The token is saved to the user's home directory with mode 0600 (owner read/write only).
pub fn store_token(token: &str) -> anyhow::Result<()> {
    let path = token_path()?;
    std::fs::write(&path, token)?;
    
    // Set restrictive permissions (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    
    Ok(())
}

/// Load the stored auth token from the user's home directory
pub fn load_token() -> anyhow::Result<Option<String>> {
    let path = token_path()?;
    if path.exists() {
        let token = std::fs::read_to_string(&path)?
            .trim()
            .to_string();
        Ok(Some(token))
    } else {
        Ok(None)
    }
}

/// Clear the stored auth token
pub fn clear_token() -> anyhow::Result<()> {
    let path = token_path()?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Get the path to the auth token file
pub fn token_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    Ok(home.join(TOKEN_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;

    #[test]
    fn test_gateway_ws_url_http() {
        let config = aisopod_config::AisopodConfig::default();
        let url = GatewayClient::gateway_ws_url(&config);
        assert!(url.starts_with("ws://"));
        assert!(url.contains(":8080")); // default port
    }

    #[test]
    fn test_gateway_ws_url_https() {
        let mut config = aisopod_config::AisopodConfig::default();
        config.gateway.tls.enabled = true;
        let url = GatewayClient::gateway_ws_url(&config);
        assert!(url.starts_with("wss://"));
        assert!(url.contains(":8080"));
    }

    #[test]
    fn test_gateway_ws_url_custom_port() {
        let mut config = aisopod_config::AisopodConfig::default();
        config.gateway.server.port = 9999;
        let url = GatewayClient::gateway_ws_url(&config);
        assert!(url.ends_with(":9999"));
    }

    #[test]
    fn test_gateway_ws_url_custom_bind() {
        let mut config = aisopod_config::AisopodConfig::default();
        config.gateway.bind.address = "0.0.0.0".to_string();
        let url = GatewayClient::gateway_ws_url(&config);
        assert!(url.contains("0.0.0.0"));
    }

    #[test]
    fn test_store_and_load_token() {
        // Use a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let token_file = temp_dir.path().join(TOKEN_FILE);
        
        // Save original home dir
        let original_home = env::var_os("HOME");
        
        // Set temp dir as home
        env::set_var("HOME", temp_dir.path());
        
        // Store token
        let result = store_token("test-token-123");
        assert!(result.is_ok());
        assert!(token_file.exists());
        
        // Verify permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&token_file).expect("Failed to get metadata");
            let mode = metadata.permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        
        // Load token
        let loaded = load_token().expect("Failed to load token");
        assert_eq!(loaded, Some("test-token-123".to_string()));
        
        // Restore home dir
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        }
    }

    #[test]
    fn test_clear_token() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let token_file = temp_dir.path().join(TOKEN_FILE);
        
        // Create token file
        std::fs::write(&token_file, "test-token").expect("Failed to write token");
        assert!(token_file.exists());
        
        // Save original home dir
        let original_home = env::var_os("HOME");
        env::set_var("HOME", temp_dir.path());
        
        // Clear token
        let result = clear_token();
        assert!(result.is_ok());
        assert!(!token_file.exists());
        
        // Verify load returns None
        let loaded = load_token().expect("Failed to load token");
        assert_eq!(loaded, None);
        
        // Restore home dir
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        }
    }

    #[test]
    fn test_load_token_nonexistent() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        let original_home = env::var_os("HOME");
        env::set_var("HOME", temp_dir.path());
        
        let loaded = load_token().expect("Failed to load token");
        assert_eq!(loaded, None);
        
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        }
    }

    #[test]
    fn test_token_path() {
        let path = token_path().expect("Failed to get token path");
        
        // Check that it ends with the token file name
        assert!(path.file_name().is_some());
        assert_eq!(path.file_name().unwrap(), TOKEN_FILE);
    }
}
