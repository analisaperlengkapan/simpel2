//! Secreton Cryptographic Library
//!
//! High-performance, post-quantum ready cryptographic primitives for the Secreton vault.
//! This crate provides secure, audited implementations of encryption, hashing, key derivation,
//! and hybrid classical/post-quantum algorithms.
//!
//! # Modules
//!
//! ## Core Cryptography
//! - [`encryption`] - Symmetric encryption (AES-256-GCM, ChaCha20-Poly1305)
//! - [`hashing`] - Cryptographic hashing (BLAKE3, SHA-256, SHA-512, Argon2)
//! - [`key_derivation`] - Key derivation functions (HKDF, PBKDF2)
//! - [`signing`] - Digital signatures (Ed25519, ECDSA)
//!
//! ## Post-Quantum Cryptography
//! - [`hybrid`] - Hybrid classical/post-quantum encryption (X25519 + ML-KEM)
//! - [`pq_key_management`] - Post-quantum key management (ML-KEM-768/1024)
//!
//! ## Vault Features
//! - [`shamir`] - Shamir's Secret Sharing (vault seal/unseal)
//! - [`transit`] - Transit secrets engine (encrypt-as-a-service)
//! - [`kv_engine`] - Key-value encryption engine
//!
//! # Security Guarantees
//!
//! - **Memory Safety**: All sensitive data uses [`zeroize`] for secure memory clearing
//! - **Constant Time**: Timing-safe operations prevent side-channel attacks
//! - **Validated Algorithms**: Uses audited RustCrypto implementations
//! - **Post-Quantum Ready**: Hybrid mode combines classical + PQ security
//!
//! # Supported Algorithms
//!
//! ## Symmetric Encryption
//! - AES-256-GCM (NIST standard, hardware accelerated)
//! - ChaCha20-Poly1305 (RFC 8439, fast software implementation)
//!
//! ## Asymmetric Encryption
//! - X25519 ECDH (classical, high performance)
//! - ML-KEM-768 (NIST PQC standard, 192-bit security)
//! - ML-KEM-1024 (NIST PQC standard, 256-bit security)
//! - Hybrid X25519+ML-KEM (best of both worlds)
//!
//! ## Hashing
//! - BLAKE3 (fastest, parallelizable)
//! - SHA-256/512 (NIST standard)
//! - Argon2id (password hashing, memory-hard)
//!
//! ## Digital Signatures
//! - Ed25519 (fast, compact signatures)
//! - ECDSA P-256/P-384 (NIST curves)
//!
//! # Example: Encrypting Data
//!
//! ```rust,no_run
//! use secreton_crypto::{CryptoEngine, SecurityParams};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let engine = CryptoEngine::new(SecurityParams::default())?;
//!
//! let plaintext = b"super secret data";
//! let encrypted = engine.encrypt(plaintext)?;
//!
//! let decrypted = engine.decrypt(&encrypted)?;
//! assert_eq!(plaintext, &decrypted[..]);
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Post-Quantum Hybrid Encryption
//!
//! ```rust,no_run
//! use secreton_crypto::hybrid::HybridCrypto;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let alice = HybridCrypto::generate_keypair()?;
//! let bob = HybridCrypto::generate_keypair()?;
//!
//! let plaintext = b"message from alice to bob";
//! let encrypted = alice.encrypt(&bob.public_key, plaintext)?;
//!
//! let decrypted = bob.decrypt(&encrypted)?;
//! assert_eq!(plaintext, &decrypted[..]);
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! Benchmarks on modern x86_64 CPU:
//! - AES-256-GCM: ~3 GB/s (hardware AES-NI)
//! - ChaCha20-Poly1305: ~1 GB/s
//! - BLAKE3: ~5 GB/s
//! - Ed25519 signing: ~100k ops/sec
//! - ML-KEM-768: ~10k encaps/sec
//!
//! # Standards Compliance
//!
//! - FIPS 140-3 validated algorithms (AES, SHA-2, ECDSA)
//! - NIST SP 800-108 (key derivation)
//! - NIST SP 800-56A (ECDH)
//! - NIST PQC Round 3 (ML-KEM)
//! - RFC 8439 (ChaCha20-Poly1305)
//! - RFC 8032 (Ed25519)

use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fmt;

// Re-export zeroize for use by other crates
pub use zeroize;

pub mod encryption;
pub mod error;
pub mod fpe;
pub mod hashing;
pub mod hybrid;
pub mod key_derivation;
pub mod kv_engine;
pub mod pq_key_management;
pub mod pqc;
pub mod prelude;
pub mod shamir;
pub mod storage_integration;
pub mod transit;
pub mod utils;

// Auth crypto modules (from authenc)
pub mod aes_gcm;
pub mod ecdsa_keys;
pub mod ecdsa_p384_keys;
pub mod ecdsa_p521_keys;
pub mod ed25519_keys;
pub mod eddsa_ed448_keys;
pub mod mtls;

pub use encryption::{
    Aes256GcmCipher, ChaCha20Poly1305Cipher, CryptoEngine, EncryptedData, SymmetricCipher,
};
pub use fpe::{FpeAlphabet, FpeEngine, FpeError, FpeKey};
pub use hybrid::{
    CryptoMode, HybridCrypto, HybridEncryptedData, HybridEncryptionMetadata, HybridPublicKeys,
    HybridSignatureData, MigrationPhase, MigrationStrategy, PerformancePriority,
    SecurityRequirements,
};

// Alias for backward compatibility
pub type EncryptionService = CryptoEngine;
pub use error::*;
pub use key_derivation::{
    DerivedKey, KdfParams, derive_key, derive_key_argon2id, derive_key_pbkdf2, presets, stretch,
};
pub use kv_engine::*;
pub use pq_key_management::{
    ArchiveEncryptionResult, ArchiveKeyEntry, ArchivePurpose, HybridKeyEntry,
    HybridKeyExchangeResult, KeyPurpose, KeyStatus, MLDsaKeyEntry, MLKemKeyEntry,
    PostQuantumKeyManager,
};
pub use pqc::*;
pub use shamir::*;
pub use storage_integration::{
    CryptoStorageBridge, EncryptedVaultEntry, EncryptionMetadata, KeyInfo,
};
pub use transit::{
    AuditLogger,
    CreateKeyRequest,
    CreateKeyResponse,
    DecryptRequest,
    DecryptResponse,
    DefaultAuditLogger,
    DeriveKeyRequest,
    DeriveKeyResponse,
    EncryptRequest,
    EncryptResponse,
    // KeyInfo, // Removed duplicate - already exported from storage_integration
    KeyOptions,
    KeyType,
    KeyUsage,
    OperationStats,
    RandomFormat,
    RandomRequest,
    RandomResponse,
    RotateKeyRequest,
    RotateKeyResponse,
    SignRequest,
    SignResponse,
    TransitEngine,
    TransitKey,
    // Structs from operations
    TransitOperations,
    VerifyRequest,
    VerifyResponse,
    algorithms,
    batch,
    // Functions from algorithms
    constant_time_eq,
    derive_key as transit_derive_key,
    generate_random,
    generate_salt,
    hash_data,
    keys,
    operations,
    policies,
};

/// Supported cryptographic algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlgorithmId {
    // Symmetric encryption
    Aes256Gcm,
    ChaCha20Poly1305,

    // Hash functions
    Sha256,
    Sha3_256,
    Blake3,

    // Key derivation
    Pbkdf2,
    Argon2id,
}

impl fmt::Display for AlgorithmId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            AlgorithmId::Aes256Gcm => "AES-256-GCM",
            AlgorithmId::ChaCha20Poly1305 => "ChaCha20-Poly1305",
            AlgorithmId::Sha256 => "SHA-256",
            AlgorithmId::Sha3_256 => "SHA3-256",
            AlgorithmId::Blake3 => "BLAKE3",
            AlgorithmId::Pbkdf2 => "PBKDF2",
            AlgorithmId::Argon2id => "Argon2id",
        };
        write!(f, "{}", name)
    }
}

/// Security parameters for cryptographic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityParams {
    pub algorithm: AlgorithmId,
    pub key_size: usize,
    pub iterations: Option<u32>,
    pub salt_size: Option<usize>,
}

impl SecurityParams {
    /// Create security parameters for a given algorithm
    pub fn new(algorithm: AlgorithmId) -> Self {
        let (key_size, iterations, salt_size) = match algorithm {
            AlgorithmId::Aes256Gcm => (32, None, Some(12)),
            AlgorithmId::ChaCha20Poly1305 => (32, None, Some(12)),
            AlgorithmId::Sha256 => (32, None, None),
            AlgorithmId::Sha3_256 => (32, None, None),
            AlgorithmId::Blake3 => (32, None, None),
            AlgorithmId::Pbkdf2 => (32, Some(100_000), Some(16)),
            AlgorithmId::Argon2id => (32, Some(3), Some(16)),
        };

        Self {
            algorithm,
            key_size,
            iterations,
            salt_size,
        }
    }

    /// Check if parameters are secure for production use
    pub fn is_secure(&self) -> bool {
        match self.algorithm {
            AlgorithmId::Aes256Gcm | AlgorithmId::ChaCha20Poly1305 => self.key_size >= 32,
            AlgorithmId::Pbkdf2 => self.iterations.unwrap_or(0) >= 100_000 && self.key_size >= 32,
            AlgorithmId::Argon2id => self.iterations.unwrap_or(0) >= 3 && self.key_size >= 32,
            _ => true,
        }
    }
}

/// Generate cryptographically secure random bytes
pub fn generate_random_bytes(len: usize) -> CryptoResult<Vec<u8>> {
    let mut bytes = vec![0u8; len];
    let mut rng = OsRng;
    rng.try_fill_bytes(&mut bytes)
        .map_err(|_| CryptoError::RandomGenerationFailed)?;
    Ok(bytes)
}

/// Generate a random key for the specified algorithm
pub fn generate_key(algorithm: AlgorithmId) -> CryptoResult<Vec<u8>> {
    let params = SecurityParams::new(algorithm);
    generate_random_bytes(params.key_size)
}

/// Generate a random nonce/IV for the specified algorithm
pub fn generate_nonce(algorithm: AlgorithmId) -> CryptoResult<Vec<u8>> {
    let params = SecurityParams::new(algorithm);
    if let Some(nonce_size) = params.salt_size {
        generate_random_bytes(nonce_size)
    } else {
        Err(CryptoError::InvalidNonceLength)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_params_for_symmetric_algorithms() {
        let aes_params = SecurityParams::new(AlgorithmId::Aes256Gcm);
        assert_eq!(aes_params.key_size, 32);
        assert_eq!(aes_params.iterations, None);
        assert_eq!(aes_params.salt_size, Some(12));
        assert!(aes_params.is_secure());

        let chacha_params = SecurityParams::new(AlgorithmId::ChaCha20Poly1305);
        assert_eq!(chacha_params.key_size, 32);
        assert!(chacha_params.is_secure());
    }

    #[test]
    fn test_security_params_for_kdf_algorithms() {
        let pbkdf2_params = SecurityParams::new(AlgorithmId::Pbkdf2);
        assert_eq!(pbkdf2_params.iterations, Some(100_000));
        assert!(pbkdf2_params.is_secure());

        let mut unsafe_pbkdf2 = pbkdf2_params.clone();
        unsafe_pbkdf2.iterations = Some(10_000);
        assert!(!unsafe_pbkdf2.is_secure());

        let argon_params = SecurityParams::new(AlgorithmId::Argon2id);
        assert_eq!(argon_params.iterations, Some(3));
        assert!(argon_params.is_secure());

        let mut unsafe_argon = argon_params.clone();
        unsafe_argon.iterations = Some(1);
        assert!(!unsafe_argon.is_secure());
    }

    #[test]
    fn test_generate_key_lengths_match_algorithm_requirements() {
        let aes_key = generate_key(AlgorithmId::Aes256Gcm).expect("AES key generation failed");
        assert_eq!(aes_key.len(), 32);

        let argon_key = generate_key(AlgorithmId::Argon2id).expect("Argon2 key generation failed");
        assert_eq!(argon_key.len(), 32);
    }

    #[test]
    fn test_generate_nonce_respects_algorithm_requirements() {
        let aes_nonce = generate_nonce(AlgorithmId::Aes256Gcm).expect("nonce generation failed");
        assert_eq!(aes_nonce.len(), 12);

        let err =
            generate_nonce(AlgorithmId::Sha256).expect_err("expected nonce generation to fail");
        assert_eq!(err, CryptoError::InvalidNonceLength);
    }

    #[test]
    fn test_generate_random_bytes_produces_requested_length() {
        let bytes = generate_random_bytes(64).expect("random generation failed");
        assert_eq!(bytes.len(), 64);
    }
}
