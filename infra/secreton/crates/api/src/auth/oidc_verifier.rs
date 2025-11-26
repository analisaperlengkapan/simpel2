//! OIDC Token Verifier for Secreton
//!
//! This module provides OpenID Connect (OIDC) token verification functionality
//! for Secreton to accept and verify JWTs issued by Authenc.

use crate::auth::{AuthError, Claims, TokenVerifier};
use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

/// JSON Web Key (from JWKS endpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    /// Key type (e.g., "RSA", "EC")
    pub kty: String,
    /// Key use (e.g., "sig" for signatures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#use: Option<String>,
    /// Key operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_ops: Option<Vec<String>>,
    /// Algorithm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg: Option<String>,
    /// Key ID
    pub kid: String,
    /// RSA modulus (base64url)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    /// RSA exponent (base64url)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
    /// EC curve
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crv: Option<String>,
    /// EC x coordinate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    /// EC y coordinate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
}

/// JSON Web Key Set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwkSet {
    pub keys: Vec<Jwk>,
}

/// OIDC verifier configuration
#[derive(Debug, Clone)]
pub struct OidcVerifierConfig {
    /// JWKS endpoint URL (e.g., "https://authenc.local/.well-known/jwks.json")
    pub jwks_url: String,
    /// Expected issuer
    pub issuer: String,
    /// Expected audience
    pub audience: String,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// HTTP client timeout in seconds
    pub http_timeout_seconds: u64,
}

impl Default for OidcVerifierConfig {
    fn default() -> Self {
        Self {
            jwks_url: "https://authenc.local/.well-known/jwks.json".to_string(),
            issuer: "authenc".to_string(),
            audience: "secreton".to_string(),
            cache_ttl_seconds: 3600, // 1 hour
            http_timeout_seconds: 10,
        }
    }
}

/// OIDC Token Verifier (RS256/ES256)
///
/// This verifier fetches public keys from Authenc's JWKS endpoint
/// and verifies tokens signed with RSA or ECDSA algorithms.
pub struct OidcVerifier {
    config: OidcVerifierConfig,
    http_client: reqwest::Client,
    /// Cached JWKS
    jwks_cache: Arc<RwLock<Option<(JwkSet, std::time::Instant)>>>,
}

impl OidcVerifier {
    /// Create a new OIDC verifier
    pub fn new(config: OidcVerifierConfig) -> Result<Self, AuthError> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.http_timeout_seconds))
            .build()
            .map_err(|e| {
                AuthError::Configuration(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self {
            config,
            http_client,
            jwks_cache: Arc::new(RwLock::new(None)),
        })
    }

    /// Fetch JWKS from the endpoint
    async fn fetch_jwks(&self) -> Result<JwkSet, AuthError> {
        debug!("Fetching JWKS from {}", self.config.jwks_url);

        let response = self
            .http_client
            .get(&self.config.jwks_url)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch JWKS: {}", e);
                AuthError::TokenValidation(format!("JWKS fetch failed: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(AuthError::TokenValidation(format!(
                "JWKS endpoint returned HTTP {}",
                response.status()
            )));
        }

        let jwks: JwkSet = response.json().await.map_err(|e| {
            error!("Failed to parse JWKS response: {}", e);
            AuthError::TokenValidation(format!("Invalid JWKS format: {}", e))
        })?;

        debug!("Successfully fetched {} keys", jwks.keys.len());
        Ok(jwks)
    }

    /// Get JWKS (from cache or fetch)
    async fn get_jwks(&self) -> Result<JwkSet, AuthError> {
        // Check cache first
        {
            let cache = self.jwks_cache.read().await;
            if let Some((jwks, cached_at)) = cache.as_ref() {
                let age = cached_at.elapsed().as_secs();
                if age < self.config.cache_ttl_seconds {
                    debug!("Using cached JWKS (age: {}s)", age);
                    return Ok(jwks.clone());
                }
            }
        }

        // Cache miss or expired, fetch new JWKS
        let jwks = self.fetch_jwks().await?;

        // Update cache
        {
            let mut cache = self.jwks_cache.write().await;
            *cache = Some((jwks.clone(), std::time::Instant::now()));
        }

        Ok(jwks)
    }

    /// Find a JWK by key ID
    async fn find_jwk(&self, kid: &str) -> Result<Jwk, AuthError> {
        let jwks = self.get_jwks().await?;

        jwks.keys.into_iter().find(|k| k.kid == kid).ok_or_else(|| {
            warn!("Key ID not found in JWKS: {}", kid);
            AuthError::TokenValidation(format!("Unknown key ID: {}", kid))
        })
    }

    /// Convert JWK to DecodingKey
    fn jwk_to_decoding_key(&self, jwk: &Jwk) -> Result<DecodingKey, AuthError> {
        match jwk.kty.as_str() {
            "RSA" => {
                let n = jwk.n.as_ref().ok_or_else(|| {
                    AuthError::TokenValidation("Missing RSA modulus (n)".to_string())
                })?;
                let e = jwk.e.as_ref().ok_or_else(|| {
                    AuthError::TokenValidation("Missing RSA exponent (e)".to_string())
                })?;

                DecodingKey::from_rsa_components(n, e)
                    .map_err(|e| AuthError::TokenValidation(format!("Invalid RSA key: {}", e)))
            }
            "EC" => {
                // ECDSA keys - we'll need the x and y coordinates
                let x = jwk.x.as_ref().ok_or_else(|| {
                    AuthError::TokenValidation("Missing EC x coordinate".to_string())
                })?;
                let y = jwk.y.as_ref().ok_or_else(|| {
                    AuthError::TokenValidation("Missing EC y coordinate".to_string())
                })?;

                // For now, construct PEM format for ES256
                // In production, use a proper ECDSA library
                let pem = format_ec_public_key_pem(x, y)?;
                DecodingKey::from_ec_pem(pem.as_bytes())
                    .map_err(|e| AuthError::TokenValidation(format!("Invalid EC key: {}", e)))
            }
            other => Err(AuthError::TokenValidation(format!(
                "Unsupported key type: {}",
                other
            ))),
        }
    }
}

#[async_trait]
impl TokenVerifier for OidcVerifier {
    async fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        // Decode header to get kid
        let header = decode_header(token)
            .map_err(|e| AuthError::TokenValidation(format!("Invalid token header: {}", e)))?;

        let kid = header
            .kid
            .ok_or_else(|| AuthError::TokenValidation("Token missing kid (key ID)".to_string()))?;

        // Get the matching JWK
        let jwk = self.find_jwk(&kid).await?;

        // Determine algorithm
        let algorithm = match jwk.alg.as_deref() {
            Some("RS256") => Algorithm::RS256,
            Some("RS384") => Algorithm::RS384,
            Some("RS512") => Algorithm::RS512,
            Some("ES256") => Algorithm::ES256,
            Some("ES384") => Algorithm::ES384,
            Some(other) => {
                return Err(AuthError::TokenValidation(format!(
                    "Unsupported algorithm: {}",
                    other
                )));
            }
            None => {
                // Default to RS256 for RSA, ES256 for EC
                match jwk.kty.as_str() {
                    "RSA" => Algorithm::RS256,
                    "EC" => Algorithm::ES256,
                    _ => {
                        return Err(AuthError::TokenValidation(
                            "Cannot determine algorithm from JWK".to_string(),
                        ));
                    }
                }
            }
        };

        // Convert JWK to decoding key
        let decoding_key = self.jwk_to_decoding_key(&jwk)?;

        // Create validation
        let mut validation = Validation::new(algorithm);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        // Decode and validate token
        decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
            warn!("Token validation failed: {}", e);
            AuthError::TokenValidation(format!("Token validation failed: {}", e))
        })
    }

    fn issuer(&self) -> &str {
        &self.config.issuer
    }

    fn audience(&self) -> &str {
        &self.config.audience
    }
}

/// Helper to format EC public key as PEM
/// This is a simplified version - in production, use a proper cryptography library
fn format_ec_public_key_pem(x: &str, y: &str) -> Result<String, AuthError> {
    // For ES256 (P-256), we need to construct the PEM
    // This is a placeholder - proper implementation would use a crypto library
    // to properly encode the EC point
    warn!("EC key formatting is simplified - consider using a proper crypto library");

    // Placeholder: return a basic structure
    // In production, properly encode the EC point as ASN.1 DER and wrap in PEM
    Err(AuthError::TokenValidation(
        "EC key support requires additional crypto library implementation".to_string(),
    ))
}

/// Multi-verifier that tries multiple verification strategies
///
/// This allows Secreton to accept tokens from multiple issuers
/// (e.g., both self-signed and Authenc tokens during migration)
pub struct MultiVerifier {
    verifiers: Vec<Box<dyn TokenVerifier>>,
}

impl MultiVerifier {
    pub fn new(verifiers: Vec<Box<dyn TokenVerifier>>) -> Self {
        Self { verifiers }
    }
}

#[async_trait]
impl TokenVerifier for MultiVerifier {
    async fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        let mut last_error = None;

        for verifier in &self.verifiers {
            match verifier.verify_token(token).await {
                Ok(token_data) => {
                    debug!("Token verified by {} verifier", verifier.issuer());
                    return Ok(token_data);
                }
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| AuthError::TokenValidation("No verifiers configured".to_string())))
    }

    fn issuer(&self) -> &str {
        "multi"
    }

    fn audience(&self) -> &str {
        "multi"
    }
}
