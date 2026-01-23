//! Crypto prelude - commonly used cryptographic types and traits
//!
//! This module re-exports the most commonly used items from the crypto crate
//! to simplify imports in other modules.

// Re-export error types
pub use crate::error::{CryptoError, CryptoResult};

// Re-export core types
pub use crate::{AlgorithmId, SecurityParams};

// Re-export encryption types
pub use crate::encryption::{
    Aes256GcmCipher, ChaCha20Poly1305Cipher, CryptoEngine, EncryptedData, SymmetricCipher,
};

// Re-export key derivation
pub use crate::key_derivation::{
    DerivedKey, KdfParams, derive_key, derive_key_argon2id, derive_key_pbkdf2,
};

// Re-export transit engine
pub use crate::transit::{
    CreateKeyRequest, CreateKeyResponse, DecryptRequest, DecryptResponse, EncryptRequest,
    EncryptResponse, KeyType, TransitEngine, TransitKey,
};

// Re-export storage integration
pub use crate::storage_integration::{
    CryptoStorageBridge, EncryptedSecretEntry, EncryptionMetadata, KeyInfo,
};

// Re-export post-quantum crypto
pub use crate::pqc::{
    AlgorithmCharacteristics, HybridSignature, PQCConfig, PQCError, PQCRegistry, PQCResult,
    PostQuantumKeyExchange, PostQuantumSignatures,
};

// Re-export Shamir secret sharing
pub use crate::shamir::{Commitment, ShamirConfig, Share};

// Re-export commonly used external crates
pub use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};
pub use zeroize::{Zeroize, ZeroizeOnDrop};
