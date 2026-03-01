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
    /// Request size limits for security
    #[serde(default)]
    pub request_size_limits: RequestSizeLimitsConfig,
    /// Pairing cleanup interval in seconds
    #[serde(default = "default_pairing_cleanup_interval")]
    pub pairing_cleanup_interval: u64,
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
            request_size_limits: RequestSizeLimitsConfig::default(),
            pairing_cleanup_interval: default_pairing_cleanup_interval(),
        }
    }
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

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            port: default_port(),
            graceful_shutdown: false,
        }
    }
}

/// Bind address configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindConfig {
    /// IP address to bind to
    #[serde(default = "default_bind_address")]
    pub address: String,
    /// Enable IPv6
    #[serde(default)]
    pub ipv6: bool,
}

impl Default for BindConfig {
    fn default() -> Self {
        Self {
            address: default_bind_address(),
            ipv6: false,
        }
    }
}

fn default_bind_address() -> String {
    "127.0.0.1".to_string()
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl Default for WebUiConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            dist_path: default_dist_path(),
            cors_origins: default_cors_origins(),
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_dist_path() -> String {
    "../web-ui/dist".to_string()
}

fn default_cors_origins() -> Vec<String> {
    vec![
        "http://localhost:8080".to_string(),
        "http://localhost:5173".to_string(),
    ]
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

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: default_max_requests(),
            window: default_window(),
        }
    }
}

fn default_max_requests() -> u64 {
    100
}

fn default_window() -> u64 {
    60
}

/// Request size limits configuration for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSizeLimitsConfig {
    /// Maximum size of request body in bytes (default: 10MB)
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
    /// Maximum size of headers in bytes (default: 8KB)
    #[serde(default = "default_max_headers_size")]
    pub max_headers_size: usize,
    /// Maximum number of headers (default: 100)
    #[serde(default = "default_max_headers_count")]
    pub max_headers_count: usize,
}

impl Default for RequestSizeLimitsConfig {
    fn default() -> Self {
        Self {
            max_body_size: default_max_body_size(),
            max_headers_size: default_max_headers_size(),
            max_headers_count: default_max_headers_count(),
        }
    }
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_max_headers_size() -> usize {
    8192 // 8KB
}

fn default_max_headers_count() -> usize {
    100
}

fn default_pairing_cleanup_interval() -> u64 {
    300 // 5 minutes
}
