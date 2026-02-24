//! Core types for the skills system.
//!
//! This module provides the foundational types for defining and managing skills
//! in the aisopod system. Skills are reusable bundles of system prompt fragments
//! and tools that can be assigned to agents.
//!
//! # Core Types
//!
//! - [`Skill`]: The main async trait that all skills must implement.
//! - [`SkillMeta`]: Metadata describing a skill's identity and requirements.
//! - [`SkillCategory`]: Category classification for skills.
//! - [`SkillContext`]: Runtime context provided to skills during initialization.
//!
//! # Manifest Types
//!
//! - [`SkillManifest`]: Parsed skill manifest from `skill.toml` files.
//! - [`ManifestError`]: Error types for manifest parsing.
//! - [`parse_manifest`]: Function to parse a skill manifest from file.
//!
//! # Discovery Types
//!
//! - [`discover_skill_dirs`]: Function to scan directories for skills.
//! - [`validate_requirements`]: Function to check skill requirements.
//! - [`load_skills`]: Function to orchestrate the full discovery pipeline.
//!
//! # Registry Types
//!
//! - [`SkillRegistry`]: Central registry for skill discovery and lifecycle management.
//! - [`SkillStatus`]: Status indicating a skill's health and availability.
//!
//! # Example
//!
//! ```ignore
//! use aisopod_plugin::{Skill, SkillMeta, SkillCategory, SkillContext};
//! use aisopod_tools::ToolResult;
//! use async_trait::async_trait;
//! use std::sync::Arc;
//!
//! struct ExampleSkill {
//!     meta: SkillMeta,
//! }
//!
//! impl ExampleSkill {
//!     pub fn new() -> Self {
//!         Self {
//!             meta: SkillMeta::new(
//!                 "example-skill",
//!                 "1.0.0",
//!                 "An example skill",
//!                 SkillCategory::Productivity,
//!                 vec![],
//!                 vec![],
//!                 None,
//!             ),
//!         }
//!     }
//! }
//!
//! #[async_trait]
//! impl Skill for ExampleSkill {
//!     fn id(&self) -> &str {
//!         "example-skill"
//!     }
//!
//!     fn meta(&self) -> &SkillMeta {
//!         &self.meta
//!     }
//!
//!     fn system_prompt_fragment(&self) -> Option<String> {
//!         Some("You have access to an example skill.".to_string())
//!     }
//!
//!     fn tools(&self) -> Vec<Arc<dyn aisopod_tools::Tool>> {
//!         vec![]
//!     }
//!
//!     async fn init(&self, _ctx: &SkillContext) -> Result<(), Box<dyn std::error::Error>> {
//!         Ok(())
//!     }
//! }
//! ```

mod context;
mod discovery;
mod manifest;
mod meta;
mod registry;
mod r#trait;
mod builtin;

pub use context::SkillContext;
pub use discovery::{discover_skill_dirs, validate_requirements, load_skills, DiscoveryError, DiscoveryResult, DiscoveredSkill};
pub use manifest::{SkillManifest, ManifestError, parse_manifest};
pub use meta::{SkillCategory, SkillMeta};
pub use registry::{SkillRegistry, SkillStatus};
pub use r#trait::Skill;
pub use builtin::healthcheck;
