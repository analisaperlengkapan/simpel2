//! Dynamic Role Database Operations
//!
//! This module provides database operations for the dynamic role system,
//! implementing the `DynamicRoleStore` trait.
//!
//! # Design
//!
//! - All role types, access levels, capabilities are stored in PostgreSQL
//! - Caching layer reduces database round-trips
//! - Bootstrap data loaded on first access
//! - Follows Vault/Keycloak best practices

use crate::Database;
use authenc_types::Result;
use authenc_core::models::dynamic_role::{
    AccessLevel, ActorType, AdminLevelType, AuthorizationPolicy, AuthorizationRequest,
    AuthorizationResult, Capability, CredentialTypeConfig, DynamicRoleStore, EffectiveCapabilities,
    PolicyEffect, RoleType, SatkerType, ScopeType,
};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// CACHE STRUCTURES
// ============================================================================

/// Cached role types with TTL
#[derive(Clone)]
struct CachedData<T: Clone> {
    data: T,
    cached_at: chrono::DateTime<Utc>,
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
    role_types: RwLock<Option<CachedData<Vec<RoleType>>>>,
    access_levels: RwLock<Option<CachedData<Vec<AccessLevel>>>>,
    admin_level_types: RwLock<Option<CachedData<Vec<AdminLevelType>>>>,
    scope_types: RwLock<Option<CachedData<Vec<ScopeType>>>>,
    capabilities: RwLock<Option<CachedData<Vec<Capability>>>>,
    actor_types: RwLock<Option<CachedData<Vec<ActorType>>>>,
    credential_types: RwLock<Option<CachedData<Vec<CredentialTypeConfig>>>>,
    satker_types: RwLock<Option<CachedData<Vec<SatkerType>>>>,
    /// Cache TTL in seconds (default: 5 minutes)
    ttl_seconds: i64,
}

impl Default for DynamicRoleCache {
    fn default() -> Self {
        Self {
            role_types: RwLock::new(None),
            access_levels: RwLock::new(None),
            admin_level_types: RwLock::new(None),
            scope_types: RwLock::new(None),
            capabilities: RwLock::new(None),
            actor_types: RwLock::new(None),
            credential_types: RwLock::new(None),
            satker_types: RwLock::new(None),
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
        *self.role_types.write().await = None;
        *self.access_levels.write().await = None;
        *self.admin_level_types.write().await = None;
        *self.scope_types.write().await = None;
        *self.capabilities.write().await = None;
        *self.actor_types.write().await = None;
        *self.credential_types.write().await = None;
        *self.satker_types.write().await = None;
    }
}

// ============================================================================
// DATABASE OPERATIONS IMPLEMENTATION
// ============================================================================

/// PostgreSQL-backed implementation of DynamicRoleStore
pub struct PostgresDynamicRoleStore {
    db: Database,
    cache: Arc<DynamicRoleCache>,
}

impl PostgresDynamicRoleStore {
    /// Create a new store with the given database connection
    pub fn new(db: Database) -> Self {
        Self {
            db,
            cache: Arc::new(DynamicRoleCache::default()),
        }
    }

    /// Create a new store with custom cache
    pub fn with_cache(db: Database, cache: Arc<DynamicRoleCache>) -> Self {
        Self { db, cache }
    }

    /// Get the database connection
    pub fn database(&self) -> &Database {
        &self.db
    }

    /// Get the cache
    pub fn cache(&self) -> Arc<DynamicRoleCache> {
        Arc::clone(&self.cache)
    }
}

#[async_trait::async_trait]
impl DynamicRoleStore for PostgresDynamicRoleStore {
    async fn get_role_types(&self) -> std::result::Result<Vec<RoleType>, String> {
        // Check cache first
        {
            let cached = self.cache.role_types.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let result = fetch_role_types(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        // Update cache
        {
            let mut cached = self.cache.role_types.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_role_type_by_code(
        &self,
        code: &str,
    ) -> std::result::Result<Option<RoleType>, String> {
        let types = self.get_role_types().await?;
        Ok(types.into_iter().find(|t| t.code == code))
    }

    async fn get_access_levels(&self) -> std::result::Result<Vec<AccessLevel>, String> {
        // Check cache first
        {
            let cached = self.cache.access_levels.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let result = fetch_access_levels(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        // Update cache
        {
            let mut cached = self.cache.access_levels.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_access_level_by_code(
        &self,
        code: &str,
    ) -> std::result::Result<Option<AccessLevel>, String> {
        let levels = self.get_access_levels().await?;
        Ok(levels.into_iter().find(|l| l.code == code))
    }

    async fn get_admin_level_types(&self) -> std::result::Result<Vec<AdminLevelType>, String> {
        // Check cache first
        {
            let cached = self.cache.admin_level_types.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let result = fetch_admin_level_types(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        // Update cache
        {
            let mut cached = self.cache.admin_level_types.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_admin_level_by_code(
        &self,
        code: &str,
    ) -> std::result::Result<Option<AdminLevelType>, String> {
        let types = self.get_admin_level_types().await?;
        Ok(types.into_iter().find(|t| t.code == code))
    }

    async fn get_scope_types(&self) -> std::result::Result<Vec<ScopeType>, String> {
        // Check cache first
        {
            let cached = self.cache.scope_types.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let result = fetch_scope_types(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        // Update cache
        {
            let mut cached = self.cache.scope_types.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_scope_type_by_code(
        &self,
        code: &str,
    ) -> std::result::Result<Option<ScopeType>, String> {
        let types = self.get_scope_types().await?;
        Ok(types.into_iter().find(|t| t.code == code))
    }

    async fn get_capabilities(&self) -> std::result::Result<Vec<Capability>, String> {
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
        let result = fetch_capabilities(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        // Update cache
        {
            let mut cached = self.cache.capabilities.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_capability_by_code(
        &self,
        code: &str,
    ) -> std::result::Result<Option<Capability>, String> {
        let caps = self.get_capabilities().await?;
        Ok(caps.into_iter().find(|c| c.code == code))
    }

    async fn get_actor_types(&self) -> std::result::Result<Vec<ActorType>, String> {
        // Check cache first
        {
            let cached = self.cache.actor_types.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let result = fetch_actor_types(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        // Update cache
        {
            let mut cached = self.cache.actor_types.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_actor_type_by_code(
        &self,
        code: &str,
    ) -> std::result::Result<Option<ActorType>, String> {
        let types = self.get_actor_types().await?;
        Ok(types.into_iter().find(|t| t.code == code))
    }

    async fn get_credential_types(&self) -> std::result::Result<Vec<CredentialTypeConfig>, String> {
        // Check cache first
        {
            let cached = self.cache.credential_types.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let result = fetch_credential_types(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        // Update cache
        {
            let mut cached = self.cache.credential_types.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_credential_type_by_code(
        &self,
        code: &str,
    ) -> std::result::Result<Option<CredentialTypeConfig>, String> {
        let types = self.get_credential_types().await?;
        Ok(types.into_iter().find(|t| t.code == code))
    }

    async fn get_satker_types(&self) -> std::result::Result<Vec<SatkerType>, String> {
        // Check cache first
        {
            let cached = self.cache.satker_types.read().await;
            if let Some(ref data) = *cached {
                if !data.is_expired(self.cache.ttl_seconds) {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch from database
        let result = fetch_satker_types(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        // Update cache
        {
            let mut cached = self.cache.satker_types.write().await;
            *cached = Some(CachedData::new(result.clone()));
        }

        Ok(result)
    }

    async fn get_satker_type_by_code(
        &self,
        code: &str,
    ) -> std::result::Result<Option<SatkerType>, String> {
        let types = self.get_satker_types().await?;
        Ok(types.into_iter().find(|t| t.code == code))
    }

    async fn get_user_effective_capabilities(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
    ) -> std::result::Result<EffectiveCapabilities, String> {
        fetch_user_effective_capabilities(&self.db, user_id, realm_id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn user_has_capability(
        &self,
        user_id: Uuid,
        capability: &str,
        realm_id: Uuid,
    ) -> std::result::Result<bool, String> {
        check_user_capability(&self.db, user_id, capability, realm_id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn user_can_access_path(
        &self,
        user_id: Uuid,
        path: &str,
        action: &str,
        realm_id: Uuid,
    ) -> std::result::Result<bool, String> {
        // Get user's effective capabilities
        let caps = self
            .get_user_effective_capabilities(user_id, realm_id)
            .await?;
        Ok(caps.can_access_path(path, action))
    }

    async fn authorize(
        &self,
        request: AuthorizationRequest,
    ) -> std::result::Result<AuthorizationResult, String> {
        authorize_request(&self.db, &request)
            .await
            .map_err(|e| e.to_string())
    }
}

// ============================================================================
// DATABASE FETCH FUNCTIONS
// ============================================================================

async fn fetch_role_types(db: &Database) -> Result<Vec<RoleType>> {
    let query = "
        SELECT id, code, name, description, category, is_system, is_assignable,
               priority, metadata, realm_id, created_at, updated_at
        FROM role_types
        WHERE deleted_at IS NULL
        ORDER BY priority DESC, name ASC
    ";

    let rows = db.query(query, &[]).await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(RoleType {
            id: row.get(0),
            code: row.get(1),
            name: row.get(2),
            description: row.get(3),
            category: row
                .get::<_, Option<String>>(4)
                .unwrap_or_else(|| "custom".to_string()),
            is_system: row.get(5),
            is_assignable: row.get::<_, Option<bool>>(6).unwrap_or(true),
            priority: row.get(7),
            metadata: row
                .get::<_, Option<serde_json::Value>>(8)
                .unwrap_or_default(),
            realm_id: row.get(9),
            created_at: row.get(10),
            updated_at: row.get(11),
        });
    }

    Ok(result)
}

async fn fetch_access_levels(db: &Database) -> Result<Vec<AccessLevel>> {
    let query = "
        SELECT id, code, name, description, numeric_level, is_system,
               capabilities, metadata, realm_id, created_at, updated_at
        FROM access_levels
        WHERE deleted_at IS NULL
        ORDER BY numeric_level ASC
    ";

    let rows = db.query(query, &[]).await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        // Parse capabilities from JSONB array
        let capabilities_json: Option<serde_json::Value> = row.get(6);
        let capabilities: Vec<String> = capabilities_json
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        result.push(AccessLevel {
            id: row.get(0),
            code: row.get(1),
            name: row.get(2),
            description: row.get(3),
            numeric_level: row.get(4),
            is_system: row.get(5),
            capabilities,
            metadata: row
                .get::<_, Option<serde_json::Value>>(7)
                .unwrap_or_default(),
            realm_id: row.get(8),
            created_at: row.get(9),
            updated_at: row.get(10),
        });
    }

    Ok(result)
}

async fn fetch_admin_level_types(db: &Database) -> Result<Vec<AdminLevelType>> {
    let query = "
        SELECT id, code, name, description, hierarchy_level, scope_type,
               parent_level_id, can_manage_levels, is_system, metadata,
               realm_id, created_at, updated_at
        FROM admin_level_types
        WHERE deleted_at IS NULL
        ORDER BY hierarchy_level ASC
    ";

    let rows = db.query(query, &[]).await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        // Parse can_manage_levels from JSONB array
        let can_manage_json: Option<serde_json::Value> = row.get(7);
        let can_manage_levels: Vec<String> = can_manage_json
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        result.push(AdminLevelType {
            id: row.get(0),
            code: row.get(1),
            name: row.get(2),
            description: row.get(3),
            hierarchy_level: row.get(4),
            scope_type: row
                .get::<_, Option<String>>(5)
                .unwrap_or_else(|| "global".to_string()),
            parent_level_id: row.get(6),
            can_manage_levels,
            is_system: row.get(8),
            metadata: row
                .get::<_, Option<serde_json::Value>>(9)
                .unwrap_or_default(),
            realm_id: row.get(10),
            created_at: row.get(11),
            updated_at: row.get(12),
        });
    }

    Ok(result)
}

async fn fetch_scope_types(db: &Database) -> Result<Vec<ScopeType>> {
    let query = "
        SELECT id, code, name, description, hierarchy_level, parent_scope_type_id,
               scope_pattern, is_system, metadata, realm_id, created_at, updated_at
        FROM scope_types
        WHERE deleted_at IS NULL
        ORDER BY hierarchy_level DESC, code ASC
    ";

    let rows = db.query(query, &[]).await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(ScopeType {
            id: row.get(0),
            code: row.get(1),
            name: row.get(2),
            description: row.get(3),
            hierarchy_level: row.get::<_, Option<i32>>(4).unwrap_or(0),
            parent_scope_type_id: row.get(5),
            scope_pattern: row.get(6),
            is_system: row.get(7),
            metadata: row
                .get::<_, Option<serde_json::Value>>(8)
                .unwrap_or_default(),
            realm_id: row.get(9),
            created_at: row.get(10),
            updated_at: row.get(11),
        });
    }

    Ok(result)
}

async fn fetch_capabilities(db: &Database) -> Result<Vec<Capability>> {
    let query = "
        SELECT id, code, name, description, resource_type, action,
               is_system, is_dangerous, requires_mfa, requires_approval,
               metadata, realm_id, created_at, updated_at
        FROM capabilities
        WHERE deleted_at IS NULL
        ORDER BY resource_type ASC, code ASC
    ";

    let rows = db.query(query, &[]).await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(Capability {
            id: row.get(0),
            code: row.get(1),
            name: row.get(2),
            description: row.get(3),
            resource_type: row.get(4),
            action: row
                .get::<_, Option<String>>(5)
                .unwrap_or_else(|| "read".to_string()),
            is_system: row.get(6),
            is_dangerous: row.get::<_, Option<bool>>(7).unwrap_or(false),
            requires_mfa: row.get::<_, Option<bool>>(8).unwrap_or(false),
            requires_approval: row.get::<_, Option<bool>>(9).unwrap_or(false),
            metadata: row
                .get::<_, Option<serde_json::Value>>(10)
                .unwrap_or_default(),
            realm_id: row.get(11),
            created_at: row.get(12),
            updated_at: row.get(13),
        });
    }

    Ok(result)
}

async fn fetch_actor_types(db: &Database) -> Result<Vec<ActorType>> {
    let query = "
        SELECT id, code, name, description, is_human, is_system,
               metadata, created_at, updated_at
        FROM actor_types
        WHERE is_active = true
        ORDER BY code ASC
    ";

    let rows = db.query(query, &[]).await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(ActorType {
            id: row.get(0),
            code: row.get(1),
            name: row.get(2),
            description: row.get(3),
            is_human: row.get(4),
            is_system: row.get(5),
            metadata: row
                .get::<_, Option<serde_json::Value>>(6)
                .unwrap_or_default(),
            created_at: row.get(7),
            updated_at: row.get(8),
        });
    }

    Ok(result)
}

async fn fetch_credential_types(db: &Database) -> Result<Vec<CredentialTypeConfig>> {
    let query = "
        SELECT id, code, name, description, is_system, is_primary,
               requires_verification, config_schema, metadata, realm_id,
               created_at, updated_at
        FROM credential_types
        WHERE deleted_at IS NULL
        ORDER BY name ASC
    ";

    let rows = db.query(query, &[]).await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(CredentialTypeConfig {
            id: row.get(0),
            code: row.get(1),
            name: row.get(2),
            description: row.get(3),
            is_system: row.get(4),
            is_primary: row.get::<_, Option<bool>>(5).unwrap_or(false),
            requires_verification: row.get::<_, Option<bool>>(6).unwrap_or(true),
            config_schema: row
                .get::<_, Option<serde_json::Value>>(7)
                .unwrap_or_default(),
            metadata: row
                .get::<_, Option<serde_json::Value>>(8)
                .unwrap_or_default(),
            realm_id: row.get(9),
            created_at: row.get(10),
            updated_at: row.get(11),
        });
    }

    Ok(result)
}

async fn fetch_satker_types(db: &Database) -> Result<Vec<SatkerType>> {
    let query = "
        SELECT id, code, name, description, hierarchy_level, parent_type_id,
               code_pattern, is_system, metadata, created_at, updated_at
        FROM satker_types
        ORDER BY hierarchy_level DESC
    ";

    let rows = db.query(query, &[]).await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(SatkerType {
            id: row.get(0),
            code: row.get(1),
            name: row.get(2),
            description: row.get(3),
            hierarchy_level: row.get(4),
            parent_type_id: row.get(5),
            code_pattern: row.get(6),
            is_system: row.get::<_, Option<bool>>(7).unwrap_or(false),
            metadata: row
                .get::<_, Option<serde_json::Value>>(8)
                .unwrap_or_default(),
            created_at: row.get(9),
            updated_at: row.get(10),
        });
    }

    Ok(result)
}

async fn fetch_user_effective_capabilities(
    db: &Database,
    user_id: Uuid,
    realm_id: Uuid,
) -> Result<EffectiveCapabilities> {
    // Use the view we created in migration
    let query = "
        SELECT capability_code, role_type_code, access_level, via_group
        FROM v_user_effective_capabilities
        WHERE user_id = $1 AND realm_id = $2
    ";

    let rows = db.query(query, &[&user_id, &realm_id]).await?;

    let mut capabilities = HashSet::new();
    let mut is_superuser = false;

    for row in rows {
        let cap_code: String = row.get(0);
        let level: String = row.get(2);

        capabilities.insert(cap_code.clone());

        // Check if user has superuser access
        if cap_code == "*" || level == "super-admin" || level == "root" {
            is_superuser = true;
        }
    }

    Ok(EffectiveCapabilities {
        capabilities,
        policies: vec![], // Policies loaded separately if needed
        access_levels: HashMap::new(),
        is_superuser,
    })
}

async fn check_user_capability(
    db: &Database,
    user_id: Uuid,
    capability: &str,
    realm_id: Uuid,
) -> Result<bool> {
    // Use database helper function for efficiency
    let query = "SELECT user_has_capability($1, $2, $3)";

    let rows = db
        .query(query, &[&user_id, &capability, &realm_id])
        .await?;

    if rows.is_empty() {
        return Ok(false);
    }

    Ok(rows[0].get::<_, Option<bool>>(0).unwrap_or(false))
}

async fn authorize_request(
    db: &Database,
    request: &AuthorizationRequest,
) -> Result<AuthorizationResult> {
    // Check if user has the required capability
    let has_cap = check_user_capability(
        db,
        request.user_id,
        &request.action,
        Uuid::nil(), // Use default realm
    )
    .await?;

    if !has_cap {
        return Ok(AuthorizationResult::deny(
            "User does not have required capability",
        ));
    }

    // Check path-based policies if resource_path is specified
    if !request.resource_path.is_empty() {
        let query = "
            SELECT ap.id, ap.name, ap.effect
            FROM authorization_policies ap
            JOIN user_policies up ON ap.id = up.policy_id
            WHERE up.user_id = $1
              AND ap.enabled = true
              AND ap.deleted_at IS NULL
              AND (
                  ap.path_pattern = '*'
                  OR $2 LIKE REPLACE(REPLACE(ap.path_pattern, '*', '%'), '?', '_')
              )
            ORDER BY ap.priority DESC
            LIMIT 1
        ";

        let rows = db
            .query(query, &[&request.user_id, &request.resource_path])
            .await?;

        if !rows.is_empty() {
            let policy_name: String = rows[0].get(1);
            let effect: String = rows[0].get(2);

            let allowed = effect == "allow";

            return Ok(AuthorizationResult {
                allowed,
                reason: format!("Policy '{}' matched: {}", policy_name, effect),
                matching_policy: Some(policy_name),
                required_capabilities: if allowed {
                    vec![request.action.clone()]
                } else {
                    vec![]
                },
            });
        }
    }

    // Default: allow if capability is present
    Ok(AuthorizationResult::allow("Capability granted"))
}

// ============================================================================
// CRUD OPERATIONS FOR MANAGEMENT
// ============================================================================

/// Create a new role type
pub async fn create_role_type(db: &Database, role: &RoleType) -> Result<Uuid> {
    let query = "
        INSERT INTO role_types (
            id, code, name, description, category, priority, is_system,
            is_assignable, metadata, realm_id, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
        RETURNING id
    ";

    let id = role.id;

    db.execute(
        query,
        &[
            &id,
            &role.code,
            &role.name,
            &role.description,
            &role.category,
            &role.priority,
            &role.is_system,
            &role.is_assignable,
            &role.metadata,
            &role.realm_id,
        ],
    )
    .await?;

    Ok(id)
}

/// Create a new capability
pub async fn create_capability(db: &Database, cap: &Capability) -> Result<Uuid> {
    let query = "
        INSERT INTO capabilities (
            id, code, name, description, resource_type, action,
            is_system, is_dangerous, requires_mfa, requires_approval,
            metadata, realm_id, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
        RETURNING id
    ";

    let id = cap.id;

    db.execute(
        query,
        &[
            &id,
            &cap.code,
            &cap.name,
            &cap.description,
            &cap.resource_type,
            &cap.action,
            &cap.is_system,
            &cap.is_dangerous,
            &cap.requires_mfa,
            &cap.requires_approval,
            &cap.metadata,
            &cap.realm_id,
        ],
    )
    .await?;

    Ok(id)
}

/// Assign capability to role
pub async fn assign_capability_to_role(
    db: &Database,
    role_id: Uuid,
    capability_id: Uuid,
    granted_by: Uuid,
) -> Result<()> {
    let query = "
        INSERT INTO role_capabilities (role_type_id, capability_id, granted_by, created_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (role_type_id, capability_id) DO NOTHING
    ";

    db.execute(query, &[&role_id, &capability_id, &granted_by])
        .await?;

    Ok(())
}

/// Assign role to user
pub async fn assign_role_to_user(
    db: &Database,
    user_id: Uuid,
    role_type_id: Uuid,
    realm_id: Uuid,
    scope_value: Option<&str>,
    granted_by: Uuid,
) -> Result<Uuid> {
    let id = Uuid::new_v4();

    let query = "
        INSERT INTO user_roles (id, user_id, role_type_id, realm_id, scope_value, granted_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        RETURNING id
    ";

    db.execute(
        query,
        &[
            &id,
            &user_id,
            &role_type_id,
            &realm_id,
            &scope_value,
            &granted_by,
        ],
    )
    .await?;

    Ok(id)
}

/// Remove role from user
pub async fn remove_role_from_user(
    db: &Database,
    user_id: Uuid,
    role_type_id: Uuid,
    realm_id: Uuid,
) -> Result<bool> {
    let query = "
        DELETE FROM user_roles
        WHERE user_id = $1 AND role_type_id = $2 AND realm_id = $3
    ";

    let affected = db
        .execute(query, &[&user_id, &role_type_id, &realm_id])
        .await?;

    Ok(affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_expiry() {
        let cache = CachedData::new(vec!["test".to_string()]);
        assert!(!cache.is_expired(300)); // 5 minutes
        assert!(!cache.is_expired(1)); // Even 1 second should not be expired immediately
    }
}
