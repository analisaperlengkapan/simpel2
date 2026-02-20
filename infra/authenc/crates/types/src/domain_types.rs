//! Core domain types for Authenc
//!
//! This module defines the fundamental domain types used throughout the system,
//! including strongly-typed IDs and authentication results.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Strongly-Typed IDs
// ============================================================================

/// Unique identifier for a user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub Uuid);

impl UserId {
    /// Create a new random UserId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse UserId from string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Create from UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<UserId> for Uuid {
    fn from(id: UserId) -> Self {
        id.0
    }
}

/// Unique identifier for a realm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RealmId(pub Uuid);

impl RealmId {
    /// Create a new random RealmId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse RealmId from string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Create from UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for RealmId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RealmId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for RealmId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<RealmId> for Uuid {
    fn from(id: RealmId) -> Self {
        id.0
    }
}

/// Unique identifier for an OAuth2 client
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(pub Uuid);

impl ClientId {
    /// Create a new random ClientId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ClientId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<ClientId> for Uuid {
    fn from(id: ClientId) -> Self {
        id.0
    }
}

/// Unique identifier for a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub Uuid);

impl SessionId {
    /// Create a new random SessionId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse SessionId from string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Create from UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SessionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<SessionId> for Uuid {
    fn from(id: SessionId) -> Self {
        id.0
    }
}

/// Unique identifier for a role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoleId(pub Uuid);

impl RoleId {
    /// Create a new random RoleId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse RoleId from string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Create from UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for RoleId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for RoleId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<RoleId> for Uuid {
    fn from(id: RoleId) -> Self {
        id.0
    }
}

// ============================================================================
// Authentication Result Types
// ============================================================================

/// Result of an authentication attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthResult {
    /// Authentication succeeded
    Success {
        /// ID of the authenticated user
        user_id: UserId,
        /// ID of the created session
        session_id: SessionId,
    },
    /// Multi-factor authentication is required
    MfaRequired {
        /// ID of the user requiring MFA
        user_id: UserId,
        /// Temporary token for MFA verification
        mfa_token: String,
    },
    /// Authentication failed
    Failed {
        /// Reason for authentication failure
        reason: AuthFailureReason,
    },
}

/// Reasons why authentication might fail
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthFailureReason {
    /// Invalid username or password
    InvalidCredentials,
    /// User account is disabled
    UserDisabled,
    /// User account is locked due to brute force protection
    AccountLocked,
    /// Realm is disabled
    RealmDisabled,
    /// Internal error occurred
    InternalError(String),
}

impl std::fmt::Display for AuthFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "Invalid credentials"),
            Self::UserDisabled => write!(f, "User account is disabled"),
            Self::AccountLocked => write!(f, "Account is locked"),
            Self::RealmDisabled => write!(f, "Realm is disabled"),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

// ============================================================================
// Core Domain Models
// ============================================================================


// ============================================================================
// Authentication Types
// ============================================================================

/// User credentials for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Username
    pub username: String,
    /// Password (plaintext, will be hashed)
    pub password: String,
}

// ============================================================================
// OAuth2 Domain Types (simplified versions for traits)
// ============================================================================
// Note: Full OIDC client model is in domain::oidc_client
// This is a simplified version for trait definitions

/// OAuth2 client entity (simplified for traits)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcClient {
    /// Unique client ID
    pub id: ClientId,
    /// Client identifier string
    pub client_id: String,
    /// Client secret (hashed)
    pub client_secret_hash: Option<String>,
    /// Client name
    pub name: String,
    /// Whether this is a public client (no secret)
    pub is_public: bool,
    /// Allowed redirect URIs
    pub redirect_uris: Vec<String>,
    /// Allowed scopes
    pub allowed_scopes: Vec<String>,
    /// Realm this client belongs to
    pub realm_id: RealmId,
    /// Whether the client is enabled
    pub enabled: bool,
    /// When the client was created
    pub created_at: DateTime<Utc>,
    /// When the client was last updated
    pub updated_at: DateTime<Utc>,
}

/// OAuth2 authorization code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCode {
    /// The authorization code value
    pub code: String,
    /// Client ID this code was issued to
    pub client_id: String,
    /// User ID this code represents
    pub user_id: UserId,
    /// Redirect URI used in the authorization request
    pub redirect_uri: String,
    /// Requested scopes
    pub scope: String,
    /// PKCE code challenge (OAuth 2.1 requirement)
    pub code_challenge: String,
    /// PKCE code challenge method (S256 or plain)
    pub code_challenge_method: String,
    /// Realm ID
    pub realm_id: RealmId,
    /// When the code expires
    pub expires_at: DateTime<Utc>,
    /// When the code was created
    pub created_at: DateTime<Utc>,
    /// Whether the code has been used
    pub used: bool,
}

/// OAuth2 refresh token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    /// The refresh token value
    pub token: String,
    /// User ID this token represents
    pub user_id: UserId,
    /// Client ID this token was issued to
    pub client_id: String,
    /// Scopes granted
    pub scope: String,
    /// Realm ID
    pub realm_id: RealmId,
    /// When the token expires
    pub expires_at: DateTime<Utc>,
    /// When the token was created
    pub created_at: DateTime<Utc>,
    /// Whether the token has been revoked
    pub revoked: bool,
}

/// OAuth2 error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Error {
    /// Error code
    pub error: String,
    /// Error description
    pub error_description: Option<String>,
    /// Error URI
    pub error_uri: Option<String>,
}

impl OAuth2Error {
    /// Create an invalid_request error
    pub fn invalid_request(description: impl Into<String>) -> Self {
        Self {
            error: "invalid_request".to_string(),
            error_description: Some(description.into()),
            error_uri: None,
        }
    }

    /// Create an invalid_client error
    pub fn invalid_client(description: impl Into<String>) -> Self {
        Self {
            error: "invalid_client".to_string(),
            error_description: Some(description.into()),
            error_uri: None,
        }
    }

    /// Create an invalid_grant error
    pub fn invalid_grant(description: impl Into<String>) -> Self {
        Self {
            error: "invalid_grant".to_string(),
            error_description: Some(description.into()),
            error_uri: None,
        }
    }
}

/// OAuth2 authorization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    /// Response type (must be "code" for authorization code flow)
    pub response_type: String,
    /// Client ID
    pub client_id: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// Requested scopes (space-separated)
    pub scope: String,
    /// State parameter for CSRF protection
    pub state: Option<String>,
    /// PKCE code challenge
    pub code_challenge: String,
    /// PKCE code challenge method (S256 or plain)
    pub code_challenge_method: String,
    /// User ID (from authenticated session)
    pub user_id: UserId,
    /// Realm ID
    pub realm_id: RealmId,
}

/// OAuth2 authorization response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    /// Authorization code
    pub code: String,
    /// State parameter (echoed back)
    pub state: Option<String>,
}

/// OAuth2 token request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest {
    /// Grant type
    pub grant_type: String,
    /// Authorization code (for authorization_code grant)
    pub code: Option<String>,
    /// Redirect URI (must match authorization request)
    pub redirect_uri: Option<String>,
    /// PKCE code verifier
    pub code_verifier: Option<String>,
    /// Client ID
    pub client_id: String,
    /// Client secret (for confidential clients)
    pub client_secret: Option<String>,
    /// Refresh token (for refresh_token grant)
    pub refresh_token: Option<String>,
    /// Scope (for client_credentials grant)
    pub scope: Option<String>,
    /// Realm ID
    pub realm_id: RealmId,
}

/// OAuth2 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Access token
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Expires in (seconds)
    pub expires_in: i64,
    /// Refresh token (optional)
    pub refresh_token: Option<String>,
    /// Scope granted
    pub scope: String,
}
