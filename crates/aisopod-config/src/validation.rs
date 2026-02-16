//! Configuration validation module.
//!
//! Provides semantic validation of configuration beyond what serde deserialization provides.

use crate::types::AisopodConfig;
use std::fmt;

/// Represents a validation error with the field path and a human-readable message.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The path to the invalid field (e.g., "gateway.port")
    pub path: String,
    /// A human-readable error message
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl AisopodConfig {
    /// Validate semantic rules across the entire configuration.
    ///
    /// Returns `Ok(())` if the configuration is valid, or `Err(Vec<ValidationError>)`
    /// containing all validation failures. All violations are collected (not fail-fast)
    /// so users can fix multiple issues at once.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        self.validate_meta(&mut errors);
        self.validate_gateway(&mut errors);
        self.validate_agents(&mut errors);
        self.validate_models(&mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_meta(&self, errors: &mut Vec<ValidationError>) {
        if self.meta.version.is_empty() {
            errors.push(ValidationError {
                path: "meta.version".to_string(),
                message: "Version must not be empty".to_string(),
            });
        }
    }

    fn validate_gateway(&self, errors: &mut Vec<ValidationError>) {
        let port = self.gateway.server.port;
        // Note: port is u16, so 0 <= port <= 65535 always holds
        // We only check for 0 which is invalid for server binding
        if port == 0 {
            errors.push(ValidationError {
                path: "gateway.server.port".to_string(),
                message: "Port must be between 1 and 65535, got 0".to_string(),
            });
        }

        if self.gateway.bind.address.is_empty() {
            errors.push(ValidationError {
                path: "gateway.bind.address".to_string(),
                message: "Address must not be empty".to_string(),
            });
        }
    }

    fn validate_agents(&self, errors: &mut Vec<ValidationError>) {
        let mut seen_names = std::collections::HashSet::new();

        for agent in &self.agents.agents {
            if agent.name.is_empty() {
                errors.push(ValidationError {
                    path: "agents[].name".to_string(),
                    message: "Agent name must not be empty".to_string(),
                });
            } else if !seen_names.insert(&agent.name) {
                errors.push(ValidationError {
                    path: format!("agents[\"{}\").name", agent.name),
                    message: format!("Duplicate agent name: {}", agent.name),
                });
            }
        }
    }

    fn validate_models(&self, errors: &mut Vec<ValidationError>) {
        let mut seen_ids = std::collections::HashSet::new();

        for model in &self.models.models {
            if model.id.is_empty() {
                errors.push(ValidationError {
                    path: "models[].id".to_string(),
                    message: "Model ID must not be empty".to_string(),
                });
            } else if !seen_ids.insert(&model.id) {
                errors.push(ValidationError {
                    path: format!("models[\"{}\").id", model.id),
                    message: format!("Duplicate model ID: {}", model.id),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Agent, AgentsConfig, GatewayConfig, MetaConfig, ModelsConfig};

    #[test]
    fn test_default_config_is_valid() {
        let config = AisopodConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_port_detected() {
        let mut config = AisopodConfig::default();
        // Use u16::MAX to get a value > 65535, which will overflow to 65535
        // So we need to use unsafe or cast from a larger type
        // Actually, since port is u16, we can't set it to 99999 directly
        // We'll just test with 0 which is also invalid
        config.gateway.server.port = 0;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.path == "gateway.server.port"));
        assert!(errors.iter().any(|e| e.message.contains("Port must be between 1 and 65535")));
    }

    #[test]
    fn test_empty_version_detected() {
        let mut config = AisopodConfig::default();
        config.meta.version = "".to_string();
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.path == "meta.version"));
    }

    #[test]
    fn test_empty_address_detected() {
        let mut config = AisopodConfig::default();
        config.gateway.bind.address = "".to_string();
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.path == "gateway.bind.address"));
    }

    #[test]
    fn test_duplicate_agent_names_detected() {
        let mut config = AisopodConfig::default();
        config.agents.agents = vec![
            Agent {
                id: "agent1".to_string(),
                name: "agent1".to_string(),
                model: "default".to_string(),
                workspace: "/workspace".to_string(),
                sandbox: false,
                subagents: vec![],
            },
            Agent {
                id: "agent2".to_string(),
                name: "agent1".to_string(),
                model: "default".to_string(),
                workspace: "/workspace".to_string(),
                sandbox: false,
                subagents: vec![],
            },
        ];
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("Duplicate agent name: agent1")));
    }

    #[test]
    fn test_multiple_errors_collected() {
        let mut config = AisopodConfig::default();
        config.gateway.server.port = 0; // Invalid port
        config.gateway.bind.address = "".to_string(); // Empty address
        config.meta.version = "".to_string(); // Empty version
        let errors = config.validate().unwrap_err();
        assert_eq!(errors.len(), 3);
    }
}
