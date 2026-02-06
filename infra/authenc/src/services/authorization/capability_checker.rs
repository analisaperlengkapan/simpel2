//! Capability-based authorization checker for dynamic permissions.
//!
//! This module provides a simplified interface for checking user capabilities
//! using the DynamicRoleStore, replacing hardcoded role string checks like:
//! ```ignore
//! if !user.roles.iter().any(|r| r == "admin" || r == "config-admin") { ... }
//! ```
//! With capability-based checks:
//! ```ignore
//! if !capabilities.user_has_capability(&db_pool, &user.id, "config:read").await? { ... }
//! ```
//!
//! ## Standard Capabilities
//!
//! The system uses hierarchical capability codes following Vault/Keycloak patterns:
//!
//! | Capability | Description |
//! |------------|-------------|
//! | `config:read` | Read configuration |
//! | `config:write` | Write configuration |
//! | `config:delete` | Delete configuration |
//! | `config:reload` | Hot reload configuration |
//! | `users:read` | View user information |
//! | `users:write` | Create/update users |
//! | `users:delete` | Delete users |
//! | `audit:read` | View audit logs |
//! | `mfa:admin` | Administer MFA settings |
//! | `mfa:bypass` | Reset/bypass MFA for users |
//! | `secrets:read` | Read secrets |
//! | `secrets:write` | Write secrets |
//! | `policies:admin` | Manage policies |

use crate::database::Database;
use crate::error::AuthencError;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Standard capability codes used across the system.
/// These map to database-defined capabilities for consistency.
pub mod capabilities {
    // Configuration capabilities
    pub const CONFIG_READ: &str = "config:read";
    pub const CONFIG_WRITE: &str = "config:write";
    pub const CONFIG_DELETE: &str = "config:delete";
    pub const CONFIG_RELOAD: &str = "config:reload";

    // User management capabilities
    pub const USERS_READ: &str = "users:read";
    pub const USERS_WRITE: &str = "users:write";
    pub const USERS_DELETE: &str = "users:delete";
    pub const USERS_ADMIN: &str = "users:admin";

    // Audit capabilities
    pub const AUDIT_READ: &str = "audit:read";
    pub const AUDIT_EXPORT: &str = "audit:export";

    // MFA capabilities
    pub const MFA_ADMIN: &str = "mfa:admin";
    pub const MFA_BYPASS: &str = "mfa:bypass";
    pub const MFA_VIEW: &str = "mfa:view";

    // Secrets capabilities (delegated to Secreton)
    pub const SECRETS_READ: &str = "secrets:read";
    pub const SECRETS_WRITE: &str = "secrets:write";
    pub const SECRETS_DELETE: &str = "secrets:delete";
    pub const SECRETS_ADMIN: &str = "secrets:admin";

    // Policy capabilities
    pub const POLICIES_READ: &str = "policies:read";
    pub const POLICIES_WRITE: &str = "policies:write";
    pub const POLICIES_ADMIN: &str = "policies:admin";

    // Session capabilities
    pub const SESSIONS_READ: &str = "sessions:read";
    pub const SESSIONS_REVOKE: &str = "sessions:revoke";

    // System administration
    pub const SYSTEM_ADMIN: &str = "system:admin";
    pub const SYSTEM_HEALTH: &str = "system:health";
}

/// Cached user capabilities with TTL
#[derive(Debug)]
struct CachedCapabilities {
    capabilities: HashSet<String>,
    cached_at: Instant,
}

/// Capability checker with caching support.
///
/// Provides efficient capability checks by caching user capabilities
/// and supporting bulk lookups.
pub struct CapabilityChecker {
    database: Arc<Database>,
    cache: Arc<RwLock<HashMap<Uuid, CachedCapabilities>>>,
    cache_ttl: Duration,
}

impl CapabilityChecker {
    /// Create a new capability checker with the given database connection.
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes default
        }
    }

    /// Create a new capability checker with custom cache TTL.
    pub fn with_ttl(database: Arc<Database>, cache_ttl: Duration) -> Self {
        Self {
            database,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
        }
    }

    /// Check if a user has a specific capability.
    ///
    /// This method first checks the cache, then queries the database
    /// via the `v_user_effective_capabilities` view if needed.
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user to check
    /// * `capability` - The capability code to check (e.g., "config:read")
    ///
    /// # Returns
    /// * `Ok(true)` if the user has the capability
    /// * `Ok(false)` if the user does not have the capability
    /// * `Err` if there was a database error
    pub async fn user_has_capability(
        &self,
        user_id: &Uuid,
        capability: &str,
    ) -> Result<bool, AuthencError> {
        let capabilities = self.get_user_capabilities(user_id).await?;

        // Check exact match first
        if capabilities.contains(capability) {
            return Ok(true);
        }

        // Check wildcard patterns (e.g., "config:*" matches "config:read")
        let parts: Vec<&str> = capability.split(':').collect();
        if parts.len() >= 2 {
            let wildcard = format!("{}:*", parts[0]);
            if capabilities.contains(&wildcard) {
                return Ok(true);
            }
        }

        // Check if user has system:admin (grants all capabilities)
        if capabilities.contains(capabilities::SYSTEM_ADMIN) {
            return Ok(true);
        }

        Ok(false)
    }

    /// Check if a user has any of the specified capabilities.
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user to check
    /// * `required_capabilities` - Slice of capability codes to check
    ///
    /// # Returns
    /// * `Ok(true)` if the user has at least one of the capabilities
    /// * `Ok(false)` if the user has none of the capabilities
    pub async fn user_has_any_capability(
        &self,
        user_id: &Uuid,
        required_capabilities: &[&str],
    ) -> Result<bool, AuthencError> {
        for cap in required_capabilities {
            if self.user_has_capability(user_id, cap).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Check if a user has all of the specified capabilities.
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user to check
    /// * `required_capabilities` - Slice of capability codes to check
    ///
    /// # Returns
    /// * `Ok(true)` if the user has all of the capabilities
    /// * `Ok(false)` if the user is missing any capability
    pub async fn user_has_all_capabilities(
        &self,
        user_id: &Uuid,
        required_capabilities: &[&str],
    ) -> Result<bool, AuthencError> {
        for cap in required_capabilities {
            if !self.user_has_capability(user_id, cap).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Get all effective capabilities for a user.
    ///
    /// Capabilities are resolved through the role hierarchy:
    /// User → Roles → Capabilities
    ///
    /// Results are cached for efficiency.
    pub async fn get_user_capabilities(
        &self,
        user_id: &Uuid,
    ) -> Result<HashSet<String>, AuthencError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(user_id) {
                if cached.cached_at.elapsed() < self.cache_ttl {
                    return Ok(cached.capabilities.clone());
                }
            }
        }

        // Query database
        let capabilities = self.load_user_capabilities(user_id).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                *user_id,
                CachedCapabilities {
                    capabilities: capabilities.clone(),
                    cached_at: Instant::now(),
                },
            );
        }

        Ok(capabilities)
    }

    /// Load user capabilities from database.
    async fn load_user_capabilities(
        &self,
        user_id: &Uuid,
    ) -> Result<HashSet<String>, AuthencError> {
        let pool = self.database.get_pool();
        let conn = pool.get().await.map_err(|e| {
            AuthencError::internal(format!("Failed to get database connection: {}", e))
        })?;

        // Use the pre-built view that handles role hierarchy and policy resolution
        // The v_user_effective_capabilities view resolves:
        // - user → roles → capabilities (through role_capabilities)
        // - user → policies → capabilities (through user_policies)
        let rows = conn
            .query(
                r#"
                SELECT DISTINCT capability_code
                FROM v_user_effective_capabilities
                WHERE user_id = $1
                "#,
                &[user_id],
            )
            .await
            .map_err(|e| AuthencError::internal(format!("Failed to query capabilities: {}", e)))?;

        let mut capabilities = HashSet::new();
        for row in rows {
            let code: String = row.get(0);
            capabilities.insert(code);
        }

        Ok(capabilities)
    }

    /// Invalidate cache for a specific user.
    ///
    /// Call this when user's roles or capabilities change.
    pub async fn invalidate_user(&self, user_id: &Uuid) {
        let mut cache = self.cache.write().await;
        cache.remove(user_id);
    }

    /// Invalidate entire cache.
    ///
    /// Call this when role-capability mappings change.
    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Check authorization and return error if denied.
    ///
    /// Convenience method for handlers that combines check + error response.
    ///
    /// # Example
    /// ```ignore
    /// async fn get_config(
    ///     State(state): State<AppState>,
    ///     AuthenticatedUser { user, .. }: AuthenticatedUser,
    /// ) -> Result<impl IntoResponse, AuthencError> {
    ///     state.capability_checker.require_capability(&user.id, "config:read").await?;
    ///     // ... handler logic
    /// }
    /// ```
    pub async fn require_capability(
        &self,
        user_id: &Uuid,
        capability: &str,
    ) -> Result<(), AuthencError> {
        if self.user_has_capability(user_id, capability).await? {
            Ok(())
        } else {
            Err(AuthencError::Forbidden {
                message: format!("Missing required capability: {}", capability),
            })
        }
    }

    /// Require any of the specified capabilities.
    pub async fn require_any_capability(
        &self,
        user_id: &Uuid,
        capabilities: &[&str],
    ) -> Result<(), AuthencError> {
        if self.user_has_any_capability(user_id, capabilities).await? {
            Ok(())
        } else {
            Err(AuthencError::Forbidden {
                message: format!(
                    "Missing required capabilities: {}",
                    capabilities.join(" or ")
                ),
            })
        }
    }

    /// Require all of the specified capabilities.
    pub async fn require_all_capabilities(
        &self,
        user_id: &Uuid,
        capabilities: &[&str],
    ) -> Result<(), AuthencError> {
        if self
            .user_has_all_capabilities(user_id, capabilities)
            .await?
        {
            Ok(())
        } else {
            Err(AuthencError::Forbidden {
                message: format!(
                    "Missing required capabilities: {}",
                    capabilities.join(" and ")
                ),
            })
        }
    }
}

/// Convenience functions for common authorization patterns.
impl CapabilityChecker {
    /// Check if user is a system administrator.
    pub async fn is_system_admin(&self, user_id: &Uuid) -> Result<bool, AuthencError> {
        self.user_has_capability(user_id, capabilities::SYSTEM_ADMIN)
            .await
    }

    /// Check if user can manage configuration.
    pub async fn can_manage_config(&self, user_id: &Uuid) -> Result<bool, AuthencError> {
        self.user_has_any_capability(
            user_id,
            &[capabilities::CONFIG_WRITE, capabilities::SYSTEM_ADMIN],
        )
        .await
    }

    /// Check if user can manage users.
    pub async fn can_manage_users(&self, user_id: &Uuid) -> Result<bool, AuthencError> {
        self.user_has_any_capability(
            user_id,
            &[capabilities::USERS_ADMIN, capabilities::SYSTEM_ADMIN],
        )
        .await
    }

    /// Check if user can view audit logs.
    pub async fn can_view_audit(&self, user_id: &Uuid) -> Result<bool, AuthencError> {
        self.user_has_capability(user_id, capabilities::AUDIT_READ)
            .await
    }

    /// Check if user can administer MFA.
    pub async fn can_admin_mfa(&self, user_id: &Uuid) -> Result<bool, AuthencError> {
        self.user_has_any_capability(
            user_id,
            &[capabilities::MFA_ADMIN, capabilities::SYSTEM_ADMIN],
        )
        .await
    }
}

/// Role-to-capability migration helper.
///
/// Maps legacy hardcoded role names to dynamic capabilities.
/// Used during migration from role-based to capability-based authorization.
pub fn legacy_role_to_capabilities(role: &str) -> Vec<&'static str> {
    match role.to_lowercase().as_str() {
        "admin" | "system_admin" => vec![
            capabilities::SYSTEM_ADMIN,
            capabilities::CONFIG_READ,
            capabilities::CONFIG_WRITE,
            capabilities::CONFIG_DELETE,
            capabilities::CONFIG_RELOAD,
            capabilities::USERS_ADMIN,
            capabilities::AUDIT_READ,
            capabilities::AUDIT_EXPORT,
            capabilities::MFA_ADMIN,
            capabilities::MFA_BYPASS,
            capabilities::POLICIES_ADMIN,
            capabilities::SESSIONS_READ,
            capabilities::SESSIONS_REVOKE,
        ],
        "config-admin" | "config_admin" => vec![
            capabilities::CONFIG_READ,
            capabilities::CONFIG_WRITE,
            capabilities::CONFIG_DELETE,
            capabilities::CONFIG_RELOAD,
        ],
        "mfa_admin" => vec![
            capabilities::MFA_ADMIN,
            capabilities::MFA_BYPASS,
            capabilities::MFA_VIEW,
        ],
        "auditor" => vec![capabilities::AUDIT_READ, capabilities::AUDIT_EXPORT],
        "user_manager" => vec![
            capabilities::USERS_READ,
            capabilities::USERS_WRITE,
            capabilities::USERS_DELETE,
        ],
        "superuser" => vec![capabilities::SYSTEM_ADMIN],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_role_mapping() {
        let admin_caps = legacy_role_to_capabilities("admin");
        assert!(admin_caps.contains(&capabilities::SYSTEM_ADMIN));
        assert!(admin_caps.contains(&capabilities::CONFIG_READ));

        let config_admin_caps = legacy_role_to_capabilities("config-admin");
        assert!(config_admin_caps.contains(&capabilities::CONFIG_READ));
        assert!(config_admin_caps.contains(&capabilities::CONFIG_WRITE));
        assert!(!config_admin_caps.contains(&capabilities::SYSTEM_ADMIN));

        let unknown_caps = legacy_role_to_capabilities("unknown_role");
        assert!(unknown_caps.is_empty());
    }

    #[test]
    fn test_capability_codes_format() {
        // Verify all capability codes follow the namespace:action pattern
        let all_caps = [
            capabilities::CONFIG_READ,
            capabilities::CONFIG_WRITE,
            capabilities::USERS_READ,
            capabilities::AUDIT_READ,
            capabilities::MFA_ADMIN,
            capabilities::SYSTEM_ADMIN,
        ];

        for cap in all_caps {
            assert!(cap.contains(':'), "Capability should contain ':': {}", cap);
            let parts: Vec<&str> = cap.split(':').collect();
            assert_eq!(
                parts.len(),
                2,
                "Capability should have exactly two parts: {}",
                cap
            );
        }
    }
}
