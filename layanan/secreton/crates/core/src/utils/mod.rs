//! Cross-cutting helpers: caching, secure memory, configuration, correlation IDs.
//!
//! Most of this module is a thin, secret-oriented facade over `lib_backend` and
//! `lib_core` — the implementations live there and are shared with the other
//! services. What Secreton adds is naming that makes the sensitivity explicit
//! ([`SecretLruCache`], [`SecureSecretMemory`]) and the [`cache::SecretCacheManager`]
//! that groups the four caches a running engine needs.
//!
//! | Area | Type | Comes from |
//! |---|---|---|
//! | caching | [`SecretLruCache`], [`ThreadSafeSecretCache`], [`cache::AsyncSecretCache`] | `lib_backend::cache` (renamed on re-export) |
//! | secure memory | [`SecureSecretMemory`], [`SecureSecretString`], [`SecretMemoryPool`] | `lib_backend::memory` |
//! | configuration | [`config::Config`] | local |
//! | tracing | [`correlation::CorrelationContext`] | local |
//! | encoding / validation | [`encoding`], [`validation`] | `lib_core` |
//!
//! Note the renames: the `LruCache` of `lib_backend` is re-exported here as
//! [`SecretLruCache`], so `utils::cache::LruCache` is not a path that resolves.
//!
//! # Caching
//!
//! Capacity is the only constructor argument; the TTL is per entry, supplied at
//! insert time, so one cache can hold values with different lifetimes.
//!
//! ```rust
//! use secreton_core::utils::SecretLruCache;
//! use std::time::Duration;
//!
//! let mut cache: SecretLruCache<String, String> = SecretLruCache::new(1_000);
//! cache.insert("token:abc".into(), "user-123".into(), Duration::from_secs(900));
//!
//! assert_eq!(cache.get(&"token:abc".to_string()), Some("user-123".to_string()));
//! assert_eq!(cache.get(&"token:missing".to_string()), None);
//!
//! let stats = cache.stats();
//! assert_eq!(stats.hit_count, 1);
//! assert_eq!(stats.miss_count, 1);
//! ```
//!
//! Entries can also carry a sensitivity level, which drives eviction order under
//! pressure — see [`cache::SecretLruCache::insert_with_sensitivity`] and
//! [`SensitivityLevel`].
//!
//! # Secure memory
//!
//! Wrappers that zeroize on drop. They do not lock pages against swapping — if
//! that guarantee is ever needed it has to be built, not assumed from the name.
//!
//! Mind which `SensitivityLevel` you reach for: `cache` and `memory` each define
//! their own, with identical variants and no conversion between them. This module
//! re-exports the cache one bare and the memory one as [`MemorySensitivityLevel`],
//! and the memory APIs take the latter.
//!
//! ```rust
//! use secreton_core::utils::{MemorySensitivityLevel, SecureSecretString};
//!
//! let password = SecureSecretString::from_str("user_password", MemorySensitivityLevel::High);
//! assert_eq!(password.as_str(), "user_password");
//! assert_eq!(password.len(), 13);
//! // Dropping `password` zeroizes the backing String.
//! ```
//!
//! [`SecretMemoryPool`] reuses buffers and zeroizes each one as it is returned,
//! so a pooled allocation never carries a previous secret into its next use.
//!
//! # Configuration
//!
//! [`config::Config`] loads from a TOML file or from the environment. There is no
//! layered "env, then file, then default" resolver — [`config::Config::from_env`]
//! reads `VAULT_*` variables and falls back to the same values as
//! [`config::Config::default`] per field.
//!
//! ```rust
//! use secreton_core::utils::config::Config;
//!
//! let config = Config::default();
//! assert_eq!(config.server_port, 8080);
//! assert_eq!(config.log_level, "info");
//! ```
//!
//! # See also
//!
//! - [`crate::pki`] — certificate and key material
//! - [`crate::storage`] — persistence, including the cache backend trait

/// Configuration utilities
pub mod config;

/// Correlation tracking for distributed tracing
pub mod correlation;

/// LRU cache implementation with TTL support optimized for secrets
pub mod cache;

/// Encoding utilities
pub mod encoding {
    pub use lib_core::encoding::*;
}

/// Memory optimization utilities and secure memory management for secrets
pub mod memory;

/// Common validation utilities to reduce code duplication
pub mod validation;

// Re-export cache types (SensitivityLevel from cache takes precedence)
pub use cache::{SecretCacheStats, SecretLruCache, SensitivityLevel, ThreadSafeSecretCache};

// Re-export memory types (but not SensitivityLevel to avoid conflict)
pub use memory::{SecretMemoryPool, SecretMemoryStats, SecureSecretMemory, SecureSecretString};
// Re-export memory's SensitivityLevel with an alias for explicit usage
pub use memory::SensitivityLevel as MemorySensitivityLevel;

// Re-export validation
pub use validation::*;
