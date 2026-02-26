//! Data Models for Secreton
//!
//! This module contains all data structures and domain models used throughout Secreton,
//! providing type-safe representations of secrets, policies, users, audit events, and more.
//!
//! # Model Categories
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │             Secreton Models                     │
//! ├─────────────────────────────────────────────────┤
//! │  Authentication                                 │
//! │  ├─ User, Token                                 │
//! │  ├─ AuthRequest, AuthResponse                   │
//! │  └─ LoginRequest, RefreshToken                  │
//! ├─────────────────────────────────────────────────┤
//! │  Authorization                                  │
//! │  ├─ Policy, PolicyRule                          │
//! │  ├─ AppRole (machine auth)                      │
//! │  └─ Sentinel (policy-as-code)                   │
//! ├─────────────────────────────────────────────────┤
//! │  Secrets Management                             │
//! │  ├─ Secret (key-value data)                     │
//! │  ├─ Lease (TTL, renewal)                        │
//! │  └─ PKI (certificates, keys)                    │
//! ├─────────────────────────────────────────────────┤
//! │  Security & Compliance                          │
//! │  ├─ AuditEvent, AuditTrailEntry                 │
//! │  ├─ MFA (multi-factor auth)                     │
//! │  └─ SecurityContext, SecurityLevel              │
//! ├─────────────────────────────────────────────────┤
//! │  Extensibility                                  │
//! │  └─ Plugin (custom auth/secret engines)         │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! # Example: User Model
//!
//! ```ignore
//! use secreton_core::models::user::{User, Token};
//! use chrono::Utc;
//!
//! # fn example() {
//! let user = User {
//!     id: "user-123".to_string(),
//!     username: "alice@kejaksaan.go.id".to_string(),
//!     email: Some("alice@kejaksaan.go.id".to_string()),
//!     roles: vec!["operator".to_string()],
//!     namespace: "/pusat/wilayah/jaktim".to_string(),
//!     clearance_level: 2, // CONFIDENTIAL
//!     mfa_enabled: true,
//!     created_at: Utc::now(),
//!     token: Some(Token {
//!         access_token: "eyJhbGc...".to_string(),
//!         refresh_token: Some("refresh...".to_string()),
//!         expires_in: 3600, // 1 hour
//!     }),
//! ;
//! # }
//! ```
//!
//! # Example: Secret Model
//!
//! ```ignore
//! use secreton_core::models::secret::Secret;
//! use std::collections::HashMap;
//!
//! # fn example() {
//! let mut secret = Secret {
//!     path: "/app/database/credentials".to_string(),
//!     data: HashMap::new(),
//!     version: 1,
//!     created_at: chrono::Utc::now(),
//!     created_by: "user-123".to_string(),
//!     metadata: HashMap::new(),
//! ;
//!
//! secret.data.insert("username".to_string(), "dbuser".to_string());
//! secret.data.insert("password".to_string(), "secure_pw".to_string());
//! secret.metadata.insert("environment".to_string(), "production".to_string());
//! # }
//! ```
//!
//! # Example: Policy Model
//!
//! ```ignore
//! use secreton_core::models::policy::{Policy, PolicyRule};
//!
//! # fn example() {
//! let policy = Policy {
//!     name: "developer-policy".to_string(),
//!     rules: vec![
//!         PolicyRule {
//!             path: "/app/dev/*".to_string(),
//!             capabilities: vec!["read".to_string(), "list".to_string()],
//!         ,
//!         PolicyRule {
//!             path: "/app/dev/secrets/*".to_string(),
//!             capabilities: vec!["create".to_string(), "update".to_string()],
//!         ,
//!     ],
//!     namespace: "/pusat/wilayah/jaktim".to_string(),
//! ;
//! # }
//! ```
//!
//! # Example: Audit Event Model
//!
//! ```ignore
//! use secreton_core::models::audit::{AuditEvent, AuditEventType, OperationResult};
//!
//! # fn example() {
//! let event = AuditEvent {
//!     id: uuid::Uuid::new_v4().to_string(),
//!     event_type: AuditEventType::SecretRead,
//!     timestamp: chrono::Utc::now(),
//!     user_id: Some("user-123".to_string()),
//!     namespace: "/pusat/wilayah/jaktim".to_string(),
//!     resource_path: "/app/database/password".to_string(),
//!     operation: "read".to_string(),
//!     result: OperationResult::Success,
//!     ip_address: Some("192.168.1.100".to_string()),
//!     user_agent: Some("secreton-cli/1.0".to_string()),
//!     details: None,
//! ;
//! # }
//! ```
//!
//! # Serialization
//!
//! All models implement `serde::Serialize` and `serde::Deserialize`:
//!
//! ```ignore
//! use secreton_core::models::user::User;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let user: User = unimplemented!();
//! // To JSON
//! let json = serde_json::to_string(&user)?;
//!
//! // From JSON
//! let user: User = serde_json::from_str(&json)?;
//!
//! // To MessagePack (binary)
//! let bytes = rmp_serde::to_vec(&user)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Database Mapping
//!
//! Models map to PostgreSQL tables:
//!
//! | Model | Table | Primary Key |
//! |-------|-------|-------------|
//! | `User` | `secreton.users` | `id` (UUID) |
//! | `Secret` | `secreton.secrets` | `path` + `version` |
//! | `Policy` | `secreton.policies` | `name` |
//! | `AuditEvent` | `secreton.audit_logs` | `id` (UUID) |
//! | `Lease` | `secreton.leases` | `lease_id` |
//!
//! # Validation
//!
//! Models include validation logic:
//!
//! ```ignore
//! use secreton_core::models::policy::PolicyRule;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let rule = PolicyRule {
//!     path: "/app/../sensitive".to_string(),
//!     capabilities: vec!["read".to_string()],
//! ;
//!
//! // Validate path (prevent traversal)
//! if rule.validate()? {
//!     println!("Valid policy rule");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Model Modules
//!
//! - [`auth`] - Authentication requests/responses
//! - [`user`] - User accounts and tokens
//! - [`policy`] - Authorization policies
//! - [`secret`] - Secret storage and versioning
//! - [`lease`] - TTL and renewal management
//! - [`audit`] - Audit events and compliance
//! - [`mfa`] - Multi-factor authentication
//! - [`pki`] - Public key infrastructure
//! - [`approle`] - Application role authentication
//! - [`sentinel`] - Policy-as-code engine
//! - [`plugin`] - Plugin system models
//!
//! # Common Patterns
//!
//! ## Builder Pattern
//!
//! Many models support builder pattern:
//!
//! ```ignore
//! use secreton_core::models::audit::AuditEvent;
//!
//! # fn example() {
//! let event = AuditEvent::builder()
//!     .user_id("user-123")
//!     .namespace("/pusat")
//!     .resource_path("/app/secret")
//!     .operation("read")
//!     .success()
//!     .build();
//! # }
//! ```
//!
//! ## Option Pattern
//!
//! Optional fields use `Option<T>`:
//!
//! ```rust,ignore
//! pub struct User {
//!     pub id: String,           // Required
//!     pub email: Option<String>, // Optional
//!     pub token: Option<Token>,  // Optional
//! }
//! ```
//!
//! ## Enum Variants
//!
//! Type-safe enums for categories:
//!
//! ```ignore
//! use secreton_core::models::audit::AuditEventType;
//!
//! # fn example() {
//! match event_type {
//!     AuditEventType::SecretRead => { /* ... */ }
//!     AuditEventType::SecretWrite => { /* ... */ }
//!     AuditEventType::UserLogin => { /* ... */ }
//!     // ... more variants
//! }
//! # }
//! ```
//!
//! # See Also
//!
//! - `crate::services` - Business logic using these models
//! - `crate::storage` - Database persistence
//! - `crate::api` - REST API endpoints
//! - `crate::grpc` - gRPC service definitions

pub mod approle;
pub mod audit;
pub mod auth;
pub mod dynamic_role;
pub mod lease;
pub mod mfa;
pub mod pki;
pub mod plugin;
pub mod policy;
pub mod secret;
pub mod sentinel;
pub mod user;

// Re-export commonly used types
pub use audit::{
    AdminLevel, AuditConfig, AuditEvent, AuditEventType, AuditQuery, AuditTrailEntry,
    ComplianceFlag, HierarchicalAuditSummary, Operation, OperationResult, OrganizationalLevel,
    SecurityContext, SecurityLevel,
};
pub use auth::{
    AuthMethod, AuthMethodType, AuthRequest, AuthResponse, LoginRequest, LoginResponse,
    RefreshTokenRequest, UserInfo,
};
pub use dynamic_role::{
    AuthMethodType as DynamicAuthMethodType, Capability, DynamicRoleStore, EffectiveCapabilities,
    EngineRoleType, PathAccessResult, PolicyEffect, PolicyRule as DynamicPolicyRule, SshKeyType,
    TokenType, UserRoleType,
};
pub use policy::{ControlGroup, Policy, PolicyRule};
pub use user::{Token, User};
