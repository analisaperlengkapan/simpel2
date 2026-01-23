// SPDX-License-Identifier: Apache-2.0
//! Secure memory handling
//!
//! This module provides secure memory primitives that zeroize data on drop.
//! It re-exports primitives from the common library and provides type aliases
//! for backward compatibility.

use lib_common::memory::{
    LazySecretCryptoContext, SecretMemoryOptimizer, SecretMemoryPool, SecretMemoryTracker,
    SecureSecretBytes, SecureSecretMemory, SecureSecretString, SensitivityLevel,
};
use zeroize::Zeroize;

// Re-exports with original names for compatibility
pub type SecureMemory<T> = SecureSecretMemory<T>;
pub type SecureString = SecureSecretString;
pub type SecureBytes = SecureSecretBytes;
pub type LazyCryptoContext<T> = LazySecretCryptoContext<T>;
pub type MemoryPool<T> = SecretMemoryPool<T>;
pub type MemoryTracker = SecretMemoryTracker;
pub type MemoryOptimizer = SecretMemoryOptimizer;

// Adapters for constructors that changed signature

pub trait SecureMemoryExt<T: Zeroize> {
    fn new_compat(data: T) -> Self;
}

impl<T: Zeroize> SecureMemoryExt<T> for SecureSecretMemory<T> {
    fn new_compat(data: T) -> Self {
        Self::new(data, SensitivityLevel::Medium)
    }
}
