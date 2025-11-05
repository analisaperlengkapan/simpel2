//! Namespace Validation for Secret Operations
//!
//! Provides validation utilities for namespace-scoped secret operations.
//! Ensures all secret operations respect namespace boundaries and access control.

use crate::audit::{AuditLog, AuditLogger, AuditStatus};
use crate::error::CoreError;
use crate::namespace::{
    JwtClaims, NamespaceAccessControl, NamespaceHierarchy, NamespacePath,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Namespace validator for secret operations
pub struct NamespaceValidator {
    /// Namespace hierarchy
    hierarchy: Arc<NamespaceHierarchy>,

    /// Access control
    access_control: Arc<NamespaceAccessControl>,

    /// Audit logger
    audit_logger: Option<Arc<AuditLogger>>,
}

impl NamespaceValidator {
    /// Create a new namespace validator
    pub fn new(
        hierarchy: Arc<NamespaceHierarchy>,
        access_control: Arc<NamespaceAccessControl>,
        audit_logger: Option<Arc<AuditLogger>>,
    ) -> Self {
        Self {
            hierarchy,
            access_control,
            audit_logger,
        }
    }

    /// Validate a secret path for read operation
    ///
    /// # Arguments
    /// * `claims` - JWT claims from user
    /// * `path` - Full secret path (e.g., "satker-kja001/db/password")
    /// * `client_ip` - Client IP address for audit logging
    ///
    /// # Returns
    /// * `Ok(NamespacePath)` if validation succeeds
    /// * `Err(CoreError)` if validation fails
    pub async fn validate_read(
        &self,
        claims: &JwtClaims,
        path: &str,
        client_ip: Option<String>,
    ) -> Result<NamespacePath, CoreError> {
        self.validate_operation(claims, path, "read", client_ip)
            .await
    }

    /// Validate a secret path for write operation
    ///
    /// # Arguments
    /// * `claims` - JWT claims from user
    /// * `path` - Full secret path (e.g., "satker-kja001/db/password")
    /// * `client_ip` - Client IP address for audit logging
    ///
    /// # Returns
    /// * `Ok(NamespacePath)` if validation succeeds
    /// * `Err(CoreError)` if validation fails
    pub async fn validate_write(
        &self,
        claims: &JwtClaims,
        path: &str,
        client_ip: Option<String>,
    ) -> Result<NamespacePath, CoreError> {
        // Check quota before allowing write
        let namespace_path = self
            .validate_operation(claims, path, "write", client_ip.clone())
            .await?;

        // Check if namespace quota is exceeded
        if let Some(namespace) = namespace_path.get_namespace(&self.hierarchy)
            && namespace.is_quota_exceeded() {
                self.log_audit(
                    claims,
                    "write",
                    path,
                    &namespace_path.namespace_id,
                    AuditStatus::Denied,
                    client_ip,
                    Some("Namespace quota exceeded".to_string()),
                )
                .await;

                return Err(CoreError::quota_exceeded(format!(
                    "Namespace {} has exceeded its quota",
                    namespace_path.namespace_id
                )));
            }

        Ok(namespace_path)
    }

    /// Validate a secret path for delete operation
    ///
    /// # Arguments
    /// * `claims` - JWT claims from user
    /// * `path` - Full secret path (e.g., "satker-kja001/db/password")
    /// * `client_ip` - Client IP address for audit logging
    ///
    /// # Returns
    /// * `Ok(NamespacePath)` if validation succeeds
    /// * `Err(CoreError)` if validation fails
    pub async fn validate_delete(
        &self,
        claims: &JwtClaims,
        path: &str,
        client_ip: Option<String>,
    ) -> Result<NamespacePath, CoreError> {
        self.validate_operation(claims, path, "delete", client_ip)
            .await
    }

    /// Validate a secret path for list operation
    ///
    /// # Arguments
    /// * `claims` - JWT claims from user
    /// * `path_prefix` - Path prefix to list (e.g., "satker-kja001/db/")
    /// * `client_ip` - Client IP address for audit logging
    ///
    /// # Returns
    /// * `Ok(NamespacePath)` if validation succeeds
    /// * `Err(CoreError)` if validation fails
    pub async fn validate_list(
        &self,
        claims: &JwtClaims,
        path_prefix: &str,
        client_ip: Option<String>,
    ) -> Result<NamespacePath, CoreError> {
        self.validate_operation(claims, path_prefix, "list", client_ip)
            .await
    }

    /// Common validation logic for all operations
    async fn validate_operation(
        &self,
        claims: &JwtClaims,
        path: &str,
        operation: &str,
        client_ip: Option<String>,
    ) -> Result<NamespacePath, CoreError> {
        // Parse the path
        let namespace_path = NamespacePath::parse(path).map_err(|e| {
            tracing::warn!("Invalid path format: {} - Error: {}", path, e);
            CoreError::from(e)
        })?;

        // Validate namespace exists in hierarchy
        namespace_path
            .validate_with_hierarchy(&self.hierarchy)
            .map_err(|e| {
                tracing::warn!(
                    "Namespace validation failed for {}: {}",
                    namespace_path.namespace_id,
                    e
                );
                CoreError::from(e)
            })?;

        // Check access control
        let has_access = self
            .access_control
            .check_access(claims, &namespace_path.namespace_id)
            .map_err(|e| {
                tracing::warn!(
                    "Access check failed for user {} on namespace {}: {}",
                    claims.sub,
                    namespace_path.namespace_id,
                    e
                );
                e
            })?;

        if !has_access {
            // Log denied access
            self.log_audit(
                claims,
                operation,
                path,
                &namespace_path.namespace_id,
                AuditStatus::Denied,
                client_ip,
                Some(format!(
                    "User does not have access to namespace {}",
                    namespace_path.namespace_id
                )),
            )
            .await;

            return Err(CoreError::authorization(format!(
                "Access denied to namespace: {}",
                namespace_path.namespace_id
            )));
        }

        // Log successful validation
        self.log_audit(
            claims,
            operation,
            path,
            &namespace_path.namespace_id,
            AuditStatus::Success,
            client_ip,
            None,
        )
        .await;

        Ok(namespace_path)
    }

    /// Log audit event
    async fn log_audit(
        &self,
        claims: &JwtClaims,
        operation: &str,
        path: &str,
        namespace_id: &str,
        status: AuditStatus,
        client_ip: Option<String>,
        error_message: Option<String>,
    ) {
        if let Some(audit_logger) = &self.audit_logger {
            let mut metadata = HashMap::new();
            metadata.insert("path".to_string(), path.to_string());
            if let Some(error) = error_message {
                metadata.insert("error".to_string(), error);
            }

            let entry = AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: format!("secret.{}", operation),
                actor: Some(claims.sub.clone()),
                resource_type: "secret".to_string(),
                resource_id: path.to_string(),
                status,
                ip: client_ip,
                user_agent: None,
                namespace: Some(namespace_id.to_string()),
                metadata,
            };

            if let Err(e) = audit_logger.log(entry).await {
                tracing::error!("Failed to log audit event: {}", e);
            }
        }
    }

    /// Filter paths by accessible namespaces
    ///
    /// Given a list of paths, returns only those in namespaces the user can access
    ///
    /// # Arguments
    /// * `claims` - JWT claims from user
    /// * `paths` - List of full secret paths
    ///
    /// # Returns
    /// * Filtered list of paths the user can access
    pub fn filter_accessible_paths(&self, claims: &JwtClaims, paths: &[String]) -> Vec<String> {
        let accessible_namespaces = self.access_control.get_accessible_namespaces(claims);

        paths
            .iter()
            .filter(|path| {
                if let Ok(namespace_path) = NamespacePath::parse(path) {
                    accessible_namespaces.contains(&namespace_path.namespace_id)
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Get accessible namespaces for a user
    ///
    /// # Arguments
    /// * `claims` - JWT claims from user
    ///
    /// # Returns
    /// * List of namespace IDs the user can access
    pub fn get_accessible_namespaces(&self, claims: &JwtClaims) -> Vec<String> {
        self.access_control.get_accessible_namespaces(claims)
    }

    /// Validate namespace exists
    ///
    /// # Arguments
    /// * `namespace_id` - Namespace identifier
    ///
    /// # Returns
    /// * `Ok(())` if namespace exists
    /// * `Err(CoreError)` if namespace doesn't exist
    pub fn validate_namespace_exists(&self, namespace_id: &str) -> Result<(), CoreError> {
        if self.hierarchy.get_namespace(namespace_id).is_none() {
            return Err(CoreError::not_found(format!("namespace: {}", namespace_id)));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespace::{AdminLevel, NamespaceHierarchy};

    fn create_test_setup() -> (
        Arc<NamespaceHierarchy>,
        Arc<NamespaceAccessControl>,
        NamespaceValidator,
    ) {
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

        let hierarchy = Arc::new(hierarchy);
        let access_control = Arc::new(NamespaceAccessControl::new((*hierarchy).clone()));
        let validator = NamespaceValidator::new(hierarchy.clone(), access_control.clone(), None);

        (hierarchy, access_control, validator)
    }

    #[tokio::test]
    async fn test_validate_read_success() {
        let (_, _, validator) = create_test_setup();

        let claims = JwtClaims::new_test(
            "user1".to_string(),
            AdminLevel::Satker,
            Some("kja001".to_string()),
            Some("sumut".to_string()),
        );

        let result = validator
            .validate_read(&claims, "satker-kja001/db/password", None)
            .await;

        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.namespace_id, "satker-kja001");
        assert_eq!(path.secret_path, "db/password");
    }

    #[tokio::test]
    async fn test_validate_read_access_denied() {
        let (_, _, validator) = create_test_setup();

        let claims = JwtClaims::new_test(
            "user1".to_string(),
            AdminLevel::Satker,
            Some("kja001".to_string()),
            Some("sumut".to_string()),
        );

        // Try to access different satker
        let result = validator
            .validate_read(&claims, "satker-kja002/db/password", None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_invalid_path() {
        let (_, _, validator) = create_test_setup();

        let claims = JwtClaims::new_test(
            "user1".to_string(),
            AdminLevel::Satker,
            Some("kja001".to_string()),
            Some("sumut".to_string()),
        );

        // Invalid path format
        let result = validator.validate_read(&claims, "invalid-path", None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_nonexistent_namespace() {
        let (_, _, validator) = create_test_setup();

        let claims = JwtClaims::new_test("user1".to_string(), AdminLevel::Pusat, None, None);

        // Namespace doesn't exist
        let result = validator
            .validate_read(&claims, "satker-kja999/db/password", None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_filter_accessible_paths() {
        let (_, _, validator) = create_test_setup();

        let claims = JwtClaims::new_test(
            "user1".to_string(),
            AdminLevel::Satker,
            Some("kja001".to_string()),
            Some("sumut".to_string()),
        );

        let paths = vec![
            "satker-kja001/db/password".to_string(),
            "satker-kja001/api/key".to_string(),
            "satker-kja002/db/password".to_string(),
            "wilayah-sumut/config/key".to_string(),
        ];

        let filtered = validator.filter_accessible_paths(&claims, &paths);

        // Should only include satker-kja001 paths
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&"satker-kja001/db/password".to_string()));
        assert!(filtered.contains(&"satker-kja001/api/key".to_string()));
    }

    #[tokio::test]
    async fn test_validate_namespace_exists() {
        let (_, _, validator) = create_test_setup();

        assert!(validator.validate_namespace_exists("satker-kja001").is_ok());
        assert!(
            validator
                .validate_namespace_exists("satker-kja999")
                .is_err()
        );
    }
}
