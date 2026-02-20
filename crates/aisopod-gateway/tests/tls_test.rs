//! Integration tests for TLS/HTTPS support

use std::fs;
use std::path::Path;
use tracing_test::traced_test;

// Import the gateway modules
use aisopod_gateway::tls::{is_tls_enabled, load_tls_config};

fn generate_self_signed_cert(
    key_path: &Path,
    cert_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate a self-signed certificate using openssl
    // This is a simple approach that works on most systems with openssl installed
    let _ = fs::remove_file(key_path);
    let _ = fs::remove_file(cert_path);

    // Generate private key and certificate in one command
    std::process::Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-keyout",
            key_path.to_str().unwrap(),
            "-out",
            cert_path.to_str().unwrap(),
            "-days",
            "1",
            "-nodes",
            "-subj",
            "/C=US/ST=Test/L=Test/O=Test/OU=Test/CN=localhost",
        ])
        .output()?;

    Ok(())
}

#[tokio::test]
async fn test_tls_enabled_check() {
    // Test with empty paths
    assert!(!is_tls_enabled("", ""));

    // Test with empty cert path
    assert!(!is_tls_enabled("", "key.pem"));

    // Test with empty key path
    assert!(!is_tls_enabled("cert.pem", ""));

    // Test with both paths
    assert!(is_tls_enabled("cert.pem", "key.pem"));
}

#[tokio::test]
#[traced_test]
async fn test_tls_config_loading() {
    // Create a temporary directory for test certificates
    let temp_dir = std::env::temp_dir();
    let key_path = temp_dir.join("test_key.pem");
    let cert_path = temp_dir.join("test_cert.pem");

    // Generate self-signed certificate
    generate_self_signed_cert(&key_path, &cert_path)
        .expect("Failed to generate self-signed certificate");

    // Ensure files exist
    assert!(key_path.exists(), "Key file was not created");
    assert!(cert_path.exists(), "Cert file was not created");

    // Load the TLS config to verify it works
    let tls_config = load_tls_config(&cert_path, &key_path)
        .await
        .expect("Failed to load TLS config");

    // Verify we got a valid config (just check it's not empty)
    // The RustlsConfig wraps an internal config, so we just verify loading worked

    // Clean up
    let _ = fs::remove_file(&key_path);
    let _ = fs::remove_file(&cert_path);
}
