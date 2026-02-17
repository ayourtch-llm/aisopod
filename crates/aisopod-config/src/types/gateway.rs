use serde::{Deserialize, Serialize};

/// Gateway configuration for HTTP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// HTTP server settings
    #[serde(default)]
    pub server: ServerConfig,
    /// Bind address configuration
    #[serde(default)]
    pub bind: BindConfig,
    /// TLS settings
    #[serde(default)]
    pub tls: TlsConfig,
    /// Web UI configuration for static file serving
    #[serde(default)]
    pub web_ui: WebUiConfig,
    /// WebSocket handshake timeout in seconds
    #[serde(default = "default_handshake_timeout")]
    pub handshake_timeout: u64,
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name
    #[serde(default)]
    pub name: String,
    /// Port to bind to
    #[serde(default = "default_port")]
    pub port: u16,
    /// Enable graceful shutdown
    #[serde(default)]
    pub graceful_shutdown: bool,
}

/// Bind address configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindConfig {
    /// IP address to bind to
    #[serde(default)]
    pub address: String,
    /// Enable IPv6
    #[serde(default)]
    pub ipv6: bool,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS
    #[serde(default)]
    pub enabled: bool,
    /// Certificate file path
    #[serde(default)]
    pub cert_path: String,
    /// Private key file path
    #[serde(default)]
    pub key_path: String,
}

fn default_port() -> u16 {
    8080
}

fn default_handshake_timeout() -> u64 {
    5
}

/// Web UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebUiConfig {
    /// Enable static file serving for the web UI
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Directory path containing the built web UI assets
    #[serde(default = "default_dist_path")]
    pub dist_path: String,
    /// Allowed origins for CORS headers
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,
}

fn default_enabled() -> bool {
    true
}

fn default_dist_path() -> String {
    "../web-ui/dist".to_string()
}

fn default_cors_origins() -> Vec<String> {
    vec!["http://localhost:8080".to_string(), "http://localhost:5173".to_string()]
}

impl Default for WebUiConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            dist_path: default_dist_path(),
            cors_origins: default_cors_origins(),
        }
    }
}

/// Rate limiting configuration for the gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the window
    #[serde(default = "default_max_requests")]
    pub max_requests: u64,
    /// Sliding window duration in seconds
    #[serde(default = "default_window")]
    pub window: u64,
}

fn default_max_requests() -> u64 {
    100
}

fn default_window() -> u64 {
    60
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: default_max_requests(),
            window: default_window(),
        }
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            bind: BindConfig::default(),
            tls: TlsConfig::default(),
            web_ui: WebUiConfig::default(),
            handshake_timeout: default_handshake_timeout(),
            rate_limit: RateLimitConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: String::from("aisopod-gateway"),
            port: default_port(),
            graceful_shutdown: true,
        }
    }
}

impl Default for BindConfig {
    fn default() -> Self {
        Self {
            address: String::from("0.0.0.0"),
            ipv6: false,
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: String::new(),
            key_path: String::new(),
        }
    }
}
