use anyhow::{anyhow, Result};
use axum_server::tls_rustls::RustlsConfig;
use std::path::Path;

// Initialize Rustls CryptoProvider to avoid runtime errors
// This must be called before any TLS operations
fn init_rustls_provider() {
    // Attempt to install the ring provider if not already installed
    // This is a no-op if already installed
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Load TLS configuration from certificate and key files
pub async fn load_tls_config(cert_path: &Path, key_path: &Path) -> Result<RustlsConfig> {
    // Ensure the provider is initialized
    init_rustls_provider();
    
    RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .map_err(|e| anyhow!("Failed to load TLS configuration from files: {}", e))
}

/// Check if TLS is enabled based on cert and key paths
pub fn is_tls_enabled(cert_path: &str, key_path: &str) -> bool {
    !cert_path.is_empty() && !key_path.is_empty()
}
