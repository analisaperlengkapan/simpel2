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

// Re-export all crypto functionality from lib-crypto crate
pub use lib_crypto::{
    // Core types
    AlgorithmId,
    CryptoError,
    CryptoResult,
    EncryptionService,
    SecurityParams,
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
        self, Blake3Hash, HashResult, Sha3_256Hash, Sha256Hash, compute_hash,
        compute_hash_multiple, compute_hmac_sha256, verify_hmac_sha256,
    },

    // Key derivation
    key_derivation::{self, DerivedKey, KdfParams, derive_key},

    // Post-quantum cryptography
    pqc::{
        self, AlgorithmCharacteristics, HybridSignature, PQCConfig, PQCError, PQCRegistry,
        PQCResult,
    },

    // Shamir secret sharing
    shamir::{
        self, Commitment, ShamirConfig, ShamirError, Share, generate_shares,
        generate_shares_with_commitments, reconstruct_secret, reconstruct_secret_verified,
        verify_share_with_commitment, verify_shares, verify_shares_batch,
    },

    // Storage integration
    storage_integration::{
        CryptoStorageBridge, EncryptedVaultEntry, EncryptionMetadata as CryptoEncryptionMetadata,
        KeyInfo,
    },

    // Transit engine
    transit::{
        self, AuditLogger, CreateKeyRequest, CreateKeyResponse, DecryptRequest, DecryptResponse,
        DefaultAuditLogger, DeriveKeyRequest, DeriveKeyResponse, EncryptRequest, EncryptResponse,
        KeyOptions, KeyType, KeyUsage, OperationStats, RandomFormat, RandomRequest, RandomResponse,
        RotateKeyRequest, RotateKeyResponse, SignRequest, SignResponse, TransitEngine, TransitKey,
        TransitOperations, VerifyRequest, VerifyResponse, algorithms, batch, keys, operations,
        policies,
    },
};

// Convenience aliases for backward compatibility
pub use lib_crypto::encryption::CryptoEngine as CryptoService;
pub use lib_crypto::hashing::password::{hash_password_argon2, verify_password_argon2};
