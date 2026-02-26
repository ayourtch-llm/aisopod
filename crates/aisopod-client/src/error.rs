use thiserror::Error;

/// Error types for the aisopod client
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Request timeout after {0} seconds")]
    Timeout(usize),
    
    #[error("Connection closed")]
    Closed,
    
    #[error("Response message ID not found: {0}")]
    MessageIdNotFound(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Result type for the aisopod client
pub type Result<T> = std::result::Result<T, ClientError>;
