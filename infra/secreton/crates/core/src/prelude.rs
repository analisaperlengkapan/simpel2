//! Core prelude - commonly used types and traits
//!
//! This module re-exports the most commonly used items from the core crate
//! to simplify imports in other modules.

// Re-export error types
pub use crate::error::CoreError;
pub use crate::CoreResult;

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
