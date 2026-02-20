//! # authenc-crypto
//!
//! Comprehensive cryptographic operations for the Authenc identity provider.
//!
//! This crate provides a complete cryptographic toolkit for enterprise-grade
//! authentication and authorization systems, with support for modern standards
//! and post-quantum cryptography.
//!
//! ## Core Features
//!
//! ### JWT Operations
//! - **JWT Generation & Validation**: Ed25519, ECDSA (P-256, P-384, P-521), EdDSA signatures
//! - **Key Management**: Dynamic key rotation, JWK/JWKS support, Secreton integration
//! - **Validation**: Token validation with caching, batch validation support
//!
//! ### Encryption
//! - **Symmetric**: ChaCha20-Poly1305, AES-256-GCM
//! - **Password Hashing**: Argon2id with configurable parameters
//! - **Enhanced Engine**: SIMKARI-specific operations with audit trails
//!
//! ### Authentication
//! - **TOTP**: Time-based One-Time Password generation and verification
//! - **mTLS**: Mutual TLS authentication support
//! - **DPoP**: Demonstrating Proof-of-Possession (RFC 9449) for FAPI-2 compliance
//!
//! ### Advanced Features
//! - **Post-Quantum Cryptography**: ML-DSA, ML-KEM, Falcon algorithms
//! - **Selective Disclosure JWT**: Privacy-preserving identity credentials
//! - **XML Digital Signatures**: SAML support with xmldsig
//! - **Shamir Secret Sharing**: Key splitting and reconstruction
//!
//! ## Module Organization
//!
//! ### Core Cryptography (`jwt`, `encryption`, `password`, `totp`)
//! Basic cryptographic operations for authentication and data protection.
//!
//! ### Key Management (`keys`, `jwt_key_manager`)
//! Multi-algorithm key management with support for Ed25519, ECDSA variants, and EdDSA.
//!
//! ### Advanced Protocols (`dpop`, `sdjwt`, `xmldsig`, `mtls`)
//! Modern authentication protocols and standards compliance.
//!
//! ### Post-Quantum (`pqc`)
//! Quantum-resistant cryptographic algorithms for future-proof security.
//!
//! ### Enhanced Operations (`enhanced`, `aes_gcm`)
//! SIMKARI-specific cryptographic operations with audit and compliance features.
//!
//! ## Usage Examples
//!
//! ### JWT Generation
//! ```rust,no_run
//! use authenc_crypto::{JwtService, TokenClaims};
//!
//! let jwt_service = JwtService::new();
//! let claims = TokenClaims {
//!     sub: "user123".to_string(),
//!     // ... other claims
//! };
//! let token = jwt_service.generate_token(&claims)?;
//! # Ok::<(), authenc_types::error::AuthencError>(())
//! ```
//!
//! ### Password Hashing
//! ```rust,no_run
//! use authenc_crypto::Argon2PasswordHasher;
//!
//! let hasher = Argon2PasswordHasher::new();
//! let hash = hasher.hash_password("secure_password")?;
//! let is_valid = hasher.verify_password("secure_password", &hash)?;
//! # Ok::<(), authenc_types::error::AuthencError>(())
//! ```
//!
//! ### DPoP Proof Generation
//! ```rust,no_run
//! use authenc_crypto::dpop::DPoPProof;
//! use ed25519_dalek::SigningKey;
//!
//! let keypair = SigningKey::generate(&mut rand::rngs::OsRng);
//! let proof = DPoPProof::new(
//!     &keypair,
//!     "POST",
//!     "https://api.example.com/token",
//!     Some("access_token"),
//!     None,
//! )?;
//! # Ok::<(), authenc_types::error::AuthencError>(())
//! ```
//!
//! ## Feature Flags
//!
//! This crate supports the following feature flags:
//! - `pqc`: Enable post-quantum cryptography support (ML-DSA, ML-KEM, Falcon)
//! - `xmldsig`: Enable XML digital signature support for SAML
//! - `dpop`: Enable DPoP (Demonstrating Proof-of-Possession) support
//! - `sdjwt`: Enable Selective Disclosure JWT support
//!
//! ## Security Considerations
//!
//! - All cryptographic operations use constant-time algorithms where applicable
//! - Keys should be loaded from secure storage (Secreton) in production
//! - Password hashing uses Argon2id with recommended parameters
//! - JWT tokens should be validated on every request
//! - Post-quantum algorithms are experimental and should be used with classical algorithms

// Re-export types from authenc-types
pub use authenc_types::*;

// ============================================================================
// Core Cryptography Modules
// ============================================================================

/// JWT generation, signing, and validation
pub mod jwt;

/// JWT key management with Secreton integration
pub mod jwt_key_manager;

/// JWT token validation with caching
pub mod jwt_validator;

/// Password hashing with Argon2id
pub mod password;

/// Symmetric encryption (ChaCha20-Poly1305)
pub mod encryption;

/// TOTP (Time-based One-Time Password) generation and verification
pub mod totp;

/// AES-256-GCM encryption service
pub mod aes_gcm;

/// Enhanced cryptographic engine for SIMKARI operations
pub mod enhanced;

// ============================================================================
// Key Management
// ============================================================================

/// Multi-algorithm key management (Ed25519, ECDSA, EdDSA)
pub mod keys;

// ============================================================================
// Advanced Cryptography Modules
// ============================================================================

/// Post-Quantum Cryptography (ML-DSA, ML-KEM, Falcon)
pub mod pqc;

/// Mutual TLS authentication support
pub mod mtls;

/// XML Digital Signatures for SAML
pub mod xmldsig;

/// DPoP (Demonstrating Proof-of-Possession) for FAPI-2
// TODO: Fix compilation errors in dpop module (Task 4.7+)
// pub mod dpop;

/// Selective Disclosure JWT for privacy-preserving credentials
// TODO: Fix compilation errors in sdjwt module (Task 4.7+)
// pub mod sdjwt;

// ============================================================================
// Core Cryptography Re-exports
// ============================================================================

// JWT operations
pub use jwt::{JwtService, TokenClaims};
pub use jwt_key_manager::{JwtKey, JwtKeyManager, SecretonClient as JwtSecretonClient};
pub use jwt_validator::{JwtValidator, ValidationResult, ValidationCache};

// Encryption and hashing
pub use password::Argon2PasswordHasher;
pub use encryption::{EncryptionService, EncryptedData, EncryptionError};
pub use aes_gcm::{AesGcmService, EncryptedData as AesEncryptedData};

// Enhanced cryptographic engine
pub use enhanced::{
    EnhancedCryptoEngine, PegawaiClaims, SecretonPermissions, UserClaims,
    EncryptedSessionData, SessionMetadata, AuditSignature, SignerInfo,
    BatchValidationRequest, BatchValidationResponse, TokenValidationResult,
    CryptoMetrics, OperationMetrics, PostQuantumMode,
};

// ============================================================================
// Key Management Re-exports
// ============================================================================

pub use keys::{
    // Ed25519 (Primary algorithm)
    ED25519_KEYPAIR, Ed25519Jwk, Ed25519JwkSet,
    get_ed25519_jwk, get_ed25519_public_pem,
    sign_ed25519, verify_ed25519,
    generate_new_ed25519_keypair,

    // ECDSA P-256 (ES256)
    ECDSA_KEYPAIR, EcdsaJwk, EcdsaJwkSet,
    get_ecdsa_jwk, get_ecdsa_public_pem, get_ecdsa_private_pem,
    sign_ecdsa, verify_ecdsa,
    generate_new_p256_keypair,

    // ECDSA P-384 (ES384)
    ECDSA_P384_KEYPAIR, EcdsaP384Jwk, EcdsaP384JwkSet,
    sign_jwt_p384, verify_jwt_p384,
    get_p384_jwk_set,
    generate_new_p384_keypair,

    // ECDSA P-521 (ES512)
    ECDSA_P521_KEYPAIR, EcdsaP521Jwk, EcdsaP521JwkSet,
    sign_jwt_p521, verify_jwt_p521,
    get_p521_jwk_set,
    generate_new_p521_keypair,

    // EdDSA Ed448
    EDDSA_KEYPAIR, EddsaJwk, EddsaJwkSet,
    sign_jwt_eddsa, verify_jwt_eddsa,
    get_eddsa_jwk_set,
};

// ============================================================================
// Advanced Cryptography Re-exports
// ============================================================================

// DPoP (Demonstrating Proof-of-Possession)
// TODO: Fix compilation errors in dpop module (Task 4.7+)
// pub use dpop::{
//     DPoPProof, DPoPHeader, DPoPProofPayload,
//     DPoPNonceManager, DPoPTokenBinder,
// };

// Selective Disclosure JWT
// TODO: Fix compilation errors in sdjwt module (Task 4.7+)
// pub use sdjwt::{
//     SdJwt, SdJwtSalt, Disclosure, DisclosureSpec,
//     IssuerSignedJwt, SdJwtClaim, SdJwtArrayElement,
//     SdJwtFacade, SdJwtVerificationContext,
//     SdJwtUtils, VisibleSdJwtClaim,
// };

// Post-Quantum Cryptography (types available when 'quantum' feature is enabled)
// Note: PQC types are feature-gated and only available with the 'quantum' feature flag
#[cfg(feature = "quantum")]
pub use pqc::mldsa::{PublicKey as MlDsaPublicKey, SecretKey as MlDsaSecretKey};
#[cfg(feature = "quantum")]
pub use pqc::mlkem::{PublicKey as MlKemPublicKey, SecretKey as MlKemSecretKey};
#[cfg(feature = "quantum")]
pub use pqc::falcon::{PublicKey as FalconPublicKey, SecretKey as FalconSecretKey};

// mTLS
pub use mtls::{
    MtlsConfig, ClientCertInfo,
};

// XML Digital Signatures
pub use xmldsig::{
    SignatureMethod, CanonicalizationMethod, DigestMethod,
};

// ============================================================================
// Shamir Secret Sharing (from lib-common)
// ============================================================================

/// Shamir Secret Sharing for key splitting and reconstruction
pub use lib_common::crypto::shamir;
