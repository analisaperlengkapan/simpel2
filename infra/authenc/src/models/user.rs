use crate::error::AuthencError;
use crate::utils::validation::{
    email_validator, email_validator_optional, nip_validator_optional, phone_validator_optional,
    sanitize_email, sanitize_satker_code, sanitize_string, sanitize_username,
    satker_code_validator, satker_code_validator_optional, username_validator,
    username_validator_optional,
};
use chrono::{DateTime, Datelike, Timelike, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Administrative levels in the Attorney General's Office hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdminLevel {
    AdminSatker(String),
    /// Admin for a specific wilayah (regional area)
    AdminWilayah(String),
    /// Admin for Eselon I level at central office
    AdminEselonI,
    /// Admin at central/national level
    AdminPusat,
}

impl AdminLevel {
    /// Check if this admin level can manage the given admin level
    /// Admin for a specific satker (satuan kerja)
    pub fn can_manage(&self, other: &AdminLevel) -> bool {
        match (self, other) {
            (AdminLevel::AdminPusat, _) => true,
            (AdminLevel::AdminEselonI, AdminLevel::AdminSatker(_)) => true,
            (AdminLevel::AdminEselonI, AdminLevel::AdminWilayah(_)) => true,
            (AdminLevel::AdminWilayah(region1), AdminLevel::AdminSatker(satker)) => {
                // Check if satker belongs to this region (simplified check)
                satker.starts_with(region1)
            }
            (AdminLevel::AdminSatker(satker1), AdminLevel::AdminSatker(satker2)) => {
                satker1 == satker2
            }
            _ => false,
        }
    }

    /// Get the scope of this admin level
    pub fn get_scope(&self) -> RoleScope {
        match self {
            AdminLevel::AdminSatker(satker) => RoleScope::Satker(satker.clone()),
            AdminLevel::AdminWilayah(wilayah) => RoleScope::Wilayah(wilayah.clone()),
            AdminLevel::AdminEselonI => RoleScope::Pusat,
            AdminLevel::AdminPusat => RoleScope::Pusat,
        }
    }
}

/// Role scope defining the organizational level where the role applies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoleScope {
    Satker(String),
    /// Role applies to a specific wilayah (regional area)
    Wilayah(String),
    /// Role applies at central/national level
    Pusat,
}

impl RoleScope {
    /// Check if this scope includes the given satker
    /// Role applies to a specific satker (satuan kerja)
    pub fn includes_satker(&self, satker_code: &str) -> bool {
        match self {
            RoleScope::Satker(scope_satker) => scope_satker == satker_code,
            RoleScope::Wilayah(wilayah) => satker_code.starts_with(wilayah),
            RoleScope::Pusat => true,
        }
    }

    /// Check if this scope can access resources from another scope
    pub fn can_access(&self, other: &RoleScope) -> bool {
        match (self, other) {
            (RoleScope::Pusat, _) => true,
            (RoleScope::Wilayah(w1), RoleScope::Wilayah(w2)) => w1 == w2,
            (RoleScope::Wilayah(w), RoleScope::Satker(s)) => s.starts_with(w),
            (RoleScope::Satker(s1), RoleScope::Satker(s2)) => s1 == s2,
            _ => false,
        }
    }
}

/// Access level for secreton operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessLevel {
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
    /// Read-only access to secrets
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
    /// Secreton access policy for this user
    pub secreton_access_policy: SecretonAccessPolicy,
    /// Security context for the user
    pub security_context: SecurityContext,
    /// Additional user attributes as JSON
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
    /// Password-based authentication
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
    /// Scope where this role applies (Satker, Wilayah, or Pusat)
    pub scope: RoleScope,
    /// Permissions granted by this role
    pub permissions: Vec<Permission>,
    /// Admin level that manages this role
    pub managed_by: AdminLevel,
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
    pub fn can_assign_to_satker(&self, satker_code: &str) -> bool {
        self.scope.includes_satker(satker_code)
    }

    /// Check if this role grants access to resources in the given scope
    pub fn grants_access_to(&self, target_scope: &RoleScope) -> bool {
        self.scope.can_access(target_scope)
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
    /// Scope where this permission applies
    pub scope: RoleScope,
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
        if !self.scope.includes_satker(satker_code) {
            return false;
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

/// Group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// Unique identifier for the group
    pub id: Uuid,
    /// Name of the group
    pub name: String,
    /// Description of the group
    pub description: Option<String>,
    /// Path of the group in the hierarchy
    pub path: String,
    /// ID of the parent group
    pub parent_id: Option<Uuid>,
    /// ID of the realm this group belongs to
    pub realm_id: Option<Uuid>,
    /// Additional group attributes as JSON
    pub attributes: Option<serde_json::Value>,
    /// Timestamp when the group was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the group was last updated
    pub updated_at: DateTime<Utc>,
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
    /// Execution is required
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
    /// Token from the provider
    pub token: Option<String>,
    /// Timestamp when the link was created
    pub linked_at: DateTime<Utc>,
    /// Timestamp of the last login via this provider
    pub last_login_at: Option<DateTime<Utc>>,
}

/// Required action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredAction {
    /// Unique identifier for the required action
    pub id: Uuid,
    /// Alias name for the action
    pub alias: String,
    /// Display name for the action
    pub name: String,
    /// Description of the required action
    pub description: Option<String>,
    /// Provider identifier for the action
    pub provider_id: String,
    /// Whether the action is enabled
    pub enabled: bool,
    /// Whether this is the default action
    pub default_action: bool,
    /// Priority order of the action
    pub priority: i32,
    /// Action configuration as JSON
    pub config: Option<serde_json::Value>,
    /// ID of the realm this action belongs to
    pub realm_id: Option<Uuid>,
    /// Timestamp when the action was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the action was last updated
    pub updated_at: DateTime<Utc>,
}

/// User required action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRequiredAction {
    /// Unique identifier for the user-action assignment
    pub id: Uuid,
    /// ID of the user
    pub user_id: Uuid,
    /// ID of the required action
    pub required_action_id: Uuid,
    /// Timestamp when the action was assigned
    pub created_at: DateTime<Utc>,
    /// Timestamp when the action expires
    pub expires_at: Option<DateTime<Utc>>,
}

/// User creation request for SIMKARI
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    /// Username for the new user
    #[garde(custom(username_validator))]
    pub username: String,

    /// Email address for the new user
    #[garde(custom(email_validator))]
    pub email: String,

    /// Kode satuan kerja (required)
    #[garde(custom(satker_code_validator))]
    pub satker_code: String,

    /// Password for the new user (optional, can be set later)
    #[garde(length(min = 8, max = 128))]
    pub password: Option<String>,

    /// First name of the user
    #[garde(length(min = 1, max = 100))]
    pub first_name: Option<String>,

    /// Last name of the user
    #[garde(length(min = 1, max = 100))]
    pub last_name: Option<String>,

    /// NIP (Nomor Induk Pegawai)
    #[garde(custom(nip_validator_optional))]
    pub nip: Option<String>,

    /// Nama lengkap pegawai
    #[garde(length(min = 1, max = 200))]
    pub nama: Option<String>,

    /// Jabatan pegawai
    #[garde(length(min = 1, max = 200))]
    pub jabatan: Option<String>,

    /// Phone number of the user
    #[garde(custom(phone_validator_optional))]
    pub phone_number: Option<String>,

    /// ID of the realm to create the user in
    #[garde(skip)]
    pub realm_id: Option<Uuid>,

    /// ID of the organization to assign the user to
    #[garde(skip)]
    pub organization_id: Option<Uuid>,

    /// Initial roles to assign to the user
    #[garde(length(max = 50))]
    pub roles: Option<Vec<Uuid>>,

    /// Secreton access policy for the user
    #[garde(skip)]
    pub secreton_access_policy: Option<SecretonAccessPolicy>,

    /// Additional user attributes as JSON
    #[garde(skip)]
    pub attributes: Option<serde_json::Value>,
}

impl CreateUserRequest {
    /// Sanitize the request fields
    pub fn sanitize(&mut self) {
        self.username = sanitize_username(&self.username);
        self.email = sanitize_email(&self.email);
        self.satker_code = sanitize_satker_code(&self.satker_code);

        if let Some(ref mut first_name) = self.first_name {
            *first_name = sanitize_string(first_name, 100);
        }
        if let Some(ref mut last_name) = self.last_name {
            *last_name = sanitize_string(last_name, 100);
        }
        if let Some(ref mut nama) = self.nama {
            *nama = sanitize_string(nama, 200);
        }
        if let Some(ref mut jabatan) = self.jabatan {
            *jabatan = sanitize_string(jabatan, 200);
        }
    }
}

/// User update request for SIMKARI
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    /// New username for the user
    #[garde(custom(username_validator_optional))]
    pub username: Option<String>,

    /// New email address for the user
    #[garde(custom(email_validator_optional))]
    pub email: Option<String>,

    /// New satker code for the user
    #[garde(custom(satker_code_validator_optional))]
    pub satker_code: Option<String>,

    /// New first name for the user
    #[garde(length(min = 1, max = 100))]
    pub first_name: Option<String>,

    /// New last name for the user
    #[garde(length(min = 1, max = 100))]
    pub last_name: Option<String>,

    /// New NIP for the user
    #[garde(custom(nip_validator_optional))]
    pub nip: Option<String>,

    /// New nama for the user
    #[garde(length(min = 1, max = 200))]
    pub nama: Option<String>,

    /// New jabatan for the user
    #[garde(length(min = 1, max = 200))]
    pub jabatan: Option<String>,

    /// New phone number for the user
    #[garde(custom(phone_validator_optional))]
    pub phone_number: Option<String>,

    /// Whether the user account is enabled
    #[garde(skip)]
    pub enabled: Option<bool>,

    /// Whether the email address has been verified
    #[garde(skip)]
    pub email_verified: Option<bool>,

    /// Whether the phone number has been verified
    #[garde(skip)]
    pub phone_verified: Option<bool>,

    /// Whether the user must change their password on next login
    #[garde(skip)]
    pub require_password_change: Option<bool>,

    /// Updated secreton access policy
    #[garde(skip)]
    pub secreton_access_policy: Option<SecretonAccessPolicy>,

    /// Additional user attributes as JSON
    #[garde(skip)]
    pub attributes: Option<serde_json::Value>,
}

/// User response (without sensitive data) for SIMKARI
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// Unique identifier for the user
    pub id: Uuid,
    /// Username of the user
    pub username: String,
    /// Email address of the user
    pub email: String,
    /// Whether the email address has been verified
    pub email_verified: bool,
    /// First name of the user
    pub first_name: Option<String>,
    /// Last name of the user
    pub last_name: Option<String>,
    /// NIP (Nomor Induk Pegawai)
    pub nip: Option<String>,
    /// Nama lengkap pegawai
    pub nama: Option<String>,
    /// Jabatan pegawai
    pub jabatan: Option<String>,
    /// Kode satuan kerja
    pub satker_code: String,
    /// Phone number of the user
    pub phone_number: Option<String>,
    /// Whether the phone number has been verified
    pub phone_verified: bool,
    /// Whether WebAuthn is enabled for this user
    pub webauthn_enabled: bool,
    /// Whether the account is currently locked
    pub account_locked: bool,
    /// Timestamp of the last successful login
    pub last_login_at: Option<DateTime<Utc>>,
    /// ID of the realm this user belongs to
    pub realm_id: Option<Uuid>,
    /// ID of the organization this user belongs to
    pub organization_id: Option<Uuid>,
    /// Roles assigned to the user
    pub roles: Vec<String>,
    /// Secreton access level
    pub secreton_access_level: AccessLevel,
    /// Whether the user account is enabled
    pub enabled: bool,
    /// Timestamp when the user was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the user was last updated
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with default values for SIMKARI
    pub fn new(
        username: String,
        email: String,
        satker_code: String,
        password_hash: Option<String>,
        realm_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        let satker_code_clone = satker_code.clone();
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
            secreton_access_policy: SecretonAccessPolicy {
                allowed_satker_secrets: vec![satker_code_clone],
                access_level: AccessLevel::ReadOnly,
                time_restrictions: None,
                audit_required: true,
                rate_limit: Some(100),
                allowed_paths: None,
                denied_paths: None,
            },
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

    /// Check if user has permission for a specific action on a resource
    pub fn has_permission(&self, resource_type: &str, resource_id: &str, action: &str) -> bool {
        self.permissions.iter().any(|p| {
            p.applies_to_resource(resource_type, resource_id, &self.satker_code)
                && p.allows_action(action)
                && p.active
        })
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role_name: &str) -> bool {
        self.roles.iter().any(|r| r.name == role_name && r.active)
    }

    /// Get all permissions for a specific resource type
    pub fn get_permissions_for_resource(&self, resource_type: &str) -> Vec<&Permission> {
        self.permissions
            .iter()
            .filter(|p| p.resource_type == resource_type && p.active)
            .collect()
    }

    /// Check if user can access secreton path
    pub fn can_access_secreton_path(&self, path: &str) -> bool {
        self.secreton_access_policy
            .can_access_path(path, &self.satker_code)
            && self.secreton_access_policy.is_time_allowed(&Utc::now())
    }

    /// Update security context
    pub fn update_security_context(&mut self, context: SecurityContext) {
        self.security_context = context;
        self.updated_at = Utc::now();
    }

    /// Add role to user
    pub fn add_role(&mut self, role: Role) {
        if !self.roles.iter().any(|r| r.id == role.id) {
            // Add role permissions to user permissions
            for permission in &role.permissions {
                if !self.permissions.iter().any(|p| p.id == permission.id) {
                    self.permissions.push(permission.clone());
                }
            }
            self.roles.push(role);
            self.updated_at = Utc::now();
        }
    }

    /// Remove role from user
    pub fn remove_role(&mut self, role_id: Uuid) {
        if let Some(pos) = self.roles.iter().position(|r| r.id == role_id) {
            let removed_role = self.roles.remove(pos);

            // Remove permissions that were only granted by this role
            self.permissions.retain(|p| {
                self.roles
                    .iter()
                    .any(|r| r.permissions.iter().any(|rp| rp.id == p.id))
            });

            self.updated_at = Utc::now();
        }
    }

    /// Update secreton access policy
    pub fn update_secreton_access_policy(&mut self, policy: SecretonAccessPolicy) {
        self.secreton_access_policy = policy;
        self.updated_at = Utc::now();
    }

    /// Check if user is active (enabled and not deleted)
    pub fn is_active(&self) -> bool {
        self.enabled && self.deleted_at.is_none() && !self.account_locked
    }

    /// Check if user account is locked
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.account_locked_until {
            self.account_locked && Utc::now() < locked_until
        } else {
            self.account_locked
        }
    }

    /// Soft delete the user
    pub fn delete(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Update user fields
    pub fn update(&mut self, request: UpdateUserRequest) {
        if let Some(username) = request.username {
            self.username = username;
        }
        if let Some(email) = request.email {
            self.email = email;
        }
        if let Some(satker_code) = request.satker_code {
            self.satker_code = satker_code;
        }
        if let Some(first_name) = request.first_name {
            self.first_name = Some(first_name);
        }
        if let Some(last_name) = request.last_name {
            self.last_name = Some(last_name);
        }
        if let Some(nip) = request.nip {
            self.nip = Some(nip);
        }
        if let Some(nama) = request.nama {
            self.nama = Some(nama);
        }
        if let Some(jabatan) = request.jabatan {
            self.jabatan = Some(jabatan);
        }
        if let Some(phone_number) = request.phone_number {
            self.phone_number = Some(phone_number);
        }
        if let Some(enabled) = request.enabled {
            self.enabled = enabled;
        }
        if let Some(email_verified) = request.email_verified {
            self.email_verified = email_verified;
        }
        if let Some(phone_verified) = request.phone_verified {
            self.phone_verified = phone_verified;
        }
        if let Some(require_password_change) = request.require_password_change {
            self.require_password_change = require_password_change;
        }
        if let Some(secreton_access_policy) = request.secreton_access_policy {
            self.secreton_access_policy = secreton_access_policy;
        }
        if let Some(attributes) = request.attributes {
            self.attributes = Some(attributes);
        }
        self.updated_at = Utc::now();
    }

    /// Record a successful login
    pub fn record_login(&mut self) {
        self.last_login_at = Some(Utc::now());
        self.failed_login_attempts = 0;
        self.account_locked = false;
        self.account_locked_until = None;
        self.updated_at = Utc::now();
    }

    /// Record a failed login attempt
    pub fn record_failed_login(&mut self) {
        self.failed_login_attempts += 1;
        self.last_failed_login_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Lock the user account
    pub fn lock_account(&mut self, until: Option<DateTime<Utc>>) {
        self.account_locked = true;
        self.account_locked_until = until;
        self.updated_at = Utc::now();
    }

    /// Unlock the user account
    pub fn unlock_account(&mut self) {
        self.account_locked = false;
        self.account_locked_until = None;
        self.failed_login_attempts = 0;
        self.updated_at = Utc::now();
    }

    /// Update password
    pub fn update_password(&mut self, new_hash: String) {
        self.password_hash = Some(new_hash);
        self.password_changed_at = Some(Utc::now());
        self.require_password_change = false;
        self.updated_at = Utc::now();
    }

    /// Enable WebAuthn
    pub fn enable_webauthn(&mut self) {
        self.webauthn_enabled = true;
        self.updated_at = Utc::now();
    }

    /// Disable WebAuthn
    pub fn disable_webauthn(&mut self) {
        self.webauthn_enabled = false;
        self.updated_at = Utc::now();
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => self.username.clone(),
        }
    }
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            email_verified: user.email_verified,
            first_name: user.first_name,
            last_name: user.last_name,
            nip: user.nip,
            nama: user.nama,
            jabatan: user.jabatan,
            satker_code: user.satker_code,
            phone_number: user.phone_number,
            phone_verified: user.phone_verified,
            webauthn_enabled: user.webauthn_enabled,
            account_locked: user.account_locked,
            last_login_at: user.last_login_at,
            realm_id: user.realm_id,
            organization_id: user.organization_id,
            roles: user.roles.iter().map(|r| r.name.clone()).collect(),
            secreton_access_level: user.secreton_access_policy.access_level,
            enabled: user.enabled,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

impl TryFrom<tokio_postgres::Row> for User {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self, Self::Error> {
        let now = Utc::now();
        let satker_code: String = row.try_get("satker_code")?;

        Ok(User {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            email_verified: row.try_get("email_verified")?,
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
            nip: row.try_get("nip")?,
            nama: row.try_get("nama")?,
            jabatan: row.try_get("jabatan")?,
            satker_code: satker_code.clone(),
            phone_number: row.try_get("phone_number")?,
            phone_verified: row.try_get("phone_verified")?,
            password_hash: row.try_get("password_hash")?,
            totp_secret: row.try_get("totp_secret")?,
            totp_backup_codes: row.try_get("totp_backup_codes")?,
            mfa_enabled: row.try_get("mfa_enabled").unwrap_or(false),
            mfa_setup_at: row.try_get("mfa_setup_at").unwrap_or(None),
            mfa_last_used: row.try_get("mfa_last_used").unwrap_or(None),
            webauthn_enabled: row.try_get("webauthn_enabled")?,
            account_locked: row.try_get("account_locked")?,
            account_locked_until: row.try_get("account_locked_until")?,
            failed_login_attempts: row.try_get("failed_login_attempts")?,
            last_login_at: row.try_get("last_login_at")?,
            last_failed_login_at: row.try_get("last_failed_login_at")?,
            password_changed_at: row.try_get("password_changed_at")?,
            password_expires_at: row.try_get("password_expires_at")?,
            require_password_change: row.try_get("require_password_change")?,
            realm_id: row.try_get("realm_id")?,
            organization_id: row.try_get("organization_id")?,
            roles: Vec::new(),       // Roles would be loaded separately
            permissions: Vec::new(), // Permissions would be loaded separately
            session_data: {
                let json_str: Option<String> = row.try_get("session_data").ok().flatten();
                json_str.and_then(|s| serde_json::from_str(&s).ok())
            },
            secreton_access_policy: {
                let policy_str: Option<String> =
                    row.try_get("secreton_access_policy").ok().flatten();
                policy_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| SecretonAccessPolicy {
                        allowed_satker_secrets: vec![satker_code.clone()],
                        access_level: AccessLevel::ReadOnly,
                        time_restrictions: None,
                        audit_required: true,
                        rate_limit: Some(100),
                        allowed_paths: None,
                        denied_paths: None,
                    })
            },
            security_context: {
                let context_str: Option<String> = row.try_get("security_context").ok().flatten();
                context_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| SecurityContext {
                        ip_address: None,
                        user_agent: None,
                        session_id: None,
                        timestamp: now,
                        risk_score: None,
                        metadata: None,
                    })
            },
            attributes: {
                let json_str: Option<String> = row.try_get("attributes")?;
                json_str.and_then(|s| serde_json::from_str(&s).ok())
            },
            enabled: row.try_get("enabled")?,
            federated: row.try_get("federated")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,
            login_count: row.try_get("login_count")?,
        })
    }
}

/// Federated identity linking a user to an external identity provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedIdentity {
    /// Unique identifier for the federated identity link
    pub id: Uuid,
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
