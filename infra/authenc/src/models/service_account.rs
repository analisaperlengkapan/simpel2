use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Service Account entity for machine-to-machine authentication
/// Service accounts represent non-human entities (services, applications, bots)
/// that need to authenticate and access resources. Unlike regular OAuth2 clients,
/// service accounts have roles and permissions assigned directly to them.
/// # Fields
/// * `id` - Unique service account identifier (UUID)
/// * `name` - Service account name (unique within realm)
/// * `description` - Optional human-readable description
/// * `client_id` - OAuth2 client identifier for authentication
/// * `client_secret_hash` - Bcrypt-hashed client secret
/// * `realm_id` - ID of the realm this service account belongs to
/// * `enabled` - Whether the service account is active
/// * `roles` - IDs of roles assigned to this service account
/// * `created_at` - Service account creation timestamp
/// * `updated_at` - Last modification timestamp
/// * `last_used_at` - Last authentication timestamp (for monitoring)
/// * `attributes` - Additional custom attributes as JSON
/// # Security Considerations
/// - Service accounts use client credentials grant (OAuth2)
/// - Client secrets must be bcrypt-hashed before storage
/// - Service accounts are scoped to realms for multi-tenancy
/// - All authentication attempts are audit logged
/// - Inactive service accounts should be disabled, not deleted (for audit trail)
/// - Role assignments control API access permissions
/// # Use Cases
/// - Microservice-to-microservice authentication
/// - Backend services accessing protected resources
/// - Scheduled jobs requiring API access
/// - CI/CD pipeline authentication
/// - Integration with external systems
/// # Example
/// ```rust
/// use uuid::Uuid;
/// use chrono::Utc;
/// let service_account = ServiceAccount {
///     id: Uuid::new_v4(),
///     name: "layanan-dasbor".to_string(),
///     description: Some("Dashboard service account".to_string()),
///     client_id: "sa-layanan-dasbor".to_string(),
///     client_secret_hash: bcrypt::hash("secret", 12).unwrap(),
///     realm_id: Uuid::new_v4(),
///     enabled: true,
///     roles: vec![],
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
///     last_used_at: None,
///     attributes: None,
/// };
/// ```
pub struct ServiceAccount {
    /// Unique identifier for the service account (UUID v4)
    pub id: Uuid,
    /// Human-readable name of the service account (must be unique within the realm)
    pub name: String,
    /// Optional description of the service account's purpose
    pub description: Option<String>,
    /// OAuth2 client identifier for authentication (must be unique)
    pub client_id: String,
    /// Bcrypt-hashed client secret (cost factor 12)
    pub client_secret_hash: String,
    /// ID of the realm this service account belongs to (enforces multi-tenancy)
    pub realm_id: Uuid,
    /// Whether the service account is enabled (disabled = cannot authenticate)
    pub enabled: bool,
    /// IDs of roles assigned to this service account
    pub roles: Vec<Uuid>,
    /// Timestamp when the service account was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the service account was last updated
    pub updated_at: DateTime<Utc>,
    /// Timestamp when the service account last authenticated (None if never used)
    pub last_used_at: Option<DateTime<Utc>>,
    /// Additional custom attributes as JSON (for extensibility)
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
/// Service account creation request
/// Parameters required to create a new service account.
/// Used when creating service accounts through the admin API.
/// # Fields
/// * `name` - Service account name (must be unique within realm)
/// * `description` - Optional human-readable description
/// * `client_secret` - Plain-text client secret (will be hashed before storage)
/// * `enabled` - Whether the service account should be enabled on creation
/// * `roles` - Optional list of role IDs to assign
/// # Security Considerations
/// - Service account names should follow naming conventions (e.g., "sa-service-name")
/// - Client secrets should be strong (min 32 characters recommended)
/// - Client secrets are auto-generated if not provided
/// - Realm ID is extracted from the request path
/// - Input validation prevents malicious names
/// - Role IDs must exist in the realm before assignment
/// # Example Request
/// ```json
/// {
///   "name": "layanan-dasbor",
///   "description": "Dashboard microservice authentication",
///   "enabled": true,
///   "roles": ["3fa85f64-5717-4562-b3fc-2c963f66afa6"]
/// }
/// ```
pub struct CreateServiceAccountRequest {
    /// Service account name (must be unique within realm)
    pub name: String,
    /// Optional human-readable description
    pub description: Option<String>,
    /// Plain-text client secret (if not provided, will be auto-generated)
    pub client_secret: Option<String>,
    /// Whether the service account should be enabled on creation (default: true)
    pub enabled: Option<bool>,
    /// Optional list of role IDs to assign to this service account
    pub roles: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, Serialize)]
/// Service account update request
/// Parameters for updating an existing service account.
/// All fields are optional to allow partial updates.
/// # Fields
/// * `name` - New service account name (if updating)
/// * `description` - New description (if updating)
/// * `enabled` - New enabled state (if updating)
/// * `roles` - New list of role IDs (replaces existing roles if provided)
/// # Security Considerations
/// - Service account name changes may affect existing integrations
/// - Updates should be authorized based on admin permissions
/// - Service account name uniqueness must be maintained
/// - Changes should trigger audit logging
/// - Disabling a service account immediately revokes access
/// - Role changes take effect on next token issuance
/// # Example Request
/// ```json
/// {
///   "description": "Updated description",
///   "enabled": false,
///   "roles": ["new-role-id-1", "new-role-id-2"]
/// }
/// ```
pub struct UpdateServiceAccountRequest {
    /// New service account name (if updating)
    pub name: Option<String>,
    /// New description (if updating)
    pub description: Option<String>,
    /// New enabled state (if updating)
    pub enabled: Option<bool>,
    /// New list of role IDs (replaces existing roles if provided)
    pub roles: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize)]
/// Service account credential regeneration response
/// Response containing the new client secret after regeneration.
/// The client secret is returned only once and cannot be retrieved again.
/// # Security Considerations
/// - The plaintext secret is only returned in this response
/// - Clients must securely store the secret
/// - Old credentials are immediately revoked
/// - Regeneration is audit logged
/// - Consider notifying service owners about secret changes
/// # Example Response
/// ```json
/// {
///   "client_id": "sa-layanan-dasbor",
///   "client_secret": "newly-generated-secret-here",
///   "message": "Client secret regenerated successfully. Store this securely - it cannot be retrieved again."
/// }
/// ```
pub struct RegenerateSecretResponse {
    /// OAuth2 client identifier
    pub client_id: String,
    /// Newly generated client secret (plaintext)
    pub client_secret: String,
    /// Informational message about secret storage
    pub message: String,
}

#[derive(Debug, Serialize)]
/// Service account list response item
/// Abbreviated service account information for list views.
/// Does not include sensitive data like client secrets.
/// # Security Considerations
/// - Client secret hash is never returned in API responses
/// - Only authorized admins can list service accounts
/// - Filtering and pagination prevent information disclosure
pub struct ServiceAccountListItem {
    /// Unique identifier
    pub id: Uuid,
    /// Service account name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// OAuth2 client identifier
    pub client_id: String,
    /// Realm identifier
    pub realm_id: Uuid,
    /// Whether enabled
    pub enabled: bool,
    /// Number of assigned roles
    pub role_count: usize,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last authentication timestamp
    pub last_used_at: Option<DateTime<Utc>>,
}

impl From<ServiceAccount> for ServiceAccountListItem {
    fn from(sa: ServiceAccount) -> Self {
        Self {
            id: sa.id,
            name: sa.name,
            description: sa.description,
            client_id: sa.client_id,
            realm_id: sa.realm_id,
            enabled: sa.enabled,
            role_count: sa.roles.len(),
            created_at: sa.created_at,
            last_used_at: sa.last_used_at,
        }
    }
}

#[derive(Debug, Serialize)]
/// Service account response (without sensitive data)
/// Full service account information for single-item views.
/// Excludes client secret hash for security.
pub struct ServiceAccountResponse {
    /// Unique identifier
    pub id: Uuid,
    /// Service account name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// OAuth2 client identifier
    pub client_id: String,
    /// Realm identifier
    pub realm_id: Uuid,
    /// Whether enabled
    pub enabled: bool,
    /// List of assigned role IDs
    pub roles: Vec<Uuid>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Last authentication timestamp
    pub last_used_at: Option<DateTime<Utc>>,
    /// Additional attributes
    pub attributes: Option<serde_json::Value>,
}

impl From<ServiceAccount> for ServiceAccountResponse {
    fn from(sa: ServiceAccount) -> Self {
        Self {
            id: sa.id,
            name: sa.name,
            description: sa.description,
            client_id: sa.client_id,
            realm_id: sa.realm_id,
            enabled: sa.enabled,
            roles: sa.roles,
            created_at: sa.created_at,
            updated_at: sa.updated_at,
            last_used_at: sa.last_used_at,
            attributes: sa.attributes,
        }
    }
}
