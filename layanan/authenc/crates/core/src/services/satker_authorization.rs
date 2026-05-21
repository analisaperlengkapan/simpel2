//! Satker authorization service for hierarchy-aware access control

use authenc_types::domain::user::Role;
use authenc_types::domain::user::User;
use authenc_types::domain::{CrossSatkerValidation, Satker, SatkerHierarchy};
use authenc_types::error::AuthencError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Satker authorization service for hierarchy-aware access control
#[derive(Clone)]
pub struct SatkerAuthorizationService {
    /// Satker hierarchy for traversal
    hierarchy: Arc<RwLock<SatkerHierarchy>>,
    /// Cache for authorization decisions
    decision_cache: Arc<RwLock<HashMap<String, CrossSatkerValidation>>>,
}

impl SatkerAuthorizationService {
    /// Create a new satker authorization service
    pub fn new(satkers: Vec<Satker>) -> Self {
        let hierarchy = SatkerHierarchy::new(satkers);

        Self {
            hierarchy: Arc::new(RwLock::new(hierarchy)),
            decision_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update the satker hierarchy
    pub async fn update_hierarchy(&self, satkers: Vec<Satker>) {
        let hierarchy = SatkerHierarchy::new(satkers);
        let mut h = self.hierarchy.write().await;
        *h = hierarchy;

        // Clear cache when hierarchy changes
        let mut cache = self.decision_cache.write().await;
        cache.clear();
    }

    /// Check if a user can access resources in a target satker
    pub async fn can_access_satker(
        &self,
        user: &User,
        target_satker_code: &str,
    ) -> Result<bool, AuthencError> {
        let hierarchy = self.hierarchy.read().await;

        // User's own satker is always accessible
        if user.satker_code == target_satker_code {
            return Ok(true);
        }

        // Check if user has roles that grant access to the target satker
        for role in &user.roles {
            if self.role_grants_access(role, target_satker_code, &hierarchy) {
                return Ok(true);
            }
        }

        // Check if user is an admin with appropriate level
        if let Some(admin_level) = self.get_user_admin_level(user)
            && self.admin_can_access(&admin_level, target_satker_code, &hierarchy)
        {
            return Ok(true);
        }

        Ok(false)
    }

    /// Check if a role grants access to a target satker
    fn role_grants_access(
        &self,
        role: &Role,
        target_satker_code: &str,
        hierarchy: &SatkerHierarchy,
    ) -> bool {
        if let Some(scope) = &role.scope {
            if scope == "global" || scope == "pusat" {
                return true;
            }

            if let Some(wilayah) = scope.strip_prefix("wilayah:") {
                return target_satker_code.starts_with(wilayah)
                    || hierarchy.is_descendant(target_satker_code, wilayah);
            }

            if let Some(satker) = scope.strip_prefix("satker:") {
                return satker == target_satker_code
                    || hierarchy.is_descendant(target_satker_code, satker);
            }

            // Legacy/Direct match support
            if scope == target_satker_code {
                return true;
            }
        }
        false
    }

    /// Check if an admin level can access a target satker
    fn admin_can_access(
        &self,
        admin_level: &str,
        target_satker_code: &str,
        hierarchy: &SatkerHierarchy,
    ) -> bool {
        match admin_level {
            "pusat" | "eselon_i" => true,
            "wilayah" => {
                // Determine user's wilayah from context (not passed here currently)
                // This is a limitation of stateless check without full user context
                // But usually admin_level string might contain scope like "wilayah:DKI"
                // If it's just "wilayah", we'd need to know WHICH wilayah.
                // Assuming admin_level might encode scope or we rely on role scope.
                // For now, let's assume strict code checks are done in role_grants_access.
                // If admin_level is just "wilayah", we can't really know unless we look up user's satker.
                // Ideally admin_level should be descriptive or paired with scope.
                false // Safer default if we can't verify scope
            }
            "satker" => {
                // Similar issue, need specific satker code.
                false
            }
            val => {
                // If value contains specific code
                if let Some(w) = val.strip_prefix("wilayah:") {
                    return target_satker_code.starts_with(w)
                        || hierarchy.is_descendant(target_satker_code, w);
                }
                if let Some(s) = val.strip_prefix("satker:") {
                    return s == target_satker_code
                        || hierarchy.is_descendant(target_satker_code, s);
                }
                false
            }
        }
    }

    /// Get the admin level of a user (if any)
    fn get_user_admin_level(&self, user: &User) -> Option<String> {
        // Check roles for admin roles
        for role in &user.roles {
            // Only consider roles that are explicitly identified as admin roles
            // This prevents privilege escalation from non-admin roles that might have managed_by set
            if role.name.to_lowercase().contains("admin") {
                if let Some(managed_by) = &role.managed_by {
                    return Some(managed_by.clone());
                }
                // Fallback for roles named "admin" without managed_by set (legacy/migration)
                if let Some(scope) = &role.scope {
                    if scope == "pusat" {
                        return Some("pusat".to_string());
                    }
                    if scope.starts_with("wilayah") {
                        return Some(scope.clone());
                    } // Treat scope as level code
                    if scope.starts_with("satker") {
                        return Some(scope.clone());
                    }
                }
            }
        }
        None
    }

    /// Validate a cross-satker operation
    pub async fn validate_cross_satker_operation(
        &self,
        user: &User,
        source_satker: &str,
        target_satker: &str,
        operation: &str,
        resource_type: &str,
    ) -> Result<CrossSatkerValidation, AuthencError> {
        // Create cache key
        let cache_key = format!(
            "{}:{}:{}:{}:{}",
            user.id, source_satker, target_satker, operation, resource_type
        );

        // Check cache
        {
            let cache = self.decision_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let hierarchy = self.hierarchy.read().await;

        // Validate that both satkers exist
        if hierarchy.get_satker(source_satker).is_none() {
            return Ok(CrossSatkerValidation::denied(
                format!("Source satker '{}' not found", source_satker),
                vec![],
                vec![source_satker.to_string(), target_satker.to_string()],
            ));
        }

        if hierarchy.get_satker(target_satker).is_none() {
            return Ok(CrossSatkerValidation::denied(
                format!("Target satker '{}' not found", target_satker),
                vec![],
                vec![source_satker.to_string(), target_satker.to_string()],
            ));
        }

        // Check if user's satker matches source
        if user.satker_code != source_satker {
            return Ok(CrossSatkerValidation::denied(
                format!(
                    "User satker '{}' does not match source satker '{}'",
                    user.satker_code, source_satker
                ),
                vec![format!("satker:{}", source_satker)],
                vec![source_satker.to_string(), target_satker.to_string()],
            ));
        }

        // Check if user has permission to access target satker
        let can_access = self.can_access_satker(user, target_satker).await?;

        if !can_access {
            let missing_permissions = vec![
                format!("{}:{}:{}", resource_type, operation, target_satker),
                format!("cross_satker:{}:{}", operation, target_satker),
            ];

            return Ok(CrossSatkerValidation::denied(
                format!(
                    "User does not have permission to perform '{}' on '{}' in satker '{}'",
                    operation, resource_type, target_satker
                ),
                missing_permissions,
                vec![source_satker.to_string(), target_satker.to_string()],
            ));
        }

        // Check if user has the specific permission for the operation
        let has_permission = user.permissions.iter().any(|p| {
            p.resource_type == resource_type
                && p.action == operation
                && p.applies_to_resource(resource_type, target_satker, target_satker)
        });

        let result = if has_permission {
            CrossSatkerValidation::allowed(
                format!(
                    "User has permission to perform '{}' on '{}' across satkers",
                    operation, resource_type
                ),
                vec![source_satker.to_string(), target_satker.to_string()],
            )
        } else {
            CrossSatkerValidation::denied(
                format!(
                    "User lacks specific permission for '{}' on '{}'",
                    operation, resource_type
                ),
                vec![format!("{}:{}:{}", resource_type, operation, target_satker)],
                vec![source_satker.to_string(), target_satker.to_string()],
            )
        };

        // Cache the result
        {
            let mut cache = self.decision_cache.write().await;
            cache.insert(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Get all satkers accessible by a user
    pub async fn get_accessible_satkers(&self, user: &User) -> Result<Vec<String>, AuthencError> {
        let hierarchy = self.hierarchy.read().await;
        let mut accessible = vec![user.satker_code.clone()];

        // Add satkers from role scopes
        for role in &user.roles {
            if let Some(scope) = &role.scope {
                if scope == "global" || scope == "pusat" {
                    return Ok(hierarchy.satkers.keys().cloned().collect());
                }

                if let Some(wilayah) = scope.strip_prefix("wilayah:") {
                    accessible.extend(
                        hierarchy
                            .satkers
                            .keys()
                            .filter(|code| code.starts_with(wilayah))
                            .cloned(),
                    );
                } else if let Some(satker) = scope.strip_prefix("satker:") {
                    accessible.push(satker.to_string());
                    accessible.extend(hierarchy.get_descendants(satker));
                }
            }
        }

        // Remove duplicates
        accessible.sort();
        accessible.dedup();

        Ok(accessible)
    }

    /// Check if a user can manage (admin) a target satker
    pub async fn can_manage_satker(
        &self,
        user: &User,
        target_satker_code: &str,
    ) -> Result<bool, AuthencError> {
        let hierarchy = self.hierarchy.read().await;

        // Get user's admin level
        let admin_level = match self.get_user_admin_level(user) {
            Some(level) => level,
            None => return Ok(false), // Not an admin
        };

        // Check if admin level can manage the target satker
        match admin_level.as_str() {
            "pusat" | "eselon_i" => Ok(true),
            val => {
                if let Some(w) = val.strip_prefix("wilayah:") {
                    Ok(target_satker_code.starts_with(w)
                        || hierarchy.is_descendant(target_satker_code, w))
                } else if let Some(s) = val.strip_prefix("satker:") {
                    Ok(s == target_satker_code || hierarchy.is_descendant(target_satker_code, s))
                } else {
                    // Fallback: If managed_by is just "wilayah", we can't check efficiently here without more context
                    Ok(false)
                }
            }
        }
    }

    /// Get satker hierarchy information
    pub async fn get_satker_hierarchy(
        &self,
        satker_code: &str,
    ) -> Result<SatkerHierarchyInfo, AuthencError> {
        let hierarchy = self.hierarchy.read().await;

        let satker = hierarchy.get_satker(satker_code).ok_or_else(|| {
            AuthencError::not_found(format!("Satker '{}' not found", satker_code))
        })?;

        let parent = hierarchy
            .get_parent(satker_code)
            .and_then(|p| hierarchy.get_satker(p))
            .cloned();

        let children: Vec<Satker> = hierarchy
            .get_children(satker_code)
            .into_iter()
            .filter_map(|c| hierarchy.get_satker(c))
            .cloned()
            .collect();

        let ancestors: Vec<Satker> = hierarchy
            .get_ancestors(satker_code)
            .into_iter()
            .filter_map(|a| hierarchy.get_satker(&a))
            .cloned()
            .collect();

        let descendants: Vec<Satker> = hierarchy
            .get_descendants(satker_code)
            .into_iter()
            .filter_map(|d| hierarchy.get_satker(&d))
            .cloned()
            .collect();

        Ok(SatkerHierarchyInfo {
            satker: satker.clone(),
            parent,
            children,
            ancestors,
            descendants,
            level: satker.level,
        })
    }

    /// Clear the authorization decision cache
    pub async fn clear_cache(&self) {
        let mut cache = self.decision_cache.write().await;
        cache.clear();
    }

    /// Clear cache for a specific user
    pub async fn clear_user_cache(&self, user_id: &uuid::Uuid) {
        let mut cache = self.decision_cache.write().await;
        let user_id_str = user_id.to_string();
        cache.retain(|key, _| !key.starts_with(&user_id_str));
    }
}

/// Satker hierarchy information response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SatkerHierarchyInfo {
    /// The satker itself
    pub satker: Satker,
    /// Parent satker (if any)
    pub parent: Option<Satker>,
    /// Direct children satkers
    pub children: Vec<Satker>,
    /// All ancestors (from parent to root)
    pub ancestors: Vec<Satker>,
    /// All descendants (all children recursively)
    pub descendants: Vec<Satker>,
    /// Level in the hierarchy
    pub level: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use authenc_types::domain::Satker;
    use authenc_types::domain::satker::SatkerType;
    use authenc_types::domain::user::Role;
    use authenc_types::domain::user::User;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_satker(code: &str, parent: Option<&str>, level: i32) -> Satker {
        Satker {
            id: Uuid::new_v4(),
            code: code.to_string(),
            name: format!("Satker {}", code),
            description: None,
            parent_code: parent.map(|s| s.to_string()),
            level,
            satker_type: SatkerType::KejaksaanNegeri,
            active: true,
            attributes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_user(satker_code: &str, roles: Vec<Role>) -> User {
        User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            email_verified: true,
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            nip: None,
            nama: None,
            jabatan: None,
            satker_code: satker_code.to_string(),
            phone_number: None,
            phone_verified: false,
            password_hash: None,
            totp_secret: None,
            totp_backup_codes: None,
            mfa_enabled: false,
            mfa_setup_at: None,
            mfa_last_used: None,
            webauthn_enabled: false,
            account_locked: false,
            account_locked_until: None,
            failed_login_attempts: 0,
            last_login_at: None,
            last_failed_login_at: None,
            password_changed_at: None,
            password_expires_at: None,
            require_password_change: false,
            realm_id: None,
            organization_id: None,
            roles,
            permissions: vec![],
            session_data: None,
            security_context: Default::default(),
            attributes: None,
            enabled: true,
            federated: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            login_count: 0,
        }
    }

    #[tokio::test]
    async fn test_can_access_own_satker() {
        let satkers = vec![create_test_satker("SAT001", None, 0)];
        let service = SatkerAuthorizationService::new(satkers);

        let user = create_test_user("SAT001", vec![]);
        let can_access = service.can_access_satker(&user, "SAT001").await.unwrap();

        assert!(can_access);
    }

    #[tokio::test]
    async fn test_pusat_can_access_all() {
        let satkers = vec![
            create_test_satker("PUSAT", None, 0),
            create_test_satker("SAT001", Some("PUSAT"), 1),
        ];
        let service = SatkerAuthorizationService::new(satkers);

        let role = Role {
            id: Uuid::new_v4(),
            name: "admin".to_string(),
            description: None,
            scope: Some("pusat".to_string()),
            permissions: vec![],
            managed_by: Some("pusat".to_string()),
            realm_id: None,
            composite: false,
            client_role: false,
            client_id: None,
            priority: 0,
            active: true,
            attributes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user = create_test_user("PUSAT", vec![role]);
        let can_access = service.can_access_satker(&user, "SAT001").await.unwrap();

        assert!(can_access);
    }

    #[tokio::test]
    async fn test_parent_can_access_child() {
        let satkers = vec![
            create_test_satker("PARENT", None, 0),
            create_test_satker("CHILD", Some("PARENT"), 1),
        ];
        let service = SatkerAuthorizationService::new(satkers);

        let role = Role {
            id: Uuid::new_v4(),
            name: "manager".to_string(),
            description: None,
            scope: Some("satker:PARENT".to_string()),
            permissions: vec![],
            managed_by: Some("satker:PARENT".to_string()),
            realm_id: None,
            composite: false,
            client_role: false,
            client_id: None,
            priority: 0,
            active: true,
            attributes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user = create_test_user("PARENT", vec![role]);
        let can_access = service.can_access_satker(&user, "CHILD").await.unwrap();

        assert!(can_access);
    }
}
