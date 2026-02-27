//! Nostr key management.
//!
//! This module provides key management functionality for Nostr,
//! supporting both nsec (private key) and npub (public key) bech32 formats.

use secp256k1::{SecretKey, PublicKey, Secp256k1, All, Message};
use std::str::FromStr;
use bech32::{encode, ToBase32, Variant, FromBase32};

/// Error types for key operations.
#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("Invalid nsec format: {0}")]
    InvalidNsec(String),
    #[error("Invalid hex format: {0}")]
    InvalidHex(String),
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    #[error("Secret key error: {0}")]
    SecretKey(#[from] secp256k1::Error),
    #[error("Bech32 error: {0}")]
    Bech32(#[from] bech32::Error),
}

/// Nostr key pair management.
pub struct NostrKeys {
    secret_key: SecretKey,
    public_key: PublicKey,
    secp: Secp256k1<All>,
}

impl NostrKeys {
    /// Create keys from an nsec (private key) string.
    ///
    /// # Arguments
    /// * `nsec` - The private key in bech32 nsec format
    ///
    /// # Returns
    /// * `Ok(NostrKeys)` - The key pair if successful
    /// * `Err(KeyError)` - An error if the key is invalid
    pub fn from_nsec(nsec: &str) -> Result<Self, KeyError> {
        // Decode bech32 nsec format (v0.8 returns 3 elements)
        let (_hrp, data, _variant) = bech32::decode(nsec)?;
        
        // Convert u5 to u8 bytes - use u5::to_u8() method
        let data_u8: Vec<u8> = data
            .into_iter()
            .map(|b| b.to_u8())
            .collect();
        
        // Convert to secret key
        let secret_key = SecretKey::from_slice(&data_u8)?;
        
        // Derive public key
        let secp = Secp256k1::<All>::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        Ok(Self {
            secret_key,
            public_key,
            secp,
        })
    }

    /// Create keys from a hex-encoded private key string.
    ///
    /// # Arguments
    /// * `hex` - The private key in hex format
    ///
    /// # Returns
    /// * `Ok(NostrKeys)` - The key pair if successful
    /// * `Err(KeyError)` - An error if the key is invalid
    pub fn from_hex(hex: &str) -> Result<Self, KeyError> {
        let secp = Secp256k1::<All>::new();
        let secret_key = SecretKey::from_str(hex)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        Ok(Self {
            secret_key,
            public_key,
            secp,
        })
    }

    /// Create keys from a public key hex string (read-only).
    ///
    /// # Arguments
    /// * `pubkey_hex` - The public key in hex format
    ///
    /// # Returns
    /// * `Ok(NostrKeys)` - The public key only (cannot sign)
    /// * `Err(KeyError)` - An error if the key is invalid
    pub fn from_pubkey_hex(pubkey_hex: &str) -> Result<Self, KeyError> {
        let secp = Secp256k1::<All>::new();
        let bytes = hex::decode(pubkey_hex)
            .map_err(|e| KeyError::InvalidHex(e.to_string()))?;
        let public_key = PublicKey::from_slice(&bytes)
            .map_err(|e| KeyError::InvalidPublicKey(e.to_string()))?;
        
        // Use a dummy secret key for read-only operations
        let secret_key = SecretKey::from_slice(&[0u8; 32])
            .map_err(|e| KeyError::InvalidHex(e.to_string()))?;
        
        Ok(Self {
            secret_key,
            public_key,
            secp,
        })
    }

    /// Get the public key as npub (bech32 format).
    pub fn npub(&self) -> String {
        // Convert bytes to base32 for bech32 encoding
        let data_bytes = self.public_key.serialize()[1..].to_vec();
        let data_base32: Vec<bech32::u5> = data_bytes.iter()
            .map(|b| bech32::u5::try_from_u8(*b).expect("value < 256"))
            .collect();
        encode("npub", &data_base32, Variant::Bech32)
            .expect("bech32 encoding should succeed")
    }

    /// Get the public key as hex string.
    pub fn pubkey_hex(&self) -> String {
        hex::encode(self.public_key.serialize()[1..].to_vec())
    }

    /// Get the secret key reference.
    pub fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }

    /// Get the public key reference.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Sign a message with the private key.
    pub fn sign(&self, msg: &[u8]) -> Result<String, KeyError> {
        // Create a message hash for signing
        let msg_hash = Message::from_slice(msg)
            .map_err(|e| KeyError::InvalidHex(e.to_string()))?;
        let signature = self.secp.sign_ecdsa(&msg_hash, &self.secret_key);
        Ok(hex::encode(signature.serialize_compact()))
    }
}

impl Clone for NostrKeys {
    fn clone(&self) -> Self {
        Self {
            secret_key: self.secret_key.clone(),
            public_key: self.public_key.clone(),
            secp: Secp256k1::<All>::new(),
        }
    }
}
