//! Namespace Management Module
//!
//! Provides hierarchical namespace management for SIMKARI multi-tenancy.
//! Supports organizational structure: Pusat -> Wilayah -> Satker

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
