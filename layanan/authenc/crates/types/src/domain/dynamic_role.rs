//! Dynamic Role System
//!
//! This module provides a fully dynamic, database-driven role and permission system
//! following HashiCorp Vault and Keycloak best practices.
//!
//! # Design Principles
//!
//! 1. **No Hardcoded Role Names**: Roles are defined in the database, not code
//! 2. **Policy-Based Authorization**: Access is determined by policies, not role names
//! 3. **Capability-Based Permissions**: Fine-grained, composable permissions
//! 4. **Hierarchical Scopes**: Flexible organizational hierarchies
//! 5. **Bootstrap-Only Defaults**: Only essential system roles are pre-defined
//!
//! # Usage
//!
//! ```rust,ignore
//! use authenc_types::domain::dynamic_role::{DynamicRoleStore, EffectiveCapabilities};
//!
//! // Check if user has a capability
//! let has_access = role_store.user_has_capability(user_id, "users:read", realm_id).await?;
//!
//! // Check path-based access (Vault-style)
//! let can_access = role_store.user_can_access_path(user_id, "/api/users/*", "read", realm_id).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Bootstrap role codes - ONLY these are referenced in code
/// These correspond to system roles that must exist for the system to function
pub mod bootstrap {
    /// Root role with all capabilities (for initial admin setup only)
    pub const ROOT: &str = "root";
    /// Default access level code
    pub const DEFAULT_ACCESS: &str = "read";

    /// Check if a role code is a bootstrap role
    pub fn is_bootstrap_role(code: &str) -> bool {
        matches!(code, ROOT)
    }
}

// ============================================================================
// DYNAMIC ROLE TYPE (Replaces hardcoded OrganizationRole enum)
// ============================================================================

/// Dynamic role type loaded from database
///
/// This replaces the hardcoded `OrganizationRole` enum. Role types are now
/// fully configurable via the database without code changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleType {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub is_system: bool,
    pub is_assignable: bool,
    pub priority: i32,
    pub metadata: serde_json::Value,
    pub realm_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RoleType {
    /// Check if this role type can be modified
    pub fn is_modifiable(&self) -> bool {
        // System roles can be modified but not deleted
        true
    }

    /// Check if this role type can be deleted
    pub fn is_deletable(&self) -> bool {
        !self.is_system
    }

    /// Compare priority with another role type
    pub fn has_higher_priority(&self, other: &RoleType) -> bool {
        self.priority > other.priority
    }
}

// ============================================================================
// DYNAMIC ACCESS LEVEL (Replaces hardcoded AccessLevel enum)
// ============================================================================

/// Dynamic access level loaded from database
///
/// This replaces the hardcoded `AccessLevel` enum. Access levels are now
/// configurable with numeric hierarchy and capability sets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLevel {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub numeric_level: i32,
    pub is_system: bool,
    pub capabilities: Vec<String>,
    pub metadata: serde_json::Value,
    pub realm_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AccessLevel {
    /// Check if this access level is higher than another
    pub fn is_higher_than(&self, other: &AccessLevel) -> bool {
        self.numeric_level > other.numeric_level
    }

    /// Check if this access level includes a capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&"*".to_string())
            || self.capabilities.iter().any(|c| c == capability)
    }
}

impl PartialOrd for AccessLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.numeric_level.cmp(&other.numeric_level))
    }
}

impl PartialEq for AccessLevel {
    fn eq(&self, other: &Self) -> bool {
        self.numeric_level == other.numeric_level
    }
}

// ============================================================================
// DYNAMIC ADMIN LEVEL (Replaces hardcoded AdminLevel enum)
// ============================================================================

/// Dynamic admin level type loaded from database
///
/// This replaces the hardcoded `AdminLevel` enum. Admin levels now support
/// flexible hierarchies defined in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLevelType {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub hierarchy_level: i32,
    pub scope_type: String,
    pub parent_level_id: Option<Uuid>,
    pub can_manage_levels: Vec<String>,
    pub is_system: bool,
    pub metadata: serde_json::Value,
    pub realm_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AdminLevelType {
    /// Check if this admin level can manage another admin level
    pub fn can_manage(&self, other_code: &str) -> bool {
        self.can_manage_levels.contains(&"*".to_string())
            || self.can_manage_levels.iter().any(|l| l == other_code)
    }

    /// Check if this admin level is higher in hierarchy
    pub fn is_above(&self, other: &AdminLevelType) -> bool {
        self.hierarchy_level > other.hierarchy_level
    }
}

// ============================================================================
// DYNAMIC SCOPE TYPE (Replaces hardcoded RoleScope enum)
// ============================================================================

/// Dynamic scope type loaded from database
///
/// This replaces the hardcoded `RoleScope` enum. Scope types now support
/// flexible hierarchies and pattern matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeType {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub hierarchy_level: i32,
    pub parent_scope_type_id: Option<Uuid>,
    pub scope_pattern: Option<String>,
    pub is_system: bool,
    pub metadata: serde_json::Value,
    pub realm_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ScopeType {
    /// Check if this scope type is broader than another
    pub fn is_broader_than(&self, other: &ScopeType) -> bool {
        self.hierarchy_level > other.hierarchy_level
    }

    /// Check if a scope identifier matches this type's pattern
    pub fn matches_identifier(&self, identifier: &str) -> bool {
        match &self.scope_pattern {
            Some(pattern) => {
                // Simple glob matching
                if pattern == "*" {
                    true
                } else if pattern.ends_with('*') {
                    identifier.starts_with(&pattern[..pattern.len() - 1])
                } else {
                    identifier == pattern
                }
            }
            None => true,
        }
    }
}

// ============================================================================
// CAPABILITY (Fine-grained permission)
// ============================================================================

/// Capability representing a fine-grained permission
///
/// Capabilities follow the pattern: `resource:action` (e.g., `users:read`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub resource_type: Option<String>,
    pub action: String,
    pub is_system: bool,
    pub is_dangerous: bool,
    pub requires_mfa: bool,
    pub requires_approval: bool,
    pub metadata: serde_json::Value,
    pub realm_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Capability {
    /// Parse capability code into resource and action
    pub fn parse_code(code: &str) -> (Option<String>, String) {
        if let Some(idx) = code.find(':') {
            let (resource, action) = code.split_at(idx);
            (Some(resource.to_string()), action[1..].to_string())
        } else {
            (None, code.to_string())
        }
    }

    /// Check if this is a wildcard capability
    pub fn is_wildcard(&self) -> bool {
        self.code == "*"
    }

    /// Check if this capability matches a specific code
    pub fn matches(&self, code: &str) -> bool {
        if self.is_wildcard() {
            return true;
        }

        if self.code == code {
            return true;
        }

        // Check pattern matching for resource:* patterns
        if self.code.ends_with(":*") {
            let prefix = &self.code[..self.code.len() - 1];
            return code.starts_with(prefix);
        }

        false
    }
}

// ============================================================================
// AUTHORIZATION POLICY (Vault-style policy)
// ============================================================================

/// Authorization policy for path-based access control
///
/// Follows HashiCorp Vault's policy model with path patterns and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub policy_type: String,
    pub effect: PolicyEffect,
    pub path_pattern: String,
    pub capabilities: Vec<String>,
    pub conditions: serde_json::Value,
    pub priority: i32,
    pub is_system: bool,
    pub enabled: bool,
    pub realm_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Policy effect (allow or deny)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

impl AuthorizationPolicy {
    /// Check if a path matches this policy
    pub fn matches_path(&self, path: &str) -> bool {
        glob_match(&self.path_pattern, path)
    }

    /// Check if this policy grants a capability
    pub fn grants_capability(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| c == "*" || c == capability)
    }
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while let Some(p) = pattern_chars.next() {
        match p {
            '*' => {
                // Consume all stars
                while pattern_chars.peek() == Some(&'*') {
                    pattern_chars.next();
                }

                // If no more pattern, match rest
                if pattern_chars.peek().is_none() {
                    return true;
                }

                // Try matching rest at each position
                let rest_pattern: String = pattern_chars.collect();
                loop {
                    let rest_text: String = text_chars.clone().collect();
                    if glob_match(&rest_pattern, &rest_text) {
                        return true;
                    }
                    if text_chars.next().is_none() {
                        return false;
                    }
                }
            }
            '?' => {
                if text_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                if text_chars.next() != Some(c) {
                    return false;
                }
            }
        }
    }

    text_chars.next().is_none()
}

// ============================================================================
// ACTOR TYPE (Replaces hardcoded actor_type checks)
// ============================================================================

/// Actor type for audit and policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorType {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_human: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// CREDENTIAL TYPE (Replaces CredentialType enum)
// ============================================================================

/// Dynamic credential type loaded from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialTypeConfig {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_primary: bool,
    pub requires_verification: bool,
    pub config_schema: serde_json::Value,
    pub metadata: serde_json::Value,
    pub realm_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// SATKER TYPE (Replaces SatkerType enum)
// ============================================================================

/// Dynamic Satker type for Kejaksaan hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerType {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub hierarchy_level: i32,
    pub parent_type_id: Option<Uuid>,
    pub code_pattern: Option<String>,
    pub is_system: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SatkerType {
    /// Check if a satker code belongs to this type
    pub fn matches_code(&self, satker_code: &str) -> bool {
        match &self.code_pattern {
            Some(pattern) => {
                // Simple prefix matching for now
                if pattern.starts_with('^') && pattern.ends_with(".*") {
                    let prefix = &pattern[1..pattern.len() - 2];
                    satker_code.starts_with(prefix)
                } else {
                    satker_code == pattern
                }
            }
            None => true,
        }
    }
}

// ============================================================================
// EFFECTIVE USER CAPABILITIES
// ============================================================================

/// Represents a user's effective capabilities from all sources
#[derive(Debug, Clone, Default)]
pub struct EffectiveCapabilities {
    /// All capability codes this user has
    pub capabilities: HashSet<String>,
    /// Policies that apply to this user
    pub policies: Vec<AuthorizationPolicy>,
    /// Access levels by resource type
    pub access_levels: HashMap<String, AccessLevel>,
    /// Whether user has root/superuser access
    pub is_superuser: bool,
}

impl EffectiveCapabilities {
    /// Check if user has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.is_superuser
            || self.capabilities.contains("*")
            || self.capabilities.contains(capability)
    }

    /// Check if user can access a path with given action
    pub fn can_access_path(&self, path: &str, action: &str) -> bool {
        if self.is_superuser {
            return true;
        }

        // Check deny policies first (higher priority)
        for policy in &self.policies {
            if policy.effect == PolicyEffect::Deny
                && policy.enabled
                && policy.matches_path(path)
                && policy.grants_capability(action)
            {
                return false;
            }
        }

        // Check allow policies
        for policy in &self.policies {
            if policy.effect == PolicyEffect::Allow
                && policy.enabled
                && policy.matches_path(path)
                && policy.grants_capability(action)
            {
                return true;
            }
        }

        false
    }
}

// ============================================================================
// AUTHORIZATION REQUEST/RESULT
// ============================================================================

/// Authorization check request
#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    pub user_id: Uuid,
    pub resource_path: String,
    pub action: String,
    pub context: HashMap<String, String>,
}

/// Authorization check result
#[derive(Debug, Clone)]
pub struct AuthorizationResult {
    pub allowed: bool,
    pub reason: String,
    pub matching_policy: Option<String>,
    pub required_capabilities: Vec<String>,
}

impl AuthorizationResult {
    pub fn allow(reason: impl Into<String>) -> Self {
        Self {
            allowed: true,
            reason: reason.into(),
            matching_policy: None,
            required_capabilities: vec![],
        }
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
            matching_policy: None,
            required_capabilities: vec![],
        }
    }
}

// ============================================================================
// DYNAMIC ROLE STORE TRAIT
// ============================================================================

/// Trait for accessing dynamic role data from database
#[async_trait::async_trait]
pub trait DynamicRoleStore: Send + Sync {
    /// Get all role types (optionally filtered by realm)
    async fn get_role_types(&self) -> Result<Vec<RoleType>, String>;

    /// Get role type by code
    async fn get_role_type_by_code(&self, code: &str) -> Result<Option<RoleType>, String>;

    /// Get all access levels
    async fn get_access_levels(&self) -> Result<Vec<AccessLevel>, String>;

    /// Get access level by code
    async fn get_access_level_by_code(&self, code: &str) -> Result<Option<AccessLevel>, String>;

    /// Get all capabilities
    async fn get_capabilities(&self) -> Result<Vec<Capability>, String>;

    /// Get capability by code
    async fn get_capability_by_code(&self, code: &str) -> Result<Option<Capability>, String>;

    /// Get effective capabilities for a user
    async fn get_user_effective_capabilities(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
    ) -> Result<EffectiveCapabilities, String>;

    /// Check if user has a specific capability
    async fn user_has_capability(
        &self,
        user_id: Uuid,
        capability: &str,
        realm_id: Uuid,
    ) -> Result<bool, String>;

    /// Check if user can access a path with given action
    async fn user_can_access_path(
        &self,
        user_id: Uuid,
        path: &str,
        action: &str,
        realm_id: Uuid,
    ) -> Result<bool, String>;

    /// Authorize a request
    async fn authorize(&self, request: AuthorizationRequest)
    -> Result<AuthorizationResult, String>;

    /// Get admin level types
    async fn get_admin_level_types(&self) -> Result<Vec<AdminLevelType>, String>;

    /// Get admin level type by code
    async fn get_admin_level_by_code(&self, code: &str) -> Result<Option<AdminLevelType>, String>;

    /// Get scope types
    async fn get_scope_types(&self) -> Result<Vec<ScopeType>, String>;

    /// Get scope type by code
    async fn get_scope_type_by_code(&self, code: &str) -> Result<Option<ScopeType>, String>;

    /// Get satker types
    async fn get_satker_types(&self) -> Result<Vec<SatkerType>, String>;

    /// Get satker type by code
    async fn get_satker_type_by_code(&self, code: &str) -> Result<Option<SatkerType>, String>;

    /// Get actor types
    async fn get_actor_types(&self) -> Result<Vec<ActorType>, String>;

    /// Get actor type by code
    async fn get_actor_type_by_code(&self, code: &str) -> Result<Option<ActorType>, String>;

    /// Get credential types
    async fn get_credential_types(&self) -> Result<Vec<CredentialTypeConfig>, String>;

    /// Get credential type by code
    async fn get_credential_type_by_code(
        &self,
        code: &str,
    ) -> Result<Option<CredentialTypeConfig>, String>;
}

// ============================================================================
// HELPER FUNCTIONS FOR MIGRATION
// ============================================================================

/// Helper module for migrating from hardcoded enums to dynamic roles
pub mod migration {


    /// Map old OrganizationRole enum value to new role type code
    pub fn map_organization_role(old_value: &str) -> &str {
        match old_value.to_uppercase().as_str() {
            "OWNER" => "owner",
            "ADMIN" => "admin",
            "MEMBER" => "member",
            "GUEST" => "guest",
            _ => "member", // Default fallback
        }
    }

    /// Map old AccessLevel enum value to new access level code
    pub fn map_access_level(old_value: &str) -> &str {
        match old_value.to_uppercase().as_str() {
            "READONLY" | "READ_ONLY" => "read",
            "READWRITE" | "READ_WRITE" => "write",
            "ADMIN" => "manage",
            "SUPERADMIN" | "SUPER_ADMIN" => "root",
            _ => "read", // Default fallback
        }
    }

    /// Map old AdminLevel enum value to new admin level type code
    pub fn map_admin_level(old_value: &str) -> &str {
        if old_value.starts_with("AdminSatker") {
            "satker"
        } else if old_value.starts_with("AdminWilayah") {
            "wilayah"
        } else if old_value == "AdminEselonI" {
            "eselon_i"
        } else if old_value == "AdminPusat" {
            "pusat"
        } else {
            "satker" // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("users/*", "users/list"));
        assert!(glob_match("users/*", "users/123"));
        assert!(!glob_match("users/*", "roles/list"));
        assert!(glob_match("*/read", "users/read"));
        assert!(glob_match("auth/token/*", "auth/token/renew"));
    }

    #[test]
    fn test_capability_matching() {
        let cap = Capability {
            id: Uuid::new_v4(),
            code: "users:*".to_string(),
            name: "All User Ops".to_string(),
            description: None,
            resource_type: Some("user".to_string()),
            action: "*".to_string(),
            is_system: true,
            is_dangerous: false,
            requires_mfa: false,
            requires_approval: false,
            metadata: serde_json::json!({}),
            realm_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(cap.matches("users:read"));
        assert!(cap.matches("users:write"));
        assert!(!cap.matches("roles:read"));
    }

    #[test]
    fn test_migration_mapping() {
        assert_eq!(migration::map_organization_role("OWNER"), "owner");
        assert_eq!(migration::map_organization_role("ADMIN"), "admin");
        assert_eq!(migration::map_access_level("READONLY"), "read");
        assert_eq!(migration::map_access_level("SUPERADMIN"), "root");
        assert_eq!(migration::map_admin_level("AdminPusat"), "pusat");
        assert_eq!(migration::map_admin_level("AdminSatker(ABC)"), "satker");
    }
}
