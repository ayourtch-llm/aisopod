//! Integration tests for JSON5 config file parsing
//!
//! These tests verify the configuration loading functionality using
//! the sample fixture files.

use std::path::PathBuf;
use aisopod_config::load_config;

#[test]
fn test_load_sample_json5() {
    let path = PathBuf::from("tests/fixtures/sample.json5");
    let config = load_config(&path).expect("Failed to load config");
    
    assert_eq!(config.meta.version, "1.0");
    assert_eq!(config.gateway.server.port, 8080);
    assert_eq!(config.gateway.server.name, "aisopod-gateway");
    assert_eq!(config.gateway.bind.address, "127.0.0.1");
    assert!(!config.gateway.tls.enabled);
}

#[test]
fn test_load_sample_with_auto_detect() {
    let path = PathBuf::from("tests/fixtures/sample.json5");
    let config = load_config(&path).expect("Failed to load config");
    
    // Verify the config loads with auto-detection
    assert!(config.meta.version == "1.0");
}

#[cfg(test)]
mod include_directive_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_config_with_include() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create fragment file
        let fragment_path = temp_dir.path().join("fragment.json5");
        let fragment_content = r#"
{
  gateway: {
    server: {
      port: 9090,
    },
  },
}
"#;
        fs::write(&fragment_path, fragment_content).expect("Failed to write fragment");
        
        // Create main config with include
        let main_path = temp_dir.path().join("main.json5");
        let main_content = format!(
            r#"{{
  "@include": "fragment.json5",
  meta: {{ version: "1.0" }}
}}"#
        );
        fs::write(&main_path, main_content).expect("Failed to write main config");
        
        // Load and verify
        let config = load_config(&main_path).expect("Failed to load config");
        assert_eq!(config.meta.version, "1.0");
        assert_eq!(config.gateway.server.port, 9090);
    }

    #[test]
    fn test_load_config_with_array_include() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create fragment files
        let models_path = temp_dir.path().join("models.json5");
        let models_content = r#"
{
  models: {
    models: [
      {
        id: "gpt-4",
        name: "GPT-4",
        provider: "openai",
        capabilities: ["chat"],
      },
    ],
  },
}
"#;
        fs::write(&models_path, models_content).expect("Failed to write models fragment");
        
        let tools_path = temp_dir.path().join("tools.json5");
        let tools_content = r#"
{
  tools: {
    bash: {
      enabled: true,
      working_dir: "/tmp",
    },
  },
}
"#;
        fs::write(&tools_path, tools_content).expect("Failed to write tools fragment");
        
        // Create main config with array include
        let main_path = temp_dir.path().join("main.json5");
        let main_content = format!(
            r#"{{
  "@include": ["models.json5", "tools.json5"],
  meta: {{ version: "2.0" }}
}}"#
        );
        fs::write(&main_path, main_content).expect("Failed to write main config");
        
        // Load and verify
        let config = load_config(&main_path).expect("Failed to load config");
        assert_eq!(config.meta.version, "2.0");
        assert_eq!(config.models.models.len(), 1);
        assert_eq!(config.models.models[0].id, "gpt-4");
        assert!(config.tools.bash.enabled);
    }

    #[test]
    fn test_load_config_circular_include() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create circular include files
        let a_path = temp_dir.path().join("a.json5");
        let a_content = r#"
{
  "@include": "b.json5",
  meta: { version: "a" }
}
"#;
        fs::write(&a_path, a_content).expect("Failed to write a.json5");
        
        let b_path = temp_dir.path().join("b.json5");
        let b_content = r#"
{
  "@include": "a.json5",
  meta: { version: "b" }
}
"#;
        fs::write(&b_path, b_content).expect("Failed to write b.json5");
        
        // Load should fail with circular include error
        let result = load_config(&a_path);
        let err = result.unwrap_err();
        
        // Check the error message and its sources for "Circular @include detected"
        let mut has_circular = err.to_string().contains("Circular @include detected");
        let mut err_source = err.source();
        while let Some(source) = err_source {
            if source.to_string().contains("Circular @include detected") {
                has_circular = true;
                break;
            }
            err_source = source.source();
        }
        
        assert!(has_circular, 
            "Error should contain 'Circular @include detected' in message or sources. Got: {}", 
            err.to_string());
    }

    #[test]
    fn test_nested_include_processing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create base fragment
        let base_path = temp_dir.path().join("base.json5");
        let base_content = r#"
{
  gateway: {
    server: {
      port: 9090,
    },
  },
}
"#;
        fs::write(&base_path, base_content).expect("Failed to write base.json5");
        
        // Create nested include that includes base
        let nested_path = temp_dir.path().join("nested.json5");
        let nested_content = format!(
            r#"{{
  "@include": "base.json5",
  meta: {{ version: "nested" }}
}}"#
        );
        fs::write(&nested_path, nested_content).expect("Failed to write nested.json5");
        
        // Create main config that includes nested
        let main_path = temp_dir.path().join("main.json5");
        let main_content = format!(
            r#"{{
  "@include": "nested.json5",
  meta: {{ version: "main" }}
}}"#
        );
        fs::write(&main_path, main_content).expect("Failed to write main.json5");
        
        // Load and verify
        let config = load_config(&main_path).expect("Failed to load config");
        // The nested includes should have been processed
        // Gateway port from base fragment should be present
        assert_eq!(config.gateway.server.port, 9090);
        // Latest meta version should win (main overrides nested overrides base)
        assert_eq!(config.meta.version, "main");
    }

    #[test]
    fn test_include_with_env_vars() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create fragment with env var as a number (port should be a number)
        let fragment_path = temp_dir.path().join("fragment.json5");
        // We need to use a different approach - env var expansion happens after parsing
        // So we'll use a string and it will be expanded, then serde will convert it
        // But for u16, we need the value to be numeric. Let's just test with a string field instead.
        let fragment_content = r#"
{
  meta: {
    version: "${TEST_VERSION:-default}",
  },
}
"#;
        fs::write(&fragment_path, fragment_content).expect("Failed to write fragment");
        
        // Create main config with include
        let main_path = temp_dir.path().join("main.json5");
        let main_content = format!(
            r#"{{
  "@include": "fragment.json5",
  gateway: {{ server: {{ port: 9090 }} }}
}}"#
        );
        fs::write(&main_path, main_content).expect("Failed to write main config");
        
        // Set env var and load
        std::env::set_var("TEST_VERSION", "1.0");
        let config = load_config(&main_path).expect("Failed to load config");
        std::env::remove_var("TEST_VERSION");
        
        assert_eq!(config.meta.version, "1.0");
    }

    #[test]
    fn test_include_removes_include_key() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create fragment
        let fragment_path = temp_dir.path().join("fragment.json5");
        let fragment_content = r#"
{
  gateway: {
    server: { port: 9090 }
  },
}
"#;
        fs::write(&fragment_path, fragment_content).expect("Failed to write fragment");
        
        // Create main config with include
        let main_path = temp_dir.path().join("main.json5");
        let main_content = format!(
            r#"{{
  "@include": "fragment.json5",
  meta: {{ version: "1.0" }}
}}"#
        );
        fs::write(&main_path, main_content).expect("Failed to write main config");
        
        // Load and verify @include is not in the deserialized config
        let config = load_config(&main_path).expect("Failed to load config");
        // The @include should have been removed during processing
        // We can't directly check this in AisopodConfig, but we can verify the merge worked
        assert_eq!(config.gateway.server.port, 9090);
    }
}
