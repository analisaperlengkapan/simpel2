//! Authentication service for user management and token validation.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::AuthConfig;
use secreton_crypto::CryptoEngine;
use secreton_storage::{SecurityLevel, StorageBackend, VaultEntry};

// Use canonical User from core
pub use secreton_core::models::User;

// Use consolidated AuthError from error module
pub use crate::error::AuthError;

const USER_PATH_PREFIX: &str = "auth/users";
const USERNAME_INDEX_PREFIX: &str = "auth/usernames";

/// User Data Transfer Object for storage persistence
///
/// This struct mirrors the core `User` model but ensures all fields,
/// specifically `password_hash`, are serialized for storage.
/// The core `User` model skips serialization of `password_hash` for API security.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserStorageDto {
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
}

impl From<&User> for UserStorageDto {
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
        }
    }
}

impl From<UserStorageDto> for User {
    fn from(stored: UserStorageDto) -> Self {
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
            namespace: stored.namespace,
            is_locked: stored.is_locked,
            failed_attempts: stored.failed_attempts,
            locked_until: stored.locked_until,
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
                self.verify_mfa_code(&user, code)?;
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
                + chrono::Duration::seconds(self.config.jwt.expiration.as_secs() as i64),
            last_accessed: chrono::Utc::now(),
        };

        self.store_session(&session).await?;

        // Update last login
        self.update_last_login(&user.id.to_string()).await?;

        Ok(AuthToken {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt.expiration.as_secs(),
            user,
        })
    }

    /// Validate access token
    pub async fn validate_token(&self, token: &str) -> Result<User, AuthError> {
        // TODO: Implement JWT token validation
        // 1. Parse JWT token
        // 2. Verify signature
        // 3. Check expiration
        // 4. Get user from token claims
        // 5. Verify session still exists

        // For now, return mock user
        Ok(User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: String::new(),
            full_name: Some("Test User".to_string()),
            is_active: true,
            is_superuser: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: Some(chrono::Utc::now()),
            mfa_enabled: false,
            roles: HashSet::new(),
            namespace: "default".to_string(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken, AuthError> {
        // TODO: Implement token refresh logic
        // 1. Validate refresh token
        // 2. Get user from token
        // 3. Generate new access token
        // 4. Optionally rotate refresh token

        Err(AuthError::Internal("Not implemented".to_string()))
    }

    /// Create user
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
        full_name: Option<&str>,
        roles: Vec<String>,
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
            is_active: true,
            is_superuser: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
            mfa_enabled: false,
            roles: roles.into_iter().collect(),
            namespace: "default".to_string(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
        };

        self.store_user(&user).await?;
        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<User, AuthError> {
        let uuid = Uuid::parse_str(user_id)
            .map_err(|_| AuthError::UserNotFound)?;

        let path = format!("auth/users/{}", uuid);
        let entry = self.storage.get_by_path(&path).await
            .map_err(|e| AuthError::Internal(format!("Failed to retrieve user: {}", e)))?;

        match entry {
            Some(entry) => self.decrypt_user(entry),
            None => Err(AuthError::UserNotFound),
        }
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<User, AuthError> {
        let path = format!("auth/usernames/{}", username);
        let index_entry = self.storage.get_by_path(&path).await
            .map_err(|e| AuthError::Internal(format!("Failed to retrieve user index: {}", e)))?;

        match index_entry {
            Some(entry) => {
                let uuid_bytes = self.crypto.decrypt_simple(&entry.encrypted_data)
                    .map_err(|e| AuthError::Internal(format!("Failed to decrypt user index: {}", e)))?;

                let uuid_str = String::from_utf8(uuid_bytes)
                    .map_err(|e| AuthError::Internal(format!("Invalid UUID string in index: {}", e)))?;

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

    /// Get role by name
    pub async fn get_role(&self, role_name: &str) -> Result<Role, AuthError> {
        // TODO: Implement role retrieval from storage
        Err(AuthError::Internal("Role not found".to_string()))
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
    fn verify_mfa_code(&self, user: &User, code: &str) -> Result<(), AuthError> {
        // TODO: Implement TOTP verification
        if code == "123456" {
            Ok(())
        } else {
            Err(AuthError::InvalidMfaCode)
        }
    }

    /// Create access token (JWT)
    fn create_access_token(&self, user: &User, session_id: &str) -> Result<String, AuthError> {
        // TODO: Implement JWT token creation
        Ok(format!("access_token_for_{}", user.username))
    }

    /// Create refresh token
    fn create_refresh_token(&self, user: &User, session_id: &str) -> Result<String, AuthError> {
        // TODO: Implement refresh token creation
        Ok(format!("refresh_token_for_{}", user.username))
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
        self.storage.store(&entry).await
            .map_err(|e| AuthError::Internal(format!("Failed to store user: {}", e)))?;

        // Store username index
        let uuid_bytes = user.id.to_string().into_bytes();
        let encrypted_uuid = self.crypto.encrypt_simple(&uuid_bytes)
            .map_err(|e| AuthError::Internal(format!("Failed to encrypt user index: {}", e)))?;

        let index_entry = VaultEntry::new(
            format!("auth/usernames/{}", user.username),
            encrypted_uuid,
            serde_json::json!({"method": "simple", "target": "user_id"}),
            SecurityLevel::Internal,
            "system".to_string(),
        );

        self.storage.store(&index_entry).await
            .map_err(|e| AuthError::Internal(format!("Failed to store user index: {}", e)))?;

        Ok(())
    }

    /// Store role in storage
    async fn store_role(&self, role: &Role) -> Result<(), AuthError> {
        // TODO: Implement role storage
        Ok(())
    }

    /// Store session
    async fn store_session(&self, session: &Session) -> Result<(), AuthError> {
        // TODO: Implement session storage
        Ok(())
    }

    /// Update last login timestamp
    async fn update_last_login(&self, user_id: &str) -> Result<(), AuthError> {
        // TODO: Implement last login update
        Ok(())
    }

    /// Encrypt user for storage
    fn encrypt_user(&self, user: &User) -> Result<VaultEntry, AuthError> {
        let stored_user = StoredUser::from(user);
        let user_bytes = serde_json::to_vec(&stored_user)
            .map_err(|e| AuthError::Internal(format!("Failed to serialize user: {}", e)))?;

        let encrypted_data = self.crypto.encrypt_simple(&user_bytes)
            .map_err(|e| AuthError::Internal(format!("Failed to encrypt user: {}", e)))?;

        let entry = VaultEntry::new(
            format!("auth/users/{}", user.id),
            encrypted_data,
            serde_json::json!({"method": "simple"}),
            SecurityLevel::Confidential,
            user.id.to_string(),
        );

        Ok(entry)
    }

    /// Decrypt user from storage
    fn decrypt_user(&self, entry: VaultEntry) -> Result<User, AuthError> {
        let decrypted_bytes = self.crypto.decrypt_simple(&entry.encrypted_data)
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
            config,
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
        let password_hash = auth_service.hash_password("password").unwrap_or_default();
        let user = User {
            id: Uuid::new_v4(),
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
            namespace: "default".into(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
        };

        auth_service.store_user(&user).await;

        let result = auth_service
            .authenticate("alice", "password", None, "127.0.0.1", "test-agent")
            .await;
        assert!(matches!(result, Err(AuthError::MfaRequired)));

        let result = auth_service
            .authenticate(
                "alice",
                "password",
                Some("123456"),
                "127.0.0.1",
                "test-agent",
            )
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
            roles: HashSet::new(),
            namespace: "default".into(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
        };

        // admin role already initialized in service with "*"
        let allowed = auth_service
            .has_permission(&user, "vault:delete")
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
            roles: HashSet::new(),
            namespace: "default".into(),
            is_locked: false,
            failed_attempts: 0,
            locked_until: None,
        };
        let allowed = service
            .has_permission(&user, "vault:read")
            .await
            .expect("permission");
        assert!(allowed);
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
