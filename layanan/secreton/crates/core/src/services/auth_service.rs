//! Authentication service for user management and token validation.

use anyhow::Result;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::config::api::AuthConfig;
use crate::models::User;
use secreton_crypto::{AlgorithmId, CryptoEngine};
use secreton_storage::{SecretEntry, SecurityLevel, StorageBackend};
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};

/// Root user ID (nil UUID) used for initial bootstrap and recovery
pub const ROOT_USER_ID: &str = "00000000-0000-0000-0000-000000000000";

/// Authentication and authorization error types.
#[derive(Error, Debug)]
pub enum AuthError {
    /// Invalid username or password
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// User not found in the system
    #[error("User not found")]
    UserNotFound,

    /// User already exists (during registration)
    #[error("User already exists")]
    UserAlreadyExists,

    /// Invalid JWT or session token
    #[error("Invalid token")]
    InvalidToken,

    /// Token has expired
    #[error("Token expired")]
    TokenExpired,

    /// Token generation failed
    #[error("Token generation failed: {0}")]
    TokenGeneration(String),

    /// Token validation failed
    #[error("Token validation failed: {0}")]
    TokenValidation(String),

    /// Missing Authorization header
    #[error("Missing authorization header")]
    MissingAuthHeader,

    /// Invalid Authorization header format
    #[error("Invalid authorization header format")]
    InvalidAuthHeader,

    /// Missing credentials in request
    #[error("Missing credentials")]
    MissingCredentials,

    /// Multi-factor authentication required
    #[error("MFA required")]
    MfaRequired,

    /// Invalid MFA code provided
    #[error("Invalid MFA code")]
    InvalidMfaCode,

    /// User lacks required permissions
    #[error("Permission denied")]
    PermissionDenied,

    /// Configuration error (invalid settings, missing required config)
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Storage layer error
    #[error("Storage error: {0}")]
    Storage(#[from] secreton_storage::StorageError),

    /// Cryptography error
    #[error("Crypto error: {0}")]
    Crypto(#[from] secreton_crypto::CryptoError),

    /// Internal authentication service error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AuthError {
    /// Get error code for programmatic handling
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::UserAlreadyExists => "USER_EXISTS",
            Self::InvalidToken => "INVALID_TOKEN",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::TokenGeneration(_) => "TOKEN_GENERATION_FAILED",
            Self::TokenValidation(_) => "TOKEN_VALIDATION_FAILED",
            Self::MissingAuthHeader => "MISSING_AUTH_HEADER",
            Self::InvalidAuthHeader => "INVALID_AUTH_HEADER",
            Self::MissingCredentials => "MISSING_CREDENTIALS",
            Self::MfaRequired => "MFA_REQUIRED",
            Self::InvalidMfaCode => "INVALID_MFA_CODE",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
            Self::Storage(_) => "STORAGE_ERROR",
            Self::Crypto(_) => "CRYPTO_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

/// User Data Transfer Object for storage persistence
///
/// This struct mirrors the core `User` model but ensures all fields,
/// specifically `password_hash`, are serialized for storage.
/// The core `User` model skips serialization of `password_hash` for API security.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_superuser: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub mfa_enabled: bool,
    pub roles: HashSet<String>,
    pub policies: HashSet<String>,
    pub namespace: String,
    pub is_locked: bool,
    pub failed_attempts: u32,
    pub locked_until: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: HashMap<String, String>,
}

impl From<&User> for StoredUser {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            password_hash: user.password_hash.clone(),
            full_name: user.full_name.clone(),
            is_active: user.is_active,
            is_superuser: user.is_superuser,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login: user.last_login,
            mfa_enabled: user.mfa_enabled,
            roles: user.roles.clone(),
            policies: user.policies.clone(),
            namespace: user.namespace.clone(),
            is_locked: user.is_locked,
            failed_attempts: user.failed_attempts,
            locked_until: user.locked_until,
            metadata: user.metadata.clone(),
        }
    }
}

impl From<StoredUser> for User {
    fn from(stored: StoredUser) -> Self {
        Self {
            id: stored.id,
            username: stored.username,
            email: stored.email,
            password_hash: stored.password_hash,
            full_name: stored.full_name,
            is_active: stored.is_active,
            is_superuser: stored.is_superuser,
            created_at: stored.created_at,
            updated_at: stored.updated_at,
            last_login: stored.last_login,
            mfa_enabled: stored.mfa_enabled,
            roles: stored.roles,
            policies: stored.policies,
            namespace: stored.namespace,
            is_locked: stored.is_locked,
            failed_attempts: stored.failed_attempts,
            locked_until: stored.locked_until,
            metadata: stored.metadata,
        }
    }
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub refresh_token: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: User,
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: String,
    exp: usize,
    iat: usize,
    jti: String,
    roles: Vec<String>,
    policies: Vec<String>,
    token_type: String, // "access" or "refresh"
    // Extended claims for stateless validation
    username: String,
    email: String,
    full_name: Option<String>,
    is_superuser: bool,
    is_active: bool,
    mfa_enabled: bool,
    namespace: String,
}

/// Authentication service
pub struct AuthService {
    storage: Arc<dyn StorageBackend + Send + Sync>,
    crypto: Arc<CryptoEngine>,
    config: AuthConfig,
}

impl AuthService {
    /// Create new authentication service
    pub async fn new(
        storage: Arc<dyn StorageBackend + Send + Sync>,
        crypto: Arc<CryptoEngine>,
        config: &AuthConfig,
    ) -> Result<Self> {
        let service = Self {
            storage,
            crypto,
            config: config.clone(),
        };

        // Initialize default roles if they don't exist
        service.initialize_default_roles().await?;

        Ok(service)
    }

    /// Authenticate user with username and password
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
        mfa_code: Option<&str>,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<AuthToken, AuthError> {
        // Get user from storage
        let mut user = self.get_user_by_username(username).await?;

        if !user.is_active {
            return Err(AuthError::InvalidCredentials);
        }

        // Verify password
        if !self.verify_password(password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Check MFA if enabled
        if user.mfa_enabled {
            if let Some(code) = mfa_code {
                self.verify_mfa_code(&user, code).await?;
            } else {
                return Err(AuthError::MfaRequired);
            }
        }

        // Create session and tokens
        let session_id = Uuid::new_v4().to_string();
        let access_token = self.create_access_token(&user, &session_id)?;
        let refresh_token = self.create_refresh_token(&user, &session_id)?;

        // Store session
        let session = Session {
            id: session_id,
            user_id: user.username.clone(),
            token: access_token.clone(),
            refresh_token: Some(refresh_token.clone()),
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now()
                + chrono::Duration::seconds(self.config.jwt.expiration as i64),
            last_accessed: chrono::Utc::now(),
        };

        self.store_session(&session).await?;

        // Update last login
        self.update_last_login(&mut user).await?;

        Ok(AuthToken {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt.expiration,
            user,
        })
    }

    /// Validate access token
    pub async fn validate_token(&self, token: &str) -> Result<User, AuthError> {
        use jsonwebtoken::{DecodingKey, Validation, decode};

        let mut validation = Validation::new(
            Algorithm::from_str(&self.config.jwt.algorithm).map_err(|e| {
                AuthError::Configuration(format!("Invalid JWT algorithm in config: {}", e))
            })?,
        );

        // Validation requires setting audience and issuer
        validation.set_audience(&[&self.config.jwt.audience]);
        validation.set_issuer(&[&self.config.jwt.issuer]);

        // DecodingKey from secret
        let decoding_key = DecodingKey::from_secret(self.config.jwt.secret.as_bytes());

        let token_data =
            decode::<Claims>(token, &decoding_key, &validation).map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            })?;

        let claims = token_data.claims;

        // Enforce token type (allow "access" for normal tokens and "root" for init root tokens)
        if claims.token_type != "access" && claims.token_type != "root" {
            return Err(AuthError::InvalidToken);
        }

        // Reconstruct user from claims
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

        // Note: In a stateful system, we might check session validity or user status in DB here.
        // For stateless/cached auth, we rely on the token signature and expiration.

        Ok(User {
            id: user_id,
            username: claims.username,
            email: claims.email,
            password_hash: String::new(), // Not present in token
            full_name: claims.full_name,
            is_active: claims.is_active,
            is_superuser: claims.is_superuser,
            created_at: chrono::Utc::now(),       // Approximation
            updated_at: chrono::Utc::now(),       // Approximation
            last_login: Some(chrono::Utc::now()), // Active now
            mfa_enabled: claims.mfa_enabled,
            roles: claims.roles.into_iter().collect(),
            policies: claims.policies.into_iter().collect(),
            namespace: claims.namespace,
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
            metadata: HashMap::new(), // Metadata not currently in JWT
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken, AuthError> {
        use jsonwebtoken::{DecodingKey, Validation, decode};

        let mut validation = Validation::new(
            Algorithm::from_str(&self.config.jwt.algorithm).map_err(|e| {
                AuthError::Configuration(format!("Invalid JWT algorithm in config: {}", e))
            })?,
        );
        validation.set_audience(&[&self.config.jwt.audience]);
        validation.set_issuer(&[&self.config.jwt.issuer]);

        let decoding_key = DecodingKey::from_secret(self.config.jwt.secret.as_bytes());

        let token_data =
            decode::<Claims>(refresh_token, &decoding_key, &validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    _ => AuthError::InvalidToken,
                }
            })?;

        let claims = token_data.claims;

        // Enforce token type must be 'refresh'
        if claims.token_type != "refresh" {
            return Err(AuthError::InvalidToken);
        }

        // Get user from token
        let user = self.get_user(&claims.sub).await?;

        // Generate new access token
        let access_token = self.create_access_token(&user, &claims.jti)?;

        Ok(AuthToken {
            access_token,
            refresh_token: refresh_token.to_string(),
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt.expiration,
            user,
        })
    }

    /// Create user
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
        full_name: Option<&str>,
        roles: Vec<String>,
        policies: Vec<String>,
        metadata: Option<HashMap<String, String>>,
        is_active: bool,
    ) -> Result<User, AuthError> {
        // Check if user already exists
        if self.user_exists(username).await? {
            return Err(AuthError::UserAlreadyExists);
        }

        // Hash password
        let password_hash = self.hash_password(password)?;

        let user = User {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            full_name: full_name.map(|s| s.to_string()),
            is_active,
            is_superuser: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
            mfa_enabled: false,
            roles: roles.into_iter().collect(),
            policies: policies.into_iter().collect(),
            namespace: "default".to_string(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
            metadata: metadata.unwrap_or_default(),
        };

        self.store_user(&user).await?;
        Ok(user)
    }

    /// Update user details
    pub async fn update_user(
        &self,
        user_id: &str,
        email: Option<String>,
        full_name: Option<String>,
        is_active: Option<bool>,
        policies: Option<Vec<String>>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<User, AuthError> {
        let mut user = self.get_user(user_id).await?;

        if let Some(email) = email {
            user.email = email;
        }
        if let Some(p) = policies {
            user.policies = p.into_iter().collect();
        }
        if let Some(full_name) = full_name {
            user.full_name = Some(full_name);
        }
        if let Some(is_active) = is_active {
            user.is_active = is_active;
        }
        if let Some(metadata) = metadata {
            user.metadata = metadata;
        }

        user.updated_at = chrono::Utc::now();

        self.store_user(&user).await?;
        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<User, AuthError> {
        let uuid = Uuid::parse_str(user_id).map_err(|_| AuthError::UserNotFound)?;

        let path = format!("auth/users/{}", uuid);
        let entry = self
            .storage
            .get_by_path(&path)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to retrieve user: {}", e)))?;

        match entry {
            Some(entry) => self.decrypt_user(entry),
            None => Err(AuthError::UserNotFound),
        }
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<User, AuthError> {
        let path = format!("auth/usernames/{}", username);
        let index_entry =
            self.storage.get_by_path(&path).await.map_err(|e| {
                AuthError::Internal(format!("Failed to retrieve user index: {}", e))
            })?;

        match index_entry {
            Some(entry) => {
                let uuid_bytes =
                    self.crypto
                        .decrypt_simple(&entry.encrypted_data)
                        .map_err(|e| {
                            AuthError::Internal(format!("Failed to decrypt user index: {}", e))
                        })?;

                let uuid_str = String::from_utf8(uuid_bytes).map_err(|e| {
                    AuthError::Internal(format!("Invalid UUID string in index: {}", e))
                })?;

                self.get_user(&uuid_str).await
            }
            None => Err(AuthError::UserNotFound),
        }
    }

    /// Check if user has permission
    pub async fn has_permission(&self, user: &User, permission: &str) -> Result<bool, AuthError> {
        // Get user roles and their permissions
        let mut all_permissions = Vec::new();

        for role_name in &user.roles {
            if let Ok(role) = self.get_role(role_name).await {
                all_permissions.extend(role.permissions);
            }
        }

        // Check for wildcard permission
        if all_permissions.contains(&"*".to_string()) {
            return Ok(true);
        }

        // Check for exact permission match
        if all_permissions.contains(&permission.to_string()) {
            return Ok(true);
        }

        // Check for pattern match (e.g., "vault:*" matches "vault:read")
        for perm in &all_permissions {
            if perm.ends_with("*") {
                let prefix = &perm[..perm.len() - 1];
                if permission.starts_with(prefix) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check if user has a specific role
    pub async fn has_role(&self, user_id: &str, role: &str) -> Result<bool, AuthError> {
        // Special case for root user
        if user_id == ROOT_USER_ID {
            return Ok(true);
        }

        let user = self.get_user(user_id).await?;

        // Superusers have all roles implicitly
        if user.is_superuser {
            return Ok(true);
        }

        Ok(user.has_role(role))
    }

    /// Get role by name
    pub async fn get_role(&self, role_name: &str) -> Result<Role, AuthError> {
        let path = format!("auth/roles/{}", role_name);

        match self.storage.get_by_path(&path).await {
            Ok(Some(entry)) => {
                let decrypted_bytes = self
                    .crypto
                    .decrypt_simple(&entry.encrypted_data)
                    .map_err(|e| AuthError::Internal(format!("Failed to decrypt role: {}", e)))?;

                let role: Role = serde_json::from_slice(&decrypted_bytes).map_err(|e| {
                    AuthError::Internal(format!("Failed to deserialize role: {}", e))
                })?;

                Ok(role)
            }
            Ok(None) => Err(AuthError::Internal(format!(
                "Role '{}' not found",
                role_name
            ))),
            Err(e) => Err(AuthError::Internal(format!(
                "Failed to retrieve role: {}",
                e
            ))),
        }
    }

    /// Get user effective permissions (aggregated from roles)
    /// NOTE: This does NOT include policy names from `user.policies`.
    /// `user.policies` should be used to load PolicyDefinition objects.
    pub async fn get_user_permissions(&self, user: &User) -> Result<Vec<String>, AuthError> {
        let mut all_permissions = Vec::new();

        for role_name in &user.roles {
            match self.get_role(role_name).await {
                Ok(role) => {
                    all_permissions.extend(role.permissions);
                }
                Err(_) => {
                    // Ignore missing roles to allow partial success
                    continue;
                }
            }
        }

        all_permissions.sort();
        all_permissions.dedup();

        Ok(all_permissions)
    }

    /// Get user effective policy names (explicit + role-derived)
    pub fn get_user_policy_names(&self, user: &User) -> Vec<String> {
        let mut policy_names: Vec<String> = user.policies.iter().cloned().collect();

        // Add role-based policies (convention: role name = policy name suffix or mapping)
        // For simple implementations, we might assume roles map 1:1 to policies if configured,
        // but here we just return the explicit ones plus any we might derive.
        // The API middleware currently derives "{role}-policy". We can replicate that or leave it to the caller.
        // For Core consistency, let's include role-based defaults if that's the model.
        for role in &user.roles {
            policy_names.push(format!("{}-policy", role.to_lowercase()));
        }

        policy_names.sort();
        policy_names.dedup();
        policy_names
    }

    /// Create role
    pub async fn create_role(
        &self,
        name: &str,
        description: Option<&str>,
        permissions: Vec<String>,
    ) -> Result<Role, AuthError> {
        let role = Role {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            permissions,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        self.store_role(&role).await?;
        Ok(role)
    }

    /// Initialize default roles
    async fn initialize_default_roles(&self) -> Result<()> {
        // Create admin role
        if self.get_role("admin").await.is_err() {
            self.create_role(
                "admin",
                Some("System administrator with full access"),
                vec!["*".to_string()],
            )
            .await?;
        }

        // Create user role
        if self.get_role("user").await.is_err() {
            self.create_role(
                "user",
                Some("Regular user with basic vault access"),
                vec![
                    "vault:read".to_string(),
                    "vault:write".to_string(),
                    "vault:list".to_string(),
                ],
            )
            .await?;
        }

        // Create viewer role
        if self.get_role("viewer").await.is_err() {
            self.create_role(
                "viewer",
                Some("Read-only access to vault"),
                vec!["vault:read".to_string(), "vault:list".to_string()],
            )
            .await?;
        }

        Ok(())
    }

    /// Hash password using crypto service
    fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        use secreton_crypto::hashing::password;

        let result = password::hash_password_argon2(password)
            .map_err(|e| AuthError::Internal(format!("Password hashing failed: {}", e)))?;

        Ok(result.hash)
    }

    /// Verify password using crypto service
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        use secreton_crypto::hashing::password;

        password::verify_password_argon2(password, hash)
            .map_err(|e| AuthError::Internal(format!("Password verification failed: {}", e)))
    }

    /// Verify MFA code
    async fn verify_mfa_code(&self, user: &User, code: &str) -> Result<(), AuthError> {
        let secret = self.get_mfa_secret(&user.id.to_string()).await?;

        if let Some(secret_str) = secret {
            // Check if user has TOTP enabled
            let totp = TOTP::new(
                TotpAlgorithm::SHA1,
                6,
                1,
                30,
                Secret::Raw(secret_str.into_bytes()).to_bytes().unwrap(),
            )
            .unwrap();

            let valid = totp
                .check_current(code)
                .map_err(|e| AuthError::Internal(format!("Failed to verify code: {}", e)))?;
            if valid {
                Ok(())
            } else {
                Err(AuthError::InvalidMfaCode)
            }
        } else {
            // If MFA is enabled but no secret is found, we cannot verify the code.
            // This is a system inconsistency or setup issue.
            Err(AuthError::Internal(
                "MFA enabled but no secret found".to_string(),
            ))
        }
    }

    /// Retrieve and decrypt MFA secret
    async fn get_mfa_secret(&self, user_id: &str) -> Result<Option<String>, AuthError> {
        let path = format!("sys/mfa/{}/totp", user_id);

        if let Some(entry) = self
            .storage
            .get_by_path(&path)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
        {
            // Derive key using PBKDF2 with user ID as salt
            let key = secreton_crypto::key_derivation::derive_key_pbkdf2(
                self.config.jwt.secret.as_bytes(),
                user_id.as_bytes(),
                10000,
                32,
            )
            .map_err(|e| AuthError::Internal(format!("Key derivation failed: {}", e)))?;

            // Decrypt the secret
            let encrypted = secreton_crypto::EncryptedData {
                algorithm: AlgorithmId::Aes256Gcm, // Assuming AES-256-GCM as standard
                nonce: entry.encryption_metadata["nonce"]
                    .as_str()
                    .and_then(|s| hex::decode(s).ok())
                    .unwrap_or_default(),
                ciphertext: entry.encrypted_data,
                tag: None,
            };

            let secret_bytes = self
                .crypto
                .decrypt(&encrypted, &key)
                .map_err(|e| AuthError::Internal(format!("Decryption failed: {}", e)))?;

            Ok(Some(String::from_utf8(secret_bytes).map_err(|_| {
                AuthError::Internal("Invalid UTF-8 in secret".to_string())
            })?))
        } else {
            Ok(None)
        }
    }

    /// Store MFA secret (helper for setup)
    pub async fn store_mfa_secret(&self, user_id: &str, secret: &str) -> Result<(), AuthError> {
        let path = format!("sys/mfa/{}/totp", user_id);

        // Derive key
        let key = secreton_crypto::key_derivation::derive_key_pbkdf2(
            self.config.jwt.secret.as_bytes(),
            user_id.as_bytes(),
            10000,
            32,
        )
        .map_err(|e| AuthError::Internal(format!("Key derivation failed: {}", e)))?;

        // Encrypt secret
        let encrypted = self
            .crypto
            .encrypt(AlgorithmId::Aes256Gcm, secret.as_bytes(), &key)
            .map_err(|e| AuthError::Internal(format!("Encryption failed: {}", e)))?;

        let entry = SecretEntry::new(
            path,
            encrypted.ciphertext,
            serde_json::json!({
                "nonce": hex::encode(encrypted.nonce),
                "algorithm": "Aes256Gcm"
            }),
            SecurityLevel::Secret,
            "system".to_string(),
        );

        self.storage
            .store(&entry)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Create access token (JWT)
    fn create_access_token(&self, user: &User, session_id: &str) -> Result<String, AuthError> {
        let now = chrono::Utc::now();
        let expiration = now + chrono::Duration::seconds(self.config.jwt.expiration as i64);

        let mut roles: Vec<String> = user.roles.iter().cloned().collect();
        roles.sort();
        let mut policies: Vec<String> = user.policies.iter().cloned().collect();
        policies.sort();

        let claims = Claims {
            sub: user.id.to_string(),
            iss: self.config.jwt.issuer.clone(),
            aud: self.config.jwt.audience.clone(),
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: session_id.to_string(),
            roles,
            policies,
            token_type: "access".to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            full_name: user.full_name.clone(),
            is_superuser: user.is_superuser,
            is_active: user.is_active,
            mfa_enabled: user.mfa_enabled,
            namespace: user.namespace.clone(),
        };

        let algorithm = Algorithm::from_str(&self.config.jwt.algorithm)
            .map_err(|e| AuthError::Configuration(format!("Invalid JWT algorithm: {}", e)))?;

        let header = Header::new(algorithm);

        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.config.jwt.secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGeneration(e.to_string()))
    }

    /// Create refresh token
    fn create_refresh_token(&self, user: &User, session_id: &str) -> Result<String, AuthError> {
        let now = chrono::Utc::now();
        let expiration = now + chrono::Duration::seconds(self.config.jwt.refresh_expiration as i64);

        let mut roles: Vec<String> = user.roles.iter().cloned().collect();
        roles.sort();
        let mut policies: Vec<String> = user.policies.iter().cloned().collect();
        policies.sort();

        let claims = Claims {
            sub: user.id.to_string(),
            iss: self.config.jwt.issuer.clone(),
            aud: self.config.jwt.audience.clone(),
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: session_id.to_string(),
            roles,
            policies,
            token_type: "refresh".to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            full_name: user.full_name.clone(),
            is_superuser: user.is_superuser,
            is_active: user.is_active,
            mfa_enabled: user.mfa_enabled,
            namespace: user.namespace.clone(),
        };

        let algorithm = Algorithm::from_str(&self.config.jwt.algorithm)
            .map_err(|e| AuthError::Configuration(format!("Invalid JWT algorithm: {}", e)))?;

        let header = Header::new(algorithm);

        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.config.jwt.secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGeneration(e.to_string()))
    }

    /// Check if user exists
    async fn user_exists(&self, username: &str) -> Result<bool, AuthError> {
        match self.get_user_by_username(username).await {
            Ok(_) => Ok(true),
            Err(AuthError::UserNotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Store user in storage
    async fn store_user(&self, user: &User) -> Result<(), AuthError> {
        // Store user entry
        let entry = self.encrypt_user(user)?;
        self.storage
            .store(&entry)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to store user: {}", e)))?;

        // Store username index
        let uuid_bytes = user.id.to_string().into_bytes();
        let encrypted_uuid = self
            .crypto
            .encrypt_simple(&uuid_bytes)
            .map_err(|e| AuthError::Internal(format!("Failed to encrypt user index: {}", e)))?;

        let index_entry = SecretEntry::new(
            format!("auth/usernames/{}", user.username),
            encrypted_uuid,
            serde_json::json!({"method": "simple", "target": "user_id"}),
            SecurityLevel::Internal,
            "system".to_string(),
        );

        self.storage
            .store(&index_entry)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to store user index: {}", e)))?;
        Ok(())
    }

    /// Store role in storage
    async fn store_role(&self, role: &Role) -> Result<(), AuthError> {
        let role_bytes = serde_json::to_vec(role)
            .map_err(|e| AuthError::Internal(format!("Failed to serialize role: {}", e)))?;

        let encrypted_data = self
            .crypto
            .encrypt_simple(&role_bytes)
            .map_err(|e| AuthError::Internal(format!("Failed to encrypt role: {}", e)))?;

        let entry = SecretEntry::new(
            format!("auth/roles/{}", role.name),
            encrypted_data,
            serde_json::json!({"method": "simple", "type": "role"}),
            SecurityLevel::Internal,
            "system".to_string(),
        );

        self.storage
            .store(&entry)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to store role: {}", e)))?;

        Ok(())
    }

    /// Store session
    async fn store_session(&self, session: &Session) -> Result<(), AuthError> {
        let session_bytes = serde_json::to_vec(session)
            .map_err(|e| AuthError::Internal(format!("Failed to serialize session: {}", e)))?;

        let encrypted_data = self
            .crypto
            .encrypt_simple(&session_bytes)
            .map_err(|e| AuthError::Internal(format!("Failed to encrypt session: {}", e)))?;

        let entry = SecretEntry::new(
            format!("auth/sessions/{}", session.id),
            encrypted_data,
            serde_json::json!({"method": "simple", "type": "session"}),
            SecurityLevel::Confidential,
            session.user_id.clone(),
        )
        .with_expiration(session.expires_at)
        .add_metadata(
            "ip_address".to_string(),
            serde_json::json!(session.ip_address),
        )
        .add_metadata(
            "user_agent".to_string(),
            serde_json::json!(session.user_agent),
        );

        self.storage
            .store(&entry)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to store session: {}", e)))?;

        Ok(())
    }

    /// Update last login timestamp
    async fn update_last_login(&self, user: &mut User) -> Result<(), AuthError> {
        user.last_login = Some(chrono::Utc::now());
        self.store_user(user).await
    }

    /// Count total users
    pub async fn count_users(&self) -> Result<u64, AuthError> {
        use secreton_storage::QueryParams;

        let params = QueryParams::new().with_path_prefix("auth/users/".to_string());
        self.storage
            .count(&params)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to count users: {}", e)))
    }

    /// Count active sessions
    pub async fn count_active_sessions(&self) -> Result<u64, AuthError> {
        use secreton_storage::QueryParams;

        // Assuming sessions are stored under "auth/sessions/"
        let params = QueryParams::new().with_path_prefix("auth/sessions/".to_string());
        self.storage
            .count(&params)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to count sessions: {}", e)))
    }

    /// Encrypt user for storage
    fn encrypt_user(&self, user: &User) -> Result<SecretEntry, AuthError> {
        let stored_user = StoredUser::from(user);
        let user_bytes = serde_json::to_vec(&stored_user)
            .map_err(|e| AuthError::Internal(format!("Failed to serialize user: {}", e)))?;

        let encrypted_data = self
            .crypto
            .encrypt_simple(&user_bytes)
            .map_err(|e| AuthError::Internal(format!("Failed to encrypt user: {}", e)))?;

        let entry = SecretEntry::new(
            format!("auth/users/{}", user.id),
            encrypted_data,
            serde_json::json!({"method": "simple"}),
            SecurityLevel::Confidential,
            user.id.to_string(),
        );

        Ok(entry)
    }

    /// Decrypt user from storage
    fn decrypt_user(&self, entry: SecretEntry) -> Result<User, AuthError> {
        let decrypted_bytes = self
            .crypto
            .decrypt_simple(&entry.encrypted_data)
            .map_err(|e| AuthError::Internal(format!("Failed to decrypt user: {}", e)))?;

        let stored_user: StoredUser = serde_json::from_slice(&decrypted_bytes)
            .map_err(|e| AuthError::Internal(format!("Failed to deserialize user: {}", e)))?;

        Ok(User::from(stored_user))
    }
}

impl AuthService {
    /// Create mock auth service for testing
    pub fn new_mock(
        storage: Arc<dyn StorageBackend + Send + Sync>,
        crypto: Arc<CryptoEngine>,
    ) -> Self {
        Self {
            storage,
            crypto,
            config: AuthConfig::default(),
        }
    }
}
