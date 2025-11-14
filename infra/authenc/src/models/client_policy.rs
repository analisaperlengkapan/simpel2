//! Client Policy and Profile Models
//!
//! Database models for client policies and profiles, supporting
//! comprehensive policy-based security enforcement for OAuth2/OIDC clients.

use crate::error::AuthencError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Client Policy database model
///
/// Represents a single security policy that can be applied to OAuth2/OIDC clients.
/// Policies contain conditions that must be met and executors that enforce security rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPolicyModel {
    /// Unique identifier for the policy
    pub id: Uuid,
    /// Realm this policy belongs to
    pub realm_id: Uuid,
    /// Policy name (unique within realm)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Whether the policy is enabled
    pub enabled: bool,
    /// List of condition identifiers that must be met
    pub conditions: Vec<String>,
    /// Configuration for conditions (JSON)
    pub condition_config: serde_json::Value,
    /// List of executor identifiers to run
    pub executors: Vec<String>,
    /// Configuration for executors (JSON)
    pub executor_config: serde_json::Value,
    /// Priority (higher = executed first)
    pub priority: i32,
    /// Policy type (e.g., "security", "compliance", "custom")
    pub policy_type: String,
    /// Timestamp when created
    pub created_at: DateTime<Utc>,
    /// Timestamp when last updated
    pub updated_at: DateTime<Utc>,
    /// User who created the policy
    pub created_by: Option<Uuid>,
}

/// Client Profile database model
///
/// A profile is a reusable collection of policies that can be applied to multiple clients.
/// Profiles enable consistent policy enforcement across similar client types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProfileModel {
    /// Unique identifier for the profile
    pub id: Uuid,
    /// Realm this profile belongs to
    pub realm_id: Uuid,
    /// Profile name (unique within realm)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Whether the profile is enabled
    pub enabled: bool,
    /// List of policy IDs included in this profile
    pub policy_ids: Vec<Uuid>,
    /// Profile type (e.g., "fapi-baseline", "fapi-advanced", "custom")
    pub profile_type: String,
    /// Is this a built-in profile (cannot be deleted)
    pub is_builtin: bool,
    /// Timestamp when created
    pub created_at: DateTime<Utc>,
    /// Timestamp when last updated
    pub updated_at: DateTime<Utc>,
    /// User who created the profile
    pub created_by: Option<Uuid>,
}

/// Client-Policy Assignment
///
/// Associates a specific policy with a client, either directly or through a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPolicyAssignment {
    /// Unique identifier for the assignment
    pub id: Uuid,
    /// Client ID
    pub client_id: Uuid,
    /// Policy ID (if direct assignment)
    pub policy_id: Option<Uuid>,
    /// Profile ID (if profile assignment)
    pub profile_id: Option<Uuid>,
    /// Assignment type ("direct" or "profile")
    pub assignment_type: String,
    /// Whether the assignment is enabled
    pub enabled: bool,
    /// Override priority for this assignment
    pub priority_override: Option<i32>,
    /// Timestamp when assigned
    pub assigned_at: DateTime<Utc>,
    /// User who made the assignment
    pub assigned_by: Option<Uuid>,
}

/// Request to create a new client policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientPolicyRequest {
    /// Realm ID
    pub realm_id: Uuid,
    /// Policy name
    pub name: String,
    /// Description
    pub description: String,
    /// Enabled status
    pub enabled: bool,
    /// Condition identifiers
    pub conditions: Vec<String>,
    /// Condition configuration
    pub condition_config: serde_json::Value,
    /// Executor identifiers
    pub executors: Vec<String>,
    /// Executor configuration
    pub executor_config: serde_json::Value,
    /// Priority
    pub priority: i32,
    /// Policy type
    pub policy_type: String,
}

/// Request to update an existing client policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientPolicyRequest {
    /// New name (optional)
    pub name: Option<String>,
    /// New description (optional)
    pub description: Option<String>,
    /// New enabled status (optional)
    pub enabled: Option<bool>,
    /// New conditions (optional)
    pub conditions: Option<Vec<String>>,
    /// New condition config (optional)
    pub condition_config: Option<serde_json::Value>,
    /// New executors (optional)
    pub executors: Option<Vec<String>>,
    /// New executor config (optional)
    pub executor_config: Option<serde_json::Value>,
    /// New priority (optional)
    pub priority: Option<i32>,
}

/// Request to create a new client profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientProfileRequest {
    /// Realm ID
    pub realm_id: Uuid,
    /// Profile name
    pub name: String,
    /// Description
    pub description: String,
    /// Enabled status
    pub enabled: bool,
    /// Policy IDs to include
    pub policy_ids: Vec<Uuid>,
    /// Profile type
    pub profile_type: String,
}

/// Request to update an existing client profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientProfileRequest {
    /// New name (optional)
    pub name: Option<String>,
    /// New description (optional)
    pub description: Option<String>,
    /// New enabled status (optional)
    pub enabled: Option<bool>,
    /// New policy IDs (optional)
    pub policy_ids: Option<Vec<Uuid>>,
}

/// Request to assign a policy or profile to a client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignClientPolicyRequest {
    /// Client ID
    pub client_id: Uuid,
    /// Policy ID (for direct assignment)
    pub policy_id: Option<Uuid>,
    /// Profile ID (for profile assignment)
    pub profile_id: Option<Uuid>,
    /// Enabled status
    pub enabled: bool,
    /// Priority override
    pub priority_override: Option<i32>,
}

/// Response for policy queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPolicyResponse {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub conditions: Vec<String>,
    pub condition_config: serde_json::Value,
    pub executors: Vec<String>,
    pub executor_config: serde_json::Value,
    pub priority: i32,
    pub policy_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for profile queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProfileResponse {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub policy_ids: Vec<Uuid>,
    pub policies: Vec<ClientPolicyResponse>,
    pub profile_type: String,
    pub is_builtin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row conversions
impl TryFrom<tokio_postgres::Row> for ClientPolicyModel {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            realm_id: row.try_get("realm_id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            enabled: row.try_get("enabled")?,
            conditions: row.try_get("conditions")?,
            condition_config: row.try_get("condition_config")?,
            executors: row.try_get("executors")?,
            executor_config: row.try_get("executor_config")?,
            priority: row.try_get("priority")?,
            policy_type: row.try_get("policy_type")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            created_by: row.try_get("created_by")?,
        })
    }
}

impl TryFrom<tokio_postgres::Row> for ClientProfileModel {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            realm_id: row.try_get("realm_id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            enabled: row.try_get("enabled")?,
            policy_ids: row.try_get("policy_ids")?,
            profile_type: row.try_get("profile_type")?,
            is_builtin: row.try_get("is_builtin")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            created_by: row.try_get("created_by")?,
        })
    }
}

impl TryFrom<tokio_postgres::Row> for ClientPolicyAssignment {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            client_id: row.try_get("client_id")?,
            policy_id: row.try_get("policy_id")?,
            profile_id: row.try_get("profile_id")?,
            assignment_type: row.try_get("assignment_type")?,
            enabled: row.try_get("enabled")?,
            priority_override: row.try_get("priority_override")?,
            assigned_at: row.try_get("assigned_at")?,
            assigned_by: row.try_get("assigned_by")?,
        })
    }
}

impl From<ClientPolicyModel> for ClientPolicyResponse {
    fn from(model: ClientPolicyModel) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id,
            name: model.name,
            description: model.description,
            enabled: model.enabled,
            conditions: model.conditions,
            condition_config: model.condition_config,
            executors: model.executors,
            executor_config: model.executor_config,
            priority: model.priority,
            policy_type: model.policy_type,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
