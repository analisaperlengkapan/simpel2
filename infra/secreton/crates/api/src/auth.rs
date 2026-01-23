//! Authentication and authorization for the Secreton API
//!
//! Provides JWT-based authentication, role-based access control,
//! and integration with external identity providers.

pub mod oidc_verifier;

use axum::http::HeaderValue;
use chrono::{Duration, Utc};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

pub use oidc_verifier::{MultiVerifier, OidcVerifier, OidcVerifierConfig};

/// Token verification trait for abstracting different JWT verification strategies
///
/// This trait allows Secreton to support multiple token verification methods:
/// - SharedSecretVerifier: Uses HS256 with a shared secret (existing behavior)
/// - OidcVerifier: Uses RS256/ES256 with JWKS from Authenc (new integration)
#[async_trait::async_trait]
pub trait TokenVerifier: Send + Sync {
    /// Verify and decode a JWT token
    async fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError>;

    /// Get the issuer this verifier handles
    fn issuer(&self) -> &str;

    /// Get the audience this verifier handles
    fn audience(&self) -> &str;
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,              // Subject (user ID)
    pub name: String,             // User name
    pub email: String,            // User email
    pub roles: Vec<String>,       // User roles
    pub permissions: Vec<String>, // Specific permissions
    pub exp: usize,               // Expiration time
    pub iat: usize,               // Issued at
    pub iss: String,              // Issuer
    pub aud: String,              // Audience
    pub jti: String,              // JWT ID
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>, // Additional metadata
}

/// JWT Authentication configuration
/// This is a simplified runtime configuration for JWT operations.
/// For comprehensive API authentication configuration including OAuth2, mTLS, and MFA,
/// see `crate::config::AuthConfig`.
#[derive(Debug, Clone)]
pub struct JwtAuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub issuer: String,
    pub audience: String,
    pub require_auth: bool,
    pub admin_roles: Vec<String>,
}

impl JwtAuthConfig {
    /// Create new JwtAuthConfig with validation
    ///
    /// # Security
    /// - JWT secret MUST be at least 32 characters (256 bits)
    /// - Issuer and audience MUST be non-empty
    /// - JWT expiration should be reasonable (1-168 hours)
    pub fn new(
        jwt_secret: String,
        jwt_expiration_hours: i64,
        issuer: String,
        audience: String,
        require_auth: bool,
        admin_roles: Vec<String>,
    ) -> Result<Self, AuthError> {
        // SECURITY: Validate JWT secret length (minimum 256 bits = 32 bytes)
        if jwt_secret.len() < 32 {
            return Err(AuthError::Configuration(format!(
                "JWT secret must be at least 32 characters (256 bits). Current length: {} characters. \
                     Use a cryptographically random string generated with: openssl rand -base64 32",
                jwt_secret.len()
            )));
        }

        // Validate issuer and audience are not empty
        if issuer.is_empty() {
            return Err(AuthError::Configuration(
                "JWT issuer cannot be empty".to_string(),
            ));
        }

        if audience.is_empty() {
            return Err(AuthError::Configuration(
                "JWT audience cannot be empty".to_string(),
            ));
        }

        // Validate expiration is reasonable (between 1 hour and 1 week)
        if !(1..=168).contains(&jwt_expiration_hours) {
            return Err(AuthError::Configuration(format!(
                "JWT expiration hours must be between 1 and 168 (1 week). Got: {}",
                jwt_expiration_hours
            )));
        }

        Ok(Self {
            jwt_secret,
            jwt_expiration_hours,
            issuer,
            audience,
            require_auth,
            admin_roles,
        })
    }

    /// Load from environment variables with validation
    ///
    /// # Required Environment Variables
    /// - `SECRETON_JWT_SECRET`: JWT signing secret (minimum 32 characters)
    ///
    /// # Optional Environment Variables
    /// - `SECRETON_JWT_EXPIRATION_HOURS`: Token expiration (default: 24)
    /// - `SECRETON_JWT_ISSUER`: JWT issuer (default: "secreton-engine")
    /// - `SECRETON_JWT_AUDIENCE`: JWT audience (default: "secreton-api")
    /// - `SECRETON_REQUIRE_AUTH`: Enable authentication (default: true)
    pub fn from_env() -> Result<Self, AuthError> {
        let jwt_secret = std::env::var("SECRETON_JWT_SECRET").map_err(|_| {
            AuthError::Configuration(
                "SECRETON_JWT_SECRET environment variable is required. \
                     Generate a secure secret with: openssl rand -base64 32"
                    .to_string(),
            )
        })?;

        let jwt_expiration_hours = std::env::var("SECRETON_JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        let issuer =
            std::env::var("SECRETON_JWT_ISSUER").unwrap_or_else(|_| "secreton-engine".to_string());

        let audience =
            std::env::var("SECRETON_JWT_AUDIENCE").unwrap_or_else(|_| "secreton-api".to_string());

        let require_auth = std::env::var("SECRETON_REQUIRE_AUTH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        let admin_roles = std::env::var("SECRETON_ADMIN_ROLES")
            .ok()
            .map(|s| s.split(',').map(|r| r.trim().to_string()).collect())
            .unwrap_or_else(|| vec!["admin".to_string(), "engine-admin".to_string()]);

        Self::new(
            jwt_secret,
            jwt_expiration_hours,
            issuer,
            audience,
            require_auth,
            admin_roles,
        )
    }
}

impl Default for JwtAuthConfig {
    /// Default configuration - ONLY for testing
    ///
    /// # Security Warning
    /// DO NOT use default configuration in production!
    /// Always load from environment variables using `JwtAuthConfig::from_env()`
    fn default() -> Self {
        warn!("⚠️  Using default JwtAuthConfig - NOT SUITABLE FOR PRODUCTION!");
        warn!("   Load configuration from environment with JwtAuthConfig::from_env()");

        Self {
            // Weak secret - only for testing
            jwt_secret: "test-secret-minimum-32-characters-long-for-security".to_string(),
            jwt_expiration_hours: 24,
            issuer: "secreton-engine-test".to_string(),
            audience: "secreton-api-test".to_string(),
            require_auth: true,
            admin_roles: vec!["admin".to_string(), "engine-admin".to_string()],
        }
    }
}

/// User roles for RBAC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    EngineAdmin,
    KeyManager,
    CryptoUser,
    ReadOnly,
}

impl UserRole {
    pub fn as_string(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_string(),
            UserRole::EngineAdmin => "engine-admin".to_string(),
            UserRole::KeyManager => "key-manager".to_string(),
            UserRole::CryptoUser => "crypto-user".to_string(),
            UserRole::ReadOnly => "read-only".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(UserRole::Admin),
            "engine-admin" => Some(UserRole::EngineAdmin),
            "key-manager" => Some(UserRole::KeyManager),
            "crypto-user" => Some(UserRole::CryptoUser),
            "read-only" => Some(UserRole::ReadOnly),
            _ => None,
        }
    }
}

/// Permissions for fine-grained access control
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Permission {
    // Key management permissions
    CreateKey,
    DeleteKey,
    RotateKey,
    ReadKey,
    ListKeys,

    // Cryptographic operation permissions
    Encrypt,
    Decrypt,
    Sign,
    Verify,

    // Utility permissions
    GenerateRandom,
    HashData,
    DeriveKey,

    // Administrative permissions
    ViewMetrics,
    ConfigureSystem,
    ManageUsers,
    AccessAuditLogs,
}

impl Permission {
    pub fn as_string(&self) -> String {
        match self {
            Permission::CreateKey => "create-key".to_string(),
            Permission::DeleteKey => "delete-key".to_string(),
            Permission::RotateKey => "rotate-key".to_string(),
            Permission::ReadKey => "read-key".to_string(),
            Permission::ListKeys => "list-keys".to_string(),
            Permission::Encrypt => "encrypt".to_string(),
            Permission::Decrypt => "decrypt".to_string(),
            Permission::Sign => "sign".to_string(),
            Permission::Verify => "verify".to_string(),
            Permission::GenerateRandom => "generate-random".to_string(),
            Permission::HashData => "hash-data".to_string(),
            Permission::DeriveKey => "derive-key".to_string(),
            Permission::ViewMetrics => "view-metrics".to_string(),
            Permission::ConfigureSystem => "configure-system".to_string(),
            Permission::ManageUsers => "manage-users".to_string(),
            Permission::AccessAuditLogs => "access-audit-logs".to_string(),
        }
    }
}

/// JWT token service for generation and validation
/// This service focuses solely on JWT operations. For full authentication
/// with user management, sessions, and roles, see `crate::services::auth::AuthService`.
#[derive(Clone)]
pub struct JwtService {
    config: JwtAuthConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(config: JwtAuthConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());

        Self {
            config,
            encoding_key,
            decoding_key,
        }
    }

    /// Generate JWT token for a user
    pub fn generate_token(
        &self,
        user_id: &str,
        name: &str,
        email: &str,
        roles: Vec<String>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.config.jwt_expiration_hours);

        // Generate permissions based on roles
        let permissions = self.generate_permissions_from_roles(&roles);

        let claims = Claims {
            sub: user_id.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            roles,
            permissions,
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
            metadata: metadata.unwrap_or_default(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::TokenGeneration(e.to_string()))
    }

    /// Validate and decode JWT token
    pub fn validate_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AuthError::TokenValidation(e.to_string()))
    }

    /// Check if user has required permission
    pub fn check_permission(&self, claims: &Claims, required_permission: Permission) -> bool {
        let permission_str = required_permission.as_string();

        // Check if user has the specific permission
        if claims.permissions.contains(&permission_str) {
            return true;
        }

        // Check if user has admin role (admins have all permissions)
        for admin_role in &self.config.admin_roles {
            if claims.roles.contains(admin_role) {
                return true;
            }
        }

        false
    }

    /// Generate permissions based on user roles
    fn generate_permissions_from_roles(&self, roles: &[String]) -> Vec<String> {
        let mut permissions = Vec::new();

        for role in roles {
            match UserRole::from_string(role) {
                Some(UserRole::Admin) => {
                    // Admins get all permissions
                    permissions.extend(vec![
                        Permission::CreateKey.as_string(),
                        Permission::DeleteKey.as_string(),
                        Permission::RotateKey.as_string(),
                        Permission::ReadKey.as_string(),
                        Permission::ListKeys.as_string(),
                        Permission::Encrypt.as_string(),
                        Permission::Decrypt.as_string(),
                        Permission::Sign.as_string(),
                        Permission::Verify.as_string(),
                        Permission::GenerateRandom.as_string(),
                        Permission::HashData.as_string(),
                        Permission::DeriveKey.as_string(),
                        Permission::ViewMetrics.as_string(),
                        Permission::ConfigureSystem.as_string(),
                        Permission::ManageUsers.as_string(),
                        Permission::AccessAuditLogs.as_string(),
                    ]);
                }
                Some(UserRole::EngineAdmin) => {
                    permissions.extend(vec![
                        Permission::CreateKey.as_string(),
                        Permission::RotateKey.as_string(),
                        Permission::ReadKey.as_string(),
                        Permission::ListKeys.as_string(),
                        Permission::Encrypt.as_string(),
                        Permission::Decrypt.as_string(),
                        Permission::Sign.as_string(),
                        Permission::Verify.as_string(),
                        Permission::GenerateRandom.as_string(),
                        Permission::HashData.as_string(),
                        Permission::DeriveKey.as_string(),
                        Permission::ViewMetrics.as_string(),
                        Permission::AccessAuditLogs.as_string(),
                    ]);
                }
                Some(UserRole::KeyManager) => {
                    permissions.extend(vec![
                        Permission::CreateKey.as_string(),
                        Permission::RotateKey.as_string(),
                        Permission::ReadKey.as_string(),
                        Permission::ListKeys.as_string(),
                        Permission::ViewMetrics.as_string(),
                    ]);
                }
                Some(UserRole::CryptoUser) => {
                    permissions.extend(vec![
                        Permission::ReadKey.as_string(),
                        Permission::ListKeys.as_string(),
                        Permission::Encrypt.as_string(),
                        Permission::Decrypt.as_string(),
                        Permission::Sign.as_string(),
                        Permission::Verify.as_string(),
                        Permission::GenerateRandom.as_string(),
                        Permission::HashData.as_string(),
                        Permission::DeriveKey.as_string(),
                    ]);
                }
                Some(UserRole::ReadOnly) => {
                    permissions.extend(vec![
                        Permission::ReadKey.as_string(),
                        Permission::ListKeys.as_string(),
                        Permission::ViewMetrics.as_string(),
                    ]);
                }
                None => {
                    warn!("Unknown role: {}", role);
                }
            }
        }

        permissions.sort();
        permissions.dedup();
        permissions
    }
}

/// Shared secret token verifier (HS256)
///
/// This verifier uses the existing JWT service with a shared secret.
/// It provides backward compatibility with the current authentication system.
pub struct SharedSecretVerifier {
    jwt_service: JwtService,
}

impl SharedSecretVerifier {
    pub fn new(config: JwtAuthConfig) -> Self {
        Self {
            jwt_service: JwtService::new(config),
        }
    }

    pub fn jwt_service(&self) -> &JwtService {
        &self.jwt_service
    }
}

#[async_trait::async_trait]
impl TokenVerifier for SharedSecretVerifier {
    async fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        self.jwt_service.validate_token(token)
    }

    fn issuer(&self) -> &str {
        &self.jwt_service.config.issuer
    }

    fn audience(&self) -> &str {
        &self.jwt_service.config.audience
    }
}

/// Extract bearer token from Authorization header
pub fn extract_bearer_token(auth_header: &HeaderValue) -> Option<String> {
    let auth_str = auth_header.to_str().ok()?;
    auth_str
        .strip_prefix("Bearer ")
        .map(|stripped| stripped.to_string())
}

// Use consolidated AuthError from error module
pub use crate::error::AuthError;

// Use canonical types from secreton_core::models
// LoginRequest, LoginResponse, UserInfo are now imported at the top

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_validation() {
        let config = JwtAuthConfig::default();
        let jwt_service = JwtService::new(config);

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("mfa_passed".to_string(), "true".to_string());

        let token = jwt_service
            .generate_token(
                "user123",
                "Test User",
                "test@example.com",
                vec!["crypto-user".to_string()],
                Some(metadata),
            )
            .expect("token generation should succeed");

        let token_data = jwt_service
            .validate_token(&token)
            .expect("Token validation should succeed");

        assert_eq!(token_data.claims.sub, "user123");
        assert_eq!(token_data.claims.name, "Test User");
        assert_eq!(token_data.claims.email, "test@example.com");
        assert!(token_data.claims.roles.contains(&"crypto-user".to_string()));
        assert_eq!(
            token_data
                .claims
                .metadata
                .get("mfa_passed")
                .map(|v| v.as_str()),
            Some("true")
        );
    }

    #[test]
    fn test_permission_checking() {
        let config = JwtAuthConfig::default();
        let jwt_service = JwtService::new(config);

        let claims = Claims {
            sub: "user123".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["crypto-user".to_string()],
            permissions: vec![
                Permission::Encrypt.as_string(),
                Permission::Decrypt.as_string(),
            ],
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            iss: "secreton-engine".to_string(),
            aud: "secreton-api".to_string(),
            jti: Uuid::new_v4().to_string(),
            metadata: std::collections::HashMap::new(),
        };

        assert!(jwt_service.check_permission(&claims, Permission::Encrypt));
        assert!(jwt_service.check_permission(&claims, Permission::Decrypt));
        assert!(!jwt_service.check_permission(&claims, Permission::DeleteKey));
    }

    #[test]
    fn test_admin_role_grants_all_permissions() {
        let config = JwtAuthConfig::default();
        let jwt_service = JwtService::new(config);

        let claims = Claims {
            sub: "admin-user".to_string(),
            name: "Admin".to_string(),
            email: "admin@example.com".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec![],
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            iss: "secreton-engine".to_string(),
            aud: "secreton-api".to_string(),
            jti: Uuid::new_v4().to_string(),
            metadata: std::collections::HashMap::new(),
        };

        assert!(jwt_service.check_permission(&claims, Permission::ManageUsers));
        assert!(jwt_service.check_permission(&claims, Permission::AccessAuditLogs));
    }

    #[test]
    fn test_generate_token_includes_role_permissions() {
        let config = JwtAuthConfig::default();
        let jwt_service = JwtService::new(config);

        let token = jwt_service
            .generate_token(
                "engine-admin",
                "Engine Admin",
                "engine.admin@example.com",
                vec!["engine-admin".to_string()],
                None,
            )
            .expect("token generation");

        let data = jwt_service
            .validate_token(&token)
            .expect("token validation");
        let permissions = data.claims.permissions;

        assert!(permissions.contains(&Permission::AccessAuditLogs.as_string()));
        assert!(permissions.contains(&Permission::Encrypt.as_string()));
        assert!(permissions.contains(&Permission::HashData.as_string()));
    }

    #[test]
    fn test_extract_bearer_token() {
        let header = HeaderValue::from_str("Bearer secret-token").expect("valid header");
        let token = extract_bearer_token(&header).expect("token expected");
        assert_eq!(token, "secret-token");
    }

    #[test]
    fn test_extract_bearer_token_invalid_format() {
        let header = HeaderValue::from_str("Basic abc123").expect("valid header");
        assert!(extract_bearer_token(&header).is_none());

        let header = HeaderValue::from_str("Bearer").expect("valid header");
        assert!(extract_bearer_token(&header).is_none());
    }
}
