//! Nostr event types and creation.
//!
//! This module provides types and functions for creating and signing
//! Nostr events (kind 1 text notes, kind 4 encrypted DMs).

use crate::keys::NostrKeys;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Error types for event operations.
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Invalid event data: {0}")]
    InvalidData(String),
    #[error("Signing error: {0}")]
    Signing(#[from] crate::keys::KeyError),
    #[error("Encoding error: {0}")]
    Encoding(String),
}

/// A Nostr event.
///
/// This struct represents a Nostr event with all required fields.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NostrEvent {
    /// Event ID (SHA256 hash of the event data)
    pub id: String,
    /// Pubkey of the event creator (hex)
    pub pubkey: String,
    /// Timestamp in seconds
    pub created_at: u64,
    /// Event kind
    pub kind: u32,
    /// Tags array
    pub tags: Vec<Vec<String>>,
    /// Event content
    pub content: String,
    /// Signature (hex)
    pub sig: String,
}

impl NostrEvent {
    /// Create a new text note event (kind 1).
    ///
    /// # Arguments
    /// * `keys` - The key pair for signing
    /// * `content` - The text content of the note
    ///
    /// # Returns
    /// * `Ok(NostrEvent)` - The created event
    /// * `Err(EventError)` - An error if event creation fails
    pub fn new_text_note(keys: &NostrKeys, content: &str) -> Result<Self, EventError> {
        let created_at = Utc::now().timestamp() as u64;
        let pubkey = keys.pubkey_hex();
        
        // Create the event without ID and signature
        let mut event = NostrEvent {
            id: String::new(),
            pubkey,
            created_at,
            kind: 1, // Text note
            tags: Vec::new(),
            content: content.to_string(),
            sig: String::new(),
        };
        
        // Compute ID and sign
        event.compute_id_and_sign(keys)?;
        
        Ok(event)
    }

    /// Create a new encrypted DM event (kind 4, NIP-04).
    ///
    /// # Arguments
    /// * `keys` - The sender's key pair for signing
    /// * `recipient_pubkey` - The recipient's public key (hex format)
    /// * `plaintext` - The plaintext content to encrypt
    ///
    /// # Returns
    /// * `Ok(NostrEvent)` - The created encrypted DM event
    /// * `Err(EventError)` - An error if event creation fails
    pub fn new_dm(
        keys: &NostrKeys,
        recipient_pubkey: &str,
        plaintext: &str,
    ) -> Result<Self, EventError> {
        let created_at = Utc::now().timestamp() as u64;
        let pubkey = keys.pubkey_hex();
        
        // Encrypt the content using NIP-04
        let encrypted_content = crate::nip04::encrypt(
            keys.secret_key(),
            &hex::decode(recipient_pubkey)
                .map_err(|e| EventError::Encoding(e.to_string()))?
                .as_slice(),
            plaintext,
        )
        .map_err(|e| EventError::Encoding(e.to_string()))?;
        
        // Create the event with recipient tag
        let mut event = NostrEvent {
            id: String::new(),
            pubkey,
            created_at,
            kind: 4, // Encrypted DM
            tags: vec![vec!["p".to_string(), recipient_pubkey.to_string()]],
            content: encrypted_content,
            sig: String::new(),
        };
        
        // Compute ID and sign
        event.compute_id_and_sign(keys)?;
        
        Ok(event)
    }

    /// Compute the event ID and sign it.
    fn compute_id_and_sign(&mut self, keys: &NostrKeys) -> Result<(), EventError> {
        // Create the event data array for hashing
        let event_data = serde_json::json!([
            0,
            self.pubkey,
            self.created_at,
            self.kind,
            self.tags,
            self.content
        ]);
        
        // Serialize to JSON without whitespace for consistent hashing
        let event_json = serde_json::to_string(&event_data)
            .map_err(|e| EventError::Encoding(e.to_string()))?;
        
        // Compute SHA256 hash for event ID
        let mut hasher = Sha256::new();
        hasher.update(event_json.as_bytes());
        let hash = hasher.finalize();
        self.id = hex::encode(hash);
        
        // Sign the event ID
        self.sig = keys.sign(self.id.as_bytes())?;
        
        Ok(())
    }

    /// Verify the event signature.
    pub fn verify(&self) -> Result<bool, EventError> {
        let public_key = hex::decode(&self.pubkey)
            .map_err(|e| EventError::Encoding(e.to_string()))?;
        let public_key = secp256k1::PublicKey::from_slice(&public_key)
            .map_err(|e| EventError::Signing(e.into()))?;
        
        let signature = hex::decode(&self.sig)
            .map_err(|e| EventError::Encoding(e.to_string()))?;
        let signature = secp256k1::ecdsa::Signature::from_compact(&signature)
            .map_err(|e| EventError::Signing(e.into()))?;
        
        let secp = secp256k1::Secp256k1::<secp256k1::All>::new();
        
        // Recreate the event data for verification
        let event_data = serde_json::json!([
            0,
            self.pubkey,
            self.created_at,
            self.kind,
            self.tags,
            self.content
        ]);
        let event_json = serde_json::to_string(&event_data)
            .map_err(|e| EventError::Encoding(e.to_string()))?;
        
        let mut hasher = Sha256::new();
        hasher.update(event_json.as_bytes());
        let hash = hasher.finalize();
        let msg = secp256k1::Message::from_slice(&hash)
            .map_err(|e| EventError::Encoding(e.to_string()))?;
        
        Ok(secp.verify_ecdsa(&msg, &signature, &public_key).is_ok())
    }

    /// Get the event as a JSON value for sending to relays.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "pubkey": self.pubkey,
            "created_at": self.created_at,
            "kind": self.kind,
            "tags": self.tags,
            "content": self.content,
            "sig": self.sig
        })
    }

    /// Parse an event from a JSON value received from a relay.
    pub fn from_json_value(value: serde_json::Value) -> Result<Self, EventError> {
        let id = value["id"].as_str()
            .ok_or_else(|| EventError::InvalidData("Missing id".to_string()))?
            .to_string();
        let pubkey = value["pubkey"].as_str()
            .ok_or_else(|| EventError::InvalidData("Missing pubkey".to_string()))?
            .to_string();
        let created_at = value["created_at"].as_u64()
            .ok_or_else(|| EventError::InvalidData("Missing created_at".to_string()))?;
        let kind = value["kind"].as_u64()
            .ok_or_else(|| EventError::InvalidData("Missing kind".to_string()))? as u32;
        let tags = value["tags"].as_array()
            .ok_or_else(|| EventError::InvalidData("Missing tags".to_string()))?
            .iter()
            .map(|t| t.as_array()
                .ok_or_else(|| EventError::InvalidData("Invalid tag".to_string()))
                .map(|arr| arr.iter().map(|s| s.as_str().unwrap_or("").to_string()).collect())
            )
            .collect::<Result<Vec<_>, _>>()?;
        let content = value["content"].as_str()
            .ok_or_else(|| EventError::InvalidData("Missing content".to_string()))?
            .to_string();
        let sig = value["sig"].as_str()
            .ok_or_else(|| EventError::InvalidData("Missing sig".to_string()))?
            .to_string();

        Ok(Self {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content,
            sig,
        })
    }
}
