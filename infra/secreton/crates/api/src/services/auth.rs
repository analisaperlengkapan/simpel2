//! Authentication service for user management and token validation.

use anyhow::Result;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::config::AuthConfig;
use secreton_crypto::{AlgorithmId, CryptoEngine};
use secreton_storage::{QueryParams, SecretEntry, SecurityLevel, StorageBackend};
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};

// Use canonical User from core
pub use secreton_core::models::User;

// Use consolidated AuthError from error module
pub use crate::error::AuthError;

/// Root user ID (nil UUID) used for initial bootstrap and recovery
pub const ROOT_USER_ID: &str = "00000000-0000-0000-0000-000000000000";

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
            policies: std::collections::HashSet::new(), // StoredUser in API crate doesn't have policies yet
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
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: String,
    exp: usize,
    iat: usize,
    jti: String,
    roles: Vec<String>,
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
    config: Arc<RwLock<AuthConfig>>,
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
            config: Arc::new(RwLock::new(config.clone())),
        };

        // Initialize default roles if they don't exist
        service.initialize_default_roles().await?;

        Ok(service)
    }

    /// Update configuration
    pub fn update_config(&self, new_config: AuthConfig) {
        if let Ok(mut config) = self.config.write() {
            *config = new_config;
        } else {
            tracing::error!("Failed to acquire write lock on AuthConfig");
        }
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
        let user = self.get_user_by_username(username).await?;

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

        let session_id = Uuid::new_v4().to_string();

        // Scope the lock to avoid holding it across await points
        let (access_token, refresh_token, expires_in, refresh_expires_in) = {
            let config = self
                .config
                .read()
                .map_err(|_| AuthError::Internal("Config lock poisoned".to_string()))?;
            let access_token = self.create_access_token(&user, &session_id, &config)?;
            let refresh_token = self.create_refresh_token(&user, &session_id, &config)?;
            (
                access_token,
                refresh_token,
                config.jwt.expiration.as_secs(),
                config.jwt.refresh_expiration.as_secs(),
            )
        };

        // Store session
        let session = Session {
            id: session_id,
            user_id: user.username.clone(), // Note: Storing username as user_id field for readability, but id in metadata
            token: access_token.clone(),
            refresh_token: Some(refresh_token.clone()),
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(refresh_expires_in as i64),
            last_accessed: chrono::Utc::now(),
        };

        // Note: session.user_id above is actually the username based on how it's assigned.
        // We should ensure we are consistent. UserInfo returns username.
        // For metadata querying, we might want the UUID.
        // Let's stick to what was there: user.username.clone()

        self.store_session(&session, &user.id.to_string()).await?;

        // Update last login
        self.update_last_login(&user.id.to_string()).await?;

        Ok(AuthToken {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user,
        })
    }

    /// Login external user (OAuth/OIDC)
    pub async fn login_external(
        &self,
        email: &str,
        full_name: Option<String>,
        provider_metadata: HashMap<String, String>,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<AuthToken, AuthError> {
        // Use email as username for external users
        let username = email;

        // Try to find user
        let mut user = match self.get_user_by_username(username).await {
            Ok(mut u) => {
                // User exists, update metadata
                u.metadata.extend(provider_metadata);
                u
            }
            Err(AuthError::UserNotFound) => {
                // Create new user
                let password = Uuid::new_v4().to_string(); // Random password
                let roles = vec!["user".to_string()]; // Default role

                self.create_user(
                    username,
                    email,
                    &password,
                    full_name.as_deref(),
                    roles,
                    Some(provider_metadata),
                    true, // is_active
                )
                .await?
            }
            Err(e) => return Err(e),
        };

        if !user.is_active {
            return Err(AuthError::InvalidCredentials);
        }

        // Update last login
        user.last_login = Some(chrono::Utc::now());
        self.store_user(&user).await?;

        // Generate tokens
        let session_id = Uuid::new_v4().to_string();

        let (access_token, refresh_token, expires_in, refresh_expires_in) = {
            let config = self
                .config
                .read()
                .map_err(|_| AuthError::Internal("Config lock poisoned".to_string()))?;
            let access_token = self.create_access_token(&user, &session_id, &config)?;
            let refresh_token = self.create_refresh_token(&user, &session_id, &config)?;
            (
                access_token,
                refresh_token,
                config.jwt.expiration.as_secs(),
                config.jwt.refresh_expiration.as_secs(),
            )
        };

        // Store session
        let session = Session {
            id: session_id,
            user_id: user.username.clone(),
            token: access_token.clone(),
            refresh_token: Some(refresh_token.clone()),
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(refresh_expires_in as i64),
            last_accessed: chrono::Utc::now(),
        };

        self.store_session(&session, &user.id.to_string()).await?;

        Ok(AuthToken {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user,
        })
    }

    /// Validate access token
    pub async fn validate_token(&self, token: &str) -> Result<User, AuthError> {
        use jsonwebtoken::{DecodingKey, Validation, decode};

        let claims = {
            let config = self
                .config
                .read()
                .map_err(|_| AuthError::Internal("Config lock poisoned".to_string()))?;

            let mut validation =
                Validation::new(Algorithm::from_str(&config.jwt.algorithm).map_err(|e| {
                    AuthError::Configuration(format!("Invalid JWT algorithm in config: {}", e))
                })?);

            // Validation requires setting audience and issuer
            validation.set_audience(&[&config.jwt.audience]);
            validation.set_issuer(&[&config.jwt.issuer]);

            // DecodingKey from secret
            let decoding_key = DecodingKey::from_secret(config.jwt.secret.as_bytes());

            let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    _ => AuthError::InvalidToken,
                }
            })?;

            token_data.claims
        };

        // Enforce token type
        if claims.token_type != "access" {
            return Err(AuthError::InvalidToken);
        }

        // Check if session exists (is valid/active)
        // If session retrieval fails (e.g. not found), we consider the token invalid/revoked
        if self.get_session(&claims.jti).await.is_err() {
            return Err(AuthError::InvalidToken);
        }

        // Reconstruct user from claims
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

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
            policies: std::collections::HashSet::new(), // Not present in legacy JWTs
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

        // 1. Validate refresh token
        let claims = {
            let config = self
                .config
                .read()
                .map_err(|_| AuthError::Internal("Config lock poisoned".to_string()))?;

            let mut validation =
                Validation::new(Algorithm::from_str(&config.jwt.algorithm).map_err(|e| {
                    AuthError::Configuration(format!("Invalid JWT algorithm: {}", e))
                })?);
            validation.set_audience(&[&config.jwt.audience]);
            validation.set_issuer(&[&config.jwt.issuer]);

            let decoding_key = DecodingKey::from_secret(config.jwt.secret.as_bytes());

            let token_data =
                decode::<Claims>(refresh_token, &decoding_key, &validation).map_err(|e| match e
                    .kind()
                {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    _ => AuthError::InvalidToken,
                })?;

            token_data.claims
        };

        // Verify token type
        if claims.token_type != "refresh" {
            return Err(AuthError::InvalidToken);
        }

        // 2. Get session
        let session_id = &claims.jti;
        let mut session = match self.get_session(session_id).await {
            Ok(s) => s,
            Err(_) => return Err(AuthError::InvalidToken),
        };

        // Check if token matches session (Rotate/Reuse detection)
        if let Some(current_refresh) = &session.refresh_token {
            if current_refresh != refresh_token {
                // Token mismatch - potential reuse detected!
                let _ = self.revoke_session(session_id).await;
                return Err(AuthError::InvalidToken);
            }
        } else {
            // Session has no refresh token but refresh was attempted
            return Err(AuthError::InvalidToken);
        }

        // 3. Get user
        let user_id = &claims.sub;
        let user = self.get_user(user_id).await?;

        if !user.is_active {
            return Err(AuthError::InvalidCredentials);
        }

        // 4. Generate new tokens (Rotate)
        let (new_access_token, new_refresh_token, expires_in, refresh_expires_in) = {
            let config = self
                .config
                .read()
                .map_err(|_| AuthError::Internal("Config lock poisoned".to_string()))?;
            let access_token = self.create_access_token(&user, session_id, &config)?;
            let refresh_token = self.create_refresh_token(&user, session_id, &config)?;
            (
                access_token,
                refresh_token,
                config.jwt.expiration.as_secs(),
                config.jwt.refresh_expiration.as_secs(),
            )
        };

        // 5. Update session
        session.token = new_access_token.clone();
        session.refresh_token = Some(new_refresh_token.clone());
        session.last_accessed = chrono::Utc::now();
        // Extend session expiration
        session.expires_at =
            chrono::Utc::now() + chrono::Duration::seconds(refresh_expires_in as i64);

        self.store_session(&session, &user.id.to_string()).await?;

        Ok(AuthToken {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
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
            policies: std::collections::HashSet::new(),
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
        metadata: Option<HashMap<String, String>>,
    ) -> Result<User, AuthError> {
        let mut user = self.get_user(user_id).await?;

        if let Some(email) = email {
            user.email = email;
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

        // Check for pattern match (e.g., "engine:*" matches "engine:read")
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

    /// Get user effective policies (permissions)
    pub async fn get_user_policies(&self, user: &User) -> Result<Vec<String>, AuthError> {
        let mut all_permissions = Vec::new();

        for role_name in &user.roles {
            match self.get_role(role_name).await {
                Ok(role) => {
                    all_permissions.extend(role.permissions);
                }
                Err(_) => {
                    // Ignore missing roles to allow partial success
                    // This can happen if a role was deleted but the user still has it assigned
                    continue;
                }
            }
        }

        all_permissions.sort();
        all_permissions.dedup();

        Ok(all_permissions)
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
                Some("Regular user with basic engine access"),
                vec![
                    "engine:read".to_string(),
                    "engine:write".to_string(),
                    "engine:list".to_string(),
                ],
            )
            .await?;
        }

        // Create viewer role
        if self.get_role("viewer").await.is_err() {
            self.create_role(
                "viewer",
                Some("Read-only access to engine"),
                vec!["engine:read".to_string(), "engine:list".to_string()],
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
            .map_err(|e| AuthError::Internal(format!("Failed to create TOTP instance: {}", e)))?;

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
            let config = self
                .config
                .read()
                .map_err(|_| AuthError::Internal("Config lock poisoned".to_string()))?;

            // Derive key using PBKDF2 with user ID as salt
            let key = secreton_crypto::key_derivation::derive_key_pbkdf2(
                config.jwt.secret.as_bytes(),
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

        let entry = {
            let config = self
                .config
                .read()
                .map_err(|_| AuthError::Internal("Config lock poisoned".to_string()))?;

            // Derive key
            let key = secreton_crypto::key_derivation::derive_key_pbkdf2(
                config.jwt.secret.as_bytes(),
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

            SecretEntry::new(
                path,
                encrypted.ciphertext,
                serde_json::json!({
                    "nonce": hex::encode(encrypted.nonce),
                    "algorithm": "Aes256Gcm"
                }),
                SecurityLevel::Secret,
                "system".to_string(),
            )
        };

        self.storage
            .store(&entry)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Create access token (JWT)
    fn create_access_token(
        &self,
        user: &User,
        session_id: &str,
        config: &AuthConfig,
    ) -> Result<String, AuthError> {
        let now = chrono::Utc::now();
        let expiration = now + chrono::Duration::seconds(config.jwt.expiration.as_secs() as i64);

        let mut roles: Vec<String> = user.roles.iter().cloned().collect();
        roles.sort();

        let claims = Claims {
            sub: user.id.to_string(),
            iss: config.jwt.issuer.clone(),
            aud: config.jwt.audience.clone(),
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: session_id.to_string(),
            roles,
            token_type: "access".to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            full_name: user.full_name.clone(),
            is_superuser: user.is_superuser,
            is_active: user.is_active,
            mfa_enabled: user.mfa_enabled,
            namespace: user.namespace.clone(),
        };

        let algorithm = Algorithm::from_str(&config.jwt.algorithm)
            .map_err(|e| AuthError::Configuration(format!("Invalid JWT algorithm: {}", e)))?;

        let header = Header::new(algorithm);

        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGeneration(e.to_string()))
    }

    /// Create refresh token
    fn create_refresh_token(
        &self,
        user: &User,
        session_id: &str,
        config: &AuthConfig,
    ) -> Result<String, AuthError> {
        let now = chrono::Utc::now();
        let expiration =
            now + chrono::Duration::seconds(config.jwt.refresh_expiration.as_secs() as i64);

        let mut roles: Vec<String> = user.roles.iter().cloned().collect();
        roles.sort();

        let claims = Claims {
            sub: user.id.to_string(),
            iss: config.jwt.issuer.clone(),
            aud: config.jwt.audience.clone(),
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: session_id.to_string(),
            roles,
            token_type: "refresh".to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            full_name: user.full_name.clone(),
            is_superuser: user.is_superuser,
            is_active: user.is_active,
            mfa_enabled: user.mfa_enabled,
            namespace: user.namespace.clone(),
        };

        let algorithm = Algorithm::from_str(&config.jwt.algorithm)
            .map_err(|e| AuthError::Configuration(format!("Invalid JWT algorithm: {}", e)))?;

        let header = Header::new(algorithm);

        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
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
    async fn store_session(&self, session: &Session, user_uuid: &str) -> Result<(), AuthError> {
        let session_bytes = serde_json::to_vec(session)
            .map_err(|e| AuthError::Internal(format!("Failed to serialize session: {}", e)))?;

        let encrypted = self
            .crypto
            .encrypt_simple(&session_bytes)
            .map_err(|e| AuthError::Internal(format!("Failed to encrypt session: {}", e)))?;

        let entry = SecretEntry::new(
            format!("auth/sessions/{}", session.id),
            encrypted,
            serde_json::json!({"method": "simple", "type": "session"}),
            SecurityLevel::Internal,
            user_uuid.to_string(),
        )
        .add_metadata("user_id".to_string(), serde_json::json!(user_uuid))
        .add_metadata("username".to_string(), serde_json::json!(session.user_id))
        .with_expiration(session.expires_at);

        self.storage
            .store(&entry)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to store session: {}", e)))?;
        Ok(())
    }

    /// Revoke session
    pub async fn revoke_session(&self, session_id: &str) -> Result<(), AuthError> {
        let path = format!("auth/sessions/{}", session_id);

        self.storage
            .delete_by_path(&path)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to revoke session: {}", e)))?;

        Ok(())
    }

    /// Revoke token (invalidate session)
    pub async fn revoke_token(&self, token: &str) -> Result<(), AuthError> {
        use jsonwebtoken::{DecodingKey, Validation, decode};

        let jti = {
            let config = self
                .config
                .read()
                .map_err(|_| AuthError::Internal("Config lock poisoned".to_string()))?;

            let mut validation =
                Validation::new(Algorithm::from_str(&config.jwt.algorithm).map_err(|e| {
                    AuthError::Configuration(format!("Invalid JWT algorithm: {}", e))
                })?);
            validation.set_audience(&[&config.jwt.audience]);
            validation.set_issuer(&[&config.jwt.issuer]);
            validation.validate_exp = false; // Allow revoking expired tokens

            let decoding_key = DecodingKey::from_secret(config.jwt.secret.as_bytes());

            let token_data = decode::<Claims>(token, &decoding_key, &validation)
                .map_err(|_| AuthError::InvalidToken)?;

            token_data.claims.jti
        };

        self.revoke_session(&jti).await
    }

    /// Get token ID (jti) from token
    pub fn get_token_id(&self, token: &str) -> Result<String, AuthError> {
        use jsonwebtoken::{DecodingKey, Validation, decode};

        let config = self
            .config
            .read()
            .map_err(|_| AuthError::Internal("Config lock poisoned".to_string()))?;

        let mut validation = Validation::new(
            Algorithm::from_str(&config.jwt.algorithm)
                .map_err(|e| AuthError::Configuration(format!("Invalid JWT algorithm: {}", e)))?,
        );
        validation.set_audience(&[&config.jwt.audience]);
        validation.set_issuer(&[&config.jwt.issuer]);
        validation.validate_exp = false; // Allow expired tokens to check ID

        let decoding_key = DecodingKey::from_secret(config.jwt.secret.as_bytes());

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims.jti)
    }

    /// List user sessions
    pub async fn list_user_sessions(&self, user_id: &str) -> Result<Vec<Session>, AuthError> {
        let mut params = QueryParams::new().with_path_prefix("auth/sessions/".to_string());
        params
            .metadata_filters
            .insert("user_id".to_string(), user_id.to_string());

        let entries = self
            .storage
            .list(&params)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to list sessions: {}", e)))?;

        let mut sessions = Vec::new();
        for entry in entries {
            let decrypted_bytes = self
                .crypto
                .decrypt_simple(&entry.encrypted_data)
                .map_err(|e| AuthError::Internal(format!("Failed to decrypt session: {}", e)))?;
            let session: Session = serde_json::from_slice(&decrypted_bytes).map_err(|e| {
                AuthError::Internal(format!("Failed to deserialize session: {}", e))
            })?;
            sessions.push(session);
        }
        Ok(sessions)
    }

    /// Get session
    pub async fn get_session(&self, session_id: &str) -> Result<Session, AuthError> {
        let path = format!("auth/sessions/{}", session_id);

        match self
            .storage
            .get_by_path(&path)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
        {
            Some(entry) => {
                let decrypted_bytes =
                    self.crypto
                        .decrypt_simple(&entry.encrypted_data)
                        .map_err(|e| {
                            AuthError::Internal(format!("Failed to decrypt session: {}", e))
                        })?;

                let session: Session = serde_json::from_slice(&decrypted_bytes).map_err(|e| {
                    AuthError::Internal(format!("Failed to deserialize session: {}", e))
                })?;

                Ok(session)
            }
            None => Err(AuthError::Internal("Session not found".to_string())),
        }
    }

    /// Update last login timestamp
    async fn update_last_login(&self, user_id: &str) -> Result<(), AuthError> {
        // Retrieve user
        match self.get_user(user_id).await {
            Ok(mut user) => {
                user.last_login = Some(chrono::Utc::now());

                // We don't have a direct update_user method that takes full user struct exposed in this impl block
                // (only partial update). But we have store_user.
                self.store_user(&user).await
            }
            Err(_) => Ok(()), // Ignore if user not found (shouldn't happen during login)
        }
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

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;
    use crate::config::AuthConfig;
    use secreton_crypto::{CryptoEngine, SecurityParams};
    use secreton_storage::MemoryBackend;

    #[tokio::test]
    async fn test_auth_service_creation() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let config = AuthConfig::default();

        let auth_service = AuthService::new(storage, crypto, &config).await;
        assert!(auth_service.is_ok());
    }

    #[test]
    fn test_password_hashing() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let config = AuthConfig::default();
        let auth_service = AuthService {
            storage,
            crypto,
            config: Arc::new(RwLock::new(config)),
        };

        let password = "test_password";
        let hash = auth_service.hash_password(password).expect("hash password");
        assert!(
            auth_service
                .verify_password(password, &hash)
                .unwrap_or(false)
        );
        assert!(
            !auth_service
                .verify_password("wrong_password", &hash)
                .unwrap_or(true)
        );
    }

    #[tokio::test]
    async fn test_authenticate_requires_mfa_code() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let mut config = AuthConfig::default();
        config.jwt.secret = "secret".into();
        let auth_service = AuthService::new(storage.clone(), crypto, &config)
            .await
            .expect("service");

        // Insert a user with MFA enabled by mocking storage behavior via store_user and get_user_by_username
        let password_hash = auth_service
            .hash_password("password")
            .expect("hash password failed");
        let user_id = Uuid::new_v4();
        let user = User {
            id: user_id,
            username: "alice".into(),
            email: "alice@example.com".into(),
            password_hash,
            full_name: None,
            is_active: true,
            is_superuser: false,
            mfa_enabled: true,
            last_login: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            roles: HashSet::new(),
            policies: HashSet::new(),
            namespace: "default".into(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
            metadata: HashMap::new(),
        };

        let _ = auth_service.store_user(&user).await;

        // Test missing code
        let result = auth_service
            .authenticate("alice", "password", None, "127.0.0.1", "test-agent")
            .await;
        assert!(matches!(result, Err(AuthError::MfaRequired)));

        // Test system inconsistency (MFA enabled but no secret)
        let result = auth_service
            .authenticate(
                "alice",
                "password",
                Some("123456"),
                "127.0.0.1",
                "test-agent",
            )
            .await;
        assert!(matches!(result, Err(AuthError::Internal(_))));

        // Setup real TOTP
        let totp_secret = Secret::Raw("JBSWY3DPEHPK3PXP".as_bytes().to_vec())
            .to_encoded()
            .to_string();
        auth_service
            .store_mfa_secret(&user_id.to_string(), &totp_secret)
            .await
            .expect("store secret");

        // Test invalid code
        let result = auth_service
            .authenticate(
                "alice",
                "password",
                Some("000000"),
                "127.0.0.1",
                "test-agent",
            )
            .await;
        assert!(matches!(result, Err(AuthError::InvalidMfaCode)));

        // Test valid code
        let totp = TOTP::new(
            TotpAlgorithm::SHA1,
            6,
            1,
            30,
            Secret::Raw(totp_secret.into_bytes()).to_bytes().unwrap(),
        )
        .unwrap();
        let code = totp.generate_current().unwrap();

        let result = auth_service
            .authenticate("alice", "password", Some(&code), "127.0.0.1", "test-agent")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_has_permission_with_wildcard_role() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let auth_service = AuthService::new(storage.clone(), crypto, &AuthConfig::default())
            .await
            .expect("service");

        let user = User {
            id: Uuid::new_v4(),
            username: "wildcard".into(),
            email: "wildcard@example.com".into(),
            password_hash: "".into(),
            full_name: None,
            is_active: true,
            is_superuser: false,
            mfa_enabled: false,
            last_login: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            roles: HashSet::from(["admin".to_string()]),
            policies: HashSet::new(),
            namespace: "default".into(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
            metadata: HashMap::new(),
        };

        // admin role already initialized in service with "*"
        let allowed = auth_service
            .has_permission(&user, "engine:delete")
            .await
            .expect("has permission");
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_initialize_default_roles_only_once() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let config = AuthConfig::default();

        // First creation initializes roles
        let service = AuthService::new(storage.clone(), crypto.clone(), &config)
            .await
            .expect("service");
        // Second creation should not fail if roles already exist
        let result = AuthService::new(storage, crypto, &config).await;
        assert!(result.is_ok());
        // Basic permission check still works
        let user = User {
            id: Uuid::new_v4(),
            username: "viewer".into(),
            email: "viewer@example.com".into(),
            password_hash: "".into(),
            full_name: None,
            is_active: true,
            is_superuser: false,
            mfa_enabled: false,
            last_login: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            roles: HashSet::from(["viewer".to_string()]),
            policies: HashSet::new(),
            namespace: "default".into(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
            metadata: HashMap::new(),
        };
        let allowed = service
            .has_permission(&user, "engine:read")
            .await
            .expect("permission");
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_create_access_token() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let mut config = AuthConfig::default();
        config.jwt.secret = "test_secret".to_string();
        config.jwt.issuer = "test_issuer".to_string();
        config.jwt.audience = "test_audience".to_string();

        let auth_service = AuthService::new(storage, crypto, &config)
            .await
            .expect("service");

        let user = User {
            id: Uuid::new_v4(),
            username: "test_token".into(),
            email: "token@example.com".into(),
            password_hash: "".into(),
            full_name: None,
            is_active: true,
            is_superuser: false,
            mfa_enabled: false,
            last_login: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            roles: HashSet::from(["user".to_string()]),
            policies: HashSet::new(),
            namespace: "default".into(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
            metadata: HashMap::new(),
        };

        let session_id = Uuid::new_v4().to_string();
        let token = auth_service
            .create_access_token(&user, &session_id, &config)
            .expect("create token");

        assert!(!token.is_empty());

        // Decode to verify
        use jsonwebtoken::{DecodingKey, Validation, decode};
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["test_audience"]);
        validation.set_issuer(&["test_issuer"]);

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret("test_secret".as_bytes()),
            &validation,
        )
        .expect("decode token");

        assert_eq!(token_data.claims.sub, user.id.to_string());
        assert_eq!(token_data.claims.jti, session_id);
        assert_eq!(token_data.claims.token_type, "access");
        assert!(token_data.claims.roles.contains(&"user".to_string()));
        assert_eq!(token_data.claims.username, user.username);
        assert_eq!(token_data.claims.email, user.email);

        // Verify validate_token works
        let validated_user = auth_service.validate_token(&token).await.expect("validate");
        assert_eq!(validated_user.id, user.id);
        assert_eq!(validated_user.username, "test_token");
        assert!(validated_user.has_role("user"));
    }

    #[tokio::test]
    async fn test_has_role() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let auth_service = AuthService::new(storage.clone(), crypto, &AuthConfig::default())
            .await
            .expect("service");

        let mut user = User {
            id: Uuid::new_v4(),
            username: "test_role".into(),
            email: "role@example.com".into(),
            password_hash: "".into(),
            full_name: None,
            is_active: true,
            is_superuser: false,
            mfa_enabled: false,
            last_login: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            roles: HashSet::new(),
            policies: HashSet::new(),
            namespace: "default".into(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
            metadata: HashMap::new(),
        };

        // Store user
        auth_service.store_user(&user).await.expect("store user");

        // Check no role
        let has_admin = auth_service
            .has_role(&user.id.to_string(), "admin")
            .await
            .expect("check role");
        assert!(!has_admin);

        // Add role
        user.roles.insert("admin".to_string());
        auth_service.store_user(&user).await.expect("update user");

        // Check role
        let has_admin = auth_service
            .has_role(&user.id.to_string(), "admin")
            .await
            .expect("check role");
        assert!(has_admin);

        // Check superuser
        user.is_superuser = true;
        user.roles.clear();
        auth_service.store_user(&user).await.expect("update user");

        let has_admin = auth_service
            .has_role(&user.id.to_string(), "admin")
            .await
            .expect("check role");
        assert!(has_admin);
        let has_random = auth_service
            .has_role(&user.id.to_string(), "random")
            .await
            .expect("check role");
        assert!(has_random);

        // Check root user
        let has_admin = auth_service
            .has_role(ROOT_USER_ID, "admin")
            .await
            .expect("check role");
        assert!(has_admin);
    }

    #[tokio::test]
    async fn test_role_persistence_and_policy_aggregation() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let auth_service = AuthService::new(storage.clone(), crypto, &AuthConfig::default())
            .await
            .expect("service");

        // 1. Create permissions and roles
        let role1_perms = vec!["perm1".to_string(), "perm2".to_string()];
        let role2_perms = vec!["perm2".to_string(), "perm3".to_string()];

        let role1 = auth_service
            .create_role("role1", None, role1_perms.clone())
            .await
            .expect("create role1");
        let _role2 = auth_service
            .create_role("role2", None, role2_perms.clone())
            .await
            .expect("create role2");

        // 2. Verify retrieval
        let retrieved_role1 = auth_service.get_role("role1").await.expect("get role1");
        assert_eq!(retrieved_role1.name, "role1");
        assert_eq!(retrieved_role1.permissions, role1_perms);

        // 3. Create user with these roles
        let user = User {
            id: Uuid::new_v4(),
            username: "policy_user".into(),
            email: "policy@example.com".into(),
            password_hash: "".into(),
            full_name: None,
            is_active: true,
            is_superuser: false,
            mfa_enabled: false,
            last_login: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            roles: HashSet::from(["role1".to_string(), "role2".to_string()]),
            policies: HashSet::new(),
            namespace: "default".into(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
            metadata: HashMap::new(),
        };

        // 4. Test policy aggregation
        let policies = auth_service
            .get_user_policies(&user)
            .await
            .expect("get user policies");

        // Should contain perm1, perm2, perm3 (deduplicated)
        assert_eq!(policies.len(), 3);
        assert!(policies.contains(&"perm1".to_string()));
        assert!(policies.contains(&"perm2".to_string()));
        assert!(policies.contains(&"perm3".to_string()));
    }

    #[tokio::test]
    async fn test_refresh_token_flow() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let mut config = AuthConfig::default();
        config.jwt.secret = "test_refresh_secret".to_string();
        config.jwt.expiration = std::time::Duration::from_secs(1); // Short access token
        config.jwt.refresh_expiration = std::time::Duration::from_secs(3600); // Long refresh

        let auth_service = AuthService::new(storage.clone(), crypto, &config)
            .await
            .expect("service");

        // Create user
        let user_id = Uuid::new_v4();
        let password_hash = auth_service.hash_password("password").expect("hash");
        let user = User {
            id: user_id,
            username: "refresh_user".into(),
            email: "refresh@example.com".into(),
            password_hash,
            full_name: None,
            is_active: true,
            is_superuser: false,
            mfa_enabled: false,
            last_login: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            roles: HashSet::from(["user".to_string()]),
            policies: HashSet::new(),
            namespace: "default".into(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
            metadata: HashMap::new(),
        };
        auth_service.store_user(&user).await.expect("store user");

        // Authenticate
        let token = auth_service
            .authenticate("refresh_user", "password", None, "127.0.0.1", "test-agent")
            .await
            .expect("authenticate");

        // Validate initial access token
        let validated = auth_service
            .validate_token(&token.access_token)
            .await
            .expect("validate");
        assert_eq!(validated.id, user_id);

        // Sleep to ensure timestamps change (optional, but good for verify)
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Note: We cannot easily test access token expiration here because jsonwebtoken
        // has a default leeway of 60 seconds which is not configurable in validate_token.
        // We proceed to test the refresh flow itself.

        // Refresh Token
        let new_token = auth_service
            .refresh_token(&token.refresh_token)
            .await
            .expect("refresh token");

        assert_ne!(new_token.access_token, token.access_token);
        assert_ne!(new_token.refresh_token, token.refresh_token);

        // Validate new access token
        let validated_new = auth_service
            .validate_token(&new_token.access_token)
            .await
            .expect("validate new");
        assert_eq!(validated_new.id, user_id);

        // Verify Reuse Detection (Old refresh token should fail)
        let reuse_result = auth_service.refresh_token(&token.refresh_token).await;
        assert!(matches!(reuse_result, Err(AuthError::InvalidToken)));

        // Verify Session Revocation after reuse attempt
        // The reuse attempt should have revoked the session.
        // So even the new refresh token should fail now.
        let subsequent_result = auth_service.refresh_token(&new_token.refresh_token).await;
        assert!(matches!(subsequent_result, Err(AuthError::InvalidToken)));
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
            config: Arc::new(RwLock::new(AuthConfig::default())),
        }
    }
}
