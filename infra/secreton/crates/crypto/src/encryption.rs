//! Symmetric encryption implementations

use crate::{AlgorithmId, CryptoError, CryptoResult, generate_random_bytes};
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use chacha20poly1305::ChaCha20Poly1305;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub algorithm: AlgorithmId,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub tag: Option<Vec<u8>>,
}

/// Symmetric encryption trait
pub trait SymmetricCipher {
    fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> CryptoResult<EncryptedData>;
    fn decrypt(&self, encrypted: &EncryptedData, key: &[u8]) -> CryptoResult<Vec<u8>>;
}

/// AES-256-GCM implementation
pub struct Aes256GcmCipher;

impl SymmetricCipher for Aes256GcmCipher {
    fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> CryptoResult<EncryptedData> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: key.len(),
            });
        }

        let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKeyLength {
            expected: 32,
            actual: key.len(),
        })?;

        // Generate random nonce
        let nonce_bytes = generate_random_bytes(12)?;
        let nonce = Nonce::from_slice(&nonce_bytes[..12]);

        let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|e| {
            CryptoError::EncryptionFailed(format!("AES-GCM encryption failed: {}", e))
        })?;

        Ok(EncryptedData {
            algorithm: AlgorithmId::Aes256Gcm,
            nonce: nonce_bytes,
            ciphertext,
            tag: None, // Tag is included in ciphertext for GCM
        })
    }

    fn decrypt(&self, encrypted: &EncryptedData, key: &[u8]) -> CryptoResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: key.len(),
            });
        }

        if encrypted.nonce.len() != 12 {
            return Err(CryptoError::InvalidNonceLength);
        }

        let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKeyLength {
            expected: 32,
            actual: key.len(),
        })?;
        let nonce = Nonce::from_slice(&encrypted.nonce[..12]);

        cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| CryptoError::DecryptionFailed(format!("AES-GCM decryption failed: {}", e)))
    }
}

/// ChaCha20-Poly1305 implementation
pub struct ChaCha20Poly1305Cipher;

impl SymmetricCipher for ChaCha20Poly1305Cipher {
    fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> CryptoResult<EncryptedData> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: key.len(),
            });
        }

        let cipher =
            ChaCha20Poly1305::new_from_slice(key).map_err(|_| CryptoError::InvalidKeyLength {
                expected: 32,
                actual: key.len(),
            })?;

        // Generate random nonce
        let nonce_bytes = generate_random_bytes(12)?;
        let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes[..12]);

        let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|e| {
            CryptoError::EncryptionFailed(format!("ChaCha20-Poly1305 encryption failed: {}", e))
        })?;

        Ok(EncryptedData {
            algorithm: AlgorithmId::ChaCha20Poly1305,
            nonce: nonce_bytes,
            ciphertext,
            tag: None, // Tag is included in ciphertext for Poly1305
        })
    }

    fn decrypt(&self, encrypted: &EncryptedData, key: &[u8]) -> CryptoResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: key.len(),
            });
        }

        if encrypted.nonce.len() != 12 {
            return Err(CryptoError::InvalidNonceLength);
        }

        let cipher =
            ChaCha20Poly1305::new_from_slice(key).map_err(|_| CryptoError::InvalidKeyLength {
                expected: 32,
                actual: key.len(),
            })?;
        let nonce = chacha20poly1305::Nonce::from_slice(&encrypted.nonce[..12]);

        cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| {
                CryptoError::DecryptionFailed(format!("ChaCha20-Poly1305 decryption failed: {}", e))
            })
    }
}

/// Unified encryption interface with optional signing capability
pub struct CryptoEngine {
    /// Ed25519 signing key (generated lazily on first use)
    signing_key: RwLock<Option<SigningKey>>,
}

impl Default for CryptoEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoEngine {
    pub fn new() -> Self {
        Self {
            signing_key: RwLock::new(None),
        }
    }

    /// Initialize or get the signing key
    fn get_or_create_signing_key(&self) -> CryptoResult<SigningKey> {
        // Try to read existing key
        {
            let read_guard = self.signing_key.read().map_err(|_| {
                CryptoError::Internal("Failed to acquire signing key lock".to_string())
            })?;
            if let Some(key) = read_guard.as_ref() {
                return Ok(key.clone());
            }
        }
        // Generate new key
        let mut write_guard = self
            .signing_key
            .write()
            .map_err(|_| CryptoError::Internal("Failed to acquire signing key lock".to_string()))?;
        if let Some(key) = write_guard.as_ref() {
            return Ok(key.clone());
        }
        let key_bytes = generate_random_bytes(32)?;
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);
        let key = SigningKey::from_bytes(&key_array);
        *write_guard = Some(key.clone());
        Ok(key)
    }

    pub fn encrypt(
        &self,
        algorithm: AlgorithmId,
        plaintext: &[u8],
        key: &[u8],
    ) -> CryptoResult<EncryptedData> {
        match algorithm {
            AlgorithmId::Aes256Gcm => {
                let cipher = Aes256GcmCipher;
                cipher.encrypt(plaintext, key)
            }
            AlgorithmId::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305Cipher;
                cipher.encrypt(plaintext, key)
            }
            _ => Err(CryptoError::EncryptionFailed(format!(
                "Unsupported encryption algorithm: {}",
                algorithm
            ))),
        }
    }

    pub fn decrypt(&self, encrypted: &EncryptedData, key: &[u8]) -> CryptoResult<Vec<u8>> {
        match encrypted.algorithm {
            AlgorithmId::Aes256Gcm => {
                let cipher = Aes256GcmCipher;
                cipher.decrypt(encrypted, key)
            }
            AlgorithmId::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305Cipher;
                cipher.decrypt(encrypted, key)
            }
            _ => Err(CryptoError::DecryptionFailed(format!(
                "Unsupported decryption algorithm: {}",
                encrypted.algorithm
            ))),
        }
    }

    /// Encrypt data using a self-managed ephemeral key
    /// Returns (encrypted_data, key) - key must be stored securely for decryption
    /// This is a convenience method for simple encryption scenarios
    pub fn encrypt_with_new_key(&self, plaintext: &[u8]) -> CryptoResult<(EncryptedData, Vec<u8>)> {
        let key = crate::generate_key(AlgorithmId::ChaCha20Poly1305)?;
        let encrypted = self.encrypt(AlgorithmId::ChaCha20Poly1305, plaintext, &key)?;
        Ok((encrypted, key))
    }

    /// Simple encrypt method for backward compatibility
    /// Generates an ephemeral key and includes it encrypted with the ciphertext
    /// WARNING: This is less secure than proper key management - use for non-sensitive data only
    pub fn encrypt_simple(&self, plaintext: &[u8]) -> CryptoResult<Vec<u8>> {
        let (encrypted, key) = self.encrypt_with_new_key(plaintext)?;
        // Serialize encrypted data with key prefix (simple format for internal use)
        let mut result =
            Vec::with_capacity(32 + encrypted.nonce.len() + encrypted.ciphertext.len());
        result.extend_from_slice(&key); // 32 bytes for ChaCha20
        result.extend_from_slice(&encrypted.nonce);
        result.extend_from_slice(&encrypted.ciphertext);
        Ok(result)
    }

    /// Simple decrypt for data encrypted with encrypt_simple
    pub fn decrypt_simple(&self, data: &[u8]) -> CryptoResult<Vec<u8>> {
        if data.len() < 44 {
            // 32 (key) + 12 (nonce) minimum
            return Err(CryptoError::DecryptionFailed(
                "Data too short for decrypt_simple".to_string(),
            ));
        }
        let key = &data[..32];
        let nonce = data[32..44].to_vec();
        let ciphertext = data[44..].to_vec();
        let encrypted = EncryptedData {
            algorithm: AlgorithmId::ChaCha20Poly1305,
            nonce,
            ciphertext,
            tag: None,
        };
        self.decrypt(&encrypted, key)
    }

    /// Sign data using Ed25519
    /// Returns the 64-byte signature
    pub fn sign(&self, message: &[u8]) -> CryptoResult<Vec<u8>> {
        let signing_key = self.get_or_create_signing_key()?;
        let signature = signing_key.sign(message);
        Ok(signature.to_bytes().to_vec())
    }

    /// Verify an Ed25519 signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> CryptoResult<bool> {
        let signing_key = self.get_or_create_signing_key()?;
        let verifying_key = signing_key.verifying_key();

        if signature.len() != 64 {
            return Err(CryptoError::VerificationFailed(
                "Invalid signature length: expected 64 bytes".to_string(),
            ));
        }

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature);
        let sig = Signature::from_bytes(&sig_bytes);

        match verifying_key.verify(message, &sig) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_key;

    #[test]
    fn test_aes256gcm_encryption_roundtrip() {
        let engine = CryptoEngine::new();
        let key = generate_key(AlgorithmId::Aes256Gcm).unwrap();
        let plaintext = b"Hello, Secreton Security System!";

        let encrypted = engine
            .encrypt(AlgorithmId::Aes256Gcm, plaintext, &key)
            .unwrap();
        let decrypted = engine.decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
        assert_eq!(encrypted.algorithm, AlgorithmId::Aes256Gcm);
        assert_eq!(encrypted.nonce.len(), 12);
    }

    #[test]
    fn test_chacha20poly1305_encryption_roundtrip() {
        let engine = CryptoEngine::new();
        let key = generate_key(AlgorithmId::ChaCha20Poly1305).unwrap();
        let plaintext = b"ChaCha20-Poly1305 test message";

        let encrypted = engine
            .encrypt(AlgorithmId::ChaCha20Poly1305, plaintext, &key)
            .unwrap();
        let decrypted = engine.decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
        assert_eq!(encrypted.algorithm, AlgorithmId::ChaCha20Poly1305);
        assert_eq!(encrypted.nonce.len(), 12);
    }

    #[test]
    fn test_wrong_key_length() {
        let engine = CryptoEngine::new();
        let short_key = vec![0u8; 16]; // Too short
        let plaintext = b"test";

        let result = engine.encrypt(AlgorithmId::Aes256Gcm, plaintext, &short_key);
        assert!(result.is_err());

        match result.unwrap_err() {
            CryptoError::InvalidKeyLength { expected, actual } => {
                assert_eq!(expected, 32);
                assert_eq!(actual, 16);
            }
            _ => panic!("Expected InvalidKeyLength error"),
        }
    }

    #[test]
    fn test_error_display() {
        let err = CryptoError::InvalidKeyLength {
            expected: 32,
            actual: 16,
        };
        assert!(err.to_string().contains("32"));
    }
}
