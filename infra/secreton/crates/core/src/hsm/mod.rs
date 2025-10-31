//! HSM (Hardware Security Module) integration for Secreton
//!
//! This module provides integration with Hardware Security Modules for secure key storage
//! and cryptographic operations. It implements the HsmVault trait from Authenc.
//!
//! # Features
//!
//! - PKCS#11 support for HSM communication
//! - Key generation in HSM
//! - Encryption/decryption using HSM keys
//! - Digital signatures using HSM keys
//! - HSM health monitoring
//!
//! # Security
//!
//! HSM operations provide hardware-backed security for cryptographic keys, ensuring:
//! - Keys never leave the HSM in plaintext
//! - FIPS 140-2 Level 3+ compliance (hardware dependent)
//! - Tamper-resistant key storage
//! - Audit logging of all HSM operations

pub mod backend;
pub mod config;
pub mod error;
pub mod pkcs11;

pub use backend::HsmBackend;
pub use config::HsmConfig;
pub use error::{HsmError, HsmResult};
pub use pkcs11::Pkcs11Provider;
