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

pub mod env;
pub mod includes;
pub mod loader;
pub mod sensitive;
pub mod types;
pub mod validation;

pub use types::AisopodConfig;
pub use loader::load_config;
pub use loader::load_config_json5;
pub use loader::load_config_toml;
pub use env::expand_env_vars;
pub use validation::ValidationError;
pub use sensitive::Sensitive;
