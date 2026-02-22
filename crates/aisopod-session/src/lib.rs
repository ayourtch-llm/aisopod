//! # aisopod-session
//!
//! Session management, state tracking, and session lifecycle for conversations.

pub mod compaction;
pub mod db;
pub mod routing;
pub mod store;
pub mod types;

pub use compaction::{CompactionRecord, CompactionStrategy};
pub use routing::{resolve_session_key, ChannelContext, PeerKind};
pub use store::SessionStore;
pub use types::{
    HistoryQuery, Session, SessionFilter, SessionKey, SessionMetadata, SessionPatch, SessionStatus,
    SessionSummary, StoredMessage,
};
