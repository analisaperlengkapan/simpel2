//! JWT generation and validation with Ed25519 signatures
//!
//! This module provides JWT token generation and validation using Ed25519
//! digital signatures for enhanced security and performance.
//!
//! # Features
//! - Ed25519 signature algorithm (preferred over RSA)
//! - Token generation with custom claims
//! - Token validation (signature, expiry, issuer)
//! - Integration with Secreton for key storage
//! - Configurable token lifetimes

use crate::{AuthencError, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Token Claims
// ============================================================================

/// JWT token claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Not before (Unix timestamp, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    /// JWT ID (unique identifier)
    pub jti: String,
    /// Scope (space-separated list of scopes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Realm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>,
    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    /// Custom claims
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl TokenClaims {
    /// Create a new TokenClaims with required fields
    pub fn new(sub: String, issuer: String, expires_in: Duration) -> Self {
        let now = Utc::now();
        let exp = (now + expires_in).timestamp();

        Self {
            sub,
            iss: issuer,
            aud: vec!["simpelv2".to_string()],
            exp,
            iat: now.timestamp(),
            nbf: None,
            jti: Uuid::new_v4().to_string(),
            scope: None,
            realm: None,
            sid: None,
            custom: HashMap::new(),
        }
    }

    /// Set audience
    pub fn with_audience(mut self, aud: Vec<String>) -> Self {
        self.aud = aud;
        self
    }

    /// Set scope
    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Set realm
    pub fn with_realm(mut self, realm: String) -> Self {
        self.realm = Some(realm);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, sid: String) -> Self {
        self.sid = Some(sid);
        self
    }

    /// Set not before time
    pub fn with_not_before(mut self, nbf: DateTime<Utc>) -> Self {
        self.nbf = Some(nbf.timestamp());
        self
    }

    /// Add a custom claim
    pub fn with_custom_claim(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom.insert(key, value);
        self
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp();
        self.exp < now
    }

    /// Check if token is not yet valid
    pub fn is_not_yet_valid(&self) -> bool {
        if let Some(nbf) = self.nbf {
            let now = Utc::now().timestamp();
            nbf > now
        } else {
            false
        }
    }
}

// ============================================================================
// JWT Service
// ============================================================================

/// JWT service for token generation and validation
///
/// Uses Ed25519 signatures for enhanced security and performance.
pub struct JwtService {
    /// Ed25519 signing key
    signing_key: SigningKey,
    /// Ed25519 verifying key (public key)
    verifying_key: VerifyingKey,
    /// Token issuer
    issuer: String,
    /// Default access token lifetime
    access_token_ttl: Duration,
    /// Default refresh token lifetime
    refresh_token_ttl: Duration,
}

impl JwtService {
    /// Create a new JwtService with the given signing key
    ///
    /// # Arguments
    /// * `signing_key_bytes` - 32-byte Ed25519 private key
    /// * `issuer` - Token issuer (e.g., "https://authenc.kejaksaan.go.id")
    /// * `access_token_ttl` - Access token lifetime (default: 15 minutes)
    /// * `refresh_token_ttl` - Refresh token lifetime (default: 7 days)
    pub fn new(
        signing_key_bytes: &[u8; 32],
        issuer: String,
        access_token_ttl: Duration,
        refresh_token_ttl: Duration,
    ) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(signing_key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
            issuer,
            access_token_ttl,
            refresh_token_ttl,
        })
    }

    /// Generate a new Ed25519 signing key (for initialization)
    ///
    /// Returns the 32-byte private key that should be stored securely in Secreton.
    pub fn generate_signing_key() -> [u8; 32] {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        signing_key.to_bytes()
    }

    /// Get the public key (verifying key) in base64 format
    ///
    /// This can be shared publicly for token verification.
    pub fn get_public_key_base64(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.verifying_key.as_bytes())
    }

    /// Get the issuer
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    /// Generate an access token
    ///
    /// # Arguments
    /// * `user_id` - User ID (subject)
    /// * `realm` - Realm name (optional)
    /// * `scope` - Space-separated scopes (optional)
    /// * `session_id` - Session ID (optional)
    pub fn generate_access_token(
        &self,
        user_id: &str,
        realm: Option<String>,
        scope: Option<String>,
        session_id: Option<String>,
    ) -> Result<String> {
        let mut claims = TokenClaims::new(
            user_id.to_string(),
            self.issuer.clone(),
            self.access_token_ttl,
        );

        if let Some(realm) = realm {
            claims = claims.with_realm(realm);
        }

        if let Some(scope) = scope {
            claims = claims.with_scope(scope);
        }

        if let Some(sid) = session_id {
            claims = claims.with_session_id(sid);
        }

        self.generate_token(&claims)
    }

    /// Generate a refresh token
    ///
    /// Refresh tokens have longer lifetime and limited scope.
    ///
    /// # Arguments
    /// * `user_id` - User ID (subject)
    /// * `session_id` - Session ID
    pub fn generate_refresh_token(&self, user_id: &str, session_id: &str) -> Result<String> {
        let claims = TokenClaims::new(
            user_id.to_string(),
            self.issuer.clone(),
            self.refresh_token_ttl,
        )
        .with_scope("refresh_token".to_string())
        .with_session_id(session_id.to_string());

        self.generate_token(&claims)
    }

    /// Generate a JWT token with custom claims
    pub fn generate_token(&self, claims: &TokenClaims) -> Result<String> {
        // Create JWT header
        let header = JwtHeader {
            alg: "EdDSA".to_string(),
            typ: "JWT".to_string(),
        };

        // Encode header and claims
        let header_json = serde_json::to_string(&header)
            .map_err(|e| AuthencError::crypto(format!("Failed to serialize header: {}", e)))?;
        let claims_json = serde_json::to_string(claims)
            .map_err(|e| AuthencError::crypto(format!("Failed to serialize claims: {}", e)))?;

        let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());
        let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json.as_bytes());

        // Create signing input
        let signing_input = format!("{}.{}", header_b64, claims_b64);

        // Sign with Ed25519
        let signature = self.signing_key.sign(signing_input.as_bytes());
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        // Construct JWT
        let jwt = format!("{}.{}", signing_input, signature_b64);

        Ok(jwt)
    }

    /// Validate and decode a JWT token
    ///
    /// Performs the following validations:
    /// - Signature verification
    /// - Expiration check
    /// - Issuer validation
    /// - Not-before check (if present)
    pub fn verify_token(&self, token: &str) -> Result<TokenClaims> {
        // Split JWT into parts
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthencError::InvalidToken(
                "Invalid JWT format".to_string(),
            ));
        }

        let header_b64 = parts[0];
        let claims_b64 = parts[1];
        let signature_b64 = parts[2];

        // Verify signature
        let signing_input = format!("{}.{}", header_b64, claims_b64);
        let signature_bytes = URL_SAFE_NO_PAD
            .decode(signature_b64)
            .map_err(|e| {
                AuthencError::InvalidToken(format!("Invalid signature encoding: {}", e))
            })?;

        let signature = Signature::from_bytes(&signature_bytes.try_into().map_err(|_| {
            AuthencError::InvalidToken("Invalid signature length".to_string())
        })?);

        self.verifying_key
            .verify(signing_input.as_bytes(), &signature)
            .map_err(|e| AuthencError::InvalidToken(format!("Signature verification failed: {}", e)))?;

        // Decode claims
        let claims_json = URL_SAFE_NO_PAD.decode(claims_b64)
            .map_err(|e| AuthencError::InvalidToken(format!("Invalid claims encoding: {}", e)))?;

        let claims: TokenClaims = serde_json::from_slice(&claims_json)
            .map_err(|e| AuthencError::InvalidToken(format!("Invalid claims format: {}", e)))?;

        // Validate expiration
        if claims.is_expired() {
            return Err(AuthencError::TokenExpired);
        }

        // Validate not-before
        if claims.is_not_yet_valid() {
            return Err(AuthencError::InvalidToken(
                "Token not yet valid".to_string(),
            ));
        }

        // Validate issuer
        if claims.iss != self.issuer {
            return Err(AuthencError::InvalidToken(format!(
                "Invalid issuer: expected {}, got {}",
                self.issuer, claims.iss
            )));
        }

        Ok(claims)
    }

    /// Decode token without verification (for debugging only)
    ///
    /// **WARNING**: This does not verify the signature! Only use for debugging.
    pub fn decode_unverified(&self, token: &str) -> Result<TokenClaims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthencError::InvalidToken(
                "Invalid JWT format".to_string(),
            ));
        }

        let claims_b64 = parts[1];
        let claims_json = URL_SAFE_NO_PAD.decode(claims_b64)
            .map_err(|e| AuthencError::InvalidToken(format!("Invalid claims encoding: {}", e)))?;

        let claims: TokenClaims = serde_json::from_slice(&claims_json)
            .map_err(|e| AuthencError::InvalidToken(format!("Invalid claims format: {}", e)))?;

        Ok(claims)
    }
}

// ============================================================================
// JWT Header
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct JwtHeader {
    /// Algorithm (EdDSA for Ed25519)
    alg: String,
    /// Type (JWT)
    typ: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service() -> JwtService {
        let key_bytes = JwtService::generate_signing_key();
        JwtService::new(
            &key_bytes,
            "https://test.example.com".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap()
    }

    #[test]
    fn test_generate_and_verify_access_token() {
        let service = create_test_service();

        let token = service
            .generate_access_token(
                "user-123",
                Some("test-realm".to_string()),
                Some("openid profile".to_string()),
                Some("session-456".to_string()),
            )
            .unwrap();

        let claims = service.verify_token(&token).unwrap();

        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.iss, "https://test.example.com");
        assert_eq!(claims.realm, Some("test-realm".to_string()));
        assert_eq!(claims.scope, Some("openid profile".to_string()));
        assert_eq!(claims.sid, Some("session-456".to_string()));
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_generate_and_verify_refresh_token() {
        let service = create_test_service();

        let token = service
            .generate_refresh_token("user-123", "session-456")
            .unwrap();

        let claims = service.verify_token(&token).unwrap();

        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.scope, Some("refresh_token".to_string()));
        assert_eq!(claims.sid, Some("session-456".to_string()));
    }

    #[test]
    fn test_verify_invalid_signature() {
        let service1 = create_test_service();
        let service2 = create_test_service(); // Different key

        let token = service1
            .generate_access_token("user-123", None, None, None)
            .unwrap();

        let result = service2.verify_token(&token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthencError::InvalidToken(_)));
    }

    #[test]
    fn test_verify_expired_token() {
        let key_bytes = JwtService::generate_signing_key();
        let service = JwtService::new(
            &key_bytes,
            "https://test.example.com".to_string(),
            Duration::seconds(-1), // Already expired
            Duration::days(7),
        )
        .unwrap();

        let token = service
            .generate_access_token("user-123", None, None, None)
            .unwrap();

        // Wait a moment to ensure expiration
        std::thread::sleep(std::time::Duration::from_millis(100));

        let result = service.verify_token(&token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthencError::TokenExpired));
    }

    #[test]
    fn test_verify_invalid_issuer() {
        let service1 = create_test_service();

        let key_bytes = JwtService::generate_signing_key();
        let service2 = JwtService::new(
            &key_bytes,
            "https://different-issuer.com".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap();

        let token = service1
            .generate_access_token("user-123", None, None, None)
            .unwrap();

        let result = service2.verify_token(&token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthencError::InvalidToken(_)));
    }

    #[test]
    fn test_decode_unverified() {
        let service = create_test_service();

        let token = service
            .generate_access_token("user-123", None, None, None)
            .unwrap();

        let claims = service.decode_unverified(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
    }

    #[test]
    fn test_custom_claims() {
        let service = create_test_service();

        let mut claims = TokenClaims::new(
            "user-123".to_string(),
            service.issuer.clone(),
            Duration::minutes(15),
        );
        claims = claims.with_custom_claim(
            "role".to_string(),
            serde_json::json!("admin"),
        );
        claims = claims.with_custom_claim(
            "permissions".to_string(),
            serde_json::json!(["read", "write"]),
        );

        let token = service.generate_token(&claims).unwrap();
        let decoded = service.verify_token(&token).unwrap();

        assert_eq!(decoded.custom.get("role"), Some(&serde_json::json!("admin")));
        assert_eq!(
            decoded.custom.get("permissions"),
            Some(&serde_json::json!(["read", "write"]))
        );
    }

    #[test]
    fn test_public_key_export() {
        let service = create_test_service();
        let public_key = service.get_public_key_base64();
        assert!(!public_key.is_empty());
    }
}
