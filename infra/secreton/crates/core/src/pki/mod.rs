//! PKI (Public Key Infrastructure) module
//!
//! This module provides PKI-specific functionality including certificate management,
//! private key operations, and signature algorithms. General cryptographic operations
//! are re-exported from the secreton-crypto crate.
//!
//! Note: This module was renamed from 'crypto' to 'pki' in v1.1.0 to better reflect
//! its actual purpose (PKI operations, not general cryptography).

pub mod pki;

pub use pki::{Certificate, PrivateKey, SignatureAlgorithm};

// Re-export all crypto functionality from secreton-crypto crate
pub use secreton_crypto::{
    // Encryption
    encryption::{
        self, Aes256GcmCipher, ChaCha20Poly1305Cipher, CryptoEngine, EncryptedData, SymmetricCipher,
    },

    // Utility functions
    generate_key,
    generate_nonce,
    generate_random_bytes,
    // Hashing
    hashing::{
        self, compute_hash, compute_hash_multiple, compute_hmac_sha256, verify_hmac_sha256,
        Blake3Hash, HashResult, Sha256Hash, Sha3_256Hash,
    },

    // Key derivation
    key_derivation::{self, derive_key, DerivedKey, KdfParams},

    // Post-quantum cryptography
    pqc::{
        self, AlgorithmCharacteristics, HybridSignature, PQCConfig, PQCError, PQCRegistry,
        PQCResult,
    },

    // Shamir secret sharing
    shamir::{
        self, generate_shares, generate_shares_with_commitments, reconstruct_secret,
        reconstruct_secret_verified, verify_share_with_commitment, verify_shares,
        verify_shares_batch, Commitment, ShamirConfig, ShamirError, Share,
    },

    // Storage integration
    storage_integration::{
        CryptoStorageBridge, EncryptedVaultEntry, EncryptionMetadata as CryptoEncryptionMetadata,
        KeyInfo,
    },

    // Transit engine
    transit::{
        self, algorithms, batch, keys, operations, policies, AuditLogger, CreateKeyRequest,
        CreateKeyResponse, DecryptRequest, DecryptResponse, DefaultAuditLogger, DeriveKeyRequest,
        DeriveKeyResponse, EncryptRequest, EncryptResponse, KeyOptions, KeyType, KeyUsage,
        OperationStats, RandomFormat, RandomRequest, RandomResponse, RotateKeyRequest,
        RotateKeyResponse, SignRequest, SignResponse, TransitEngine, TransitKey, TransitOperations,
        VerifyRequest, VerifyResponse,
    },
    // Core types
    AlgorithmId,
    CryptoError,
    CryptoResult,
    EncryptionService,
    SecurityParams,
};

// Convenience aliases for backward compatibility
pub use secreton_crypto::encryption::CryptoEngine as CryptoService;
pub use secreton_crypto::hashing::password::{hash_password_argon2, verify_password_argon2};
