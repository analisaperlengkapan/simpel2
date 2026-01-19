use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Client Scope entity - reusable OAuth2/OIDC scope definition
/// Client scopes are reusable scope configurations that can be assigned to clients
/// as either default scopes (automatically granted) or optional scopes (require user consent).
/// # Standards Compliance
/// - OAuth 2.0 RFC 6749 (scopes for access delegation)
/// - OpenID Connect Core 1.0 (standard OIDC scopes: openid, profile, email, etc.)
/// - OAuth 2.0 Incremental Authorization (consent management)
/// # Use Cases
/// - Standard OIDC scopes (openid, profile, email, address, phone)
/// - Custom application scopes (read:aset, write:aset, admin:satker)
/// - Consent management with human-readable descriptions
/// - Protocol mapper associations for claim generation
pub struct ClientScope {
    /// Unique identifier for the scope
    pub id: Uuid,
    /// Realm this scope belongs to
    pub realm_id: Uuid,

    /// Scope name (e.g., "openid", "profile", "read:aset")
    pub name: String,
    /// Human-readable display name for consent UI
    pub display_name: Option<String>,
    /// Detailed description for users
    pub description: Option<String>,

    /// Protocol this scope applies to (openid-connect, saml)
    pub protocol: String,

    /// Whether user consent is required for this scope
    pub consent_required: bool,
    /// Whether to display this scope on consent screen
    pub display_on_consent_screen: bool,
    /// Custom text for consent screen
    pub consent_screen_text: Option<String>,

    /// Whether to include this scope in token scope claim
    pub include_in_token_scope: bool,
    /// Display order in UI (lower = higher priority)
    pub gui_order: i32,

    /// Icon URI for UI presentation
    pub icon_uri: Option<String>,

    /// Custom attributes (audience, resources, etc.)
    pub attributes: serde_json::Value,

    /// Whether this scope is enabled
    pub enabled: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Client scope creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientScopeRequest {
    /// Scope name (required, must be unique per realm)
    pub name: String,
    /// Human-readable display name
    pub display_name: Option<String>,
    /// Detailed description
    pub description: Option<String>,
    /// Protocol (default: "openid-connect")
    pub protocol: Option<String>,
    /// Consent required (default: true)
    pub consent_required: Option<bool>,
    /// Display on consent screen (default: true)
    pub display_on_consent_screen: Option<bool>,
    /// Custom consent screen text
    pub consent_screen_text: Option<String>,
    /// Include in token scope (default: true)
    pub include_in_token_scope: Option<bool>,
    /// Display order (default: 0)
    pub gui_order: Option<i32>,
    /// Icon URI
    pub icon_uri: Option<String>,
    /// Custom attributes
    pub attributes: Option<serde_json::Value>,
}

/// Client scope update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientScopeRequest {
    /// New display name
    pub display_name: Option<String>,
    /// New description
    pub description: Option<String>,
    /// Update consent required
    pub consent_required: Option<bool>,
    /// Update display on consent screen
    pub display_on_consent_screen: Option<bool>,
    /// Update consent screen text
    pub consent_screen_text: Option<String>,
    /// Update include in token scope
    pub include_in_token_scope: Option<bool>,
    /// Update display order
    pub gui_order: Option<i32>,
    /// Update icon URI
    pub icon_uri: Option<String>,
    /// Update attributes
    pub attributes: Option<serde_json::Value>,
    /// Update enabled status
    pub enabled: Option<bool>,
}

/// Client scope response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientScopeResponse {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub protocol: String,
    pub consent_required: bool,
    pub display_on_consent_screen: bool,
    pub consent_screen_text: Option<String>,
    pub include_in_token_scope: bool,
    pub gui_order: i32,
    pub icon_uri: Option<String>,
    pub attributes: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ClientScope> for ClientScopeResponse {
    fn from(scope: ClientScope) -> Self {
        Self {
            id: scope.id,
            realm_id: scope.realm_id,
            name: scope.name,
            display_name: scope.display_name,
            description: scope.description,
            protocol: scope.protocol,
            consent_required: scope.consent_required,
            display_on_consent_screen: scope.display_on_consent_screen,
            consent_screen_text: scope.consent_screen_text,
            include_in_token_scope: scope.include_in_token_scope,
            gui_order: scope.gui_order,
            icon_uri: scope.icon_uri,
            attributes: scope.attributes,
            enabled: scope.enabled,
            created_at: scope.created_at,
            updated_at: scope.updated_at,
        }
    }
}

impl TryFrom<tokio_postgres::Row> for ClientScope {
    type Error = crate::error::AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get id: {}", e))
            })?,
            realm_id: row.try_get("realm_id").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get realm_id: {}", e))
            })?,
            name: row.try_get("name").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get name: {}", e))
            })?,
            display_name: row.try_get("display_name").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get display_name: {}", e))
            })?,
            description: row.try_get("description").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get description: {}", e))
            })?,
            protocol: row.try_get("protocol").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get protocol: {}", e))
            })?,
            consent_required: row.try_get("consent_required").map_err(|e| {
                crate::error::AuthencError::database(format!(
                    "Failed to get consent_required: {}",
                    e
                ))
            })?,
            display_on_consent_screen: row.try_get("display_on_consent_screen").map_err(|e| {
                crate::error::AuthencError::database(format!(
                    "Failed to get display_on_consent_screen: {}",
                    e
                ))
            })?,
            consent_screen_text: row.try_get("consent_screen_text").map_err(|e| {
                crate::error::AuthencError::database(format!(
                    "Failed to get consent_screen_text: {}",
                    e
                ))
            })?,
            include_in_token_scope: row.try_get("include_in_token_scope").map_err(|e| {
                crate::error::AuthencError::database(format!(
                    "Failed to get include_in_token_scope: {}",
                    e
                ))
            })?,
            gui_order: row.try_get("gui_order").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get gui_order: {}", e))
            })?,
            icon_uri: row.try_get("icon_uri").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get icon_uri: {}", e))
            })?,
            attributes: row.try_get("attributes").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get attributes: {}", e))
            })?,
            enabled: row.try_get("enabled").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get enabled: {}", e))
            })?,
            created_at: row.try_get("created_at").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get created_at: {}", e))
            })?,
            updated_at: row.try_get("updated_at").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get updated_at: {}", e))
            })?,
        })
    }
}

/// Client default scope assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientDefaultScope {
    pub id: Uuid,
    pub client_id: Uuid,
    pub scope_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Client optional scope assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientOptionalScope {
    pub id: Uuid,
    pub client_id: Uuid,
    pub scope_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Request to assign scopes to a client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignClientScopesRequest {
    /// Scope IDs to assign as default scopes
    pub default_scope_ids: Vec<Uuid>,
    /// Scope IDs to assign as optional scopes
    pub optional_scope_ids: Vec<Uuid>,
}

/// Scope validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeValidationResult {
    /// Whether all scopes are valid
    pub valid: bool,
    /// List of invalid scope names
    pub invalid_scopes: Vec<String>,
    /// List of valid scope objects
    pub valid_scopes: Vec<ClientScope>,
}

/// Consent check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentCheckResult {
    /// Whether consent is needed
    pub consent_needed: bool,
    /// Scopes that require consent
    pub scopes_requiring_consent: Vec<ClientScopeResponse>,
    /// Scopes already consented
    pub consented_scopes: Vec<String>,
    /// Scopes missing consent
    pub missing_consent_scopes: Vec<String>,
}

/// User consent scope (structured consent tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConsentScope {
    pub id: Uuid,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub scope_id: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub consent_source: String, // explicit, implicit, pre-authorized
}

/// Request to grant consent for scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantScopeConsentRequest {
    /// Scope IDs to grant consent for
    pub scope_ids: Vec<Uuid>,
    /// Consent expiration in seconds (optional)
    pub expires_in: Option<i64>,
    /// Consent source
    pub consent_source: Option<String>,
}

/// Scope assignment info (for client configuration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientScopeAssignment {
    /// Scope details
    pub scope: ClientScopeResponse,
    /// Assignment type (default or optional)
    pub assignment_type: ScopeAssignmentType,
}

/// Enum `ScopeAssignmentType`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScopeAssignmentType {
    Default,
    Optional,
}

/// Standard OIDC scope names
pub mod standard_scopes {
    pub const OPENID: &str = "openid";
    pub const PROFILE: &str = "profile";
    pub const EMAIL: &str = "email";
    pub const ADDRESS: &str = "address";
    pub const PHONE: &str = "phone";
    pub const OFFLINE_ACCESS: &str = "offline_access";
    pub const ROLES: &str = "roles";
    pub const GROUPS: &str = "groups";

    /// Returns all standard OIDC scope names
    pub fn all() -> Vec<&'static str> {
        vec![
            OPENID,
            PROFILE,
            EMAIL,
            ADDRESS,
            PHONE,
            OFFLINE_ACCESS,
            ROLES,
            GROUPS,
        ]
    }
}

/// SIMPelv2-specific scope names for asset management
pub mod simpel_scopes {
    pub const READ_ASET: &str = "read:aset";
    pub const WRITE_ASET: &str = "write:aset";
    pub const READ_LAPORAN: &str = "read:laporan";
    pub const WRITE_LAPORAN: &str = "write:laporan";
    pub const ADMIN_SATKER: &str = "admin:satker";
    pub const ADMIN_WILAYAH: &str = "admin:wilayah";
    pub const ADMIN_PUSAT: &str = "admin:pusat";

    /// Returns all SIMPelv2 scope names
    pub fn all() -> Vec<&'static str> {
        vec![
            READ_ASET,
            WRITE_ASET,
            READ_LAPORAN,
            WRITE_LAPORAN,
            ADMIN_SATKER,
            ADMIN_WILAYAH,
            ADMIN_PUSAT,
        ]
    }
}
