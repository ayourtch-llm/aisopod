//! # aisopod-session
//!
//! Session management, state tracking, and session lifecycle for conversations.

pub mod store;
pub mod types;

pub use store::{Session, SessionStore};
pub use types::{
    SessionFilter, SessionKey, SessionMetadata, SessionPatch, SessionStatus, SessionSummary,
    StoredMessage,
};
