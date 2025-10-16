/// Configuration utilities
pub mod config;

/// Error handling utilities
pub mod error;

/// LRU cache implementation with TTL support optimized for secrets
///
/// Provides thread-safe caching with sensitivity-based eviction policies.
/// Optimized for secret storage with security-focused cache management.
pub mod cache;

/// Memory optimization utilities and secure memory management for secrets
///
/// Provides secure memory containers, lazy loading, and memory pooling.
/// Includes automatic zeroization and sensitivity-based memory management.
pub mod memory;

// Re-exports for convenience
pub use cache::*;
pub use memory::*;
