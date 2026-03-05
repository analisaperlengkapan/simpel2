use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Access level for secreton operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessLevel {
    /// Read-only access to secrets
    ReadOnly,
    /// Read and write access to secrets
    ReadWrite,
    /// Administrative access including secret management
    Admin,
    /// Super admin access with full control
    SuperAdmin,
}

/// Time-based access restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
    /// Start time for access (hour of day, 0-23)
    pub start_hour: u8,
    /// End time for access (hour of day, 0-23)
    pub end_hour: u8,
    /// Days of week when access is allowed (0=Sunday, 6=Saturday)
    pub allowed_days: Vec<u8>,
    /// Timezone for time calculations
    pub timezone: String,
}

/// Security context for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// IP address of the request
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// Risk score (0.0 = low risk, 1.0 = high risk)
    pub risk_score: Option<f64>,
    /// Additional security metadata
    pub metadata: Option<serde_json::Value>,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            ip_address: None,
            user_agent: None,
            session_id: None,
            timestamp: Utc::now(),
            risk_score: None,
            metadata: None,
        }
    }
}

/// Secreton access policy defining what secrets a user can access
/// Note: This struct is now decoupled from the User model.
/// Policies should be stored in the `attributes` JSON field or managed externally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretonAccessPolicy {
    /// Satker codes where user can access secrets
    pub allowed_satker_secrets: Vec<String>,
    /// Access level for secreton operations
    pub access_level: AccessLevel,
    /// Time-based access restrictions
    pub time_restrictions: Option<TimeRestrictions>,
    /// Whether all secret access requires audit logging
    pub audit_required: bool,
    /// Maximum number of secrets that can be accessed per hour
    pub rate_limit: Option<u32>,
    /// Specific secret paths that are explicitly allowed
    pub allowed_paths: Option<Vec<String>>,
    /// Specific secret paths that are explicitly denied
    pub denied_paths: Option<Vec<String>>,
}

impl SecretonAccessPolicy {
    /// Check if access to a secret path is allowed
    pub fn can_access_path(&self, path: &str, satker_code: &str) -> bool {
        // Check denied paths first
        if let Some(denied) = &self.denied_paths {
            if denied.iter().any(|p| path.starts_with(p)) {
                return false;
            }
        }

        // Check if satker is allowed
        if !self.allowed_satker_secrets.iter().any(|s| s == satker_code) {
            return false;
        }

        // Check allowed paths if specified
        if let Some(allowed) = &self.allowed_paths {
            allowed.iter().any(|p| path.starts_with(p))
        } else {
            true
        }
    }

    /// Check if access is allowed at the current time
    pub fn is_time_allowed(&self, current_time: &DateTime<Utc>) -> bool {
        if let Some(restrictions) = &self.time_restrictions {
            // Simplified time check - in production, would need proper timezone handling
            let hour = current_time.hour() as u8;
            let weekday = current_time.weekday().num_days_from_sunday() as u8;

            hour >= restrictions.start_hour
                && hour <= restrictions.end_hour
                && restrictions.allowed_days.contains(&weekday)
        } else {
            true
        }
    }
}

impl Default for SecretonAccessPolicy {
    fn default() -> Self {
        Self {
            allowed_satker_secrets: Vec::new(),
            access_level: AccessLevel::ReadOnly,
            time_restrictions: None,
            audit_required: false,
            rate_limit: None,
            allowed_paths: None,
            denied_paths: None,
        }
    }
}

/// JWT Claims for user authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    /// Subject identifier (user ID)
    pub sub: String,
    /// Username of the authenticated user
    pub username: String,
    /// Email address of the user
    pub email: String,
    /// Realm identifier the user belongs to
    pub realm_id: String,
    /// List of roles assigned to the user
    pub roles: Vec<String>,
    /// Token expiration timestamp
    pub exp: usize,
    /// Token issued at timestamp
    pub iat: usize,
    /// Token issuer identifier
    pub iss: String,
}

/// User entity representing an authenticated user (enhanced for SIMKARI operations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for the user
    pub id: Uuid,
    /// Username for authentication
    pub username: String,
    /// Email address of the user
    pub email: String,
    /// Whether the email address has been verified
    pub email_verified: bool,
    /// User's first name
    pub first_name: Option<String>,
    /// User's last name
    pub last_name: Option<String>,
    /// NIP (Nomor Induk Pegawai) - Employee ID number
    pub nip: Option<String>,
    /// Nama lengkap pegawai
    pub nama: Option<String>,
    /// Jabatan pegawai dalam organisasi
    pub jabatan: Option<String>,
    /// Kode satuan kerja (organizational unit code)
    pub satker_code: String,
    /// User's phone number
    pub phone_number: Option<String>,
    /// Whether the phone number has been verified
    pub phone_verified: bool,
    /// Hashed password for authentication
    pub password_hash: Option<String>,
    /// TOTP secret for two-factor authentication
    pub totp_secret: Option<String>,
    /// Backup codes for TOTP recovery
    pub totp_backup_codes: Option<Vec<String>>,
    /// Whether MFA is enabled for this user
    pub mfa_enabled: bool,
    /// Timestamp when MFA was set up
    pub mfa_setup_at: Option<DateTime<Utc>>,
    /// Timestamp when MFA was last used
    pub mfa_last_used: Option<DateTime<Utc>>,
    /// Whether WebAuthn is enabled for this user
    pub webauthn_enabled: bool,
    /// Whether the account is currently locked
    pub account_locked: bool,
    /// Timestamp until which the account is locked
    pub account_locked_until: Option<DateTime<Utc>>,
    /// Number of consecutive failed login attempts
    pub failed_login_attempts: i32,
    /// Timestamp of the last successful login
    pub last_login_at: Option<DateTime<Utc>>,
    /// Timestamp of the last failed login attempt
    pub last_failed_login_at: Option<DateTime<Utc>>,
    /// Timestamp when the password was last changed
    pub password_changed_at: Option<DateTime<Utc>>,
    /// Timestamp when the password expires
    pub password_expires_at: Option<DateTime<Utc>>,
    /// Whether the user must change their password on next login
    pub require_password_change: bool,
    /// ID of the realm this user belongs to
    pub realm_id: Option<Uuid>,
    /// ID of the organization this user belongs to
    pub organization_id: Option<Uuid>,
    /// Roles assigned to this user
    pub roles: Vec<Role>,
    /// Permissions derived from roles
    pub permissions: Vec<Permission>,
    /// Encrypted session data for SIMKARI operations
    pub session_data: Option<serde_json::Value>,
    /// Security context for the user
    pub security_context: SecurityContext,
    /// Additional user attributes as JSON (Stores Secreton Policy here if needed)
    pub attributes: Option<serde_json::Value>,
    /// Whether the user account is enabled
    pub enabled: bool,
    /// Whether this user was created via identity provider federation
    pub federated: bool,
    /// Timestamp when the user was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the user was last updated
    pub updated_at: DateTime<Utc>,
    /// Timestamp when the user was soft deleted
    pub deleted_at: Option<DateTime<Utc>>,
    /// Number of successful logins for this user
    pub login_count: i32,
}

/// User credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredential {
    /// Unique identifier for the credential
    pub id: Uuid,
    /// ID of the user this credential belongs to
    pub user_id: Uuid,
    /// Type of credential (password, TOTP, WebAuthn, etc.)
    pub credential_type: CredentialType,
    /// Credential data stored as JSON
    pub credential_data: serde_json::Value,
    /// Priority order for credential usage
    pub priority: i32,
    /// Whether this credential is enabled
    pub enabled: bool,
    /// Timestamp when the credential was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the credential was last used
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Credential type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialType {
    /// Password-based authentication
    Password,
    /// Time-based One-Time Password (TOTP)
    Totp,
    /// WebAuthn/FIDO2 authentication
    Webauthn,
    /// Recovery codes for account recovery
    RecoveryCode,
    /// Magic link authentication
    MagicLink,
    /// Social login authentication
    Social,
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Unique identifier for the session
    pub id: Uuid,
    /// ID of the user this session belongs to
    pub user_id: Uuid,
    /// Session identifier string
    pub session_id: String,
    /// Client ID associated with the session
    pub client_id: Option<String>,
    /// IP address of the client
    pub ip_address: Option<String>,
    /// User agent string from the client
    pub user_agent: Option<String>,
    /// Timestamp when the session started
    pub started_at: DateTime<Utc>,
    /// Timestamp when the session expires
    pub expires_at: DateTime<Utc>,
    /// Timestamp of the last activity in this session
    pub last_activity_at: DateTime<Utc>,
    /// Timestamp when the session was terminated
    pub terminated_at: Option<DateTime<Utc>>,
    /// Reason for session termination
    pub termination_reason: Option<String>,
    /// ID of the refresh token associated with this session
    pub refresh_token_id: Option<Uuid>,
    /// Additional session attributes as JSON
    pub attributes: Option<serde_json::Value>,
}

/// User role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    /// Unique identifier for the user-role assignment
    pub id: Uuid,
    /// ID of the user
    pub user_id: Uuid,
    /// ID of the role
    pub role_id: Uuid,
    /// ID of the user who assigned this role
    pub assigned_by: Uuid,
    /// Timestamp when the role was assigned
    pub assigned_at: DateTime<Utc>,
    /// Timestamp when the role assignment expires
    pub expires_at: Option<DateTime<Utc>>,
    /// Additional attributes for the role assignment
    pub attributes: Option<serde_json::Value>,
}

/// Role with hierarchical structure for Attorney General's Office
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier for the role
    pub id: Uuid,
    /// Name of the role
    pub name: String,
    /// Description of the role
    pub description: Option<String>,
    /// Permissions granted by this role
    pub permissions: Vec<Permission>,
    /// Admin level code that manages this role (dynamic)
    pub managed_by: Option<String>,
    /// Scope code where this role applies (dynamic)
    pub scope: Option<String>,
    /// ID of the realm this role belongs to
    pub realm_id: Option<Uuid>,
    /// Whether this is a composite role (contains other roles)
    pub composite: bool,
    /// Whether this is a client-specific role
    pub client_role: bool,
    /// Client identifier if this is a client role
    pub client_id: Option<String>,
    /// Priority level for role resolution (higher number = higher priority)
    pub priority: i32,
    /// Whether this role is active
    pub active: bool,
    /// Additional role attributes as JSON
    pub attributes: Option<serde_json::Value>,
    /// Timestamp when the role was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the role was last updated
    pub updated_at: DateTime<Utc>,
}

impl Role {
    /// Check if this role can be assigned to a user in the given satker
    /// Now uses dynamic scope checking (simplified for now)
    pub fn can_assign_to_satker(&self, satker_code: &str) -> bool {
        if let Some(scope) = &self.scope {
            // Simplified logic: strict match or wildcard
            // In a real dynamic system, we'd query ScopeType
            if scope == "global" || scope == "pusat" {
                return true;
            }
            if scope.starts_with("satker:") {
                let scope_satker = &scope[7..];
                return scope_satker == satker_code;
            }
            // Fallback for migration compatibility
            if scope == satker_code {
                return true;
            }
        } else {
            // If scope is None, check for legacy attributes
            if let Some(attrs) = &self.attributes {
                if let Some(legacy_scope) = attrs.get("role_scope").and_then(|v| v.as_str()) {
                    if legacy_scope == "global" || legacy_scope == "pusat" {
                        return true;
                    }
                    if legacy_scope.starts_with("satker:") {
                        let scope_satker = &legacy_scope[7..];
                        return scope_satker == satker_code;
                    }
                    if legacy_scope == satker_code {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if this role grants access to resources in the given scope
    pub fn grants_access_to(&self, target_scope: &str) -> bool {
        if let Some(scope) = &self.scope {
            if scope == "global" || scope == "pusat" {
                return true;
            }
            // Simple prefix matching
            return target_scope.starts_with(scope);
        }
        false
    }

    /// Get all permissions for this role that apply to the given resource type
    pub fn get_permissions_for_resource(&self, resource_type: &str) -> Vec<&Permission> {
        self.permissions
            .iter()
            .filter(|p| p.resource_type == resource_type)
            .collect()
    }
}

/// Permission for flexible resource access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Unique identifier for the permission
    pub id: Uuid,
    /// Name of the permission
    pub name: String,
    /// Description of the permission
    pub description: Option<String>,
    /// Type of resource this permission applies to (flexible for any resource type)
    pub resource_type: String,
    /// Specific resource identifier pattern (supports wildcards)
    pub resource_pattern: Option<String>,
    /// Action allowed on the resource (read, write, delete, admin, etc.)
    pub action: String,
    /// Scope where this permission applies (dynamic string)
    pub scope: Option<String>,
    /// Conditions that must be met for this permission to apply
    pub conditions: Option<serde_json::Value>,
    /// ID of the realm this permission belongs to
    pub realm_id: Option<Uuid>,
    /// Whether this permission is active
    pub active: bool,
    /// Additional permission attributes as JSON
    pub attributes: Option<serde_json::Value>,
    /// Timestamp when the permission was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the permission was last updated
    pub updated_at: DateTime<Utc>,
}

impl Permission {
    /// Check if this permission applies to the given resource
    pub fn applies_to_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
        satker_code: &str,
    ) -> bool {
        // Check resource type match
        if self.resource_type != resource_type {
            return false;
        }

        // Check scope includes the satker
        if let Some(scope) = &self.scope {
            // Simplified check - assumes scope is either global or satker prefix
            if scope != "global" && scope != "pusat" {
                if scope.starts_with("satker:") {
                    let scope_satker = &scope[7..];
                    if scope_satker != satker_code {
                        return false;
                    }
                } else if scope != satker_code {
                    return false;
                }
            }
        }

        // Check resource pattern match if specified
        if let Some(pattern) = &self.resource_pattern {
            // Simple pattern matching - in production would use proper regex
            if pattern.contains('*') {
                let prefix = pattern.trim_end_matches('*');
                resource_id.starts_with(prefix)
            } else {
                pattern == resource_id
            }
        } else {
            true
        }
    }

    /// Check if this permission allows the given action
    pub fn allows_action(&self, action: &str) -> bool {
        self.action == action || self.action == "admin" || self.action == "*"
    }
}

/// Role permission mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermission {
    /// Unique identifier for the role-permission assignment
    pub id: Uuid,
    /// ID of the role
    pub role_id: Uuid,
    /// ID of the permission
    pub permission_id: Uuid,
    /// Timestamp when the permission was assigned to the role
    pub assigned_at: DateTime<Utc>,
}

/// User group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroup {
    /// Unique identifier for the user-group assignment
    pub id: Uuid,
    /// ID of the user
    pub user_id: Uuid,
    /// ID of the group
    pub group_id: Uuid,
    /// ID of the user who assigned this group membership
    pub assigned_by: Uuid,
    /// Timestamp when the user was assigned to the group
    pub assigned_at: DateTime<Utc>,
    /// Timestamp when the group membership expires
    pub expires_at: Option<DateTime<Utc>>,
}

/// Group role mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRole {
    /// Unique identifier for the group-role assignment
    pub id: Uuid,
    /// ID of the group
    pub group_id: Uuid,
    /// ID of the role
    pub role_id: Uuid,
    /// Timestamp when the role was assigned to the group
    pub assigned_at: DateTime<Utc>,
}

/// User profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// ID of the user this profile belongs to
    pub user_id: Uuid,
    /// URL to the user's avatar image
    pub avatar_url: Option<String>,
    /// User's biography or description
    pub bio: Option<String>,
    /// User's website URL
    pub website: Option<String>,
    /// User's location
    pub location: Option<String>,
    /// User's timezone
    pub timezone: Option<String>,
    /// User's preferred locale
    pub locale: Option<String>,
    /// User's preferred theme
    pub theme: Option<String>,
    /// Additional user preferences as JSON
    pub preferences: Option<serde_json::Value>,
    /// Timestamp when the profile was last updated
    pub updated_at: DateTime<Utc>,
}

/// Authentication flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationFlow {
    /// Unique identifier for the authentication flow
    pub id: Uuid,
    /// Alias name for the flow
    pub alias: String,
    /// Description of the authentication flow
    pub description: Option<String>,
    /// ID of the realm this flow belongs to
    pub realm_id: Option<Uuid>,
    /// Provider identifier for the flow
    pub provider_id: String,
    /// Whether this is a top-level flow
    pub top_level: bool,
    /// Whether this is a built-in flow
    pub built_in: bool,
    /// Additional flow attributes as JSON
    pub attributes: Option<serde_json::Value>,
    /// Timestamp when the flow was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the flow was last updated
    pub updated_at: DateTime<Utc>,
}

/// Authentication execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationExecution {
    /// Unique identifier for the execution
    pub id: Uuid,
    /// ID of the flow this execution belongs to
    pub flow_id: Uuid,
    /// Alias name for the execution
    pub alias: String,
    /// Description of the execution
    pub description: Option<String>,
    /// Provider identifier for the execution
    pub provider_id: String,
    /// Requirement level for this execution
    pub requirement: ExecutionRequirement,
    /// Priority order of the execution
    pub priority: i32,
    /// ID of the parent flow
    pub parent_flow: Option<Uuid>,
    /// Authenticator configuration identifier
    pub authenticator_config: Option<String>,
    /// Additional execution attributes as JSON
    pub attributes: Option<serde_json::Value>,
    /// Timestamp when the execution was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the execution was last updated
    pub updated_at: DateTime<Utc>,
}

/// Execution requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionRequirement {
    /// Execution is required
    Required,
    /// Execution is an alternative option
    Alternative,
    /// Execution is disabled
    Disabled,
    /// Execution is conditional
    Conditional,
}

/// Authenticator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorConfig {
    /// Unique identifier for the configuration
    pub id: Uuid,
    /// Alias name for the configuration
    pub alias: String,
    /// Description of the configuration
    pub description: Option<String>,
    /// ID of the realm this configuration belongs to
    pub realm_id: Option<Uuid>,
    /// Provider identifier for the authenticator
    pub provider_id: String,
    /// Configuration data as JSON
    pub config: serde_json::Value,
    /// Timestamp when the configuration was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the configuration was last updated
    pub updated_at: DateTime<Utc>,
}

/// Identity provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProvider {
    /// Unique identifier for the identity provider
    pub id: Uuid,
    /// Alias name for the provider
    pub alias: String,
    /// Display name for the provider
    pub display_name: Option<String>,
    /// Provider identifier
    pub provider_id: String,
    /// Whether the provider is enabled
    pub enabled: bool,
    /// Whether to trust email from this provider
    pub trust_email: bool,
    /// Whether to store tokens from this provider
    pub store_token: bool,
    /// Whether to add read token role on user creation
    pub add_read_token_role_on_create: bool,
    /// Whether to authenticate by default with this provider
    pub authenticate_by_default: bool,
    /// Whether this provider is for linking only
    pub link_only: bool,
    /// ID of the flow for first broker login
    pub first_broker_login_flow_id: Option<Uuid>,
    /// ID of the flow for post broker login
    pub post_broker_login_flow_id: Option<Uuid>,
    /// Provider configuration as JSON
    pub config: serde_json::Value,
    /// ID of the realm this provider belongs to
    pub realm_id: Option<Uuid>,
    /// ID of the organization this provider belongs to
    pub organization_id: Option<Uuid>,
    /// Timestamp when the provider was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the provider was last updated
    pub updated_at: DateTime<Utc>,
}

/// Identity provider mapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProviderMapper {
    /// Unique identifier for the mapper
    pub id: Uuid,
    /// Name of the mapper
    pub name: String,
    /// Alias of the identity provider
    pub identity_provider_alias: String,
    /// Mapper provider identifier
    pub identity_provider_mapper: String,
    /// Mapper configuration as JSON
    pub config: serde_json::Value,
    /// ID of the realm this mapper belongs to
    pub realm_id: Option<Uuid>,
    /// Timestamp when the mapper was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the mapper was last updated
    pub updated_at: DateTime<Utc>,
}

/// User identity provider link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentityProviderLink {
    /// Unique identifier for the link
    pub id: Uuid,
    /// ID of the user
    pub user_id: Uuid,
    /// ID of the identity provider
    pub identity_provider_id: Uuid,
    /// External identifier from the provider
    pub external_id: String,
    /// External username from the provider
    pub external_username: Option<String>,
    /// External email from the identity provider
    pub external_email: Option<String>,
    /// Additional attributes from the identity provider
    pub external_attributes: Option<serde_json::Value>,
    /// Timestamp of the last login via this identity provider
    pub last_login_at: Option<DateTime<Utc>>,
    /// Timestamp when this link was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when this link was last updated
    pub updated_at: DateTime<Utc>,
}

/// Request to create a federated identity link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFederatedIdentityRequest {
    /// ID of the user in Authenc
    pub user_id: Uuid,
    /// ID of the identity provider
    pub identity_provider_id: Uuid,
    /// External user ID from the identity provider
    pub external_id: String,
    /// External username from the identity provider
    pub external_username: Option<String>,
    /// External email from the identity provider
    pub external_email: Option<String>,
    /// Additional attributes from the identity provider
    pub external_attributes: Option<serde_json::Value>,
}

/// JIT user provisioning request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JITUserProvisioningRequest {
    /// ID of the identity provider
    pub identity_provider_id: Uuid,
    /// External user ID from the identity provider
    pub external_id: String,
    /// External username from the identity provider
    pub external_username: Option<String>,
    /// External email from the identity provider
    pub external_email: Option<String>,
    /// User's first name from the identity provider
    pub first_name: Option<String>,
    /// User's last name from the identity provider
    pub last_name: Option<String>,
    /// Additional attributes from the identity provider
    pub external_attributes: Option<serde_json::Value>,
    /// ID of the realm where the user should be created
    pub realm_id: Uuid,
}

/// JIT user provisioning response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JITUserProvisioningResponse {
    /// The created or existing user
    pub user: User,
    /// Whether a new user was created (true) or existing user was found (false)
    pub created: bool,
    /// The federated identity link
    pub federated_identity: FederatedIdentity,
}

/// Federated identity (alias for UserIdentityProviderLink)
pub type FederatedIdentity = UserIdentityProviderLink;

/// Request to create a new user in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    /// Username for the new user
    pub username: String,
    /// Email address for the new user
    pub email: String,
    /// Kode satuan kerja
    pub satker_code: String,
    /// Password (will be hashed before storage)
    pub password: Option<String>,
    /// First name
    pub first_name: Option<String>,
    /// Last name
    pub last_name: Option<String>,
    /// NIP (Nomor Induk Pegawai)
    pub nip: Option<String>,
    /// Nama lengkap pegawai
    pub nama: Option<String>,
    /// Jabatan pegawai
    pub jabatan: Option<String>,
    /// Phone number
    pub phone_number: Option<String>,
    /// Realm ID
    pub realm_id: Option<Uuid>,
    /// Organization ID
    pub organization_id: Option<Uuid>,
    /// Roles to assign
    pub roles: Option<Vec<Uuid>>,
    /// Additional attributes
    pub attributes: Option<serde_json::Value>,
}

/// Request to update an existing user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub totp_secret: Option<String>,
    pub clear_totp_secret: Option<bool>,
    /// New username
    pub username: Option<String>,
    /// New email
    pub email: Option<String>,
    /// New kode satuan kerja
    pub satker_code: Option<String>,
    /// New first name
    pub first_name: Option<String>,
    /// New last name
    pub last_name: Option<String>,
    /// New NIP
    pub nip: Option<String>,
    /// New nama lengkap
    pub nama: Option<String>,
    /// New jabatan
    pub jabatan: Option<String>,
    /// New phone number
    pub phone_number: Option<String>,
    /// Enable/disable account
    pub enabled: Option<bool>,
    /// Email verification status
    pub email_verified: Option<bool>,
    /// Phone verification status
    pub phone_verified: Option<bool>,
    /// Force password change on next login
    pub require_password_change: Option<bool>,
    /// New password (plain text, will be hashed by service layer)
    pub password: Option<String>,
    /// Enable/disable MFA
    pub mfa_enabled: Option<bool>,
    /// Additional attributes
    pub attributes: Option<serde_json::Value>,
}

/// User response for API endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    /// Unique identifier
    pub id: Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// First name
    pub first_name: Option<String>,
    /// Last name
    pub last_name: Option<String>,
    /// Whether the account is enabled
    pub enabled: bool,
    /// Whether email is verified
    pub email_verified: bool,
    /// Realm ID
    pub realm_id: Option<Uuid>,
    /// Additional attributes
    pub attributes: Option<serde_json::Value>,
    /// When the user was created
    pub created_at: DateTime<Utc>,
    /// When the user was last updated
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            enabled: user.enabled,
            email_verified: user.email_verified,
            realm_id: user.realm_id,
            attributes: user.attributes,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

impl User {
    /// Create a new User with default values
    pub fn new(
        username: String,
        email: String,
        satker_code: String,
        password_hash: Option<String>,
        realm_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            email_verified: false,
            first_name: None,
            last_name: None,
            nip: None,
            nama: None,
            jabatan: None,
            satker_code,
            phone_number: None,
            phone_verified: false,
            password_hash,
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
            realm_id,
            organization_id: None,
            roles: Vec::new(),
            permissions: Vec::new(),
            session_data: None,
            security_context: SecurityContext {
                ip_address: None,
                user_agent: None,
                session_id: None,
                timestamp: now,
                risk_score: None,
                metadata: None,
            },
            attributes: None,
            enabled: true,
            federated: false,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            login_count: 0,
        }
    }
}

impl From<tokio_postgres::Row> for User {
    fn from(row: tokio_postgres::Row) -> Self {
        let satker_code: String = row
            .try_get("satker_code")
            .unwrap_or_else(|_| "UNKNOWN".to_string());
        Self {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            email_verified: row.get("email_verified"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            nip: row.try_get("nip").ok(),
            nama: row.try_get("nama").ok(),
            jabatan: row.try_get("jabatan").ok(),
            satker_code,
            phone_number: row.get("phone_number"),
            phone_verified: row.get("phone_verified"),
            password_hash: row.get("password_hash"),
            totp_secret: row.get("totp_secret"),
            totp_backup_codes: row.get("totp_backup_codes"),
            mfa_enabled: row.try_get("mfa_enabled").unwrap_or(false),
            mfa_setup_at: row.try_get("mfa_setup_at").ok().flatten(),
            mfa_last_used: row.try_get("mfa_last_used").ok().flatten(),
            webauthn_enabled: row.get("webauthn_enabled"),
            account_locked: row.get("account_locked"),
            account_locked_until: row.get("account_locked_until"),
            failed_login_attempts: row.get("failed_login_attempts"),
            last_login_at: row.get("last_login_at"),
            last_failed_login_at: row.get("last_failed_login_at"),
            password_changed_at: row.get("password_changed_at"),
            password_expires_at: row.get("password_expires_at"),
            require_password_change: row.get("require_password_change"),
            realm_id: row.get("realm_id"),
            organization_id: row.get("organization_id"),
            roles: Vec::new(),
            permissions: Vec::new(),
            session_data: None,
            security_context: SecurityContext {
                ip_address: None,
                user_agent: None,
                session_id: None,
                timestamp: Utc::now(),
                risk_score: None,
                metadata: None,
            },
            attributes: row.try_get("attributes").ok(),
            enabled: row.get("enabled"),
            federated: row.get("federated"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
            login_count: row.try_get("login_count").unwrap_or(0),
        }
    }
}
