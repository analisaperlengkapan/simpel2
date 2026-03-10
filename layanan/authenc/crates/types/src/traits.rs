//! Service traits for dependency injection
//!
//! This module defines the trait interfaces for all major services in Authenc,
//! enabling dependency injection and testability.

use async_trait::async_trait;

use crate::{
    domain,          // Domain models module
    domain_types::*, // Strongly-typed IDs (UserId, RealmId, etc.) and OAuth2 types
    result::Result,
};

// Explicitly import domain types to avoid ambiguity with legacy types
use crate::domain::oidc_client::OidcClient as DomainOidcClient;
use crate::domain::permission::Permission;
use crate::domain::realm::Realm;
use crate::domain::role::Role;
use crate::domain::session::Session;
use crate::domain::user::User;
use crate::domain::user::{CreateUserRequest, UpdateUserRequest};

// Use domain version of OidcClient (full model), but domain_types versions for OAuth2 types
type OidcClient = DomainOidcClient;

// ============================================================================
// Storage Traits
// ============================================================================

/// Trait for user storage operations
#[async_trait]
pub trait UserStore: Send + Sync {
    /// Get a user by ID
    async fn get_user(&self, id: UserId) -> Result<User>;

    /// Get a user by username
    async fn get_user_by_username(&self, username: &str, realm_id: RealmId) -> Result<User>;

    /// Get a user by email
    async fn get_user_by_email(&self, email: &str, realm_id: RealmId) -> Result<User>;

    /// Create a new user
    async fn create_user(&self, req: CreateUserRequest) -> Result<User>;

    /// Update a user
    async fn update_user(&self, id: UserId, req: UpdateUserRequest) -> Result<User>;

    /// Delete a user (soft delete)
    async fn delete_user(&self, id: UserId) -> Result<()>;

    /// List users in a realm with pagination
    async fn list_users(&self, realm_id: RealmId, offset: usize, limit: usize)
    -> Result<Vec<User>>;

    /// Check if a username exists in a realm
    async fn username_exists(&self, username: &str, realm_id: RealmId) -> Result<bool>;

    /// Check if an email exists in a realm
    async fn email_exists(&self, email: &str, realm_id: RealmId) -> Result<bool>;

    /// Count total users in a realm (for pagination)
    async fn count_users(&self, realm_id: RealmId) -> Result<i64> {
        let users = self.list_users(realm_id, 0, i64::MAX as usize).await?;
        Ok(users.len() as i64)
    }

    /// Search users by query string (username, email, nip, nama)
    async fn search_users(
        &self,
        realm_id: RealmId,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<User>> {
        // Default: fall back to list_users (no search)
        self.list_users(realm_id, offset, limit).await
    }

    /// Count users matching a search query
    async fn count_search_users(&self, realm_id: RealmId, query: &str) -> Result<i64> {
        let users = self
            .search_users(realm_id, query, 0, i64::MAX as usize)
            .await?;
        Ok(users.len() as i64)
    }
}

/// Trait for session storage operations
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a new session
    async fn create_session(&self, user_id: UserId) -> Result<Session>;

    /// Get a session by ID
    async fn get_session(&self, id: SessionId) -> Result<Option<Session>>;

    /// Update session last accessed time
    async fn update_last_accessed(&self, id: SessionId) -> Result<()>;

    /// Invalidate a session
    async fn invalidate_session(&self, id: SessionId) -> Result<()>;

    /// Invalidate all sessions for a user
    async fn invalidate_user_sessions(&self, user_id: UserId) -> Result<()>;

    /// List active sessions for a user
    async fn list_user_sessions(&self, user_id: UserId) -> Result<Vec<Session>>;

    /// Clean up expired sessions
    async fn cleanup_expired_sessions(&self) -> Result<usize>;
}

/// Trait for realm storage operations
#[async_trait]
pub trait RealmStore: Send + Sync {
    /// Get a realm by ID
    async fn get_realm(&self, id: RealmId) -> Result<Realm>;

    /// Get a realm by name
    async fn get_realm_by_name(&self, name: &str) -> Result<Realm>;

    /// Create a new realm
    async fn create_realm(&self, name: String, display_name: String) -> Result<Realm>;

    /// Update a realm
    async fn update_realm(
        &self,
        id: RealmId,
        display_name: Option<String>,
        enabled: Option<bool>,
    ) -> Result<Realm>;

    /// Delete a realm
    async fn delete_realm(&self, id: RealmId) -> Result<()>;

    /// List all realms
    async fn list_realms(&self) -> Result<Vec<Realm>>;

    /// Check if a realm name exists
    async fn realm_name_exists(&self, name: &str) -> Result<bool>;
}

/// Trait for OAuth2 client storage operations
#[async_trait]
pub trait ClientStore: Send + Sync {
    /// Get a client by ID
    async fn get_client(&self, id: ClientId) -> Result<OidcClient>;

    /// Get a client by client_id string
    async fn get_client_by_client_id(
        &self,
        client_id: &str,
        realm_id: RealmId,
    ) -> Result<OidcClient>;

    /// Create a new client
    async fn create_client(
        &self,
        client_id: String,
        name: String,
        is_public: bool,
        realm_id: RealmId,
    ) -> Result<OidcClient>;

    /// Update a client
    async fn update_client(
        &self,
        id: ClientId,
        name: Option<String>,
        redirect_uris: Option<Vec<String>>,
        allowed_scopes: Option<Vec<String>>,
        enabled: Option<bool>,
    ) -> Result<OidcClient>;

    /// Delete a client
    async fn delete_client(&self, id: ClientId) -> Result<()>;

    /// List clients in a realm
    async fn list_clients(&self, realm_id: RealmId) -> Result<Vec<OidcClient>>;

    /// Update client secret
    async fn update_client_secret(&self, id: ClientId, secret_hash: String) -> Result<()>;
}

// ============================================================================
// Service Traits
// ============================================================================

/// Trait for authentication service
#[async_trait]
pub trait AuthenticationService: Send + Sync {
    /// Authenticate a user with credentials
    async fn authenticate(&self, credentials: Credentials, realm_id: RealmId)
    -> Result<AuthResult>;

    /// Verify MFA code
    async fn verify_mfa(&self, user_id: UserId, code: String) -> Result<AuthResult>;

    /// Validate a session
    async fn validate_session(&self, session_id: SessionId) -> Result<User>;

    /// Logout (invalidate session)
    async fn logout(&self, session_id: SessionId) -> Result<()>;
}

/// Trait for password hashing
pub trait PasswordHasher: Send + Sync {
    /// Hash a password
    fn hash(&self, password: &str) -> Result<String>;

    /// Verify a password against a hash
    fn verify(&self, password: &str, hash: &str) -> Result<bool>;
}

/// Trait for brute force protection
#[async_trait]
pub trait BruteForceProtector: Send + Sync {
    /// Check if a username is locked
    async fn check(&self, username: &str) -> Result<()>;

    /// Record a failed login attempt
    async fn record_failure(&self, username: &str) -> Result<()>;

    /// Record a successful login (reset failure count)
    async fn record_success(&self, username: &str) -> Result<()>;

    /// Unlock a username
    async fn unlock(&self, username: &str) -> Result<()>;
}

/// Trait for token generation
pub trait TokenGenerator: Send + Sync {
    /// Generate an access token
    fn generate_access_token(&self, user_id: UserId, scope: &str) -> Result<String>;

    /// Generate a refresh token
    fn generate_refresh_token(&self, user_id: UserId) -> Result<String>;

    /// Validate a token and extract claims
    fn validate_token(&self, token: &str) -> Result<TokenClaims>;
}

/// Token claims
#[derive(Debug, Clone)]
pub struct TokenClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: Vec<String>,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Scope
    pub scope: String,
}

// ============================================================================
// OAuth2 Storage Traits
// ============================================================================

/// Trait for authorization code storage operations
#[async_trait]
pub trait AuthorizationCodeStore: Send + Sync {
    /// Store an authorization code
    async fn store_code(&self, code: AuthorizationCode) -> Result<()>;

    /// Get an authorization code by code value
    async fn get_code(&self, code: &str) -> Result<Option<AuthorizationCode>>;

    /// Mark an authorization code as used
    async fn mark_code_used(&self, code: &str) -> Result<()>;

    /// Delete expired authorization codes
    async fn cleanup_expired_codes(&self) -> Result<usize>;
}

/// Trait for refresh token storage operations
#[async_trait]
pub trait RefreshTokenStore: Send + Sync {
    /// Store a refresh token
    async fn store_token(&self, token: RefreshToken) -> Result<()>;

    /// Get a refresh token by token value
    async fn get_token(&self, token: &str) -> Result<Option<RefreshToken>>;

    /// Revoke a refresh token
    async fn revoke_token(&self, token: &str) -> Result<()>;

    /// Revoke all refresh tokens for a user
    async fn revoke_user_tokens(&self, user_id: UserId) -> Result<()>;

    /// Delete expired refresh tokens
    async fn cleanup_expired_tokens(&self) -> Result<usize>;
}

/// Trait for OAuth2 service
#[async_trait]
pub trait OAuth2Service: Send + Sync {
    /// Handle authorization request (Authorization Code flow)
    async fn authorize(&self, request: AuthorizationRequest) -> Result<AuthorizationResponse>;

    /// Handle token request (all grant types)
    async fn token(&self, request: TokenRequest) -> Result<TokenResponse>;

    /// Validate redirect URI against client configuration
    fn validate_redirect_uri(&self, client: &OidcClient, redirect_uri: &str) -> Result<()>;

    /// Validate scopes against client configuration
    fn validate_scopes(&self, client: &OidcClient, requested_scopes: &str) -> Result<()>;

    /// Verify PKCE code challenge
    fn verify_pkce(&self, code_verifier: &str, code_challenge: &str, method: &str) -> Result<()>;
}
