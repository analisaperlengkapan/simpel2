//! # Key Management Module
//!
//! This module provides JWT signing key management for various algorithms:
//! - Ed25519 (EdDSA) - Primary signing algorithm
//! - ECDSA P-256 (ES256) - NIST standard curve
//! - ECDSA P-384 (ES384) - Enterprise-grade security
//! - ECDSA P-521 (ES512) - Maximum security
//! - EdDSA Ed448 - Post-quantum ready (using Ed25519 as fallback)
//!
//! All keys support:
//! - Environment variable loading (production)
//! - File-based loading (alternative production)
//! - Ephemeral key generation (development/testing)
//! - JWK (JSON Web Key) export
//! - PEM format export

pub mod ed25519;
pub mod ecdsa;
pub mod ecdsa_p384;
pub mod ecdsa_p521;
pub mod eddsa_ed448;

// Re-export key types and functions for convenience
pub use ed25519::{
    ED25519_KEYPAIR,
    Ed25519Jwk,
    Ed25519JwkSet,
    get_ed25519_jwk,
    get_ed25519_public_pem,
    sign_ed25519,
    verify_ed25519,
    generate_new_keypair as generate_new_ed25519_keypair,
};

pub use ecdsa::{
    ECDSA_KEYPAIR,
    EcdsaJwk,
    EcdsaJwkSet,
    get_ecdsa_jwk,
    get_ecdsa_public_pem,
    get_ecdsa_private_pem,
    sign_ecdsa,
    verify_ecdsa,
    generate_new_p256_keypair,
};

pub use ecdsa_p384::{
    ECDSA_P384_KEYPAIR,
    EcdsaP384Jwk,
    EcdsaP384JwkSet,
    sign_jwt_p384,
    verify_jwt_p384,
    get_p384_jwk_set,
    generate_new_p384_keypair,
};

pub use ecdsa_p521::{
    ECDSA_P521_KEYPAIR,
    EcdsaP521Jwk,
    EcdsaP521JwkSet,
    sign_jwt_p521,
    verify_jwt_p521,
    get_p521_jwk_set,
    generate_new_p521_keypair,
};

pub use eddsa_ed448::{
    EDDSA_KEYPAIR,
    EddsaJwk,
    EddsaJwkSet,
    sign_jwt_eddsa,
    verify_jwt_eddsa,
    get_eddsa_jwk_set,
};
