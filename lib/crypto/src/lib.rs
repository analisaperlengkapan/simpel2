//! lib-crypto: Cryptographic primitives for SIMPEL
//!
//! WASM-compatible cryptography including Shamir Secret Sharing,
//! password hashing (Argon2id, bcrypt), AES-GCM encryption, and KDF.

// Re-export lib-core for convenience
pub use lib_core;

pub mod shamir;
pub mod password;

#[cfg(feature = "encryption")]
pub mod aes;

#[cfg(feature = "encryption")]
pub mod kdf;
