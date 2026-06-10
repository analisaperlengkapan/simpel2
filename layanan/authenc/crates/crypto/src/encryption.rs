//! Symmetric encryption with ChaCha20-Poly1305
//!
//! This module provides symmetric encryption using ChaCha20-Poly1305 AEAD cipher.
//! It supports:
//! - Encryption/decryption of arbitrary data
//! - Key derivation from passwords (Argon2id)
//! - Integration with Secreton for secure key storage
//! - Nonce generation and management

use base64::Engine;
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Encryption errors
#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key length: expected 32 bytes, got {0}")]
    InvalidKeyLength(usize),

    #[error("Invalid nonce length: expected 12 bytes, got {0}")]
    InvalidNonceLength(usize),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Secreton integration error: {0}")]
    SecretonError(String),
}

pub type Result<T> = std::result::Result<T, EncryptionError>;

/// Encrypted data with nonce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Ciphertext
    pub ciphertext: Vec<u8>,
    /// Nonce used for encryption (12 bytes)
    pub nonce: Vec<u8>,
}

/// Encryption service using ChaCha20-Poly1305
pub struct EncryptionService {
    cipher: ChaCha20Poly1305,
}

impl EncryptionService {
    /// Create a new encryption service with the given key
    ///
    /// # Arguments
    /// * `key` - 32-byte encryption key
    ///
    /// # Errors
    /// Returns error if key length is not 32 bytes
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength(key.len()));
        }

        let key = Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);

        Ok(Self { cipher })
    }

    /// Create a new encryption service with a randomly generated key
    pub fn new_with_random_key() -> Self {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);

        Self { cipher }
    }

    /// Encrypt plaintext data
    ///
    /// # Arguments
    /// * `plaintext` - Data to encrypt
    ///
    /// # Returns
    /// Encrypted data with nonce
    ///
    /// # Errors
    /// Returns error if encryption fails
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        // Generate random nonce (12 bytes for ChaCha20-Poly1305)
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    /// Decrypt ciphertext data
    ///
    /// # Arguments
    /// * `encrypted_data` - Encrypted data with nonce
    ///
    /// # Returns
    /// Decrypted plaintext
    ///
    /// # Errors
    /// Returns error if decryption fails or nonce is invalid
    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>> {
        if encrypted_data.nonce.len() != 12 {
            return Err(EncryptionError::InvalidNonceLength(
                encrypted_data.nonce.len(),
            ));
        }

        let nonce = Nonce::from_slice(&encrypted_data.nonce);

        let plaintext = self
            .cipher
            .decrypt(nonce, encrypted_data.ciphertext.as_ref())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }

    /// Encrypt plaintext and return base64-encoded result
    pub fn encrypt_to_base64(&self, plaintext: &[u8]) -> Result<String> {
        let encrypted = self.encrypt(plaintext)?;
        let serialized = serde_json::to_vec(&encrypted)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        Ok(base64::engine::general_purpose::STANDARD.encode(serialized))
    }

    /// Decrypt base64-encoded ciphertext
    pub fn decrypt_from_base64(&self, encoded: &str) -> Result<Vec<u8>> {
        let serialized = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        let encrypted: EncryptedData = serde_json::from_slice(&serialized)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        self.decrypt(&encrypted)
    }
}

/// Key derivation using Argon2id
pub mod key_derivation {
    use super::*;
    use argon2::{
        Argon2, Params,
        password_hash::{SaltString, rand_core::OsRng},
    };

    /// Derive a 32-byte encryption key from a password using Argon2id
    ///
    /// # Arguments
    /// * `password` - Password to derive key from
    /// * `salt` - Salt for key derivation (16 bytes recommended)
    ///
    /// # Returns
    /// 32-byte encryption key
    ///
    /// # Errors
    /// Returns error if key derivation fails
    pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
        // Configure Argon2id parameters
        // Memory cost: 64 MB, Time cost: 3 iterations, Parallelism: 4 threads
        let params = Params::new(65536, 3, 4, Some(32))
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

        // Derive key
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;

        Ok(key)
    }

    /// Generate a random salt for key derivation
    pub fn generate_salt() -> Vec<u8> {
        let salt = SaltString::generate(&mut OsRng);
        salt.as_str().as_bytes().to_vec()
    }
}

/// Secreton integration for key management
#[cfg(feature = "secreton")]
pub mod secreton_integration {
    use super::*;

    /// Secreton client trait for key storage
    // Internal trait used only behind `&dyn`/generics in-crate; `async fn` here
    // is fine and keeps the signatures readable.
    #[allow(async_fn_in_trait)]
    pub trait SecretonClient: Send + Sync {
        /// Store a key in Secreton
        async fn store_key(&self, path: &str, key: &[u8]) -> Result<()>;

        /// Retrieve a key from Secreton
        async fn get_key(&self, path: &str) -> Result<Vec<u8>>;

        /// Delete a key from Secreton
        async fn delete_key(&self, path: &str) -> Result<()>;
    }

    /// Encryption service with Secreton integration
    pub struct SecretonEncryptionService<C: SecretonClient> {
        secreton_client: C,
        key_path: String,
    }

    impl<C: SecretonClient> SecretonEncryptionService<C> {
        /// Create a new encryption service with Secreton integration
        ///
        /// # Arguments
        /// * `secreton_client` - Secreton client for key storage
        /// * `key_path` - Path to store/retrieve the encryption key in Secreton
        pub fn new(secreton_client: C, key_path: String) -> Self {
            Self {
                secreton_client,
                key_path,
            }
        }

        /// Initialize with a new random key and store in Secreton
        pub async fn initialize(&self) -> Result<()> {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            self.secreton_client
                .store_key(&self.key_path, key.as_slice())
                .await
                .map_err(|e| EncryptionError::SecretonError(e.to_string()))?;
            Ok(())
        }

        /// Get encryption service with key from Secreton
        pub async fn get_service(&self) -> Result<EncryptionService> {
            let key = self
                .secreton_client
                .get_key(&self.key_path)
                .await
                .map_err(|e| EncryptionError::SecretonError(e.to_string()))?;

            EncryptionService::new(&key)
        }

        /// Rotate encryption key in Secreton
        pub async fn rotate_key(&self) -> Result<()> {
            let new_key = ChaCha20Poly1305::generate_key(&mut OsRng);
            self.secreton_client
                .store_key(&self.key_path, new_key.as_slice())
                .await
                .map_err(|e| EncryptionError::SecretonError(e.to_string()))?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = b"Hello, World!";

        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = b"";

        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_large_data() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = vec![0u8; 1024 * 1024]; // 1 MB

        let encrypted = service.encrypt(&plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_to_base64() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = b"Secret message";

        let encoded = service.encrypt_to_base64(plaintext).unwrap();
        let decrypted = service.decrypt_from_base64(&encoded).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_invalid_key_length() {
        let result = EncryptionService::new(&[0u8; 16]);
        assert!(result.is_err());
        if let Err(EncryptionError::InvalidKeyLength(16)) = result {
            // Test passed
        } else {
            panic!("Expected InvalidKeyLength(16) error");
        }
    }

    #[test]
    fn test_invalid_nonce_length() {
        let service = EncryptionService::new_with_random_key();
        let encrypted = EncryptedData {
            ciphertext: vec![0u8; 32],
            nonce: vec![0u8; 8], // Invalid nonce length
        };

        let result = service.decrypt(&encrypted);
        assert!(result.is_err());
        if let Err(EncryptionError::InvalidNonceLength(8)) = result {
            // Test passed
        } else {
            panic!("Expected InvalidNonceLength(8) error");
        }
    }

    #[test]
    fn test_decryption_with_wrong_key() {
        let service1 = EncryptionService::new_with_random_key();
        let service2 = EncryptionService::new_with_random_key();

        let plaintext = b"Secret";
        let encrypted = service1.encrypt(plaintext).unwrap();

        // Decryption with wrong key should fail
        let result = service2.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_derivation() {
        let password = "my-secure-password";
        let salt = key_derivation::generate_salt();

        let key1 = key_derivation::derive_key_from_password(password, &salt).unwrap();
        let key2 = key_derivation::derive_key_from_password(password, &salt).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1, key2);

        // Different salt should produce different key
        let different_salt = key_derivation::generate_salt();
        let key3 = key_derivation::derive_key_from_password(password, &different_salt).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_key_derivation_different_passwords() {
        let salt = key_derivation::generate_salt();

        let key1 = key_derivation::derive_key_from_password("password1", &salt).unwrap();
        let key2 = key_derivation::derive_key_from_password("password2", &salt).unwrap();

        // Different passwords should produce different keys
        assert_ne!(key1, key2);
    }
}
