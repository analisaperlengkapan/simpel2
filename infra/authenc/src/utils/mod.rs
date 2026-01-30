// Utility modules

/// Cryptographic utilities for secure operations
/// Provides cryptographic functions including key generation,
/// encryption/decryption, and secure random number generation.
/// Used throughout the authentication platform for security operations.
pub mod crypto;

/// Cryptographic operation monitoring and security auditing
/// Monitors cryptographic operations for security compliance,
/// performance metrics, and anomaly detection.
/// Helps maintain security standards and detect potential attacks.
pub mod crypto_monitor;

/// Internationalization (i18n) utilities for multi-language support
/// Handles localization, message translation, and cultural formatting.
/// Supports multiple languages for user-facing messages and interfaces.
pub mod i18n;

/// JWT token utilities and helpers
/// Provides JWT token parsing, validation, and utility functions.
/// Supports both encoding and decoding of JWT tokens with various algorithms.
pub mod jwt;

/// JWT key management with Secreton integration
/// Manages JWT signing keys retrieved from Secreton with caching and rotation.
/// Provides secure key storage and automatic initialization.
pub mod jwt_key_manager;
/// Framework for loading and managing authentication plugins.
/// Enables third-party extensions and custom authentication methods.
pub mod plugin;

// Authentication utilities

/// Authentication context management utilities
/// Manages authentication state, user context, and session information.
/// Provides utilities for authentication flow management and user state tracking.
pub mod auth_context;

// Legacy modules (to be refactored)

/// Core utility functions (legacy - to be refactored)
/// Contains legacy utility functions that need refactoring.
/// These utilities are maintained for backward compatibility during migration.
pub mod core;

/// Integration utilities for external systems (legacy - to be refactored)
/// Utilities for integrating with external authentication systems.
/// Contains legacy integration code that needs modernization.
pub mod integration;

/// LRU cache implementation with TTL support
/// Provides thread-safe caching with TTL and LRU eviction policies.
/// Optimized for authentication data caching and performance.
pub mod cache;

/// HTTP connection pooling for external service communication
/// Manages connection pools for HTTP requests to external services.
/// Includes retry logic, timeout handling, and connection reuse.
pub mod connection_pool;

/// Memory optimization utilities and secure memory management
/// Provides secure memory containers, lazy loading, and memory pooling.
/// Includes automatic zeroization and memory usage tracking.
pub mod memory;

/// Input validation and sanitization utilities
/// Provides comprehensive input validation using garde and custom validators.
/// Includes sanitization functions to prevent injection attacks.
pub mod validation;

/// SSO cookie management utilities
/// Provides secure SSO cookie creation, validation, and session management.
/// Implements secure cookie attributes (Secure, HttpOnly, SameSite) for Portal integration.
pub mod sso_cookie;

/// Request context extraction utilities
/// Extracts IP address, user agent, and other contextual information from HTTP requests.
/// Used for comprehensive audit logging and security monitoring.
pub mod request_context;

/// Geolocation utilities for IP address lookup
/// Provides optional geolocation data for audit logging.
/// Supports integration with external geolocation services.
pub mod geolocation;

/// Payload sanitization utilities for audit logging
pub mod sanitizer {
    pub use lib_common::sanitizer::*;
}

/// Encoding utilities
pub mod encoding {
    pub use lib_common::encoding::*;
}

// Re-exports for convenience
pub use auth_context::*;
pub use cache::*;
pub use connection_pool::*;
pub use geolocation::*;
pub use i18n::*;
pub use memory::*;
pub use request_context::*;
pub use sanitizer::*;
pub use sso_cookie::*;
pub use validation::*;
