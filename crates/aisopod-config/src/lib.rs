//! # aisopod-config
//!
//! Configuration loading, parsing, and validation for the aisopod project.
//!
//! ## Modules
//!
//! - `types`: Core configuration types for the application
//! - `loader`: Configuration file loading functionality
//! - `env`: Environment variable substitution functionality

pub mod env;
pub mod loader;
pub mod types;

pub use types::AisopodConfig;
pub use loader::load_config;
pub use loader::load_config_json5;
pub use loader::load_config_toml;
pub use env::expand_env_vars;
