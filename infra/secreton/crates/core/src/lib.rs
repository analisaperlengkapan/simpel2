//! # Secreton Core
//!
//! Core types, traits, and utilities shared across the Secreton security system.
//! Provides foundational abstractions for security levels, audit logging,
//! error handling, and common data structures.

#![allow(async_fn_in_trait)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

// Re-export shared types from secreton-types
pub use secreton_types::{Metadata, ResourceId, SecurityLevel, Tags};

pub mod audit;
pub mod auth;
pub mod config;
pub mod error;
// pub mod hsm; // MOVED: Extracted to secreton-hsm crate
pub mod models;
pub mod namespace;
pub mod pki; // Renamed from 'crypto' - contains PKI/certificate code only
pub mod prelude;
pub mod sdk_libraries;
pub mod security;
pub mod services;
pub mod storage;
pub mod types;
pub mod utils;

// Alias for engines module (points to services::secrets::enhanced)
// This provides backward compatibility with test expectations
pub mod engines {
    pub use crate::services::secrets::enhanced::*;
}

pub use audit::{AuditLog, AuditLogger, AuditStatus};
pub use auth::{
    AuthProvider, AuthResult, AuthencAuthProvider, Credentials, PqSignature, TokenValidation,
};
pub use error::CoreError;
// Commented out imports that don't exist yet
// pub use auth::mfa::MfaMethod;
// pub use graphql_api::{create_graphql_schema, GraphQLConfig, DefaultSecretsManager as GraphQLSecretsManager};
// pub use grpc_api::{GrpcConfig, SecretsGrpcService};
// pub use utils::error::AppError;

// Re-export commonly used models
pub use models::auth::{
    AuthMethod, AuthMethodType, AuthRequest, AuthResponse, LoginRequest, LoginResponse,
    RefreshTokenRequest, UserInfo,
};
pub use models::policy::{Policy, PolicyRule};
pub use models::user::{Token, User};

// Include integration tests - Disabled: missing dependencies
// #[cfg(test)]
// mod integration_tests;

// SecurityLevel is now re-exported from secreton-types

/// Result type for core operations
pub type CoreResult<T> = Result<T, error::CoreError>;

// Metadata is now re-exported from secreton-types

// Tags is now re-exported from secreton-types

// ResourceId is now re-exported from secreton-types

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_levels() {
        assert!(SecurityLevel::TopSecret > SecurityLevel::Secret);
        assert!(SecurityLevel::Secret > SecurityLevel::Confidential);
        assert!(SecurityLevel::Confidential > SecurityLevel::Internal);
        assert!(SecurityLevel::Internal > SecurityLevel::Public);

        assert!(SecurityLevel::TopSecret.can_access(SecurityLevel::Public));
        assert!(!SecurityLevel::Public.can_access(SecurityLevel::Secret));
    }

    #[test]
    fn test_security_level_from_string() {
        assert_eq!(SecurityLevel::parse("public"), Some(SecurityLevel::Public));
        assert_eq!(
            SecurityLevel::parse("CONFIDENTIAL"),
            Some(SecurityLevel::Confidential)
        );
        assert_eq!(
            SecurityLevel::parse("top-secret"),
            Some(SecurityLevel::TopSecret)
        );
        assert_eq!(SecurityLevel::parse("invalid"), None);

        // Test FromStr trait implementation
        assert_eq!("public".parse::<SecurityLevel>(), Ok(SecurityLevel::Public));
        assert_eq!(
            "confidential".parse::<SecurityLevel>(),
            Ok(SecurityLevel::Confidential)
        );
        assert!("invalid".parse::<SecurityLevel>().is_err());
    }

    #[test]
    fn test_metadata() {
        let mut metadata = Metadata::new();
        assert!(metadata.is_empty());

        metadata.set("key1", "value1");
        metadata.set("key2", 42);

        assert_eq!(metadata.len(), 2);
        assert!(metadata.contains_key("key1"));
        assert_eq!(
            metadata.get_typed::<String>("key1"),
            Some("value1".to_string())
        );
        assert_eq!(metadata.get_typed::<i32>("key2"), Some(42));
    }

    #[test]
    fn test_tags() {
        let mut tags = Tags::new();
        assert!(tags.is_empty());

        tags.add("important");
        tags.add("secure");
        tags.add("important"); // Should not duplicate

        assert_eq!(tags.len(), 2);
        assert!(tags.contains("important"));
        assert!(tags.contains("secure"));
        assert!(!tags.contains("other"));

        tags.remove("important");
        assert_eq!(tags.len(), 1);
        assert!(!tags.contains("important"));
    }

    #[test]
    fn test_resource_id() {
        let id = ResourceId::new("secrets".to_string(), "database-password".to_string());
        assert_eq!(id.full_path(), "secrets/database-password");
        assert_eq!(id.to_string(), "secrets/database-password");
    }
}
