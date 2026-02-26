//! Dynamic Role Service for Secreton
//!
//! This module provides the service layer for the dynamic role system,
//! implementing the `DynamicRoleStore` trait for PostgreSQL storage.
//!
//! # Design
//!
//! - Role types, capabilities, policy rules stored in PostgreSQL
//! - Policies can be cached in Raft for HA (optional)
//! - Caching layer reduces database round-trips
//! - Follows HashiCorp Vault best practices

use crate::models::dynamic_role::{
    AuthMethodType, Capability, DynamicRoleStore, EffectiveCapabilities, EngineRoleType,
    PathAccessResult, PolicyEffect, PolicyRule, SshKeyType, TokenType, UserRoleType,
};
use chrono::Utc;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// CACHE STRUCTURES
// ============================================================================

/// Cached data with TTL
#[derive(Clone)]
struct CachedData<T: Clone> {
    data: T,
    cached_at: chrono::DateTime<chrono::Utc>,
}

impl<T: Clone> CachedData<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            cached_at: Utc::now(),
        }
    }

    fn is_expired(&self, ttl_seconds: i64) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.cached_at);
        elapsed.num_seconds() > ttl_seconds
    }
}

/// Cache for dynamic role data
pub struct DynamicRoleCache {
    user_role_types: RwLock<Option<CachedData<Vec<UserRoleType>>>>,
    capabilities: RwLock<Option<CachedData<Vec<Capability>>>>,
    auth_method_types: RwLock<Option<CachedData<Vec<AuthMethodType>>>>,
    token_types: RwLock<Option<CachedData<Vec<TokenType>>>>,
    ssh_key_types: RwLock<Option<CachedData<Vec<SshKeyType>>>>,
    engine_role_types: RwLock<Option<CachedData<Vec<EngineRoleType>>>>,
    /// Cache TTL in seconds (default: 5 minutes)
    ttl_seconds: i64,
}

impl Default for DynamicRoleCache {
    fn default() -> Self {
        Self {
            user_role_types: RwLock::new(None),
            capabilities: RwLock::new(None),
            auth_method_types: RwLock::new(None),
            token_types: RwLock::new(None),
            ssh_key_types: RwLock::new(None),
            engine_role_types: RwLock::new(None),
            ttl_seconds: 300, // 5 minutes
        }
    }
}

impl DynamicRoleCache {
    /// Create new cache with custom TTL
    pub fn with_ttl(ttl_seconds: i64) -> Self {
        Self {
            ttl_seconds,
            ..Default::default()
        }
    }

    /// Invalidate all cached data
    pub async fn invalidate_all(&self) {
        *self.user_role_types.write().await = None;
        *self.capabilities.write().await = None;
        *self.auth_method_types.write().await = None;
        *self.token_types.write().await = None;
        *self.ssh_key_types.write().await = None;
        *self.engine_role_types.write().await = None;
    }
}

// ============================================================================
// STORAGE BACKEND TRAIT
// ============================================================================

/// Trait for backend storage operations
/// This abstraction allows switching between PostgreSQL and Raft storage
#[async_trait::async_trait]
pub trait DynamicRoleBackend: Send + Sync {
    /// Query rows from database
    async fn query_rows(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, String>;

    /// Execute a command
    async fn execute(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, String>;
}

// ============================================================================
// POSTGRESQL IMPLEMENTATION
// ============================================================================

/// PostgreSQL-backed implementation of DynamicRoleStore
pub struct PostgresDynamicRoleService<B: DynamicRoleBackend> {
    backend: B,
    cache: Arc<DynamicRoleCache>,
}

impl<B: DynamicRoleBackend> PostgresDynamicRoleService<B> {
    /// Create a new service with the given backend
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            cache: Arc::new(DynamicRoleCache::default()),
        }
    }

    /// Create a new service with custom cache
    pub fn with_cache(backend: B, cache: Arc<DynamicRoleCache>) -> Self {
        Self { backend, cache }
    }

    /// Get the cache
    pub fn cache(&self) -> Arc<DynamicRoleCache> {
        Arc::clone(&self.cache)
    }
}

#[async_trait::async_trait]
impl<B: DynamicRoleBackend + 'static> DynamicRoleStore for PostgresDynamicRoleService<B> {
    async fn get_user_role_types(&self) -> Result<Vec<UserRoleType>, String> {
        // Check cache first
        {
            let cached = self.cache.user_role_types.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let query = "
            SELECT id, code, name, description, priority, is_system,
                   default_capabilities, metadata, created_at, updated_at
            FROM user_role_types
            WHERE is_active = true
            ORDER BY priority DESC, name ASC
        ";

        let rows = self.backend.query_rows(query, &[]).await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(UserRoleType {
                id: row.get(0),
                code: row.get(1),
                name: row.get(2),
                description: row.get(3),
                priority: row.get(4),
                is_system: row.get(5),
                default_capabilities: row.get::<_, Option<Vec<String>>>(6).unwrap_or_default(),
                metadata: row
                    .get::<_, Option<serde_json::Value>>(7)
                    .unwrap_or_default(),
                created_at: row.get(8),
                updated_at: row.get(9),
            });
        }

        // Update cache
        {
            let mut cached = self.cache.user_role_types.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_user_role_type_by_code(&self, code: &str) -> Result<Option<UserRoleType>, String> {
        let types = self.get_user_role_types().await?;
        Ok(types.into_iter().find(|t| t.code == code))
    }

    async fn get_capabilities(&self) -> Result<Vec<Capability>, String> {
        // Check cache first
        {
            let cached = self.cache.capabilities.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let query = "
            SELECT id, code, name, description, category, is_system,
                   is_dangerous, requires_sudo, metadata, created_at, updated_at
            FROM capabilities
            WHERE is_active = true
            ORDER BY category ASC, code ASC
        ";

        let rows = self.backend.query_rows(query, &[]).await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(Capability {
                id: row.get(0),
                code: row.get(1),
                name: row.get(2),
                description: row.get(3),
                category: row.get(4),
                is_system: row.get(5),
                is_dangerous: row.get(6),
                requires_sudo: row.get(7),
                metadata: row
                    .get::<_, Option<serde_json::Value>>(8)
                    .unwrap_or_default(),
                created_at: row.get(9),
                updated_at: row.get(10),
            });
        }

        // Update cache
        {
            let mut cached = self.cache.capabilities.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_capability_by_code(&self, code: &str) -> Result<Option<Capability>, String> {
        let caps = self.get_capabilities().await?;
        Ok(caps.into_iter().find(|c| c.code == code))
    }

    async fn get_auth_method_types(&self) -> Result<Vec<AuthMethodType>, String> {
        // Check cache first
        {
            let cached = self.cache.auth_method_types.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let query = "
            SELECT id, code, name, description, is_system, is_external,
                   config_schema, metadata, created_at, updated_at
            FROM auth_method_types
            WHERE is_active = true
            ORDER BY code ASC
        ";

        let rows = self.backend.query_rows(query, &[]).await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(AuthMethodType {
                id: row.get(0),
                code: row.get(1),
                name: row.get(2),
                description: row.get(3),
                is_system: row.get(4),
                is_external: row.get(5),
                config_schema: row
                    .get::<_, Option<serde_json::Value>>(6)
                    .unwrap_or_default(),
                metadata: row
                    .get::<_, Option<serde_json::Value>>(7)
                    .unwrap_or_default(),
                created_at: row.get(8),
                updated_at: row.get(9),
            });
        }

        // Update cache
        {
            let mut cached = self.cache.auth_method_types.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_auth_method_type_by_code(
        &self,
        code: &str,
    ) -> Result<Option<AuthMethodType>, String> {
        let types = self.get_auth_method_types().await?;
        Ok(types.into_iter().find(|t| t.code == code))
    }

    async fn get_token_types(&self) -> Result<Vec<TokenType>, String> {
        // Check cache first
        {
            let cached = self.cache.token_types.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let query = "
            SELECT id, code, name, description, is_system, is_renewable,
                   is_orphan_allowed, max_ttl_seconds, default_ttl_seconds,
                   metadata, created_at, updated_at
            FROM token_types
            WHERE is_active = true
            ORDER BY code ASC
        ";

        let rows = self.backend.query_rows(query, &[]).await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(TokenType {
                id: row.get(0),
                code: row.get(1),
                name: row.get(2),
                description: row.get(3),
                is_system: row.get(4),
                is_renewable: row.get(5),
                is_orphan_allowed: row.get(6),
                max_ttl_seconds: row.get(7),
                default_ttl_seconds: row.get(8),
                metadata: row
                    .get::<_, Option<serde_json::Value>>(9)
                    .unwrap_or_default(),
                created_at: row.get(10),
                updated_at: row.get(11),
            });
        }

        // Update cache
        {
            let mut cached = self.cache.token_types.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_token_type_by_code(&self, code: &str) -> Result<Option<TokenType>, String> {
        let types = self.get_token_types().await?;
        Ok(types.into_iter().find(|t| t.code == code))
    }

    async fn get_ssh_key_types(&self) -> Result<Vec<SshKeyType>, String> {
        // Check cache first
        {
            let cached = self.cache.ssh_key_types.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let query = "
            SELECT id, code, name, algorithm, key_size, is_system,
                   is_deprecated, security_level, metadata, created_at, updated_at
            FROM ssh_key_types
            WHERE is_active = true
            ORDER BY security_level DESC, code ASC
        ";

        let rows = self.backend.query_rows(query, &[]).await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(SshKeyType {
                id: row.get(0),
                code: row.get(1),
                name: row.get(2),
                algorithm: row.get(3),
                key_size: row.get(4),
                is_system: row.get(5),
                is_deprecated: row.get(6),
                security_level: row.get(7),
                metadata: row
                    .get::<_, Option<serde_json::Value>>(8)
                    .unwrap_or_default(),
                created_at: row.get(9),
                updated_at: row.get(10),
            });
        }

        // Update cache
        {
            let mut cached = self.cache.ssh_key_types.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_ssh_key_type_by_code(&self, code: &str) -> Result<Option<SshKeyType>, String> {
        let types = self.get_ssh_key_types().await?;
        Ok(types.into_iter().find(|t| t.code == code))
    }

    async fn get_engine_role_types(
        &self,
        engine_type: &str,
    ) -> Result<Vec<EngineRoleType>, String> {
        let query = "
            SELECT id, engine_type, code, name, description, default_config,
                   allowed_operations, is_system, created_at, updated_at
            FROM engine_role_types
            WHERE engine_type = $1 AND is_active = true
            ORDER BY code ASC
        ";

        let rows = self.backend.query_rows(query, &[&engine_type]).await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(EngineRoleType {
                id: row.get(0),
                engine_type: row.get(1),
                code: row.get(2),
                name: row.get(3),
                description: row.get(4),
                default_config: row
                    .get::<_, Option<serde_json::Value>>(5)
                    .unwrap_or_default(),
                allowed_operations: row.get::<_, Option<Vec<String>>>(6).unwrap_or_default(),
                is_system: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            });
        }

        Ok(result)
    }

    async fn get_user_effective_capabilities(
        &self,
        user_id: Uuid,
        namespace: &str,
    ) -> Result<EffectiveCapabilities, String> {
        // Use the view we created in migration
        let query = "
            SELECT capability_code, role_type_code, is_root
            FROM v_user_effective_capabilities
            WHERE user_id = $1 AND namespace_id = (
                SELECT id FROM namespaces WHERE path = $2 LIMIT 1
            )
        ";

        let rows = self
            .backend
            .query_rows(query, &[&user_id, &namespace])
            .await?;

        let mut capabilities = HashSet::new();
        let mut role_types = Vec::new();
        let mut is_root = false;

        for row in &rows {
            let cap_code: String = row.get(0);
            let role_code: String = row.get(1);
            let root: bool = row.get(2);

            capabilities.insert(cap_code);
            if !role_types
                .iter()
                .any(|r: &UserRoleType| r.code == role_code)
            {
                // Fetch full role type if needed
                if let Ok(Some(role_type)) = self.get_user_role_type_by_code(&role_code).await {
                    role_types.push(role_type);
                }
            }
            if root {
                is_root = true;
            }
        }

        Ok(EffectiveCapabilities {
            user_id,
            capabilities,
            role_types,
            policies: vec![], // Loaded separately
            namespace: namespace.to_string(),
            is_root,
        })
    }

    async fn user_has_capability(
        &self,
        user_id: Uuid,
        capability: &str,
        namespace: &str,
    ) -> Result<bool, String> {
        // Use database helper function
        let query = "SELECT user_has_capability($1, $2, $3)";

        let rows = self
            .backend
            .query_rows(query, &[&user_id, &capability, &namespace])
            .await?;

        if rows.is_empty() {
            return Ok(false);
        }

        Ok(rows[0].get::<_, Option<bool>>(0).unwrap_or(false))
    }

    async fn check_path_access(
        &self,
        policies: &[String],
        path: &str,
        action: &str,
        namespace: &str,
    ) -> Result<PathAccessResult, String> {
        if policies.is_empty() {
            return Ok(PathAccessResult::deny("No policies assigned"));
        }

        // Check all policies for this path
        let query = "
            SELECT pr.policy_id, p.name, pr.effect, pr.path_pattern, pr.capabilities
            FROM policy_rules pr
            JOIN policies p ON pr.policy_id = p.id
            WHERE p.name = ANY($1)
              AND p.namespace_id = (SELECT id FROM namespaces WHERE path = $2 LIMIT 1)
              AND (
                  pr.path_pattern = '*'
                  OR $3 LIKE REPLACE(REPLACE(pr.path_pattern, '**', '%'), '*', '%')
              )
            ORDER BY
                CASE WHEN pr.effect = 'deny' THEN 0 ELSE 1 END,  -- Deny takes precedence
                LENGTH(pr.path_pattern) DESC  -- More specific patterns first
        ";

        let rows = self
            .backend
            .query_rows(query, &[&policies, &namespace, &path])
            .await?;

        for row in rows {
            let policy_id: i64 = row.get(0);
            let policy_name: String = row.get(1);
            let effect: String = row.get(2);
            let capabilities: Vec<String> =
                row.get::<_, Option<Vec<String>>>(4).unwrap_or_default();

            // Check if this rule grants the requested action
            let has_capability = capabilities.iter().any(|c| c == "*" || c == action);

            if effect == "deny" && has_capability {
                return Ok(PathAccessResult::deny_by_policy(&policy_name));
            }

            if effect == "allow" && has_capability {
                return Ok(PathAccessResult::allow(&policy_name));
            }
        }

        Ok(PathAccessResult::deny("No matching policy rule found"))
    }
}

// ============================================================================
// CRUD OPERATIONS
// ============================================================================

/// Create a new user role type
pub async fn create_user_role_type<B: DynamicRoleBackend>(
    backend: &B,
    role: &UserRoleType,
) -> Result<Uuid, String> {
    let query = "
        INSERT INTO user_role_types (
            id, code, name, description, priority, is_system,
            default_capabilities, metadata, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        RETURNING id
    ";

    let caps: Vec<String> = role.default_capabilities.clone();

    backend
        .execute(
            query,
            &[
                &role.id,
                &role.code,
                &role.name,
                &role.description,
                &role.priority,
                &role.is_system,
                &caps,
                &role.metadata,
            ],
        )
        .await?;

    Ok(role.id)
}

/// Create a new capability
pub async fn create_capability<B: DynamicRoleBackend>(
    backend: &B,
    cap: &Capability,
) -> Result<Uuid, String> {
    let query = "
        INSERT INTO capabilities (
            id, code, name, description, category, is_system,
            is_dangerous, requires_sudo, metadata, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        RETURNING id
    ";

    backend
        .execute(
            query,
            &[
                &cap.id,
                &cap.code,
                &cap.name,
                &cap.description,
                &cap.category,
                &cap.is_system,
                &cap.is_dangerous,
                &cap.requires_sudo,
                &cap.metadata,
            ],
        )
        .await?;

    Ok(cap.id)
}

/// Assign role to user
pub async fn assign_role_to_user<B: DynamicRoleBackend>(
    backend: &B,
    user_id: Uuid,
    role_type_id: Uuid,
    namespace: &str,
) -> Result<Uuid, String> {
    let id = Uuid::new_v4();

    let query = "
        INSERT INTO user_roles (id, user_id, role_type_id, namespace_id, created_at)
        VALUES ($1, $2, $3, (SELECT id FROM namespaces WHERE path = $4 LIMIT 1), NOW())
        RETURNING id
    ";

    backend
        .execute(query, &[&id, &user_id, &role_type_id, &namespace])
        .await?;

    Ok(id)
}

/// Remove role from user
pub async fn remove_role_from_user<B: DynamicRoleBackend>(
    backend: &B,
    user_id: Uuid,
    role_type_id: Uuid,
    namespace: &str,
) -> Result<bool, String> {
    let query = "
        DELETE FROM user_roles
        WHERE user_id = $1
          AND role_type_id = $2
          AND namespace_id = (SELECT id FROM namespaces WHERE path = $3 LIMIT 1)
    ";

    let affected = backend
        .execute(query, &[&user_id, &role_type_id, &namespace])
        .await?;

    Ok(affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_expiry() {
        let cache = CachedData::new(vec!["test".to_string()]);
        assert!(!cache.is_expired(300));
        assert!(!cache.is_expired(1));
    }
}
