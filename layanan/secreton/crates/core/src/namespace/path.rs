//! Namespace Path Resolution
//!
//! Provides path parsing and validation for namespace-scoped secrets.
//! Format: {namespace_id}/path/to/secret
//! Example: satker-kja001/db/password

use crate::error::CoreError;
use crate::namespace::{Namespace, NamespaceHierarchy};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Path resolution error
#[derive(Error, Debug)]
pub enum PathResolutionError {
    #[error("Invalid path format: {0}")]
    InvalidFormat(String),

    #[error("Namespace not found: {0}")]
    NamespaceNotFound(String),

    #[error("Empty path")]
    EmptyPath,

    #[error("Invalid namespace ID: {0}")]
    InvalidNamespaceId(String),

    #[error("Path validation failed: {0}")]
    ValidationFailed(String),
}

impl From<PathResolutionError> for CoreError {
    fn from(err: PathResolutionError) -> Self {
        match err {
            PathResolutionError::NamespaceNotFound(ns) => {
                CoreError::not_found(format!("namespace: {}", ns))
            }
            PathResolutionError::EmptyPath => CoreError::validation("Path cannot be empty"),
            PathResolutionError::InvalidFormat(msg) => CoreError::validation(msg),
            PathResolutionError::InvalidNamespaceId(msg) => CoreError::validation(msg),
            PathResolutionError::ValidationFailed(msg) => CoreError::validation(msg),
        }
    }
}

/// Parsed namespace path
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamespacePath {
    /// Namespace ID (e.g., "satker-kja001")
    pub namespace_id: String,

    /// Secret path within namespace (e.g., "db/password")
    pub secret_path: String,

    /// Full path (e.g., "satker-kja001/db/password")
    pub full_path: String,
}

impl NamespacePath {
    /// Parse a full path into namespace and secret components
    ///
    /// # Format
    /// {namespace_id}/path/to/secret
    ///
    /// # Examples
    /// - "satker-kja001/db/password" -> namespace: "satker-kja001", secret: "db/password"
    /// - "wilayah-sumut/config/api-key" -> namespace: "wilayah-sumut", secret: "config/api-key"
    /// - "pusat/master/encryption-key" -> namespace: "pusat", secret: "master/encryption-key"
    ///
    /// # Arguments
    /// * `path` - Full path string
    ///
    /// # Returns
    /// * `Ok(NamespacePath)` if path is valid
    /// * `Err(PathResolutionError)` if path is invalid
    pub fn parse(path: &str) -> Result<Self, PathResolutionError> {
        // Validate path is not empty
        if path.is_empty() {
            return Err(PathResolutionError::EmptyPath);
        }

        // Trim whitespace
        let path = path.trim();

        // Split by first '/'
        let parts: Vec<&str> = path.splitn(2, '/').collect();

        if parts.len() < 2 {
            return Err(PathResolutionError::InvalidFormat(
                "Path must contain namespace and secret path separated by '/'".to_string(),
            ));
        }

        let namespace_id = parts[0].to_string();
        let secret_path = parts[1].to_string();

        // Validate namespace ID format
        Self::validate_namespace_id(&namespace_id)?;

        // Validate secret path
        Self::validate_secret_path(&secret_path)?;

        Ok(Self {
            namespace_id: namespace_id.clone(),
            secret_path: secret_path.clone(),
            full_path: path.to_string(),
        })
    }

    /// Validate namespace ID format
    ///
    /// Valid formats:
    /// - "pusat" (root namespace)
    /// - "wilayah-{code}" (regional namespace)
    /// - "satker-{code}" (work unit namespace)
    fn validate_namespace_id(namespace_id: &str) -> Result<(), PathResolutionError> {
        if namespace_id.is_empty() {
            return Err(PathResolutionError::InvalidNamespaceId(
                "Namespace ID cannot be empty".to_string(),
            ));
        }

        // Check for invalid characters
        if namespace_id.contains(char::is_whitespace) {
            return Err(PathResolutionError::InvalidNamespaceId(
                "Namespace ID cannot contain whitespace".to_string(),
            ));
        }

        // Validate format
        if namespace_id == "pusat" {
            return Ok(());
        }

        if let Some(code) = namespace_id.strip_prefix("wilayah-") {
            if code.is_empty() {
                return Err(PathResolutionError::InvalidNamespaceId(
                    "Wilayah code cannot be empty".to_string(),
                ));
            }
            return Ok(());
        }

        if let Some(code) = namespace_id.strip_prefix("satker-") {
            if code.is_empty() {
                return Err(PathResolutionError::InvalidNamespaceId(
                    "Satker code cannot be empty".to_string(),
                ));
            }
            return Ok(());
        }

        Err(PathResolutionError::InvalidNamespaceId(format!(
            "Invalid namespace ID format: {}. Must be 'pusat', 'wilayah-{{code}}', or 'satker-{{code}}'",
            namespace_id
        )))
    }

    /// Validate secret path format
    fn validate_secret_path(secret_path: &str) -> Result<(), PathResolutionError> {
        if secret_path.is_empty() {
            return Err(PathResolutionError::ValidationFailed(
                "Secret path cannot be empty".to_string(),
            ));
        }

        // Check for invalid characters
        let invalid_chars = ['\\', '\0', '\n', '\r', '\t'];
        for ch in invalid_chars {
            if secret_path.contains(ch) {
                return Err(PathResolutionError::ValidationFailed(format!(
                    "Secret path contains invalid character: {:?}",
                    ch
                )));
            }
        }

        // Check for path traversal attempts
        if secret_path.contains("..") {
            return Err(PathResolutionError::ValidationFailed(
                "Secret path cannot contain '..' (path traversal)".to_string(),
            ));
        }

        // Check for leading/trailing slashes
        if secret_path.starts_with('/') || secret_path.ends_with('/') {
            return Err(PathResolutionError::ValidationFailed(
                "Secret path cannot start or end with '/'".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate path against namespace hierarchy
    ///
    /// Ensures the namespace exists in the hierarchy
    ///
    /// # Arguments
    /// * `hierarchy` - Namespace hierarchy to validate against
    ///
    /// # Returns
    /// * `Ok(())` if namespace exists
    /// * `Err(PathResolutionError)` if namespace doesn't exist
    pub fn validate_with_hierarchy(
        &self,
        hierarchy: &NamespaceHierarchy,
    ) -> Result<(), PathResolutionError> {
        if hierarchy.get_namespace(&self.namespace_id).is_none() {
            return Err(PathResolutionError::NamespaceNotFound(
                self.namespace_id.clone(),
            ));
        }
        Ok(())
    }

    /// Get the namespace from hierarchy
    ///
    /// # Arguments
    /// * `hierarchy` - Namespace hierarchy
    ///
    /// # Returns
    /// * `Some(&Namespace)` if namespace exists
    /// * `None` if namespace doesn't exist
    pub fn get_namespace<'a>(&self, hierarchy: &'a NamespaceHierarchy) -> Option<&'a Namespace> {
        hierarchy.get_namespace(&self.namespace_id)
    }

    /// Construct a full path from namespace ID and secret path
    ///
    /// # Arguments
    /// * `namespace_id` - Namespace identifier
    /// * `secret_path` - Secret path within namespace
    ///
    /// # Returns
    /// * Full path string
    pub fn construct(namespace_id: &str, secret_path: &str) -> String {
        format!("{}/{}", namespace_id, secret_path)
    }

    /// Check if this path is within a specific namespace
    pub fn is_in_namespace(&self, namespace_id: &str) -> bool {
        self.namespace_id == namespace_id
    }

    /// Get the namespace type from the ID
    pub fn namespace_type(&self) -> Option<crate::namespace::NamespaceType> {
        if self.namespace_id == "pusat" {
            Some(crate::namespace::NamespaceType::Pusat)
        } else if self.namespace_id.starts_with("wilayah-") {
            Some(crate::namespace::NamespaceType::Wilayah)
        } else if self.namespace_id.starts_with("satker-") {
            Some(crate::namespace::NamespaceType::Satker)
        } else {
            None
        }
    }
}

impl std::fmt::Display for NamespacePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_path)
    }
}

impl std::str::FromStr for NamespacePath {
    type Err = PathResolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespace::NamespaceHierarchy;

    #[test]
    fn test_parse_valid_paths() {
        // Satker path
        let path = NamespacePath::parse("satker-kja001/db/password").unwrap();
        assert_eq!(path.namespace_id, "satker-kja001");
        assert_eq!(path.secret_path, "db/password");
        assert_eq!(path.full_path, "satker-kja001/db/password");

        // Wilayah path
        let path = NamespacePath::parse("wilayah-sumut/config/api-key").unwrap();
        assert_eq!(path.namespace_id, "wilayah-sumut");
        assert_eq!(path.secret_path, "config/api-key");

        // Pusat path
        let path = NamespacePath::parse("pusat/master/encryption-key").unwrap();
        assert_eq!(path.namespace_id, "pusat");
        assert_eq!(path.secret_path, "master/encryption-key");

        // Nested secret path
        let path = NamespacePath::parse("satker-kja001/app/db/prod/password").unwrap();
        assert_eq!(path.namespace_id, "satker-kja001");
        assert_eq!(path.secret_path, "app/db/prod/password");
    }

    #[test]
    fn test_parse_invalid_paths() {
        // Empty path
        assert!(NamespacePath::parse("").is_err());

        // No separator
        assert!(NamespacePath::parse("satker-kja001").is_err());

        // Invalid namespace format
        assert!(NamespacePath::parse("invalid/db/password").is_err());

        // Path traversal
        assert!(NamespacePath::parse("satker-kja001/../password").is_err());

        // Leading slash in secret path
        assert!(NamespacePath::parse("satker-kja001//password").is_err());

        // Whitespace in namespace
        assert!(NamespacePath::parse("satker kja001/password").is_err());

        // Invalid characters
        assert!(NamespacePath::parse("satker-kja001/pass\0word").is_err());
        assert!(NamespacePath::parse("satker-kja001/pass\nword").is_err());
    }

    #[test]
    fn test_validate_namespace_id() {
        // Valid IDs
        assert!(NamespacePath::validate_namespace_id("pusat").is_ok());
        assert!(NamespacePath::validate_namespace_id("wilayah-sumut").is_ok());
        assert!(NamespacePath::validate_namespace_id("satker-kja001").is_ok());

        // Invalid IDs
        assert!(NamespacePath::validate_namespace_id("").is_err());
        assert!(NamespacePath::validate_namespace_id("wilayah-").is_err());
        assert!(NamespacePath::validate_namespace_id("satker-").is_err());
        assert!(NamespacePath::validate_namespace_id("invalid").is_err());
        assert!(NamespacePath::validate_namespace_id("satker kja001").is_err());
    }

    #[test]
    fn test_validate_secret_path() {
        // Valid paths
        assert!(NamespacePath::validate_secret_path("db/password").is_ok());
        assert!(NamespacePath::validate_secret_path("app/config/api-key").is_ok());
        assert!(NamespacePath::validate_secret_path("simple").is_ok());

        // Invalid paths
        assert!(NamespacePath::validate_secret_path("").is_err());
        assert!(NamespacePath::validate_secret_path("../password").is_err());
        assert!(NamespacePath::validate_secret_path("/password").is_err());
        assert!(NamespacePath::validate_secret_path("password/").is_err());
        assert!(NamespacePath::validate_secret_path("pass\0word").is_err());
    }

    #[test]
    fn test_validate_with_hierarchy() {
        let mut hierarchy =
            NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        hierarchy
            .add_wilayah(
                "wilayah-sumut".to_string(),
                "Kejaksaan Tinggi Sumatera Utara".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        hierarchy
            .add_satker(
                "satker-kja001".to_string(),
                "Kejaksaan Negeri Medan".to_string(),
                "wilayah-sumut".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        // Valid namespace
        let path = NamespacePath::parse("satker-kja001/db/password").unwrap();
        assert!(path.validate_with_hierarchy(&hierarchy).is_ok());

        // Invalid namespace (doesn't exist)
        let path = NamespacePath::parse("satker-kja999/db/password").unwrap();
        assert!(path.validate_with_hierarchy(&hierarchy).is_err());
    }

    #[test]
    fn test_get_namespace() {
        let mut hierarchy =
            NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        hierarchy
            .add_satker(
                "satker-kja001".to_string(),
                "Kejaksaan Negeri Medan".to_string(),
                "wilayah-sumut".to_string(),
                "admin".to_string(),
            )
            .unwrap_or_else(|_| {
                hierarchy
                    .add_wilayah(
                        "wilayah-sumut".to_string(),
                        "Kejaksaan Tinggi Sumatera Utara".to_string(),
                        "admin".to_string(),
                    )
                    .unwrap();
                hierarchy
                    .add_satker(
                        "satker-kja001".to_string(),
                        "Kejaksaan Negeri Medan".to_string(),
                        "wilayah-sumut".to_string(),
                        "admin".to_string(),
                    )
                    .unwrap()
            });

        let path = NamespacePath::parse("satker-kja001/db/password").unwrap();
        let namespace = path.get_namespace(&hierarchy);
        assert!(namespace.is_some());
        assert_eq!(namespace.unwrap().id, "satker-kja001");
    }

    #[test]
    fn test_construct() {
        let path = NamespacePath::construct("satker-kja001", "db/password");
        assert_eq!(path, "satker-kja001/db/password");
    }

    #[test]
    fn test_is_in_namespace() {
        let path = NamespacePath::parse("satker-kja001/db/password").unwrap();
        assert!(path.is_in_namespace("satker-kja001"));
        assert!(!path.is_in_namespace("satker-kja002"));
    }

    #[test]
    fn test_namespace_type() {
        let path = NamespacePath::parse("pusat/master/key").unwrap();
        assert_eq!(
            path.namespace_type(),
            Some(crate::namespace::NamespaceType::Pusat)
        );

        let path = NamespacePath::parse("wilayah-sumut/config/key").unwrap();
        assert_eq!(
            path.namespace_type(),
            Some(crate::namespace::NamespaceType::Wilayah)
        );

        let path = NamespacePath::parse("satker-kja001/db/password").unwrap();
        assert_eq!(
            path.namespace_type(),
            Some(crate::namespace::NamespaceType::Satker)
        );
    }

    #[test]
    fn test_display() {
        let path = NamespacePath::parse("satker-kja001/db/password").unwrap();
        assert_eq!(path.to_string(), "satker-kja001/db/password");
    }

    #[test]
    fn test_from_str() {
        let path: NamespacePath = "satker-kja001/db/password".parse().unwrap();
        assert_eq!(path.namespace_id, "satker-kja001");
        assert_eq!(path.secret_path, "db/password");
    }
}
