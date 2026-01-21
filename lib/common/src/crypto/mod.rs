//! Cryptographic primitives for shared use across SIMPelv2 services
//!
//! This module provides:
//! - Shamir Secret Sharing for key splitting
//! - Password hashing with Argon2id
//! - AES-GCM and ChaCha20-Poly1305 encryption
//! - Key derivation functions (Argon2, HKDF, PBKDF2)
//!
//! # Features
//!
//! Enable the `crypto` feature in Cargo.toml to use this module.

#[cfg(feature = "crypto")]
pub mod shamir;

#[cfg(feature = "crypto")]
pub mod password;

#[cfg(feature = "encryption")]
pub mod aes;

#[cfg(feature = "encryption")]
pub mod kdf;

