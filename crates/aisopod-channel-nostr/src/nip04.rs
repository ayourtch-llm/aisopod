//! NIP-04: Encrypted Direct Message encryption/decryption.
//!
//! This module implements the Nostr NIP-04 specification for encrypting
//! and decrypting direct messages using shared secrets derived from
//! Elliptic Curve Diffie-Hellman (ECDH) key exchange.

use aes::Aes256;
use cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use cipher::generic_array::GenericArray;
use rand::Rng;
use secp256k1::{PublicKey, SecretKey, Secp256k1, All, ecdh::SharedSecret};

/// Error types for NIP-04 operations.
#[derive(Debug, thiserror::Error)]
pub enum Nip04Error {
    #[error("Invalid base64 encoding: {0}")]
    InvalidBase64(String),
    #[error("AES encryption error: {0}")]
    AesError(String),
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),
}

/// Encrypt a message using NIP-04.
///
/// This function computes a shared secret using ECDH between the sender's
/// private key and the recipient's public key, then uses AES-256-CBC to
/// encrypt the plaintext message.
///
/// # Arguments
/// * `sender_secret` - The sender's secret key
/// * `recipient_pubkey` - The recipient's public key
/// * `plaintext` - The message to encrypt
///
/// # Returns
/// * `Ok(String)` - The encrypted message in format: `base64(ciphertext)?iv=base64(iv)`
/// * `Err<Nip04Error>` - An error if encryption fails
pub fn encrypt(
    sender_secret: &SecretKey,
    recipient_pubkey: &[u8],
    plaintext: &str,
) -> Result<String, Nip04Error> {
    // Parse recipient public key
    let recipient_pubkey = PublicKey::from_slice(recipient_pubkey)
        .map_err(|e| Nip04Error::KeyDerivation(e.to_string()))?;

    // Compute shared secret using ECDH
    // Nostr uses the x-coordinate of the ECDH result
    let shared_secret = SharedSecret::new(&recipient_pubkey, sender_secret);

    // Use the shared secret as the AES key (32 bytes = 256 bits)
    let aes_key: &[u8] = &shared_secret.secret_bytes();

    // Generate random IV (16 bytes for AES)
    let mut iv = [0u8; 16];
    rand::rngs::OsRng.fill(&mut iv);

    // Encrypt using AES-256-CBC - convert to GenericArray
    let cipher = Aes256::new(GenericArray::from_slice(aes_key));
    
    // Pad plaintext to multiple of 16 bytes (PKCS#7)
    let padded_len = ((plaintext.len() + 15) / 16) * 16;
    let mut padded = vec![0u8; padded_len];
    padded[..plaintext.len()].copy_from_slice(plaintext.as_bytes());
    let pad_len = padded_len - plaintext.len();
    for i in 0..pad_len {
        padded[plaintext.len() + i] = pad_len as u8;
    }

    // Encrypt each block
    let mut ciphertext = vec![0u8; padded.len()];
    let mut prev_block = iv.clone();
    
    for (i, block) in padded.chunks(16).enumerate() {
        let mut block_array = [0u8; 16];
        block_array.copy_from_slice(block);
        
        // XOR with previous ciphertext block (or IV)
        for j in 0..16 {
            block_array[j] ^= prev_block[j];
        }
        
        // Encrypt
        cipher.encrypt_block(&mut block_array.into());
        
        ciphertext[i * 16..(i + 1) * 16].copy_from_slice(&block_array);
        prev_block.copy_from_slice(&block_array);
    }

    // Encode ciphertext and IV in base64
    let ciphertext_b64 = base64::encode(&ciphertext);
    let iv_b64 = base64::encode(&iv);

    Ok(format!("{}?iv={}", ciphertext_b64, iv_b64))
}

/// Decrypt a message using NIP-04.
///
/// This function computes a shared secret using ECDH between the recipient's
/// private key and the sender's public key, then uses AES-256-CBC to
/// decrypt the ciphertext message.
///
/// # Arguments
/// * `recipient_secret` - The recipient's secret key
/// * `sender_pubkey` - The sender's public key (as bytes)
/// * `ciphertext` - The encrypted message in format: `base64(ciphertext)?iv=base64(iv)`
///
/// # Returns
/// * `Ok(String)` - The decrypted plaintext message
/// * `Err<Nip04Error>` - An error if decryption fails
pub fn decrypt(
    recipient_secret: &SecretKey,
    sender_pubkey: &[u8],
    ciphertext: &str,
) -> Result<String, Nip04Error> {
    // Parse the ciphertext format: base64(ciphertext)?iv=base64(iv)
    let (ciphertext_b64, iv_b64) = ciphertext
        .split_once("?iv=")
        .ok_or_else(|| Nip04Error::InvalidBase64("Missing ?iv= separator".to_string()))?;

    let ciphertext = base64::decode(ciphertext_b64)
        .map_err(|e| Nip04Error::InvalidBase64(e.to_string()))?;
    let iv = base64::decode(iv_b64)
        .map_err(|e| Nip04Error::InvalidBase64(e.to_string()))?;

    if iv.len() != 16 {
        return Err(Nip04Error::InvalidBase64(
            "IV must be 16 bytes".to_string(),
        ));
    }

    // Parse sender public key
    let sender_pubkey = PublicKey::from_slice(sender_pubkey)
        .map_err(|e| Nip04Error::KeyDerivation(e.to_string()))?;

    // Compute shared secret using ECDH
    let shared_secret = SharedSecret::new(&sender_pubkey, recipient_secret);

    // Use the shared secret as the AES key (32 bytes = 256 bits)
    let aes_key: &[u8] = &shared_secret.secret_bytes();

    // Decrypt using AES-256-CBC - convert to GenericArray
    let cipher = Aes256::new(GenericArray::from_slice(aes_key));
    
    if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err(Nip04Error::AesError(
            "Invalid ciphertext length".to_string(),
        ));
    }

    let mut plaintext = vec![0u8; ciphertext.len()];
    let mut prev_block = iv.clone();

    for (i, block) in ciphertext.chunks(16).enumerate() {
        let mut block_array = [0u8; 16];
        block_array.copy_from_slice(block);

        // Decrypt
        cipher.decrypt_block(&mut block_array.into());

        // XOR with previous ciphertext block (or IV)
        for j in 0..16 {
            plaintext[i * 16 + j] = block_array[j] ^ prev_block[j];
        }

        prev_block.copy_from_slice(block);
    }

    // Remove PKCS#7 padding
    let pad_len = plaintext[plaintext.len() - 1] as usize;
    if pad_len == 0 || pad_len > 16 || pad_len > plaintext.len() {
        return Err(Nip04Error::AesError(
            "Invalid padding".to_string(),
        ));
    }

    for i in 0..pad_len {
        if plaintext[plaintext.len() - 1 - i] != pad_len as u8 {
            return Err(Nip04Error::AesError(
                "Invalid padding".to_string(),
            ));
        }
    }

    let plaintext_str = String::from_utf8_lossy(&plaintext[..plaintext.len() - pad_len]);

    Ok(plaintext_str.to_string())
}
