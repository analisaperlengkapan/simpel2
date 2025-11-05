//! Namespace Management for Multi-Tenant Secreton
//!
//! This module provides hierarchical namespace isolation for SIMPelv2's organizational structure,
//! enabling secure multi-tenancy across Indonesian prosecutorial hierarchy (Kejaksaan RI).
//!
//! # Organizational Hierarchy (Indonesian Government Structure)
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │  PUSAT (Central - Kejaksaan Agung RI)           │
//! │  - National policies and secrets                │
//! │  - Full visibility into all namespaces          │
//! └────────────────────┬────────────────────────────┘
//!                      │
//!          ┌───────────┴───────────┐
//!          ▼                       ▼
//! ┌──────────────────┐    ┌──────────────────┐
//! │  WILAYAH (Region)│    │  WILAYAH (Region)│
//! │  - Regional ops  │    │  - Regional ops  │
//! │  - See own data  │    │  - See own data  │
//! └────────┬─────────┘    └────────┬─────────┘
//!          │                       │
//!     ┌────┴────┐             ┌────┴────┐
//!     ▼         ▼             ▼         ▼
//! ┌──────┐ ┌──────┐      ┌──────┐ ┌──────┐
//! │SATKER│ │SATKER│      │SATKER│ │SATKER│
//! │(Unit)│ │(Unit)│      │(Unit)│ │(Unit)│
//! └──────┘ └──────┘      └──────┘ └──────┘
//! ```
//!
//! # Namespace Types
//!
//! - **PUSAT** - Central level (Kejaksaan Agung RI) - full system access
//! - **WILAYAH** - Regional level (Kejaksaan Tinggi) - regional scope
//! - **SATKER** - Unit level (Kejaksaan Negeri/individual units) - unit-specific
//!
//! # Namespace Paths
//!
//! Namespaces use hierarchical paths:
//!
//! ```text
//! /                           → Root (Pusat only)
//! /pusat                      → Central namespace
//! /pusat/wilayah/jaktim       → East Java regional namespace
//! /pusat/wilayah/jaktim/jaksa → Satker under East Java
//! ```
//!
//! # Example: Create Namespace Hierarchy
//!
//! ```rust,no_run
//! use secreton_core::namespace::{NamespaceService, NamespaceType};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let service = NamespaceService::new(/* storage */);
//!
//! // Create regional namespace
//! let wilayah = service.create_namespace(
//!     "jaktim",
//!     NamespaceType::Wilayah,
//!     Some("/pusat/wilayah"), // Parent path
//! ).await?;
//!
//! // Create unit namespace under region
//! let satker = service.create_namespace(
//!     "jaksa-depok",
//!     NamespaceType::Satker,
//!     Some(&wilayah.path),
//! ).await?;
//!
//! println!("Created: {}", satker.path); // /pusat/wilayah/jaktim/jaksa-depok
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Access Control Check
//!
//! ```rust,no_run
//! use secreton_core::namespace::{NamespaceAccessControl, AdminLevel};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let access_control = NamespaceAccessControl::new(/* deps */);
//!
//! // Check if user can access namespace
//! let result = access_control.check_access(
//!     "user-123",
//!     "/pusat/wilayah/jaktim/jaksa-depok",
//!     "read",
//! ).await?;
//!
//! if result.allowed {
//!     println!("Access granted: {}", result.reason);
//! } else {
//!     println!("Access denied: {}", result.reason);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Namespace Isolation
//!
//! Each namespace is isolated:
//!
//! - **Secrets** - Only visible within namespace and ancestors
//! - **Policies** - Namespace-scoped permission rules
//! - **Quotas** - Resource limits per namespace
//! - **Audit Logs** - Namespace-filtered queries
//!
//! Example: User in `/pusat/wilayah/jaktim` can:
//! - ✅ Read secrets in `/pusat/wilayah/jaktim/*`
//! - ✅ Read secrets in `/pusat/*` (parent)
//! - ❌ Read secrets in `/pusat/wilayah/jakbar/*` (sibling)
//!
//! # Namespace Quotas
//!
//! Resource limits enforced per namespace:
//!
//! ```rust,no_run
//! use secreton_core::namespace::{NamespaceQuotas, QuotaUsage};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let service: secreton_core::namespace::NamespaceService = unimplemented!();
//! let quotas = NamespaceQuotas {
//!     max_secrets: 10_000,
//!     max_storage_bytes: 1_073_741_824, // 1 GB
//!     max_secret_size_bytes: 1_048_576,  // 1 MB per secret
//! };
//!
//! service.set_quotas("/pusat/wilayah/jaktim", quotas).await?;
//!
//! // Check current usage
//! let usage: QuotaUsage = service.get_quota_usage("/pusat/wilayah/jaktim").await?;
//! println!("Secrets: {}/{}", usage.secrets_count, quotas.max_secrets);
//! # Ok(())
//! # }
//! ```
//!
//! # Path Validation
//!
//! Namespace paths must follow rules:
//!
//! - Start with `/`
//! - No trailing `/`
//! - No `..` or `.` components
//! - Max depth: 5 levels
//! - Valid characters: `[a-z0-9-_]`
//!
//! ```rust,no_run
//! use secreton_core::namespace::{NamespacePath, PathResolutionError};
//!
//! # fn example() -> Result<(), PathResolutionError> {
//! // Valid paths
//! let path1 = NamespacePath::parse("/pusat/wilayah/jaktim")?;
//! let path2 = NamespacePath::parse("/pusat/wilayah/jakbar/satker-01")?;
//!
//! // Invalid paths
//! assert!(NamespacePath::parse("pusat").is_err());        // No leading /
//! assert!(NamespacePath::parse("/pusat/").is_err());      // Trailing /
//! assert!(NamespacePath::parse("/pusat/../wilayah").is_err()); // Path traversal
//! # Ok(())
//! # }
//! ```
//!
//! # Admin Levels
//!
//! Three admin privilege levels:
//!
//! - **Pusat Admin** - Full system access, all namespaces
//! - **Wilayah Admin** - Regional scope, own wilayah + child satkers
//! - **Satker Admin** - Unit scope, own satker only
//!
//! # JWT Integration
//!
//! User's namespace extracted from JWT claims:
//!
//! ```rust,no_run
//! use secreton_core::namespace::JwtClaims;
//!
//! # fn example() {
//! let claims = JwtClaims {
//!     sub: "user-123".to_string(),
//!     namespace: "/pusat/wilayah/jaktim".to_string(),
//!     admin_level: Some("wilayah".to_string()),
//!     // ... other claims
//! };
//!
//! // User automatically scoped to their namespace
//! // All operations filtered by this path
//! # }
//! ```
//!
//! # Performance Considerations
//!
//! - **Path Lookups**: O(1) with indexed storage (PostgreSQL `gin` index)
//! - **Hierarchy Traversal**: O(depth) - typically 3-5 levels
//! - **Access Checks**: Cached for 5 minutes per user+namespace pair
//! - **Quota Checks**: Incremental updates, not full scans
//!
//! # Migration Support
//!
//! Moving secrets between namespaces:
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let service: secreton_core::namespace::NamespaceService = unimplemented!();
//! // Move secrets from one namespace to another
//! service.migrate_secrets(
//!     "/pusat/wilayah/old-jaksa",
//!     "/pusat/wilayah/new-jaksa",
//!     /* admin_user */
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Security
//!
//! - Namespace paths validated before storage operations
//! - Access checks enforced at API gateway level
//! - Audit logs include namespace context
//! - Cross-namespace access requires explicit Pusat admin role
//!
//! # See Also
//!
//! - [`NamespaceService`] - Main service for namespace CRUD
//! - [`NamespaceAccessControl`] - Access validation logic
//! - [`NamespaceHierarchy`] - Hierarchy traversal utilities
//! - [`NamespacePath`] - Path parsing and validation
//! - `crate::policy` - Namespace-scoped policies

pub mod access;
pub mod hierarchy;
pub mod path;
pub mod service;
pub mod validation;

pub use access::{AccessCheckResult, AdminLevel, JwtClaims, NamespaceAccessControl};
pub use hierarchy::{Namespace, NamespaceHierarchy, NamespaceQuotas, NamespaceType, QuotaUsage};
pub use path::{NamespacePath, PathResolutionError};
pub use service::NamespaceService;
pub use validation::NamespaceValidator;
