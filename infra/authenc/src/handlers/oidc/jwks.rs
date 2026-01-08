use crate::app::AppState;
use crate::crypto::ed25519_keys::{Ed25519Jwk, get_ed25519_jwk};
use crate::error::AuthencError;
use crate::services::cache::Cache;
use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// JSON Web Key Set response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksResponse {
    /// Array of JSON Web Keys
    pub keys: Vec<Ed25519Jwk>,
}

/// Cache key for JWKS response
const JWKS_CACHE_KEY: &str = "jwks:response";

/// Cache TTL for JWKS response (1 hour)
const JWKS_CACHE_TTL_SECONDS: u64 = 3600;

/// JWKS endpoint at /.well-known/jwks.json
/// Provides JSON Web Key Set containing Ed25519 public keys for JWT signature verification.
/// Implements caching with 1-hour TTL for performance optimization.
/// # Features
/// - Exposes Ed25519 public keys in JWK format
/// - Supports multiple keys with key ID (kid) for key rotation
/// - Caches response for 1 hour to reduce computation
/// - Compatible with OIDC discovery specification
/// # Returns
/// JWKS document containing all active Ed25519 public keys
/// # Security Considerations
/// - Only exposes public keys for signature verification
/// - Private keys never leave the server
/// - Supports key rotation through multiple keys with unique kid
/// - Enables secure token validation by clients
/// - Cache invalidation on key rotation events
pub async fn jwks_endpoint(
    State(state): State<Arc<AppState>>,
) -> Result<Json<JwksResponse>, AuthencError> {
    // Try to get from cache first
    if let Some(redis_cache) = &state.redis_cache {
        if let Ok(Some(cached)) = redis_cache.get(JWKS_CACHE_KEY).await {
            if let Ok(jwks) = serde_json::from_value::<JwksResponse>(cached) {
                tracing::debug!("JWKS cache hit");
                return Ok(Json(jwks));
            }
        }
    }

    // Cache miss or no cache available - generate JWKS
    tracing::debug!("JWKS cache miss, generating fresh response");
    let jwks = generate_jwks_response();

    // Cache the response if Redis is available
    if let Some(redis_cache) = &state.redis_cache {
        let cache_value = serde_json::to_value(&jwks)
            .map_err(|e| AuthencError::internal(format!("Failed to serialize JWKS: {}", e)))?;

        if let Err(e) = redis_cache
            .set(
                JWKS_CACHE_KEY,
                &cache_value,
                std::time::Duration::from_secs(JWKS_CACHE_TTL_SECONDS),
            )
            .await
        {
            tracing::warn!("Failed to cache JWKS response: {}", e);
            // Continue without caching - not a critical error
        } else {
            tracing::debug!(
                "JWKS response cached for {} seconds",
                JWKS_CACHE_TTL_SECONDS
            );
        }
    }

    Ok(Json(jwks))
}

/// Generate JWKS response with all active keys
/// Currently returns the primary Ed25519 key. In the future, this will support
/// multiple keys for key rotation scenarios.
/// # Returns
/// JwksResponse containing all active public keys
fn generate_jwks_response() -> JwksResponse {
    // Get the primary Ed25519 key
    let primary_key = get_ed25519_jwk();

    // TODO: Add support for multiple keys during rotation
    // When implementing key rotation:
    // 1. Load all active keys from key store
    // 2. Include both current and previous keys
    // 3. Mark keys with appropriate kid for identification
    // 4. Clients can validate tokens with any active key

    JwksResponse {
        keys: vec![primary_key],
    }
}

/// Invalidate JWKS cache
/// Should be called when keys are rotated to ensure clients get fresh keys.
/// This is a utility function for key rotation service.
/// # Arguments
/// * `state` - Application state containing Redis cache
/// # Returns
/// Result indicating success or failure of cache invalidation
pub async fn invalidate_jwks_cache(state: &AppState) -> Result<(), AuthencError> {
    if let Some(redis_cache) = &state.redis_cache {
        redis_cache.delete(JWKS_CACHE_KEY).await.map_err(|e| {
            AuthencError::internal(format!("Failed to invalidate JWKS cache: {}", e))
        })?;
        tracing::info!("JWKS cache invalidated");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_jwks_response() {
        let jwks = generate_jwks_response();

        // Should have at least one key
        assert!(!jwks.keys.is_empty());

        // First key should be Ed25519
        let key = &jwks.keys[0];
        assert_eq!(key.kty, "OKP");
        assert_eq!(key.crv, "Ed25519");
        assert_eq!(key.alg, "EdDSA");
        assert_eq!(key.key_use, "sig");
        assert!(!key.x.is_empty());
        assert!(!key.kid.is_empty());
    }

    #[test]
    fn test_jwks_serialization() {
        let jwks = generate_jwks_response();
        let json = serde_json::to_string(&jwks).unwrap();

        // Should be valid JSON
        assert!(json.contains("\"keys\""));
        assert!(json.contains("\"kty\""));
        assert!(json.contains("\"OKP\""));
        assert!(json.contains("\"Ed25519\""));
    }

    #[test]
    fn test_jwks_deserialization() {
        let jwks = generate_jwks_response();
        let json = serde_json::to_string(&jwks).unwrap();
        let deserialized: JwksResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.keys.len(), jwks.keys.len());
        assert_eq!(deserialized.keys[0].kty, jwks.keys[0].kty);
        assert_eq!(deserialized.keys[0].crv, jwks.keys[0].crv);
    }
}
