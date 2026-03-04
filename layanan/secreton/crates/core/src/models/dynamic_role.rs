//! Dynamic Role and Capability System for Secreton
//!
//! This module provides a fully dynamic, database-driven role and permission system
//! following HashiCorp Vault best practices.
//!
//! # Design Principles
//!
//! 1. **Policy-Based Authorization**: Access determined by policies, not role names
//! 2. **Path-Based Access Control**: Vault-style path patterns with glob matching
//! 3. **Capability-Based Permissions**: Fine-grained, composable permissions
//! 4. **Namespace Isolation**: Multi-tenant support
//! 5. **Bootstrap-Only Defaults**: Only essential system roles pre-defined
//!
//! # Usage
//!
//! ```ignore
//! use secreton_core::models::dynamic_role::{DynamicRoleStore, CapabilityChecker};
//!
//! // Check if user has a capability
//! let has_access = role_store.user_has_capability(user_id, "encrypt", "default").await?;
//!
//! // Check path-based access (Vault-style)
//! let can_access = role_store.check_path_access(&policies, "secret/data/app/*", "read").await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Bootstrap capability codes - these are the ONLY capabilities referenced directly in code
pub mod bootstrap {
    /// Root capability (all access)
    pub const ROOT: &str = "*";
    /// Sudo capability (protected paths)
    pub const SUDO: &str = "sudo";

    /// Check if a capability is a bootstrap capability
    pub fn is_bootstrap_capability(code: &str) -> bool {
        matches!(code, ROOT | SUDO)
    }
}

// ============================================================================
// DYNAMIC CAPABILITY (Replaces Permission enum in auth.rs)
// ============================================================================

/// Dynamic capability loaded from database
///
/// This replaces the hardcoded `Permission` enum. Capabilities are now
/// fully configurable via the database without code changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub is_system: bool,
    pub is_dangerous: bool,
    pub requires_sudo: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Capability {
    /// Check if this is a wildcard capability
    pub fn is_wildcard(&self) -> bool {
        self.code == "*"
    }

    /// Check if this capability matches another code
    pub fn matches(&self, code: &str) -> bool {
        if self.is_wildcard() {
            return true;
        }
        self.code == code
    }
}

// ============================================================================
// DYNAMIC USER ROLE TYPE (Replaces UserRole enum in auth.rs)
// ============================================================================

/// Dynamic user role type loaded from database
///
/// This replaces the hardcoded `UserRole` enum. Role types are now
/// fully configurable via the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleType {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
    pub is_system: bool,
    pub default_capabilities: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserRoleType {
    /// Check if this role has a default capability
    pub fn has_default_capability(&self, capability: &str) -> bool {
        self.default_capabilities
            .iter()
            .any(|c| c == "*" || c == capability)
    }

    /// Compare priority with another role type
    pub fn has_higher_priority(&self, other: &UserRoleType) -> bool {
        self.priority > other.priority
    }
}

// ============================================================================
// DYNAMIC AUTH METHOD TYPE (Replaces AuthMethodType enum)
// ============================================================================

/// Dynamic auth method type loaded from database
///
/// This replaces the hardcoded `AuthMethodType` enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMethodType {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_external: bool,
    pub config_schema: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AuthMethodType {
    /// Check if this auth method requires external identity provider
    pub fn requires_external_idp(&self) -> bool {
        self.is_external
    }
}

// ============================================================================
// DYNAMIC TOKEN TYPE (Replaces TokenType enum)
// ============================================================================

/// Dynamic token type loaded from database
///
/// This replaces the hardcoded `TokenType` enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenType {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_renewable: bool,
    pub is_orphan_allowed: bool,
    pub max_ttl_seconds: Option<i32>,
    pub default_ttl_seconds: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TokenType {
    /// Check if tokens of this type can be renewed
    pub fn can_renew(&self) -> bool {
        self.is_renewable
    }

    /// Check if orphan tokens are allowed
    pub fn allows_orphan(&self) -> bool {
        self.is_orphan_allowed
    }

    /// Get effective TTL (capped at max if specified)
    pub fn effective_ttl(&self, requested: Option<i32>) -> i32 {
        let ttl = requested.unwrap_or(self.default_ttl_seconds);
        match self.max_ttl_seconds {
            Some(max) => ttl.min(max),
            None => ttl,
        }
    }
}

// ============================================================================
// DYNAMIC SSH KEY TYPE (Replaces SshKeyType enum)
// ============================================================================

/// Dynamic SSH key type loaded from database
///
/// This replaces the hardcoded `SshKeyType` enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyType {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub algorithm: String,
    pub key_size: Option<i32>,
    pub is_system: bool,
    pub is_deprecated: bool,
    pub security_level: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SshKeyType {
    /// Check if this key type is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.is_deprecated
    }

    /// Get security level (standard, high, critical)
    pub fn security_level(&self) -> &str {
        &self.security_level
    }
}

// ============================================================================
// ENGINE ROLE TYPE (For transit, PKI, SSH engines)
// ============================================================================

/// Engine-specific role type loaded from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRoleType {
    pub id: Uuid,
    pub engine_type: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub default_config: serde_json::Value,
    pub allowed_operations: Vec<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EngineRoleType {
    /// Check if an operation is allowed for this role
    pub fn allows_operation(&self, operation: &str) -> bool {
        self.allowed_operations
            .iter()
            .any(|o| o == "*" || o == operation)
    }
}

// ============================================================================
// POLICY RULE (Vault-style)
// ============================================================================

/// Policy rule for path-based access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: Uuid,
    pub policy_id: i64,
    pub effect: PolicyEffect,
    pub path_pattern: String,
    pub capabilities: Vec<String>,
    pub required_parameters: serde_json::Value,
    pub allowed_parameters: serde_json::Value,
    pub denied_parameters: serde_json::Value,
    pub min_wrapping_ttl: Option<i32>,
    pub max_wrapping_ttl: Option<i32>,
    pub conditions: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Policy effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

impl PolicyRule {
    /// Check if a path matches this rule
    pub fn matches_path(&self, path: &str) -> bool {
        glob_match(&self.path_pattern, path)
    }

    /// Check if this rule grants a capability
    pub fn grants_capability(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| c == "*" || c == capability)
    }

    /// Check if request parameters are valid for this rule
    pub fn validate_parameters(&self, params: &serde_json::Value) -> bool {
        // Check required parameters
        if let serde_json::Value::Object(required) = &self.required_parameters {
            for (key, _) in required {
                if params.get(key).is_none() {
                    return false;
                }
            }
        }

        // Check denied parameters
        if let serde_json::Value::Object(denied) = &self.denied_parameters {
            if let serde_json::Value::Object(params_obj) = params {
                for key in denied.keys() {
                    if params_obj.contains_key(key) {
                        return false;
                    }
                }
            }
        }

        true
    }
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" || pattern == "/*" {
        return true;
    }

    // Handle ** for recursive matching
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            if !text.starts_with(prefix) {
                return false;
            }

            if suffix.is_empty() {
                return true;
            }

            return text.ends_with(suffix);
        }
    }

    // Handle single * for segment matching
    if pattern.ends_with("/*") {
        let prefix = &pattern[..pattern.len() - 1];
        return text.starts_with(prefix);
    }

    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return text.starts_with(prefix);
    }

    pattern == text
}

// ============================================================================
// EFFECTIVE USER CAPABILITIES
// ============================================================================

/// Represents a user's effective capabilities from all sources
#[derive(Debug, Clone, Default)]
pub struct EffectiveCapabilities {
    /// User ID
    pub user_id: Uuid,
    /// All capability codes this user has
    pub capabilities: HashSet<String>,
    /// Role types assigned to this user
    pub role_types: Vec<UserRoleType>,
    /// Policies assigned to this user
    pub policies: Vec<String>,
    /// Namespace
    pub namespace: String,
    /// Whether user has root access
    pub is_root: bool,
}

impl EffectiveCapabilities {
    /// Check if user has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.is_root || self.capabilities.contains("*") || self.capabilities.contains(capability)
    }

    /// Get all unique capabilities
    pub fn all_capabilities(&self) -> Vec<String> {
        self.capabilities.iter().cloned().collect()
    }
}

// ============================================================================
// PATH ACCESS CHECK
// ============================================================================

/// Result of a path access check
#[derive(Debug, Clone)]
pub struct PathAccessResult {
    pub allowed: bool,
    pub matching_policy: Option<String>,
    pub matching_rule: Option<PolicyRule>,
    pub reason: String,
}

impl PathAccessResult {
    pub fn allow(policy: &str) -> Self {
        Self {
            allowed: true,
            matching_policy: Some(policy.to_string()),
            matching_rule: None,
            reason: "Access granted by policy".to_string(),
        }
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            matching_policy: None,
            matching_rule: None,
            reason: reason.into(),
        }
    }

    pub fn deny_by_policy(policy: &str) -> Self {
        Self {
            allowed: false,
            matching_policy: Some(policy.to_string()),
            matching_rule: None,
            reason: "Access denied by policy".to_string(),
        }
    }
}

// ============================================================================
// DYNAMIC ROLE STORE TRAIT
// ============================================================================

/// Trait for accessing dynamic role data from database
#[async_trait::async_trait]
pub trait DynamicRoleStore: Send + Sync {
    /// Get all user role types
    async fn get_user_role_types(&self) -> Result<Vec<UserRoleType>, String>;

    /// Get user role type by code
    async fn get_user_role_type_by_code(&self, code: &str) -> Result<Option<UserRoleType>, String>;

    /// Get all capabilities
    async fn get_capabilities(&self) -> Result<Vec<Capability>, String>;

    /// Get capability by code
    async fn get_capability_by_code(&self, code: &str) -> Result<Option<Capability>, String>;

    /// Get all auth method types
    async fn get_auth_method_types(&self) -> Result<Vec<AuthMethodType>, String>;

    /// Get auth method type by code
    async fn get_auth_method_type_by_code(
        &self,
        code: &str,
    ) -> Result<Option<AuthMethodType>, String>;

    /// Get all token types
    async fn get_token_types(&self) -> Result<Vec<TokenType>, String>;

    /// Get token type by code
    async fn get_token_type_by_code(&self, code: &str) -> Result<Option<TokenType>, String>;

    /// Get all SSH key types
    async fn get_ssh_key_types(&self) -> Result<Vec<SshKeyType>, String>;

    /// Get SSH key type by code
    async fn get_ssh_key_type_by_code(&self, code: &str) -> Result<Option<SshKeyType>, String>;

    /// Get engine role types for an engine
    async fn get_engine_role_types(&self, engine_type: &str)
    -> Result<Vec<EngineRoleType>, String>;

    /// Get effective capabilities for a user in a namespace
    async fn get_user_effective_capabilities(
        &self,
        user_id: Uuid,
        namespace: &str,
    ) -> Result<EffectiveCapabilities, String>;

    /// Check if user has a capability in a namespace
    async fn user_has_capability(
        &self,
        user_id: Uuid,
        capability: &str,
        namespace: &str,
    ) -> Result<bool, String>;

    /// Check path access based on policies
    async fn check_path_access(
        &self,
        policies: &[String],
        path: &str,
        action: &str,
        namespace: &str,
    ) -> Result<PathAccessResult, String>;
}

// ============================================================================
// HELPER FUNCTIONS FOR MIGRATION
// ============================================================================

/// Helper module for migrating from hardcoded enums to dynamic types
pub mod migration {
    use super::*;

    /// Map old UserRole enum value to new role type code
    pub fn map_user_role(old_value: &str) -> &str {
        match old_value.to_lowercase().as_str() {
            "admin" => "admin",
            "engineadmin" | "engine-admin" | "engine_admin" => "engine-admin",
            "keymanager" | "key-manager" | "key_manager" => "key-manager",
            "cryptouser" | "crypto-user" | "crypto_user" => "crypto-user",
            "readonly" | "read-only" | "read_only" => "read-only",
            _ => "read-only", // Default fallback
        }
    }

    /// Map old Permission enum value to new capability code
    pub fn map_permission(old_value: &str) -> &str {
        match old_value.to_lowercase().as_str() {
            "createkey" | "create-key" | "create_key" => "create-key",
            "deletekey" | "delete-key" | "delete_key" => "delete-key",
            "rotatekey" | "rotate-key" | "rotate_key" => "rotate-key",
            "readkey" | "read-key" | "read_key" => "read-key",
            "listkeys" | "list-keys" | "list_keys" => "list-keys",
            "encrypt" => "encrypt",
            "decrypt" => "decrypt",
            "sign" => "sign",
            "verify" => "verify",
            "generaterandom" | "generate-random" | "generate_random" => "generate-random",
            "hashdata" | "hash-data" | "hash_data" => "hash-data",
            "derivekey" | "derive-key" | "derive_key" => "derive-key",
            "viewmetrics" | "view-metrics" | "view_metrics" => "view-metrics",
            "configuresystem" | "configure-system" | "configure_system" => "configure-system",
            "manageusers" | "manage-users" | "manage_users" => "manage-users",
            "accessauditlogs" | "access-audit-logs" | "access_audit_logs" => "access-audit-logs",
            _ => "read", // Default fallback
        }
    }

    /// Map old AuthMethodType enum value to new auth method code
    pub fn map_auth_method(old_value: &str) -> &str {
        match old_value.to_lowercase().as_str() {
            "token" => "token",
            "userpass" => "userpass",
            "ldap" => "ldap",
            "oidc" => "oidc",
            "okta" => "okta",
            "github" => "github",
            "radius" => "radius",
            "approle" => "approle",
            "kubernetes" => "kubernetes",
            _ => "token", // Default fallback
        }
    }

    /// Map old TokenType enum value to new token type code
    pub fn map_token_type(old_value: &str) -> &str {
        match old_value.to_lowercase().as_str() {
            "service" => "service",
            "batch" => "batch",
            "root" => "root",
            _ => "service", // Default fallback
        }
    }

    /// Map old SshKeyType enum value to new SSH key type code
    pub fn map_ssh_key_type(old_value: &str) -> &str {
        match old_value.to_lowercase().as_str() {
            "rsa2048" | "rsa-2048" | "rsa_2048" => "rsa-2048",
            "rsa4096" | "rsa-4096" | "rsa_4096" => "rsa-4096",
            "ed25519" => "ed25519",
            "ecdsa256" | "ecdsa-256" | "ecdsa_256" => "ecdsa-256",
            "ecdsa384" | "ecdsa-384" | "ecdsa_384" => "ecdsa-384",
            "ecdsa521" | "ecdsa-521" | "ecdsa_521" => "ecdsa-521",
            _ => "ed25519", // Default to most secure
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("/*", "/anything"));
        assert!(glob_match("secret/*", "secret/foo"));
        assert!(glob_match("secret/*", "secret/bar/baz"));
        assert!(glob_match("secret/data/*", "secret/data/myapp"));
        assert!(!glob_match("secret/data/*", "secret/metadata/myapp"));
    }

    #[test]
    fn test_capability_matching() {
        let cap = Capability {
            id: Uuid::new_v4(),
            code: "*".to_string(),
            name: "Root".to_string(),
            description: None,
            category: "root".to_string(),
            is_system: true,
            is_dangerous: true,
            requires_sudo: false,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(cap.is_wildcard());
        assert!(cap.matches("anything"));
        assert!(cap.matches("encrypt"));
    }

    #[test]
    fn test_migration_mapping() {
        assert_eq!(migration::map_user_role("Admin"), "admin");
        assert_eq!(migration::map_user_role("engine-admin"), "engine-admin");
        assert_eq!(migration::map_permission("CreateKey"), "create-key");
        assert_eq!(migration::map_permission("ENCRYPT"), "encrypt");
        assert_eq!(migration::map_token_type("Service"), "service");
        assert_eq!(migration::map_ssh_key_type("Ed25519"), "ed25519");
    }

    #[test]
    fn test_policy_rule_matching() {
        let rule = PolicyRule {
            id: Uuid::new_v4(),
            policy_id: 1,
            effect: PolicyEffect::Allow,
            path_pattern: "secret/data/*".to_string(),
            capabilities: vec!["read".to_string(), "list".to_string()],
            required_parameters: serde_json::json!({}),
            allowed_parameters: serde_json::json!({}),
            denied_parameters: serde_json::json!({}),
            min_wrapping_ttl: None,
            max_wrapping_ttl: None,
            conditions: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(rule.matches_path("secret/data/myapp"));
        assert!(rule.grants_capability("read"));
        assert!(rule.grants_capability("list"));
        assert!(!rule.grants_capability("write"));
    }
}
