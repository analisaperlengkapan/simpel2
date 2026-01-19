use crate::models::user::{AccessLevel, AdminLevel, RoleScope};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Authentication token types for SIMKARI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    AccessToken,
    /// Refresh token for obtaining new access tokens
    RefreshToken,
    /// ID token containing user identity information
    IdToken,
    /// Token for email verification
    VerificationToken,
    /// Token for password reset
    PasswordResetToken,
    /// Admin token for hierarchical operations
    AdminToken,
    /// Service token for inter-service communication
    ServiceToken,
}

/// Flexible scope system for SIMKARI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Access token for API authorization
pub struct FlexibleScope {
    /// Base scope (e.g., "read", "write", "admin")
    pub base: String,
    /// Resource type (e.g., "secrets", "users", "audit")
    pub resource: String,
    /// Organizational scope (Satker, Wilayah, Pusat)
    pub org_scope: RoleScope,
    /// Additional constraints
    pub constraints: Option<serde_json::Value>,
}

impl FlexibleScope {
    /// Create a new flexible scope
    pub fn new(base: &str, resource: &str, org_scope: RoleScope) -> Self {
        Self {
            base: base.to_string(),
            resource: resource.to_string(),
            org_scope,
            constraints: None,
        }
    }

    /// Check if this scope allows access to a resource in a specific satker
    pub fn allows_access(&self, action: &str, resource_type: &str, satker_code: &str) -> bool {
        // Check base scope
        if self.base != action && self.base != "admin" && self.base != "*" {
            return false;
        }

        // Check resource type
        if self.resource != resource_type && self.resource != "*" {
            return false;
        }

        // Check organizational scope
        self.org_scope.includes_satker(satker_code)
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.base,
            self.resource,
            match &self.org_scope {
                RoleScope::Satker(s) => format!("satker:{}", s),
                RoleScope::Wilayah(w) => format!("wilayah:{}", w),
                RoleScope::Pusat => "pusat".to_string(),
            }
        )
    }
}

/// Role-based permissions for Secreton access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretonPermissions {
    /// Access level for secreton operations
    pub access_level: AccessLevel,
    /// Allowed operations based on roles
    pub allowed_operations: Vec<String>,
    /// Scopes where operations are allowed
    pub allowed_scopes: Vec<RoleScope>,
    /// Admin level for hierarchical operations
    pub admin_level: Option<AdminLevel>,
    /// Rate limiting (operations per hour)
    pub rate_limit: Option<u32>,
    /// Time-based restrictions
    pub time_restrictions: Option<TimeRestrictions>,
    /// Audit requirements
    pub audit_required: bool,
}

impl SecretonPermissions {
    /// Create default permissions for a user
    pub fn default_for_satker(satker_code: &str) -> Self {
        Self {
            access_level: AccessLevel::ReadOnly,
            allowed_operations: vec!["read".to_string()],
            allowed_scopes: vec![RoleScope::Satker(satker_code.to_string())],
            admin_level: None,
            rate_limit: Some(100),
            time_restrictions: None,
            audit_required: true,
        }
    }

    /// Create admin permissions
    pub fn admin_permissions(admin_level: AdminLevel) -> Self {
        let allowed_scopes = vec![admin_level.get_scope()];
        Self {
            access_level: AccessLevel::Admin,
            allowed_operations: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
                "admin".to_string(),
            ],
            allowed_scopes,
            admin_level: Some(admin_level),
            rate_limit: Some(1000),
            time_restrictions: None,
            audit_required: true,
        }
    }

    /// Check if operation is allowed in the given scope
    pub fn can_perform_operation(&self, operation: &str, target_scope: &RoleScope) -> bool {
        // Check if operation is allowed
        if !self.allowed_operations.contains(&operation.to_string())
            && !self.allowed_operations.contains(&"*".to_string())
        {
            return false;
        }

        // Check if scope is allowed
        self.allowed_scopes
            .iter()
            .any(|scope| scope.can_access(target_scope))
    }

    /// Check if admin operation is allowed
    pub fn can_perform_admin_operation(&self, target_admin_level: &AdminLevel) -> bool {
        if let Some(admin_level) = &self.admin_level {
            admin_level.can_manage(target_admin_level)
        } else {
            false
        }
    }
}

/// Time-based restrictions for token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23)
    pub end_hour: u8,
    /// Allowed days of week (0=Sunday, 6=Saturday)
    pub allowed_days: Vec<u8>,
    /// Timezone for calculations
    pub timezone: String,
}

/// Enhanced token structure for SIMKARI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Unique identifier for the token
    pub id: Uuid,
    /// Type of the token
    pub token_type: TokenType,
    /// Actual token value (hashed or encrypted)
    pub value: String,
    /// ID of the user this token belongs to
    pub user_id: Option<Uuid>,
    /// Satker code of the user/client
    pub satker_code: String,
    /// ID of the client this token is issued to
    pub client_id: Option<String>,
    /// Flexible scopes for the token
    pub scopes: Vec<FlexibleScope>,
    /// Secreton permissions for this token
    pub secreton_permissions: SecretonPermissions,
    /// Timestamp when the token expires
    pub expires_at: DateTime<Utc>,
    /// Timestamp when the token was created
    pub created_at: DateTime<Utc>,
    /// Whether the token has been revoked
    pub revoked: bool,
    /// Whether the token has been used (for one-time tokens)
    pub used: bool,
    /// IP address where token was issued
    pub issued_ip: Option<String>,
    /// Last IP address where token was used
    pub last_used_ip: Option<String>,
    /// Timestamp when token was last used
    pub last_used_at: Option<DateTime<Utc>>,
    /// Number of times token has been used
    pub usage_count: i32,
    /// Maximum number of times token can be used (None = unlimited)
    pub max_usage: Option<i32>,
}

/// Token creation request for SIMKARI
#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    /// Type of token to create
    pub token_type: TokenType,
    /// ID of the user for whom the token is created
    pub user_id: Option<Uuid>,
    /// Satker code for the token
    pub satker_code: String,
    /// ID of the client for whom the token is created
    pub client_id: Option<String>,
    /// Flexible scopes for the token
    pub scopes: Vec<FlexibleScope>,
    /// Secreton permissions for the token
    pub secreton_permissions: Option<SecretonPermissions>,
    /// Token lifetime in seconds
    pub expires_in: i64,
    /// IP address where token is being issued
    pub issued_ip: Option<String>,
    /// Maximum number of times token can be used
    pub max_usage: Option<i32>,
}

/// Token response (safe for client) for SIMKARI
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// Access token value
    pub access_token: String,
    /// Type of the token (usually "Bearer")
    pub token_type: String,
    /// Token lifetime in seconds
    pub expires_in: i64,
    /// Refresh token value (if issued)
    pub refresh_token: Option<String>,
    /// Scopes granted to the token
    pub scopes: Vec<String>,
    /// Satker code associated with the token
    pub satker_code: String,
    /// Access level for secreton operations
    pub secreton_access_level: AccessLevel,
    /// Whether audit is required for operations
    pub audit_required: bool,
}

/// Enhanced JWT token claims for SIMKARI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject identifier (user ID)
    pub sub: String,
    /// Audience (client ID)
    pub aud: String,
    /// Issuer of the token
    pub iss: String,
    /// Expiration timestamp
    pub exp: i64,
    /// Issued at timestamp
    pub iat: i64,
    /// JWT unique identifier
    pub jti: String,
    /// Satker code
    pub satker_code: String,
    /// NIP (Nomor Induk Pegawai) if available
    pub nip: Option<String>,
    /// Flexible scopes
    pub scopes: Vec<String>,
    /// Admin level if applicable
    pub admin_level: Option<String>,
    /// Secreton access level
    pub secreton_access_level: String,
    /// Whether audit is required
    pub audit_required: bool,
    /// Roles assigned to the user
    pub roles: Vec<String>,
}

impl Token {
    /// Create a new token for SIMKARI
    pub fn new(request: CreateTokenRequest, value: String) -> Self {
        let now = Utc::now();
        let secreton_permissions = request
            .secreton_permissions
            .unwrap_or_else(|| SecretonPermissions::default_for_satker(&request.satker_code));

        Self {
            id: Uuid::new_v4(),
            token_type: request.token_type,
            value,
            user_id: request.user_id,
            satker_code: request.satker_code,
            client_id: request.client_id,
            scopes: request.scopes,
            secreton_permissions,
            expires_at: now + chrono::Duration::seconds(request.expires_in),
            created_at: now,
            revoked: false,
            used: false,
            issued_ip: request.issued_ip,
            last_used_ip: None,
            last_used_at: None,
            usage_count: 0,
            max_usage: request.max_usage,
        }
    }

    /// Check if token is valid (not expired, revoked, or used)
    pub fn is_valid(&self) -> bool {
        if self.revoked || Utc::now() >= self.expires_at {
            return false;
        }

        // Check usage limits
        if let Some(max_usage) = self.max_usage {
            if self.usage_count >= max_usage {
                return false;
            }
        }

        // Check time restrictions
        if let Some(restrictions) = &self.secreton_permissions.time_restrictions {
            let now = Utc::now();
            let hour = now.hour() as u8;
            let weekday = now.weekday().num_days_from_sunday() as u8;

            if hour < restrictions.start_hour || hour > restrictions.end_hour {
                return false;
            }

            if !restrictions.allowed_days.contains(&weekday) {
                return false;
            }
        }

        true
    }

    /// Revoke the token
    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    /// Mark token as used and update usage statistics
    pub fn mark_used(&mut self, ip_address: Option<String>) {
        self.usage_count += 1;
        self.last_used_at = Some(Utc::now());
        if let Some(ip) = ip_address {
            self.last_used_ip = Some(ip);
        }

        // Mark as used if it's a one-time token
        if matches!(
            self.token_type,
            TokenType::VerificationToken | TokenType::PasswordResetToken
        ) {
            self.used = true;
        }
    }

    /// Check in is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// Get remaining time to live in seconds
    pub fn ttl(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds()
    }

    /// Check if token allows access to a specific resource
    pub fn allows_access(&self, action: &str, resource_type: &str, target_satker: &str) -> bool {
        if !self.is_valid() {
            return false;
        }

        // Check scopes
        self.scopes
            .iter()
            .any(|scope| scope.allows_access(action, resource_type, target_satker))
    }

    /// Check if token allows secreton operation
    pub fn allows_secreton_operation(&self, operation: &str, target_scope: &RoleScope) -> bool {
        if !self.is_valid() {
            return false;
        }

        self.secreton_permissions
            .can_perform_operation(operation, target_scope)
    }

    /// Check if token allows admin operation
    pub fn allows_admin_operation(&self, target_admin_level: &AdminLevel) -> bool {
        if !self.is_valid() {
            return false;
        }

        self.secreton_permissions
            .can_perform_admin_operation(target_admin_level)
    }

    /// Get scope strings for response
    pub fn get_scope_strings(&self) -> Vec<String> {
        self.scopes.iter().map(|s| s.to_string()).collect()
    }

    /// Check if usage limit is reached
    pub fn is_usage_limit_reached(&self) -> bool {
        if let Some(max_usage) = self.max_usage {
            self.usage_count >= max_usage
        } else {
            false
        }
    }

    /// Get usage statistics
    pub fn get_usage_stats(&self) -> (i32, Option<i32>) {
        (self.usage_count, self.max_usage)
    }
}
/// Admin token creation request for hierarchical operations
#[derive(Debug, Deserialize)]
pub struct CreateAdminTokenRequest {
    /// Admin level for the token
    pub admin_level: AdminLevel,
    /// ID of the admin user
    pub admin_user_id: Uuid,
    /// Satker code of the admin
    pub satker_code: String,
    /// Specific operations allowed
    pub allowed_operations: Vec<String>,
    /// Token lifetime in seconds
    pub expires_in: i64,
    /// IP address where token is being issued
    pub issued_ip: Option<String>,
}

impl CreateAdminTokenRequest {
    /// Convert to standard CreateTokenRequest
    pub fn to_create_token_request(self) -> CreateTokenRequest {
        let scopes = vec![FlexibleScope::new(
            "admin",
            "*",
            self.admin_level.get_scope(),
        )];

        let secreton_permissions = SecretonPermissions::admin_permissions(self.admin_level.clone());

        CreateTokenRequest {
            token_type: TokenType::AdminToken,
            user_id: Some(self.admin_user_id),
            satker_code: self.satker_code,
            client_id: None,
            scopes,
            secreton_permissions: Some(secreton_permissions),
            expires_in: self.expires_in,
            issued_ip: self.issued_ip,
            max_usage: None, // Admin tokens typically have unlimited usage
        }
    }
}

/// Service token creation request for inter-service communication
#[derive(Debug, Deserialize)]
pub struct CreateServiceTokenRequest {
    /// Service identifier
    pub service_id: String,
    /// Satker code of the service
    pub satker_code: String,
    /// Allowed operations for the service
    pub allowed_operations: Vec<String>,
    /// Allowed resource types
    pub allowed_resources: Vec<String>,
    /// Token lifetime in seconds
    pub expires_in: i64,
    /// Maximum number of uses
    pub max_usage: Option<i32>,
}

impl CreateServiceTokenRequest {
    /// Convert to standard CreateTokenRequest
    pub fn to_create_token_request(self) -> CreateTokenRequest {
        let mut scopes = Vec::new();

        for operation in &self.allowed_operations {
            for resource in &self.allowed_resources {
                scopes.push(FlexibleScope::new(
                    operation,
                    resource,
                    RoleScope::Satker(self.satker_code.clone()),
                ));
            }
        }

        let secreton_permissions = SecretonPermissions {
            access_level: AccessLevel::ReadWrite,
            allowed_operations: self.allowed_operations.clone(),
            allowed_scopes: vec![RoleScope::Satker(self.satker_code.clone())],
            admin_level: None,
            rate_limit: Some(10000), // Higher limit for services
            time_restrictions: None,
            audit_required: true,
        };

        CreateTokenRequest {
            token_type: TokenType::ServiceToken,
            user_id: None,
            satker_code: self.satker_code,
            client_id: Some(self.service_id),
            scopes,
            secreton_permissions: Some(secreton_permissions),
            expires_in: self.expires_in,
            issued_ip: None,
            max_usage: self.max_usage,
        }
    }
}

/// Token validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationResult {
    /// Whether the token is valid
    pub valid: bool,
    /// User ID if token is valid
    pub user_id: Option<Uuid>,
    /// Satker code
    pub satker_code: Option<String>,
    /// Scopes granted by the token
    pub scopes: Vec<String>,
    /// Admin level if applicable
    pub admin_level: Option<AdminLevel>,
    /// Secreton access level
    pub secreton_access_level: Option<AccessLevel>,
    /// Error message if token is invalid
    pub error: Option<String>,
    /// Remaining TTL in seconds
    pub ttl: Option<i64>,
    /// Usage statistics
    pub usage_count: Option<i32>,
    /// Maximum usage allowed
    pub max_usage: Option<i32>,
}

impl From<&Token> for TokenValidationResult {
    fn from(token: &Token) -> Self {
        if token.is_valid() {
            Self {
                valid: true,
                user_id: token.user_id,
                satker_code: Some(token.satker_code.clone()),
                scopes: token.get_scope_strings(),
                admin_level: token.secreton_permissions.admin_level.clone(),
                secreton_access_level: Some(token.secreton_permissions.access_level.clone()),
                error: None,
                ttl: Some(token.ttl()),
                usage_count: Some(token.usage_count),
                max_usage: token.max_usage,
            }
        } else {
            let error = if token.is_expired() {
                "Token expired"
            } else if token.revoked {
                "Token revoked"
            } else if token.is_usage_limit_reached() {
                "Usage limit reached"
            } else {
                "Token invalid"
            };

            Self {
                valid: false,
                user_id: None,
                satker_code: None,
                scopes: Vec::new(),
                admin_level: None,
                secreton_access_level: None,
                error: Some(error.to_string()),
                ttl: None,
                usage_count: None,
                max_usage: None,
            }
        }
    }
}

/// Hierarchical admin operation request
#[derive(Debug, Deserialize)]
pub struct AdminOperationRequest {
    /// Type of admin operation
    pub operation: String,
    /// Target resource type
    pub resource_type: String,
    /// Target resource ID
    pub resource_id: String,
    /// Target satker code
    pub target_satker: String,
    /// Additional operation parameters
    pub parameters: Option<serde_json::Value>,
}

impl AdminOperationRequest {
    /// Validate if the operation can be performed with the given token
    pub fn validate_with_token(&self, token: &Token) -> Result<(), String> {
        if !token.is_valid() {
            return Err("Invalid token".to_string());
        }

        // Check if token allows this operation
        if !token.allows_access(&self.operation, &self.resource_type, &self.target_satker) {
            return Err("Insufficient permissions for operation".to_string());
        }

        // For admin operations, check admin level
        if self.operation == "admin" {
            let target_admin_level = AdminLevel::AdminSatker(self.target_satker.clone());
            if !token.allows_admin_operation(&target_admin_level) {
                return Err("Insufficient admin privileges".to_string());
            }
        }

        Ok(())
    }
}
