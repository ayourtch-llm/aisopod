//! # aisopod-session
//!
//! Session management, state tracking, and session lifecycle for conversations.

pub mod db;
pub mod store;
pub mod types;

pub use store::SessionStore;
pub use types::{
    Session, SessionFilter, SessionKey, SessionMetadata, SessionPatch, SessionStatus, SessionSummary,
    StoredMessage,
};
