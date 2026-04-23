//! Key Derivation Functions (KDF)
//!
//! Provides password-based and key-based key derivation functions.
//!
//! # Example
//!
//! ```rust,no_run
//! use lib_crypto::kdf::{derive_key_argon2, generate_salt};
//!
//! let salt = generate_salt();
//! let key = derive_key_argon2("my_password", &salt).unwrap();
//! // Use key for encryption
//! ```

use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::Sha256;
use thiserror::Error;

/// KDF errors
#[derive(Debug, Error, PartialEq, Eq)]
pub enum KdfError {
    #[error("Key derivation failed")]
    DerivationFailed,
    #[error("Invalid salt length: minimum 16 bytes required")]
    InvalidSaltLength,
    #[error("Invalid parameters")]
    InvalidParameters,
}

pub type Result<T> = std::result::Result<T, KdfError>;

/// Generate a cryptographically secure salt (16 bytes)
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Derive a 32-byte key from a password using Argon2id
///
/// Uses hardened parameters:
/// - Memory: 64 MB
/// - Iterations: 10
/// - Parallelism: 4
///
/// # Arguments
/// * `password` - User password
/// * `salt` - Random salt (minimum 16 bytes)
pub fn derive_key_argon2(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    if salt.len() < 16 {
        return Err(KdfError::InvalidSaltLength);
    }

    let params = Params::new(65536, 10, 4, Some(32)).map_err(|_| KdfError::InvalidParameters)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|_| KdfError::DerivationFailed)?;

    Ok(key)
}

/// Derive a 32-byte key using HKDF-SHA256
///
/// Used for key expansion from existing keying material.
///
/// # Arguments
/// * `ikm` - Input keying material
/// * `salt` - Optional salt (use empty slice for no salt)
/// * `info` - Context information for key separation
pub fn derive_key_hkdf(ikm: &[u8], salt: &[u8], info: &[u8]) -> Result<[u8; 32]> {
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut okm = [0u8; 32];
    hk.expand(info, &mut okm)
        .map_err(|_| KdfError::DerivationFailed)?;
    Ok(okm)
}

/// Derive a key using PBKDF2-HMAC-SHA256
///
/// For legacy compatibility - prefer Argon2 for new implementations.
///
/// # Arguments
/// * `password` - User password
/// * `salt` - Random salt (minimum 16 bytes)
/// * `iterations` - Number of iterations (minimum 100,000 recommended)
pub fn derive_key_pbkdf2(password: &str, salt: &[u8], iterations: u32) -> Result<[u8; 32]> {
    if salt.len() < 16 {
        return Err(KdfError::InvalidSaltLength);
    }

    let mut key = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2_derive_key() {
        let salt = generate_salt();
        let key1 = derive_key_argon2("password123", &salt).unwrap();
        let key2 = derive_key_argon2("password123", &salt).unwrap();

        // Same password + salt = same key
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_argon2_different_passwords() {
        let salt = generate_salt();
        let key1 = derive_key_argon2("password1", &salt).unwrap();
        let key2 = derive_key_argon2("password2", &salt).unwrap();

        // Different passwords = different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_hkdf_derive_key() {
        let ikm = [0x0b; 22];
        let salt = generate_salt();
        let info = b"app-specific-info";

        let key = derive_key_hkdf(&ikm, &salt, info).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_pbkdf2_derive_key() {
        let salt = generate_salt();
        let key = derive_key_pbkdf2("password", &salt, 100_000).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_short_salt_rejected() {
        let short_salt = [0u8; 8];
        let result = derive_key_argon2("password", &short_salt);
        assert!(matches!(result, Err(KdfError::InvalidSaltLength)));
    }
}
