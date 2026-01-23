//! AES-GCM and ChaCha20-Poly1305 Encryption Primitives
//!
//! Provides core stateless encryption functions for secure data protection.
//!
//! # Example
//!
//! ```rust,no_run
//! use lib_common::crypto::aes::{aes_gcm_encrypt, aes_gcm_decrypt, generate_key};
//!
//! let key = generate_key();
//! let plaintext = b"sensitive data";
//!
//! let encrypted = aes_gcm_encrypt(&key, plaintext).unwrap();
//! let decrypted = aes_gcm_decrypt(&key, &encrypted).unwrap();
//! assert_eq!(plaintext.to_vec(), decrypted);
//! ```

use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use chacha20poly1305::ChaCha20Poly1305;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Algorithm identifier for encrypted data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::Aes256Gcm => write!(f, "AES-256-GCM"),
            Algorithm::ChaCha20Poly1305 => write!(f, "ChaCha20-Poly1305"),
        }
    }
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Algorithm used for encryption
    pub algorithm: Algorithm,
    /// 12-byte nonce
    pub nonce: Vec<u8>,
    /// Ciphertext with authentication tag
    pub ciphertext: Vec<u8>,
}

impl EncryptedData {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|_| EncryptionError::SerializationError)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|_| EncryptionError::SerializationError)
    }
}

/// Encryption errors
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EncryptionError {
    #[error("Invalid key length: expected 32 bytes, got {0}")]
    InvalidKeyLength(usize),
    #[error("Invalid nonce length: expected 12 bytes")]
    InvalidNonceLength,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed: authentication error")]
    DecryptionFailed,
    #[error("Serialization error")]
    SerializationError,
    #[error("Algorithm mismatch")]
    AlgorithmMismatch,
}

pub type Result<T> = std::result::Result<T, EncryptionError>;

/// Generate a cryptographically secure 32-byte key
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// Generate a cryptographically secure 12-byte nonce
fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Encrypt data using AES-256-GCM
///
/// # Arguments
/// * `key` - 32-byte encryption key
/// * `plaintext` - Data to encrypt
///
/// # Returns
/// `EncryptedData` containing nonce and ciphertext with auth tag
pub fn aes_gcm_encrypt(key: &[u8], plaintext: &[u8]) -> Result<EncryptedData> {
    if key.len() != 32 {
        return Err(EncryptionError::InvalidKeyLength(key.len()));
    }

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|_| EncryptionError::InvalidKeyLength(key.len()))?;

    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| EncryptionError::EncryptionFailed)?;

    Ok(EncryptedData {
        algorithm: Algorithm::Aes256Gcm,
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

/// Decrypt data using AES-256-GCM
///
/// # Arguments
/// * `key` - 32-byte encryption key (must match key used for encryption)
/// * `encrypted` - `EncryptedData` to decrypt
///
/// # Returns
/// Decrypted plaintext
pub fn aes_gcm_decrypt(key: &[u8], encrypted: &EncryptedData) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(EncryptionError::InvalidKeyLength(key.len()));
    }
    if encrypted.nonce.len() != 12 {
        return Err(EncryptionError::InvalidNonceLength);
    }
    if encrypted.algorithm != Algorithm::Aes256Gcm {
        return Err(EncryptionError::AlgorithmMismatch);
    }

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|_| EncryptionError::InvalidKeyLength(key.len()))?;

    let nonce = Nonce::from_slice(&encrypted.nonce);

    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|_| EncryptionError::DecryptionFailed)
}

/// Encrypt data using ChaCha20-Poly1305
///
/// ChaCha20-Poly1305 is a fast, secure AEAD cipher that performs well
/// on systems without AES hardware acceleration.
pub fn chacha20_encrypt(key: &[u8], plaintext: &[u8]) -> Result<EncryptedData> {
    if key.len() != 32 {
        return Err(EncryptionError::InvalidKeyLength(key.len()));
    }

    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| EncryptionError::InvalidKeyLength(key.len()))?;

    let nonce_bytes = generate_nonce();
    let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| EncryptionError::EncryptionFailed)?;

    Ok(EncryptedData {
        algorithm: Algorithm::ChaCha20Poly1305,
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

/// Decrypt data using ChaCha20-Poly1305
pub fn chacha20_decrypt(key: &[u8], encrypted: &EncryptedData) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(EncryptionError::InvalidKeyLength(key.len()));
    }
    if encrypted.nonce.len() != 12 {
        return Err(EncryptionError::InvalidNonceLength);
    }
    if encrypted.algorithm != Algorithm::ChaCha20Poly1305 {
        return Err(EncryptionError::AlgorithmMismatch);
    }

    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| EncryptionError::InvalidKeyLength(key.len()))?;

    let nonce = chacha20poly1305::Nonce::from_slice(&encrypted.nonce);

    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|_| EncryptionError::DecryptionFailed)
}

/// Generic encrypt function that dispatches by algorithm
pub fn encrypt(algorithm: Algorithm, key: &[u8], plaintext: &[u8]) -> Result<EncryptedData> {
    match algorithm {
        Algorithm::Aes256Gcm => aes_gcm_encrypt(key, plaintext),
        Algorithm::ChaCha20Poly1305 => chacha20_encrypt(key, plaintext),
    }
}

/// Generic decrypt function that uses algorithm from encrypted data
pub fn decrypt(key: &[u8], encrypted: &EncryptedData) -> Result<Vec<u8>> {
    match encrypted.algorithm {
        Algorithm::Aes256Gcm => aes_gcm_decrypt(key, encrypted),
        Algorithm::ChaCha20Poly1305 => chacha20_decrypt(key, encrypted),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_roundtrip() {
        let key = generate_key();
        let plaintext = b"Hello, World!";

        let encrypted = aes_gcm_encrypt(&key, plaintext).unwrap();
        assert_eq!(encrypted.algorithm, Algorithm::Aes256Gcm);
        assert_eq!(encrypted.nonce.len(), 12);

        let decrypted = aes_gcm_decrypt(&key, &encrypted).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha20_roundtrip() {
        let key = generate_key();
        let plaintext = b"ChaCha20-Poly1305 test";

        let encrypted = chacha20_encrypt(&key, plaintext).unwrap();
        assert_eq!(encrypted.algorithm, Algorithm::ChaCha20Poly1305);

        let decrypted = chacha20_decrypt(&key, &encrypted).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key();
        let key2 = generate_key();
        let plaintext = b"test";

        let encrypted = aes_gcm_encrypt(&key1, plaintext).unwrap();
        let result = aes_gcm_decrypt(&key2, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = vec![0u8; 16];
        let result = aes_gcm_encrypt(&short_key, b"test");
        assert!(matches!(result, Err(EncryptionError::InvalidKeyLength(16))));
    }

    #[test]
    fn test_generic_encrypt_decrypt() {
        let key = generate_key();
        let plaintext = b"Generic API test";

        let encrypted = encrypt(Algorithm::Aes256Gcm, &key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
