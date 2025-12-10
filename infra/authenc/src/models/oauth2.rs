use crate::error::AuthencError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OAuth2 client model with RFC 7591 Dynamic Client Registration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Client {
    /// Unique identifier for the OAuth2 client
    pub id: Uuid,
    /// The client identifier used in OAuth2 flows
    pub client_id: String,
    /// Hashed client secret for authentication
    pub client_secret_hash: String,
    /// Human-readable name of the client
    pub client_name: String,
    /// Type of client (confidential or public)
    pub client_type: String,
    /// Allowed redirect URIs for authorization responses
    pub redirect_uris: Vec<String>,
    /// OAuth2 scopes the client is allowed to request
    pub scopes: Vec<String>,
    /// Supported OAuth2 grant types
    pub grant_types: Vec<String>,
    /// Supported OAuth2 response types
    pub response_types: Vec<String>,
    /// Authentication method for token endpoint
    pub token_endpoint_auth_method: String,
    /// ID of the user who owns this client
    pub owner_id: Option<Uuid>,
    /// ID of the realm this client belongs to
    pub realm_id: Option<Uuid>,
    /// Whether the client is enabled
    pub enabled: bool,
    /// Timestamp when the client was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the client was last updated
    pub updated_at: DateTime<Utc>,
    /// Timestamp when the client was deleted (soft delete)
    pub deleted_at: Option<DateTime<Utc>>,

    // RFC 7591 Dynamic Client Registration metadata
    /// URL that references a logo for the client
    pub logo_uri: Option<String>,
    /// URL of the home page of the client
    pub client_uri: Option<String>,
    /// URL of the client's policy document
    pub policy_uri: Option<String>,
    /// URL of the client's terms of service
    pub tos_uri: Option<String>,
    /// URL for the client's JSON Web Key Set document
    pub jwks_uri: Option<String>,
    /// Client's JSON Web Key Set document
    pub jwks: Option<serde_json::Value>,
    /// URI using the https scheme to calculate pairwise subject identifiers
    pub sector_identifier_uri: Option<String>,
    /// Subject type (public or pairwise)
    pub subject_type: Option<String>,
    /// JWS algorithm for signing the ID Token
    pub id_token_signed_response_alg: Option<String>,
    /// JWE algorithm for encrypting the ID Token
    pub id_token_encrypted_response_alg: Option<String>,
    /// JWE encryption method for encrypting the ID Token
    pub id_token_encrypted_response_enc: Option<String>,
    /// JWS algorithm for signing UserInfo responses
    pub userinfo_signed_response_alg: Option<String>,
    /// JWE algorithm for encrypting UserInfo responses
    pub userinfo_encrypted_response_alg: Option<String>,
    /// JWE encryption method for encrypting UserInfo responses
    pub userinfo_encrypted_response_enc: Option<String>,
    /// JWS algorithm for signing request objects
    pub request_object_signing_alg: Option<String>,
    /// JWE algorithm for encrypting request objects
    pub request_object_encryption_alg: Option<String>,
    /// JWE encryption method for encrypting request objects
    pub request_object_encryption_enc: Option<String>,
    /// JWS algorithm for signing the JWT used to authenticate
    pub token_endpoint_auth_signing_alg: Option<String>,
    /// Default maximum authentication age in seconds
    pub default_max_age: Option<i32>,
    /// Whether the auth_time claim in the ID Token is required
    pub require_auth_time: Option<bool>,
    /// Default Authentication Context Class Reference values
    pub default_acr_values: Option<Vec<String>>,
    /// URI using the https scheme for initiating login
    pub initiate_login_uri: Option<String>,
    /// Array of request URIs that are pre-registered
    pub request_uris: Option<Vec<String>>,
    /// Application type (web or native)
    pub application_type: Option<String>,
    /// Array of contact email addresses
    pub contacts: Option<Vec<String>>,
    /// Timestamp when the client_id was issued
    pub client_id_issued_at: Option<DateTime<Utc>>,
    /// Timestamp when the client_secret expires
    pub client_secret_expires_at: Option<DateTime<Utc>>,
    /// Software identifier
    pub software_id: Option<String>,
    /// Software version
    pub software_version: Option<String>,
    /// Hashed registration access token for this client
    pub registration_access_token_hash: Option<String>,
}

/// OAuth2 authorization code model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2AuthorizationCode {
    /// Unique identifier for the authorization code
    pub id: Uuid,
    /// The authorization code value
    pub code: String,
    /// ID of the client that requested the code
    pub client_id: Uuid,
    /// ID of the user who authorized the code
    pub user_id: Uuid,
    /// Redirect URI used in the authorization request
    pub redirect_uri: String,
    /// OAuth2 scopes granted in the authorization
    pub scopes: Vec<String>,
    /// PKCE code challenge for security
    pub code_challenge: Option<String>,
    /// PKCE code challenge method (S256 or plain)
    pub code_challenge_method: Option<String>,
    /// Expiration timestamp of the code
    pub expires_at: DateTime<Utc>,
    /// Whether the code has been used
    pub used: bool,
    /// Timestamp when the code was created
    pub created_at: DateTime<Utc>,
}

/// OAuth2 access token model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2AccessToken {
    /// Unique identifier for the access token
    pub id: Uuid,
    /// Hashed access token value
    pub token_hash: String,
    /// Hashed refresh token value (if issued)
    pub refresh_token_hash: Option<String>,
    /// ID of the client that requested the token
    pub client_id: Uuid,
    /// ID of the user the token was issued for
    pub user_id: Option<Uuid>,
    /// OAuth2 scopes granted in the token
    pub scopes: Vec<String>,
    /// Expiration timestamp of the access token
    pub expires_at: DateTime<Utc>,
    /// Expiration timestamp of the refresh token
    pub refresh_expires_at: Option<DateTime<Utc>>,
    /// Whether the token has been revoked
    pub revoked: bool,
    /// Timestamp when the token was revoked
    pub revoked_at: Option<DateTime<Utc>>,
    /// Timestamp when the token was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the token was last used
    pub last_used_at: Option<DateTime<Utc>>,
}

/// OAuth2 client creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOAuth2ClientRequest {
    /// Human-readable name for the new client
    pub client_name: String,
    /// Type of client (confidential or public)
    pub client_type: String,
    /// Allowed redirect URIs for the client
    pub redirect_uris: Vec<String>,
    /// OAuth2 scopes the client can request
    pub scopes: Vec<String>,
    /// Supported OAuth2 grant types
    pub grant_types: Vec<String>,
    /// Supported OAuth2 response types
    pub response_types: Vec<String>,
    /// Authentication method for token endpoint (optional)
    pub token_endpoint_auth_method: Option<String>,
}

/// OAuth2 authorization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2AuthorizeRequest {
    /// The response type requested (code, token, id_token)
    pub response_type: String,
    /// The client identifier
    pub client_id: String,
    /// The redirect URI for the authorization response
    pub redirect_uri: Option<String>,
    /// The requested OAuth2 scope
    pub scope: Option<String>,
    /// Opaque value for state maintenance
    pub state: Option<String>,
    /// PKCE code challenge for security
    pub code_challenge: Option<String>,
    /// PKCE code challenge method (S256 or plain)
    pub code_challenge_method: Option<String>,
    /// Nonce for replay attack protection
    pub nonce: Option<String>,
    /// Prompt parameter to force re-authentication
    pub prompt: Option<String>,
    /// Maximum authentication age in seconds
    pub max_age: Option<i64>,
}

/// OAuth2 token request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2TokenRequest {
    /// The OAuth2 grant type
    pub grant_type: String,
    /// Authorization code for authorization_code grant
    pub code: Option<String>,
    /// Redirect URI used in authorization request
    pub redirect_uri: Option<String>,
    /// Client identifier
    pub client_id: Option<String>,
    /// Client secret for confidential clients
    pub client_secret: Option<String>,
    /// PKCE code verifier
    pub code_verifier: Option<String>,
    /// Refresh token for refresh_token grant
    pub refresh_token: Option<String>,
    /// Requested scope (subset of authorized scope)
    pub scope: Option<String>,
    /// Username for password grant
    pub username: Option<String>,
    /// Password for password grant
    pub password: Option<String>,
}

/// OAuth2 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    /// The access token issued to the client
    pub access_token: String,
    /// The type of the token (usually "Bearer")
    pub token_type: String,
    /// The lifetime of the access token in seconds
    pub expires_in: i64,
    /// The refresh token (if issued)
    pub refresh_token: Option<String>,
    /// The scope of the access token
    pub scope: Option<String>,
    /// The ID token for OpenID Connect flows
    pub id_token: Option<String>,
}

use crate::error::Result;

/// Converts a PostgreSQL database row into an OAuth2Client instance
/// This implementation extracts all OAuth2Client fields from a database row
/// and constructs a new OAuth2Client struct. All fields are required in the row.
impl TryFrom<tokio_postgres::Row> for OAuth2Client {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            client_id: row.try_get("client_id")?,
            client_secret_hash: row.try_get("client_secret_hash")?,
            client_name: row.try_get("client_name")?,
            client_type: row.try_get("client_type")?,
            redirect_uris: row.try_get("redirect_uris")?,
            scopes: row.try_get("scopes")?,
            grant_types: row.try_get("grant_types")?,
            response_types: row.try_get("response_types")?,
            token_endpoint_auth_method: row.try_get("token_endpoint_auth_method")?,
            owner_id: row.try_get("owner_id")?,
            realm_id: row.try_get("realm_id")?,
            enabled: row.try_get("enabled")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,

            // RFC 7591 metadata
            logo_uri: row.try_get("logo_uri").ok(),
            client_uri: row.try_get("client_uri").ok(),
            policy_uri: row.try_get("policy_uri").ok(),
            tos_uri: row.try_get("tos_uri").ok(),
            jwks_uri: row.try_get("jwks_uri").ok(),
            jwks: row.try_get("jwks").ok(),
            sector_identifier_uri: row.try_get("sector_identifier_uri").ok(),
            subject_type: row.try_get("subject_type").ok(),
            id_token_signed_response_alg: row.try_get("id_token_signed_response_alg").ok(),
            id_token_encrypted_response_alg: row.try_get("id_token_encrypted_response_alg").ok(),
            id_token_encrypted_response_enc: row.try_get("id_token_encrypted_response_enc").ok(),
            userinfo_signed_response_alg: row.try_get("userinfo_signed_response_alg").ok(),
            userinfo_encrypted_response_alg: row.try_get("userinfo_encrypted_response_alg").ok(),
            userinfo_encrypted_response_enc: row.try_get("userinfo_encrypted_response_enc").ok(),
            request_object_signing_alg: row.try_get("request_object_signing_alg").ok(),
            request_object_encryption_alg: row.try_get("request_object_encryption_alg").ok(),
            request_object_encryption_enc: row.try_get("request_object_encryption_enc").ok(),
            token_endpoint_auth_signing_alg: row.try_get("token_endpoint_auth_signing_alg").ok(),
            default_max_age: row.try_get("default_max_age").ok(),
            require_auth_time: row.try_get("require_auth_time").ok(),
            default_acr_values: row.try_get("default_acr_values").ok(),
            initiate_login_uri: row.try_get("initiate_login_uri").ok(),
            request_uris: row.try_get("request_uris").ok(),
            application_type: row.try_get("application_type").ok(),
            contacts: row.try_get("contacts").ok(),
            client_id_issued_at: row.try_get("client_id_issued_at").ok(),
            client_secret_expires_at: row.try_get("client_secret_expires_at").ok(),
            software_id: row.try_get("software_id").ok(),
            software_version: row.try_get("software_version").ok(),
            registration_access_token_hash: row.try_get("registration_access_token_hash").ok(),
        })
    }
}

/// Converts a PostgreSQL database row into an OAuth2AuthorizationCode instance
/// This implementation extracts all OAuth2AuthorizationCode fields from a database row
/// and constructs a new OAuth2AuthorizationCode struct. All fields are required in the row.
impl TryFrom<tokio_postgres::Row> for OAuth2AuthorizationCode {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            code: row.try_get("code")?,
            client_id: row.try_get("client_id")?,
            user_id: row.try_get("user_id")?,
            redirect_uri: row.try_get("redirect_uri")?,
            scopes: row.try_get("scopes")?,
            code_challenge: row.try_get("code_challenge")?,
            code_challenge_method: row.try_get("code_challenge_method")?,
            expires_at: row.try_get("expires_at")?,
            used: row.try_get("used")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

/// Converts a PostgreSQL database row into an OAuth2AccessToken instance
/// This implementation extracts all OAuth2AccessToken fields from a database row
/// and constructs a new OAuth2AccessToken struct. All fields are required in the row.
impl TryFrom<tokio_postgres::Row> for OAuth2AccessToken {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            token_hash: row.try_get("token_hash")?,
            refresh_token_hash: row.try_get("refresh_token_hash")?,
            client_id: row.try_get("client_id")?,
            user_id: row.try_get("user_id")?,
            scopes: row.try_get("scopes")?,
            expires_at: row.try_get("expires_at")?,
            refresh_expires_at: row.try_get("refresh_expires_at")?,
            revoked: row.try_get("revoked")?,
            revoked_at: row.try_get("revoked_at")?,
            created_at: row.try_get("created_at")?,
            last_used_at: row.try_get("last_used_at")?,
        })
    }
}

/// Client registration access token (RFC 7592)
/// Tokens issued for managing dynamically registered clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRegistrationToken {
    /// Unique identifier
    pub id: Uuid,
    /// Hashed token value
    pub token_hash: String,
    /// Associated client ID
    pub client_id: Uuid,
    /// Realm this token belongs to
    pub realm_id: Option<Uuid>,
    /// Token expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether the token has been revoked
    pub revoked: bool,
    /// Timestamp when revoked
    pub revoked_at: Option<DateTime<Utc>>,
    /// Timestamp when created
    pub created_at: DateTime<Utc>,
    /// Timestamp when last used
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Initial access token (RFC 7591)
/// Tokens used to protect the client registration endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialAccessToken {
    /// Unique identifier
    pub id: Uuid,
    /// Hashed token value
    pub token_hash: String,
    /// Realm this token belongs to
    pub realm_id: Option<Uuid>,
    /// Maximum number of registrations allowed
    pub count: i32,
    /// Remaining number of registrations
    pub remaining_count: i32,
    /// Token expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether the token has been revoked
    pub revoked: bool,
    /// Timestamp when revoked
    pub revoked_at: Option<DateTime<Utc>>,
    /// Timestamp when created
    pub created_at: DateTime<Utc>,
    /// User who created this token
    pub created_by: Option<Uuid>,
    /// Timestamp when last used
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Client registration policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRegistrationPolicy {
    /// Unique identifier
    pub id: Uuid,
    /// Realm this policy applies to
    pub realm_id: Uuid,
    /// Policy name
    pub name: String,
    /// Whether dynamic registration is allowed
    pub allow_dynamic_registration: bool,
    /// Whether initial access token is required
    pub require_initial_access_token: bool,
    /// Whether software statement is required
    pub require_software_statement: bool,
    /// Allowed redirect URI patterns (regex)
    pub allowed_redirect_uri_patterns: Option<Vec<String>>,
    /// Blocked redirect URI patterns (regex)
    pub blocked_redirect_uri_patterns: Option<Vec<String>>,
    /// Maximum number of redirect URIs
    pub max_redirect_uris: Option<i32>,
    /// Allowed scopes for registration
    pub allowed_scopes: Option<Vec<String>>,
    /// Default scopes if not specified
    pub default_scopes: Option<Vec<String>>,
    /// Allowed grant types
    pub allowed_grant_types: Option<Vec<String>>,
    /// Allowed response types
    pub allowed_response_types: Option<Vec<String>>,
    /// Whether HTTPS is required for redirect URIs
    pub require_https_redirect_uris: bool,
    /// Whether localhost redirects are allowed
    pub allow_localhost_redirect: bool,
    /// Client secret expiration in seconds
    pub client_secret_expires_in: Option<i32>,
    /// Registration token expiration in seconds
    pub registration_token_expires_in: Option<i32>,
    /// Whether the policy is enabled
    pub enabled: bool,
    /// Timestamp when created
    pub created_at: DateTime<Utc>,
    /// Timestamp when updated
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<tokio_postgres::Row> for ClientRegistrationToken {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            token_hash: row.try_get("token_hash")?,
            client_id: row.try_get("client_id")?,
            realm_id: row.try_get("realm_id")?,
            expires_at: row.try_get("expires_at")?,
            revoked: row.try_get("revoked")?,
            revoked_at: row.try_get("revoked_at")?,
            created_at: row.try_get("created_at")?,
            last_used_at: row.try_get("last_used_at")?,
        })
    }
}

impl TryFrom<tokio_postgres::Row> for InitialAccessToken {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            token_hash: row.try_get("token_hash")?,
            realm_id: row.try_get("realm_id")?,
            count: row.try_get("count")?,
            remaining_count: row.try_get("remaining_count")?,
            expires_at: row.try_get("expires_at")?,
            revoked: row.try_get("revoked")?,
            revoked_at: row.try_get("revoked_at")?,
            created_at: row.try_get("created_at")?,
            created_by: row.try_get("created_by")?,
            last_used_at: row.try_get("last_used_at")?,
        })
    }
}

impl TryFrom<tokio_postgres::Row> for ClientRegistrationPolicy {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            realm_id: row.try_get("realm_id")?,
            name: row.try_get("name")?,
            allow_dynamic_registration: row.try_get("allow_dynamic_registration")?,
            require_initial_access_token: row.try_get("require_initial_access_token")?,
            require_software_statement: row.try_get("require_software_statement")?,
            allowed_redirect_uri_patterns: row.try_get("allowed_redirect_uri_patterns")?,
            blocked_redirect_uri_patterns: row.try_get("blocked_redirect_uri_patterns")?,
            max_redirect_uris: row.try_get("max_redirect_uris")?,
            allowed_scopes: row.try_get("allowed_scopes")?,
            default_scopes: row.try_get("default_scopes")?,
            allowed_grant_types: row.try_get("allowed_grant_types")?,
            allowed_response_types: row.try_get("allowed_response_types")?,
            require_https_redirect_uris: row.try_get("require_https_redirect_uris")?,
            allow_localhost_redirect: row.try_get("allow_localhost_redirect")?,
            client_secret_expires_in: row.try_get("client_secret_expires_in")?,
            registration_token_expires_in: row.try_get("registration_token_expires_in")?,
            enabled: row.try_get("enabled")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// Software Statement Issuer (Trusted third-party for client registration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareStatementIssuer {
    /// Unique identifier
    pub id: Uuid,
    /// Human-readable name
    pub name: String,
    /// Issuer identifier (must match 'iss' claim in JWT)
    pub issuer: String,
    /// URL to fetch JWKS for signature verification
    pub jwks_uri: Option<String>,
    /// Embedded JWKS for signature verification
    pub jwks: Option<serde_json::Value>,
    /// Realm this issuer belongs to
    pub realm_id: Option<Uuid>,
    /// Whether this issuer is enabled
    pub enabled: bool,
    /// Timestamp when created
    pub created_at: DateTime<Utc>,
    /// Timestamp when updated
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<tokio_postgres::Row> for SoftwareStatementIssuer {
    type Error = AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            issuer: row.try_get("issuer")?,
            jwks_uri: row.try_get("jwks_uri")?,
            jwks: row.try_get("jwks")?,
            realm_id: row.try_get("realm_id")?,
            enabled: row.try_get("enabled")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
