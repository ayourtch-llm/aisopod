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

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            bind: BindConfig::default(),
            tls: TlsConfig::default(),
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
