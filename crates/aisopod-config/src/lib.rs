//! # aisopod-config
//!
//! Configuration loading, parsing, and validation for the aisopod project.
//!
//! ## Modules
//!
//! - `types`: Core configuration types for the application
//! - `loader`: Configuration file loading functionality
//! - `env`: Environment variable substitution functionality
//! - `includes`: @include directive processing functionality
//! - `validation`: Configuration semantic validation
//! - `sensitive`: Sensitive field handling with redaction
//! - `generate`: Default configuration generation functionality
//! - `watcher`: Configuration file watcher for hot reload

pub mod env;
pub mod generate;
pub mod includes;
pub mod loader;
pub mod sensitive;
pub mod types;
pub mod validation;
pub mod watcher;

pub use types::AisopodConfig;
pub use types::AgentDefaults;
pub use types::ModelFallback;
pub use loader::load_config;
pub use loader::load_config_json5;
pub use loader::load_config_toml;
pub use loader::load_config_json5_str;
pub use loader::load_config_toml_str;
pub use env::expand_env_vars;
pub use validation::ValidationError;
pub use sensitive::Sensitive;
pub use generate::{generate_default_config, generate_config_with_format, ConfigFormat};
pub use watcher::ConfigWatcher;
pub use watcher::diff_sections;
