//! Prelude Module - Convenient Imports
//!
//! This module provides a curated collection of commonly used types, traits, and functions
//! from across the Secreton core crate, enabling simplified imports in application code.
//!
//! # Usage Pattern
//!
//! Instead of importing individual items:
//!
//! ```rust,ignore
//! use secreton_core::error::CoreError;
//! use secreton_core::auth::{AuthProvider, AuthResult, Credentials};
//! use secreton_core::audit::{AuditLog, AuditLogger};
//! use secreton_core::models::user::User;
//! use secreton_core::SecurityLevel;
//! // ... many more imports
//! ```
//!
//! Simply use the prelude:
//!
//! ```rust,no_run
//! use secreton_core::prelude::*;
//!
//! // All common types now available
//! # fn example() {
//! let user: User = unimplemented!();
//! let level: SecurityLevel = SecurityLevel::Confidential;
//! let error: CoreError = CoreError::Unauthorized;
//! # }
//! ```
//!
//! # What's Included
//!
//! ## Error Types
//! - `CoreError` - Main error enum
//! - `CoreResult` - Result type alias
//!
//! ## Core Types
//! - `SecurityLevel` - Security classification levels
//! - `ResourceId` - Unique resource identifiers
//! - `Metadata` - Key-value metadata
//! - `Tags` - Resource tagging
//!
//! ## Authentication
//! - `AuthProvider` - Authentication trait
//! - `AuthResult` - Auth operation result
//! - `Credentials` - User credentials
//! - `TokenValidation` - JWT validation
//!
//! ## Audit Logging
//! - `AuditLog` - Audit log entry
//! - `AuditLogger` - Audit logging interface
//! - `AuditStatus` - Operation status (success/failure)
//!
//! ## Data Models
//! - `User` - User account model
//! - `Token` - JWT token model
//! - `Policy` - Authorization policy
//! - `PolicyRule` - Policy rule entry
//! - `AuthMethod` - Authentication method enum
//! - `LoginRequest` - Login request payload
//! - `LoginResponse` - Login response payload
//!
//! ## External Traits
//! - `async_trait` - Async trait support
//! - `Serialize` / `Deserialize` - Serde serialization
//! - `DateTime<Utc>` - Timezone-aware timestamps
//! - `Arc` / `RwLock` - Thread-safe shared state
//! - `Uuid` - Universally unique identifiers
//!
//! # Example: Service Implementation
//!
//! ```rust,no_run
//! use secreton_core::prelude::*;
//!
//! pub struct SecretService {
//!     auth: Arc<dyn AuthProvider>,
//!     audit: Arc<AuditLogger>,
//! }
//!
//! impl SecretService {
//!     pub async fn read_secret(&self, path: &str, user: &User) -> CoreResult<String> {
//!         // Check authorization
//!         if user.clearance_level < 2 {
//!             return Err(CoreError::Unauthorized);
//!         }
//!
//!         // Audit the operation
//!         self.audit.log(AuditLog::new(
//!             "secret.read",
//!             Some(&user.id),
//!             "secret",
//!             path,
//!             AuditStatus::Success,
//!         )).await?;
//!
//!         Ok("secret_value".to_string())
//!     }
//! }
//! ```
//!
//! # Example: Error Handling
//!
//! ```rust,no_run
//! use secreton_core::prelude::*;
//!
//! async fn process_request(user: &User) -> CoreResult<()> {
//!     if !user.mfa_enabled {
//!         return Err(CoreError::MfaRequired);
//!     }
//!
//!     // Process...
//!     Ok(())
//! }
//! ```
//!
//! # When to Use Prelude
//!
//! ✅ **Use prelude when**:
//! - Writing application code that uses many core types
//! - Implementing services or handlers
//! - Building internal tools
//! - Rapid prototyping
//!
//! ❌ **Don't use prelude when**:
//! - Writing library code (be explicit with imports)
//! - Name conflicts exist with local types
//! - Only need 1-2 items (import explicitly)
//! - Building public APIs (explicit imports aid documentation)
//!
//! # Prelude Design Philosophy
//!
//! The prelude follows Rust ecosystem conventions:
//!
//! 1. **Common not comprehensive** - Only frequently-used items
//! 2. **No namespace pollution** - Avoids generic names like `Config`
//! 3. **Type-focused** - Traits and types, not functions
//! 4. **Stable** - Rarely changes to avoid churn
//!
//! # See Also
//!
//! For specific functionality, see:
//! - `crate::error` - Error types and handling
//! - `crate::auth` - Authentication providers
//! - `crate::audit` - Audit logging
//! - `crate::models` - Data models
//! - `crate::services` - Business logic services

// Re-export error types
pub use crate::CoreResult;
pub use crate::error::CoreError;

// Re-export common types
pub use crate::{Metadata, ResourceId, SecurityLevel, Tags};

// Re-export auth types
pub use crate::auth::{AuthProvider, AuthResult, Credentials, TokenValidation};

// Re-export audit types
pub use crate::audit::{AuditLog, AuditLogger, AuditStatus};

// Re-export model types
pub use crate::models::{
    auth::{AuthMethod, AuthMethodType, AuthRequest, AuthResponse, LoginRequest, LoginResponse},
    policy::{Policy, PolicyRule},
    user::{Token, User},
};

// Re-export commonly used external crates
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub use tokio::sync::RwLock;
pub use uuid::Uuid;
