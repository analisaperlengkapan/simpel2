//! Utility Functions and Helper Modules
//!
//! This module provides cross-cutting utility functions used throughout Secreton,
//! including caching, memory management, configuration helpers, and common algorithms.
//!
//! # Utility Categories
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │           Secreton Utilities                    │
//! ├─────────────────────────────────────────────────┤
//! │  Caching (Performance)                          │
//! │  ├─ LRU cache with TTL                          │
//! │  ├─ Token validation cache                      │
//! │  └─ Policy evaluation cache                     │
//! ├─────────────────────────────────────────────────┤
//! │  Memory Management (Security)                   │
//! │  ├─ Secure memory allocation                    │
//! │  ├─ Automatic zeroing on drop                   │
//! │  └─ Memory locking (prevent swapping)           │
//! ├─────────────────────────────────────────────────┤
//! │  Configuration (Convenience)                    │
//! │  ├─ Environment variable parsing                │
//! │  ├─ Config file loading                         │
//! │  └─ Default value handling                      │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! # Example: LRU Cache with TTL
//!
//! ```rust,no_run
//! use secreton_core::utils::cache::LruCache;
//! use std::time::Duration;
//!
//! # fn example() {
//! // Cache with max 1000 items, 5 minute TTL
//! let mut cache = LruCache::new(1000, Duration::from_secs(300));
//!
//! // Insert
//! cache.insert("key1".to_string(), "value1".to_string());
//!
//! // Get (returns Option)
//! if let Some(value) = cache.get("key1") {
//!     println!("Cached value: {}", value);
//! }
//!
//! // Automatically evicts expired and least-recently-used items
//! # }
//! ```
//!
//! # Example: Secure Memory for Secrets
//!
//! ```rust,no_run
//! use secreton_core::utils::memory::SecureMemory;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Allocate secure memory for sensitive data
//! let mut secure_mem = SecureMemory::new(32)?; // 32 bytes
//!
//! // Write secret data
//! secure_mem.write(b"super_secret_key_123456789012")?;
//!
//! // Memory is:
//! // - Locked (won't be swapped to disk)
//! // - Automatically zeroed when dropped
//! // - Protected from core dumps
//!
//! // Use the data
//! let secret_bytes = secure_mem.as_slice();
//! println!("Secret length: {}", secret_bytes.len());
//!
//! // SecureMemory automatically zeroed on drop
//! drop(secure_mem);
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Configuration Loading
//!
//! ```rust,no_run
//! use secreton_core::utils::config::ConfigLoader;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let loader = ConfigLoader::new();
//!
//! // Try environment variable, then config file, then default
//! let db_url = loader.get_string(
//!     "DATABASE_URL",                       // env var name
//!     Some("config.yaml:database.url"),    // config path
//!     "postgresql://localhost/secreton",   // default
//! )?;
//!
//! println!("Database URL: {}", db_url);
//! # Ok(())
//! # }
//! ```
//!
//! # Caching Strategies
//!
//! ## Token Validation Cache
//!
//! Reduce Authenc load by caching validated tokens:
//!
//! ```rust,no_run
//! use secreton_core::utils::cache::LruCache;
//! use std::time::Duration;
//!
//! # fn example() {
//! // Cache validated tokens for 15 minutes
//! let mut token_cache = LruCache::new(10_000, Duration::from_secs(900));
//!
//! let token = "eyJhbGc...";
//!
//! // Check cache first
//! if let Some(user_id) = token_cache.get(token) {
//!     println!("Cache hit: {}", user_id);
//!     // Skip Authenc validation
//! } else {
//!     // Validate with Authenc
//!     let user_id = validate_with_authenc(token);
//!     token_cache.insert(token.to_string(), user_id);
//! }
//! # }
//! # fn validate_with_authenc(token: &str) -> String { "user-123".to_string() }
//! ```
//!
//! ## Policy Evaluation Cache
//!
//! Cache expensive policy evaluations:
//!
//! ```rust,no_run
//! use secreton_core::utils::cache::LruCache;
//! use std::time::Duration;
//!
//! # fn example() {
//! // Cache policy decisions for 1 minute
//! let mut policy_cache = LruCache::new(5_000, Duration::from_secs(60));
//!
//! let cache_key = format!("{}:{}", user_id, resource_path);
//!
//! if let Some(allowed) = policy_cache.get(&cache_key) {
//!     return *allowed; // Use cached decision
//! }
//!
//! // Evaluate policy
//! let allowed = evaluate_policy(user_id, resource_path);
//! policy_cache.insert(cache_key, allowed);
//! # }
//! # let user_id = "user-123";
//! # let resource_path = "/app/secret";
//! # fn evaluate_policy(u: &str, p: &str) -> bool { true }
//! ```
//!
//! # Memory Security
//!
//! ## Automatic Zeroing
//!
//! Sensitive data automatically cleared from memory:
//!
//! ```rust,no_run
//! use secreton_core::utils::memory::SecureVec;
//!
//! # fn example() {
//! {
//!     let mut secret = SecureVec::new();
//!     secret.extend_from_slice(b"password123");
//!
//!     // Use secret...
//! } // <- Memory zeroed here automatically
//!
//! // Memory now contains all zeros, not "password123"
//! # }
//! ```
//!
//! ## Memory Locking
//!
//! Prevent sensitive data from being swapped to disk:
//!
//! ```rust,no_run
//! use secreton_core::utils::memory::LockedMemory;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Allocate and lock 256 bytes
//! let locked = LockedMemory::new(256)?;
//!
//! // This memory:
//! // - Won't be swapped to disk
//! // - Won't appear in core dumps
//! // - Zeroed on drop
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Optimizations
//!
//! ## Cache Hit Rates
//!
//! Monitor cache effectiveness:
//!
//! ```rust,no_run
//! use secreton_core::utils::cache::LruCache;
//!
//! # fn example() {
//! # let cache: LruCache<String, String> = LruCache::new(100, std::time::Duration::from_secs(60));
//! let stats = cache.stats();
//! println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
//! println!("Hits: {}, Misses: {}", stats.hits, stats.misses);
//! # }
//! ```
//!
//! ## Memory Pool
//!
//! Reuse memory allocations to reduce overhead:
//!
//! ```rust,no_run
//! use secreton_core::utils::memory::MemoryPool;
//!
//! # fn example() {
//! let pool = MemoryPool::new(1024, 100); // 1KB buffers, max 100
//!
//! // Get buffer from pool (reused if available)
//! let buffer = pool.get();
//!
//! // Use buffer...
//!
//! // Return to pool (memory zeroed automatically)
//! pool.return_buffer(buffer);
//! # }
//! ```
//!
//! # Configuration Helpers
//!
//! ## Environment Variable Parsing
//!
//! ```rust,no_run
//! use secreton_core::utils::config;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse with type conversion
//! let port: u16 = config::env_or("SECRETON_PORT", 8200)?;
//! let debug: bool = config::env_or("SECRETON_DEBUG", false)?;
//! let workers: usize = config::env_or("SECRETON_WORKERS", 4)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Default Values
//!
//! ```rust,no_run
//! use secreton_core::utils::config::default_if_empty;
//!
//! # fn example() {
//! let host = default_if_empty(
//!     std::env::var("HOST").ok(),
//!     "127.0.0.1".to_string(),
//! );
//! # }
//! ```
//!
//! # Common Patterns
//!
//! ## Result Caching
//!
//! Cache expensive computations:
//!
//! ```rust,no_run
//! use secreton_core::utils::cache::LruCache;
//! use std::time::Duration;
//!
//! # async fn example() {
//! # let mut cache = LruCache::new(100, Duration::from_secs(300));
//! # let key = "key";
//! if let Some(result) = cache.get(key) {
//!     return result.clone();
//! }
//!
//! let result = expensive_operation().await;
//! cache.insert(key.to_string(), result.clone());
//! # }
//! # async fn expensive_operation() -> String { "result".to_string() }
//! ```
//!
//! ## Secure String Handling
//!
//! ```rust,no_run
//! use secreton_core::utils::memory::SecureString;
//!
//! # fn example() {
//! let password = SecureString::from("user_password");
//! // Use password...
//! // Automatically zeroed on drop
//! # }
//! ```
//!
//! # See Also
//!
//! - [`cache`] - LRU cache implementation
//! - [`memory`] - Secure memory management
//! - [`config`] - Configuration utilities
//! - `crate::crypto` - Cryptographic operations
//! - `crate::storage` - Database caching layer

/// Configuration utilities
pub mod config;

/// LRU cache implementation with TTL support optimized for secrets
pub mod cache;

/// Memory optimization utilities and secure memory management for secrets
pub mod memory;

/// Common validation utilities to reduce code duplication
pub mod validation;

// Re-exports for convenience
pub use cache::*;
pub use memory::*;
pub use validation::*;
