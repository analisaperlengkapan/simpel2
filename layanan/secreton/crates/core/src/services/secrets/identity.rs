//! Identity Secrets Engine with OIDC Provider
//!
//! Provides OpenID Connect Provider functionality for federated identity,
//! entity management, and group-based access control.
//!
//! # Features
//! - OIDC Provider (discovery, JWKS, token, userinfo)
//! - Entity management with aliases
//! - Group management with hierarchies
//! - Token generation and introspection
//! - Integration with existing IdentityService

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::utils::encoding::base64_encode;

/// Identity engine errors
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Alias not found: {0}")]
    AliasNotFound(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("OIDC error: {0}")]
    OidcError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// OIDC Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProviderConfig {
    /// Issuer URL (e.g., https://secreton.example.com)
    pub issuer: String,

    /// Supported signing algorithms
    pub signing_algorithms: Vec<String>,

    /// Token TTL (seconds)
    pub token_ttl: u64,

    /// ID token TTL (seconds)
    pub id_token_ttl: u64,

    /// Supported scopes
    pub scopes_supported: Vec<String>,

    /// Supported response types
    pub response_types_supported: Vec<String>,

    /// Supported grant types
    pub grant_types_supported: Vec<String>,

    /// JWKS rotation interval (seconds)
    pub jwks_rotation_interval: u64,
}

impl Default for OidcProviderConfig {
    fn default() -> Self {
        Self {
            issuer: "https://secreton.local".to_string(),
            signing_algorithms: vec!["RS256".to_string(), "ES256".to_string()],
            token_ttl: 3600,
            id_token_ttl: 600,
            scopes_supported: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
                "groups".to_string(),
            ],
            response_types_supported: vec!["code".to_string(), "id_token".to_string()],
            grant_types_supported: vec![
                "authorization_code".to_string(),
                "client_credentials".to_string(),
            ],
            jwks_rotation_interval: 86400, // 24 hours
        }
    }
}

/// OIDC Discovery document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub introspection_endpoint: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
}

/// JSON Web Key (JWK)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    pub kty: String,  // Key type (RSA, EC)
    pub use_: String, // Usage (sig, enc)
    #[serde(rename = "use")]
    pub use_field: String,
    pub kid: String,         // Key ID
    pub alg: String,         // Algorithm
    pub n: Option<String>,   // RSA modulus
    pub e: Option<String>,   // RSA exponent
    pub x: Option<String>,   // EC x coordinate
    pub y: Option<String>,   // EC y coordinate
    pub crv: Option<String>, // EC curve
}

/// JSON Web Key Set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

/// Identity token (ID Token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityToken {
    /// Issuer
    pub iss: String,

    /// Subject (entity ID)
    pub sub: String,

    /// Audience (client ID)
    pub aud: Vec<String>,

    /// Expiration time
    pub exp: i64,

    /// Issued at
    pub iat: i64,

    /// Nonce
    pub nonce: Option<String>,

    /// Authentication time
    pub auth_time: Option<i64>,

    /// Email
    pub email: Option<String>,

    /// Email verified
    pub email_verified: Option<bool>,

    /// Name
    pub name: Option<String>,

    /// Groups
    pub groups: Option<Vec<String>>,

    /// Entity metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token introspection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenIntrospection {
    pub active: bool,
    pub scope: Option<String>,
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub token_type: Option<String>,
    pub exp: Option<i64>,
    pub iat: Option<i64>,
    pub sub: Option<String>,
    pub aud: Option<Vec<String>>,
    pub iss: Option<String>,
}

/// UserInfo response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub groups: Option<Vec<String>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub scope: Option<String>,
}

/// Token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Stored token for tracking
#[derive(Debug, Clone)]
struct StoredToken {
    token_id: String,
    entity_id: String,
    client_id: String,
    scopes: Vec<String>,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

/// Identity Secrets Engine with OIDC Provider
pub struct IdentityEngine {
    config: Arc<RwLock<OidcProviderConfig>>,

    // Existing IdentityService integration
    identity_service: Arc<crate::services::identity::IdentityService>,

    // OIDC-specific state
    tokens: Arc<RwLock<HashMap<String, StoredToken>>>,
    authorization_codes: Arc<RwLock<HashMap<String, (String, DateTime<Utc>)>>>, // code -> (entity_id, expiry)
    jwks: Arc<RwLock<Jwks>>,
    current_kid: Arc<RwLock<String>>,
}

impl IdentityEngine {
    /// Create new identity engine
    pub fn new(identity_service: Arc<crate::services::identity::IdentityService>) -> Self {
        let config = OidcProviderConfig::default();
        let jwks = Self::generate_initial_jwks();
        let current_kid = jwks.keys.first().map(|k| k.kid.clone()).unwrap_or_default();

        Self {
            config: Arc::new(RwLock::new(config)),
            identity_service,
            tokens: Arc::new(RwLock::new(HashMap::new())),
            authorization_codes: Arc::new(RwLock::new(HashMap::new())),
            jwks: Arc::new(RwLock::new(jwks)),
            current_kid: Arc::new(RwLock::new(current_kid)),
        }
    }

    /// Configure OIDC provider
    #[instrument(skip(self))]
    pub async fn configure(&self, config: OidcProviderConfig) -> Result<(), IdentityError> {
        // Validate configuration
        if config.issuer.is_empty() {
            return Err(IdentityError::InvalidConfig(
                "Issuer is required".to_string(),
            ));
        }

        if config.token_ttl == 0 || config.token_ttl > 86400 {
            return Err(IdentityError::InvalidConfig(
                "Token TTL must be between 1 and 86400 seconds".to_string(),
            ));
        }

        let mut cfg = self.config.write().await;
        *cfg = config;

        info!("OIDC Provider configured successfully");
        Ok(())
    }

    /// Get OIDC discovery document
    pub async fn get_discovery(&self) -> OidcDiscovery {
        let config = self.config.read().await;

        OidcDiscovery {
            issuer: config.issuer.clone(),
            authorization_endpoint: format!("{}/v1/identity/oidc/authorize", config.issuer),
            token_endpoint: format!("{}/v1/identity/oidc/token", config.issuer),
            userinfo_endpoint: format!("{}/v1/identity/oidc/userinfo", config.issuer),
            jwks_uri: format!("{}/.well-known/jwks.json", config.issuer),
            introspection_endpoint: format!("{}/v1/identity/oidc/introspect", config.issuer),
            scopes_supported: config.scopes_supported.clone(),
            response_types_supported: config.response_types_supported.clone(),
            grant_types_supported: config.grant_types_supported.clone(),
            id_token_signing_alg_values_supported: config.signing_algorithms.clone(),
            subject_types_supported: vec!["public".to_string()],
            token_endpoint_auth_methods_supported: vec![
                "client_secret_basic".to_string(),
                "client_secret_post".to_string(),
            ],
        }
    }

    /// Get JWKS (JSON Web Key Set)
    pub async fn get_jwks(&self) -> Jwks {
        let jwks = self.jwks.read().await;
        jwks.clone()
    }

    /// Generate token from authorization code
    #[instrument(skip(self, request))]
    pub async fn generate_token(
        &self,
        request: TokenRequest,
    ) -> Result<TokenResponse, IdentityError> {
        let config = self.config.read().await;

        match request.grant_type.as_str() {
            "authorization_code" => {
                let code = request
                    .code
                    .ok_or_else(|| IdentityError::InvalidToken("Code required".to_string()))?;

                // Validate authorization code
                let entity_id = {
                    let mut codes = self.authorization_codes.write().await;
                    let (entity_id, expiry) = codes
                        .remove(&code)
                        .ok_or_else(|| IdentityError::InvalidToken("Invalid code".to_string()))?;

                    if Utc::now() > expiry {
                        return Err(IdentityError::TokenExpired);
                    }

                    entity_id
                };

                // Get entity
                let entity = self
                    .identity_service
                    .get_entity(&entity_id)
                    .await
                    .map_err(|e| IdentityError::EntityNotFound(e.to_string()))?;

                // Generate tokens
                let access_token = self
                    .generate_access_token(&entity, &request.client_id)
                    .await?;
                let id_token = self.generate_id_token(&entity, &request.client_id).await?;

                // Store token
                let token_id = Uuid::new_v4().to_string();
                let stored_token = StoredToken {
                    token_id: token_id.clone(),
                    entity_id: entity.id.clone(),
                    client_id: request.client_id.clone(),
                    scopes: request
                        .scope
                        .as_ref()
                        .map(|s| s.split_whitespace().map(String::from).collect())
                        .unwrap_or_default(),
                    issued_at: Utc::now(),
                    expires_at: Utc::now() + Duration::seconds(config.token_ttl as i64),
                };

                let mut tokens = self.tokens.write().await;
                tokens.insert(access_token.clone(), stored_token);

                Ok(TokenResponse {
                    access_token,
                    token_type: "Bearer".to_string(),
                    expires_in: config.token_ttl,
                    id_token: Some(id_token),
                    refresh_token: None,
                    scope: request.scope,
                })
            }
            "client_credentials" => {
                // Service account token generation
                Err(IdentityError::OidcError(
                    "Client credentials not yet implemented".to_string(),
                ))
            }
            _ => Err(IdentityError::OidcError(format!(
                "Unsupported grant type: {}",
                request.grant_type
            ))),
        }
    }

    /// Get user info from access token
    #[instrument(skip(self))]
    pub async fn get_userinfo(&self, access_token: &str) -> Result<UserInfo, IdentityError> {
        // Validate token
        let stored_token = {
            let tokens = self.tokens.read().await;
            tokens
                .get(access_token)
                .cloned()
                .ok_or_else(|| IdentityError::InvalidToken("Token not found".to_string()))?
        };

        // Check expiration
        if Utc::now() > stored_token.expires_at {
            return Err(IdentityError::TokenExpired);
        }

        // Get entity
        let entity = self
            .identity_service
            .get_entity(&stored_token.entity_id)
            .await
            .map_err(|e| IdentityError::EntityNotFound(e.to_string()))?;

        // Get groups
        let groups = self.get_entity_groups(&entity.id).await;

        Ok(UserInfo {
            sub: entity.id.clone(),
            name: Some(entity.name.clone()),
            email: entity.metadata.get("email").cloned(),
            email_verified: Some(true),
            groups: Some(groups),
            metadata: entity
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        })
    }

    /// Introspect token
    #[instrument(skip(self))]
    pub async fn introspect_token(&self, token: &str) -> TokenIntrospection {
        let tokens = self.tokens.read().await;

        if let Some(stored_token) = tokens.get(token) {
            let active = Utc::now() <= stored_token.expires_at;

            TokenIntrospection {
                active,
                scope: Some(stored_token.scopes.join(" ")),
                client_id: Some(stored_token.client_id.clone()),
                username: Some(stored_token.entity_id.clone()),
                token_type: Some("Bearer".to_string()),
                exp: Some(stored_token.expires_at.timestamp()),
                iat: Some(stored_token.issued_at.timestamp()),
                sub: Some(stored_token.entity_id.clone()),
                aud: Some(vec![stored_token.client_id.clone()]),
                iss: None,
            }
        } else {
            TokenIntrospection {
                active: false,
                scope: None,
                client_id: None,
                username: None,
                token_type: None,
                exp: None,
                iat: None,
                sub: None,
                aud: None,
                iss: None,
            }
        }
    }

    /// Create authorization code for entity
    pub async fn create_authorization_code(&self, entity_id: String) -> String {
        let code = Uuid::new_v4().to_string();
        let expiry = Utc::now() + Duration::minutes(5);

        let mut codes = self.authorization_codes.write().await;
        codes.insert(code.clone(), (entity_id, expiry));

        code
    }

    /// Generate access token (simplified JWT)
    async fn generate_access_token(
        &self,
        _entity: &crate::services::identity::Entity,
        _client_id: &str,
    ) -> Result<String, IdentityError> {
        // In production, this would generate a proper JWT
        // For now, return a unique token identifier
        Ok(format!("secreton_access_{}", Uuid::new_v4()))
    }

    /// Generate ID token (simplified JWT)
    async fn generate_id_token(
        &self,
        entity: &crate::services::identity::Entity,
        client_id: &str,
    ) -> Result<String, IdentityError> {
        let config = self.config.read().await;

        let token = IdentityToken {
            iss: config.issuer.clone(),
            sub: entity.id.clone(),
            aud: vec![client_id.to_string()],
            exp: (Utc::now() + Duration::seconds(config.id_token_ttl as i64)).timestamp(),
            iat: Utc::now().timestamp(),
            nonce: None,
            auth_time: Some(Utc::now().timestamp()),
            email: entity.metadata.get("email").cloned(),
            email_verified: Some(true),
            name: Some(entity.name.clone()),
            groups: Some(self.get_entity_groups(&entity.id).await),
            metadata: entity
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        };

        // In production, sign this with private key
        // For now, return base64-encoded JSON
        let json =
            serde_json::to_string(&token).map_err(|e| IdentityError::Internal(e.to_string()))?;
        Ok(format!("secreton_id_token.{}", base64_encode(json)))
    }

    /// Get entity groups
    async fn get_entity_groups(&self, _entity_id: &str) -> Vec<String> {
        // TODO: Query groups from IdentityService
        // For now, return empty
        Vec::new()
    }

    /// Generate initial JWKS
    fn generate_initial_jwks() -> Jwks {
        // Generate a sample RSA key
        let kid = Uuid::new_v4().to_string();

        Jwks {
            keys: vec![Jwk {
                kty: "RSA".to_string(),
                use_: "sig".to_string(),
                use_field: "sig".to_string(),
                kid,
                alg: "RS256".to_string(),
                n: Some("sample_modulus".to_string()),
                e: Some("AQAB".to_string()),
                x: None,
                y: None,
                crv: None,
            }],
        }
    }

    /// Rotate JWKS (should be called periodically)
    pub async fn rotate_jwks(&self) -> Result<(), IdentityError> {
        info!("Rotating JWKS");

        let new_jwks = Self::generate_initial_jwks();
        let new_kid = new_jwks
            .keys
            .first()
            .map(|k| k.kid.clone())
            .unwrap_or_default();

        let mut jwks = self.jwks.write().await;
        *jwks = new_jwks;

        let mut kid = self.current_kid.write().await;
        *kid = new_kid;

        Ok(())
    }

    /// Cleanup expired tokens and codes
    pub async fn cleanup_expired(&self) -> Result<usize, IdentityError> {
        let now = Utc::now();
        let mut count = 0;

        // Cleanup tokens
        {
            let mut tokens = self.tokens.write().await;
            tokens.retain(|_, token| {
                let keep = token.expires_at > now;
                if !keep {
                    count += 1;
                }
                keep
            });
        }

        // Cleanup authorization codes
        {
            let mut codes = self.authorization_codes.write().await;
            codes.retain(|_, (_, expiry)| {
                let keep = *expiry > now;
                if !keep {
                    count += 1;
                }
                keep
            });
        }

        if count > 0 {
            info!("Cleaned up {} expired tokens/codes", count);
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oidc_discovery() {
        let identity_service = Arc::new(crate::services::identity::IdentityService::new());
        let engine = IdentityEngine::new(identity_service);

        let discovery = engine.get_discovery().await;
        assert!(!discovery.issuer.is_empty());
        assert!(!discovery.token_endpoint.is_empty());
    }

    #[tokio::test]
    async fn test_jwks() {
        let identity_service = Arc::new(crate::services::identity::IdentityService::new());
        let engine = IdentityEngine::new(identity_service);

        let jwks = engine.get_jwks().await;
        assert!(!jwks.keys.is_empty());
    }

    #[tokio::test]
    async fn test_token_introspection_inactive() {
        let identity_service = Arc::new(crate::services::identity::IdentityService::new());
        let engine = IdentityEngine::new(identity_service);

        let result = engine.introspect_token("invalid_token").await;
        assert!(!result.active);
    }
}
