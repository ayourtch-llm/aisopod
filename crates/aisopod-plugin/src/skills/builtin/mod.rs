//! Built-in skills for the aisopod system.
//!
//! This module contains skills that are compiled directly into the plugin crate.
//! Built-in skills are always available and do not require external plugin files.

#[cfg(feature = "skill-healthcheck")]
pub mod healthcheck;
