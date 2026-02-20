//! Cryptographic operations - Re-export from authenc-crypto crate
//!
//! This module provides backward compatibility by re-exporting
//! all cryptographic functionality from the authenc-crypto crate.

// Re-export everything from authenc-crypto
pub use authenc_crypto::*;

// Additional re-exports for convenience (if needed)
pub use authenc_crypto::{
    // JWT
    jwt::{JwtService, TokenClaims},
    jwt_key_manager::{JwtKey, JwtKeyManager, SecretonClient as JwtSecretonClient},
    jwt_validator::{JwtValidator, ValidationResult, ValidationCache},

    // Password
    password::Argon2PasswordHasher,

    // Encryption
    encryption::{EncryptionService, EncryptedData, EncryptionError},
    aes_gcm::{AesGcmService, EncryptedData as AesEncryptedData},

    // Enhanced crypto
    enhanced::{
        EnhancedCryptoEngine, PegawaiClaims, SecretonPermissions, UserClaims,
        EncryptedSessionData, SessionMetadata, AuditSignature, SignerInfo,
        BatchValidationRequest, BatchValidationResponse, TokenValidationResult,
        CryptoMetrics, OperationMetrics, PostQuantumMode,
    },

    // Keys
    keys::{
        ED25519_KEYPAIR, Ed25519Jwk, Ed25519JwkSet,
        ECDSA_KEYPAIR, EcdsaJwk, EcdsaJwkSet,
        ECDSA_P384_KEYPAIR, EcdsaP384Jwk, EcdsaP384JwkSet,
        ECDSA_P521_KEYPAIR, EcdsaP521Jwk, EcdsaP521JwkSet,
        EDDSA_KEYPAIR, EddsaJwk, EddsaJwkSet,
    },
};
