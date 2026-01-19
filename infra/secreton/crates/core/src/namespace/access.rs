//! Namespace Access Control based on JWT Claims
//!
//! Implements hierarchical access control for SIMKARI organizational structure
//! based on JWT claims from Authenc. Enforces access rules:
//! - AdminLevel::Pusat -> access all namespaces
//! - AdminLevel::EselonI -> access directorate namespaces
//! - AdminLevel::Wilayah -> access wilayah and child satker namespaces
//! - AdminLevel::Satker -> access only own satker namespace

use crate::error::CoreError;
use crate::namespace::{Namespace, NamespaceHierarchy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JWT claims extracted from Authenc tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `JwtClaims`.
pub struct JwtClaims {
    /// Standard JWT subject (user ID)
    pub sub: String,

    /// User's name
    pub name: String,

    /// User's email
    pub email: String,

    /// Satker code (e.g., "KJA001" for Kejaksaan Negeri Medan)
    pub satker_code: Option<String>,

    /// Wilayah code (e.g., "SUMUT" for Sumatera Utara)
    pub wilayah_code: Option<String>,

    /// Administrative level in the hierarchy
    pub admin_level: AdminLevel,

    /// User roles
    pub roles: Vec<String>,

    /// User permissions
    pub permissions: Vec<String>,

    /// Token expiration (Unix timestamp)
    pub exp: i64,

    /// Token issued at (Unix timestamp)
    pub iat: i64,

    /// Token issuer
    pub iss: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Administrative level in Kejaksaan RI hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Mewakili pub `AdminLevel`.
pub enum AdminLevel {
    /// Central/National level (Kejaksaan Agung)
    /// Full access to all namespaces
    Pusat,

    /// Eselon I level (Directorate level)
    /// Access to directorate-specific namespaces
    EselonI,

    /// Regional level (Kejaksaan Tinggi)
    /// Access to wilayah and child satker namespaces
    Wilayah,

    /// Work unit level (Kejaksaan Negeri, etc.)
    /// Access only to own satker namespace
    Satker,
}

/// Access control result
#[derive(Debug, Clone)]
/// Mewakili pub `AccessCheckResult`.
pub struct AccessCheckResult {
    /// Whether access is allowed
    pub allowed: bool,

    /// Reason for denial (if not allowed)
    pub reason: Option<String>,

    /// Accessible namespace IDs
    pub accessible_namespaces: Vec<String>,
}

/// Namespace access controller
#[derive(Debug)]
/// Mewakili pub `NamespaceAccessControl`.
pub struct NamespaceAccessControl {
    /// Namespace hierarchy
    hierarchy: NamespaceHierarchy,
}

impl NamespaceAccessControl {
    /// Create a new namespace access controller
    pub fn new(hierarchy: NamespaceHierarchy) -> Self {
        Self { hierarchy }
    }

    /// Check if user has access to a specific namespace
    ///
    /// # Arguments
    /// * `claims` - JWT claims from Authenc
    /// * `namespace_id` - Target namespace ID to check access for
    ///
    /// # Returns
    /// * `Ok(true)` if access is allowed
    /// * `Ok(false)` if access is denied
    /// * `Err` if namespace doesn't exist or other error
    pub fn check_access(&self, claims: &JwtClaims, namespace_id: &str) -> Result<bool, CoreError> {
        Self::verify_access(&self.hierarchy, claims, namespace_id)
    }

    /// Verify if user has access to a specific namespace (static version)
    ///
    /// # Arguments
    /// * `hierarchy` - Reference to namespace hierarchy
    /// * `claims` - JWT claims from Authenc
    /// * `namespace_id` - Target namespace ID to check access for
    ///
    /// # Returns
    /// * `Ok(true)` if access is allowed
    /// * `Ok(false)` if access is denied
    /// * `Err` if namespace doesn't exist or other error
    pub fn verify_access(
        hierarchy: &NamespaceHierarchy,
        claims: &JwtClaims,
        namespace_id: &str,
    ) -> Result<bool, CoreError> {
        // Validate namespace exists
        let namespace = hierarchy
            .get_namespace(namespace_id)
            .ok_or_else(|| CoreError::not_found(format!("namespace: {}", namespace_id)))?;

        // Apply hierarchical access rules
        match claims.admin_level {
            AdminLevel::Pusat => {
                // Pusat admin has access to all namespaces
                Ok(true)
            }
            AdminLevel::EselonI => {
                // Eselon I has access to directorate namespaces
                // For now, treat similar to Pusat (can be refined based on directorate field)
                // TODO: Add directorate-specific filtering when directorate field is added
                Ok(true)
            }
            AdminLevel::Wilayah => {
                // Wilayah admin has access to their wilayah and all child satkers
                Self::check_wilayah_access(hierarchy, claims, namespace)
            }
            AdminLevel::Satker => {
                // Satker admin has access only to their own satker
                Self::check_satker_access(hierarchy, claims, namespace)
            }
        }
    }

    /// Check wilayah-level access
    fn check_wilayah_access(
        hierarchy: &NamespaceHierarchy,
        claims: &JwtClaims,
        namespace: &Namespace,
    ) -> Result<bool, CoreError> {
        let wilayah_code = claims.wilayah_code.as_ref().ok_or_else(|| {
            CoreError::authorization("Wilayah admin must have wilayah_code in JWT claims")
        })?;

        // Construct expected wilayah namespace ID
        let wilayah_namespace_id = format!("wilayah-{}", wilayah_code.to_lowercase());

        // Check if target namespace is the wilayah itself
        if namespace.id == wilayah_namespace_id {
            return Ok(true);
        }

        // Check if target namespace is a child of this wilayah
        let ancestors = hierarchy.get_ancestors(&namespace.id);
        for ancestor in ancestors {
            if ancestor.id == wilayah_namespace_id {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check satker-level access
    fn check_satker_access(
        _hierarchy: &NamespaceHierarchy,
        claims: &JwtClaims,
        namespace: &Namespace,
    ) -> Result<bool, CoreError> {
        let satker_code = claims.satker_code.as_ref().ok_or_else(|| {
            CoreError::authorization("Satker admin must have satker_code in JWT claims")
        })?;

        // Construct expected satker namespace ID
        let satker_namespace_id = format!("satker-{}", satker_code.to_lowercase());

        // Satker admin can only access their own satker namespace
        Ok(namespace.id == satker_namespace_id)
    }

    /// Get all accessible namespaces for a user
    ///
    /// # Arguments
    /// * `claims` - JWT claims from Authenc
    ///
    /// # Returns
    /// * List of namespace IDs the user can access
    pub fn get_accessible_namespaces(&self, claims: &JwtClaims) -> Vec<String> {
        match claims.admin_level {
            AdminLevel::Pusat | AdminLevel::EselonI => {
                // Pusat and Eselon I can access all namespaces
                self.hierarchy
                    .list_all()
                    .iter()
                    .map(|ns| ns.id.clone())
                    .collect()
            }
            AdminLevel::Wilayah => {
                // Wilayah admin can access their wilayah and all child satkers
                if let Some(wilayah_code) = &claims.wilayah_code {
                    let wilayah_id = format!("wilayah-{}", wilayah_code.to_lowercase());

                    let mut accessible = vec![wilayah_id.clone()];

                    // Add all descendants (satkers under this wilayah)
                    let descendants = self.hierarchy.get_descendants(&wilayah_id);
                    accessible.extend(descendants.iter().map(|ns| ns.id.clone()));

                    accessible
                } else {
                    vec![]
                }
            }
            AdminLevel::Satker => {
                // Satker admin can only access their own satker
                if let Some(satker_code) = &claims.satker_code {
                    vec![format!("satker-{}", satker_code.to_lowercase())]
                } else {
                    vec![]
                }
            }
        }
    }

    /// Perform comprehensive access check with detailed result
    ///
    /// # Arguments
    /// * `claims` - JWT claims from Authenc
    /// * `namespace_id` - Target namespace ID to check access for
    ///
    /// # Returns
    /// * Detailed access check result with reason and accessible namespaces
    pub fn check_access_detailed(
        &self,
        claims: &JwtClaims,
        namespace_id: &str,
    ) -> AccessCheckResult {
        match self.check_access(claims, namespace_id) {
            Ok(true) => AccessCheckResult {
                allowed: true,
                reason: None,
                accessible_namespaces: self.get_accessible_namespaces(claims),
            },
            Ok(false) => AccessCheckResult {
                allowed: false,
                reason: Some(format!(
                    "User with admin level {:?} does not have access to namespace {}",
                    claims.admin_level, namespace_id
                )),
                accessible_namespaces: self.get_accessible_namespaces(claims),
            },
            Err(e) => AccessCheckResult {
                allowed: false,
                reason: Some(format!("Access check failed: {}", e)),
                accessible_namespaces: vec![],
            },
        }
    }

    /// Validate namespace access for secret operations
    ///
    /// This is a convenience method for validating access before
    /// performing secret operations (get, put, delete, list)
    ///
    /// # Arguments
    /// * `claims` - JWT claims from Authenc
    /// * `secret_path` - Full secret path (e.g., "satker-kja001/db/password")
    ///
    /// # Returns
    /// * `Ok(namespace_id)` if access is allowed, returns the namespace ID
    /// * `Err` if access is denied or path is invalid
    pub fn validate_secret_access(
        &self,
        claims: &JwtClaims,
        secret_path: &str,
    ) -> Result<String, CoreError> {
        // Extract namespace ID from secret path
        // Format: {namespace_id}/path/to/secret
        let parts: Vec<&str> = secret_path.split('/').collect();

        if parts.is_empty() {
            return Err(CoreError::validation("Secret path cannot be empty"));
        }

        let namespace_id = parts[0];

        // Validate namespace exists
        if self.hierarchy.get_namespace(namespace_id).is_none() {
            return Err(CoreError::not_found(format!("namespace: {}", namespace_id)));
        }

        // Check access
        if self.check_access(claims, namespace_id)? {
            Ok(namespace_id.to_string())
        } else {
            Err(CoreError::authorization(format!(
                "Access denied to namespace: {}",
                namespace_id
            )))
        }
    }

    /// Filter namespaces based on user access
    ///
    /// # Arguments
    /// * `claims` - JWT claims from Authenc
    /// * `namespace_ids` - List of namespace IDs to filter
    ///
    /// # Returns
    /// * Filtered list of namespace IDs the user can access
    pub fn filter_accessible_namespaces(
        &self,
        claims: &JwtClaims,
        namespace_ids: &[String],
    ) -> Vec<String> {
        let accessible = self.get_accessible_namespaces(claims);

        namespace_ids
            .iter()
            .filter(|id| accessible.contains(id))
            .cloned()
            .collect()
    }

    /// Update hierarchy (for dynamic namespace management)
    pub fn update_hierarchy(&mut self, hierarchy: NamespaceHierarchy) {
        self.hierarchy = hierarchy;
    }

    /// Get reference to hierarchy
    pub fn hierarchy(&self) -> &NamespaceHierarchy {
        &self.hierarchy
    }
}

impl std::fmt::Display for AdminLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminLevel::Pusat => write!(f, "Pusat"),
            AdminLevel::EselonI => write!(f, "Eselon I"),
            AdminLevel::Wilayah => write!(f, "Wilayah"),
            AdminLevel::Satker => write!(f, "Satker"),
        }
    }
}

impl JwtClaims {
    /// Create JWT claims for testing
    #[cfg(test)]
    /// Mewakili pub `new_test(`.
    pub fn new_test(
        sub: String,
        admin_level: AdminLevel,
        satker_code: Option<String>,
        wilayah_code: Option<String>,
    ) -> Self {
        Self {
            sub,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            satker_code,
            wilayah_code,
            admin_level,
            roles: vec![],
            permissions: vec![],
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            iss: "authenc".to_string(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hierarchy() -> NamespaceHierarchy {
        let mut hierarchy =
            NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

        // Add wilayah
        hierarchy
            .add_wilayah(
                "wilayah-sumut".to_string(),
                "Kejaksaan Tinggi Sumatera Utara".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        hierarchy
            .add_wilayah(
                "wilayah-jabar".to_string(),
                "Kejaksaan Tinggi Jawa Barat".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        // Add satkers
        hierarchy
            .add_satker(
                "satker-kja001".to_string(),
                "Kejaksaan Negeri Medan".to_string(),
                "wilayah-sumut".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        hierarchy
            .add_satker(
                "satker-kja002".to_string(),
                "Kejaksaan Negeri Binjai".to_string(),
                "wilayah-sumut".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        hierarchy
            .add_satker(
                "satker-kja101".to_string(),
                "Kejaksaan Negeri Bandung".to_string(),
                "wilayah-jabar".to_string(),
                "admin".to_string(),
            )
            .unwrap();

        hierarchy
    }

    #[test]
    fn test_pusat_admin_access_all() {
        let hierarchy = create_test_hierarchy();
        let access_control = NamespaceAccessControl::new(hierarchy);

        let claims = JwtClaims::new_test("admin-pusat".to_string(), AdminLevel::Pusat, None, None);

        // Pusat admin should have access to all namespaces
        assert!(access_control.check_access(&claims, "pusat").unwrap());
        assert!(
            access_control
                .check_access(&claims, "wilayah-sumut")
                .unwrap()
        );
        assert!(
            access_control
                .check_access(&claims, "wilayah-jabar")
                .unwrap()
        );
        assert!(
            access_control
                .check_access(&claims, "satker-kja001")
                .unwrap()
        );
        assert!(
            access_control
                .check_access(&claims, "satker-kja101")
                .unwrap()
        );
    }

    #[test]
    fn test_wilayah_admin_access() {
        let hierarchy = create_test_hierarchy();
        let access_control = NamespaceAccessControl::new(hierarchy);

        let claims = JwtClaims::new_test(
            "admin-wilayah-sumut".to_string(),
            AdminLevel::Wilayah,
            None,
            Some("sumut".to_string()),
        );

        // Wilayah admin should have access to their wilayah
        assert!(
            access_control
                .check_access(&claims, "wilayah-sumut")
                .unwrap()
        );

        // And child satkers
        assert!(
            access_control
                .check_access(&claims, "satker-kja001")
                .unwrap()
        );
        assert!(
            access_control
                .check_access(&claims, "satker-kja002")
                .unwrap()
        );

        // But not other wilayah or their satkers
        assert!(
            !access_control
                .check_access(&claims, "wilayah-jabar")
                .unwrap()
        );
        assert!(
            !access_control
                .check_access(&claims, "satker-kja101")
                .unwrap()
        );

        // And not pusat
        assert!(!access_control.check_access(&claims, "pusat").unwrap());
    }

    #[test]
    fn test_satker_admin_access() {
        let hierarchy = create_test_hierarchy();
        let access_control = NamespaceAccessControl::new(hierarchy);

        let claims = JwtClaims::new_test(
            "admin-satker-kja001".to_string(),
            AdminLevel::Satker,
            Some("kja001".to_string()),
            Some("sumut".to_string()),
        );

        // Satker admin should only have access to their own satker
        assert!(
            access_control
                .check_access(&claims, "satker-kja001")
                .unwrap()
        );

        // Not other satkers
        assert!(
            !access_control
                .check_access(&claims, "satker-kja002")
                .unwrap()
        );
        assert!(
            !access_control
                .check_access(&claims, "satker-kja101")
                .unwrap()
        );

        // Not wilayah
        assert!(
            !access_control
                .check_access(&claims, "wilayah-sumut")
                .unwrap()
        );

        // Not pusat
        assert!(!access_control.check_access(&claims, "pusat").unwrap());
    }

    #[test]
    fn test_get_accessible_namespaces() {
        let hierarchy = create_test_hierarchy();
        let access_control = NamespaceAccessControl::new(hierarchy);

        // Pusat admin
        let pusat_claims =
            JwtClaims::new_test("admin-pusat".to_string(), AdminLevel::Pusat, None, None);
        let pusat_accessible = access_control.get_accessible_namespaces(&pusat_claims);
        assert_eq!(pusat_accessible.len(), 6); // All namespaces

        // Wilayah admin
        let wilayah_claims = JwtClaims::new_test(
            "admin-wilayah-sumut".to_string(),
            AdminLevel::Wilayah,
            None,
            Some("sumut".to_string()),
        );
        let wilayah_accessible = access_control.get_accessible_namespaces(&wilayah_claims);
        assert_eq!(wilayah_accessible.len(), 3); // wilayah-sumut + 2 satkers
        assert!(wilayah_accessible.contains(&"wilayah-sumut".to_string()));
        assert!(wilayah_accessible.contains(&"satker-kja001".to_string()));
        assert!(wilayah_accessible.contains(&"satker-kja002".to_string()));

        // Satker admin
        let satker_claims = JwtClaims::new_test(
            "admin-satker-kja001".to_string(),
            AdminLevel::Satker,
            Some("kja001".to_string()),
            Some("sumut".to_string()),
        );
        let satker_accessible = access_control.get_accessible_namespaces(&satker_claims);
        assert_eq!(satker_accessible.len(), 1); // Only own satker
        assert_eq!(satker_accessible[0], "satker-kja001");
    }

    #[test]
    fn test_validate_secret_access() {
        let hierarchy = create_test_hierarchy();
        let access_control = NamespaceAccessControl::new(hierarchy);

        let claims = JwtClaims::new_test(
            "admin-satker-kja001".to_string(),
            AdminLevel::Satker,
            Some("kja001".to_string()),
            Some("sumut".to_string()),
        );

        // Valid access to own satker
        let result = access_control.validate_secret_access(&claims, "satker-kja001/db/password");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "satker-kja001");

        // Invalid access to other satker
        let result = access_control.validate_secret_access(&claims, "satker-kja002/db/password");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_access_detailed() {
        let hierarchy = create_test_hierarchy();
        let access_control = NamespaceAccessControl::new(hierarchy);

        let claims = JwtClaims::new_test(
            "admin-satker-kja001".to_string(),
            AdminLevel::Satker,
            Some("kja001".to_string()),
            Some("sumut".to_string()),
        );

        // Allowed access
        let result = access_control.check_access_detailed(&claims, "satker-kja001");
        assert!(result.allowed);
        assert!(result.reason.is_none());
        assert_eq!(result.accessible_namespaces.len(), 1);

        // Denied access
        let result = access_control.check_access_detailed(&claims, "satker-kja002");
        assert!(!result.allowed);
        assert!(result.reason.is_some());
    }

    #[test]
    fn test_filter_accessible_namespaces() {
        let hierarchy = create_test_hierarchy();
        let access_control = NamespaceAccessControl::new(hierarchy);

        let claims = JwtClaims::new_test(
            "admin-wilayah-sumut".to_string(),
            AdminLevel::Wilayah,
            None,
            Some("sumut".to_string()),
        );

        let all_namespaces = vec![
            "pusat".to_string(),
            "wilayah-sumut".to_string(),
            "wilayah-jabar".to_string(),
            "satker-kja001".to_string(),
            "satker-kja002".to_string(),
            "satker-kja101".to_string(),
        ];

        let filtered = access_control.filter_accessible_namespaces(&claims, &all_namespaces);

        // Should only include wilayah-sumut and its satkers
        assert_eq!(filtered.len(), 3);
        assert!(filtered.contains(&"wilayah-sumut".to_string()));
        assert!(filtered.contains(&"satker-kja001".to_string()));
        assert!(filtered.contains(&"satker-kja002".to_string()));
        assert!(!filtered.contains(&"pusat".to_string()));
        assert!(!filtered.contains(&"wilayah-jabar".to_string()));
        assert!(!filtered.contains(&"satker-kja101".to_string()));
    }

    #[test]
    fn test_missing_claims() {
        let hierarchy = create_test_hierarchy();
        let access_control = NamespaceAccessControl::new(hierarchy);

        // Wilayah admin without wilayah_code
        let claims =
            JwtClaims::new_test("admin-wilayah".to_string(), AdminLevel::Wilayah, None, None);

        let result = access_control.check_access(&claims, "wilayah-sumut");
        assert!(result.is_err());

        // Satker admin without satker_code
        let claims =
            JwtClaims::new_test("admin-satker".to_string(), AdminLevel::Satker, None, None);

        let result = access_control.check_access(&claims, "satker-kja001");
        assert!(result.is_err());
    }
}
