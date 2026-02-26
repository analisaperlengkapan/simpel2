//! Helper types and functions for token exchange operations

use authenc_types::{AuthencError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Access Token Claims for JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenClaims {
    /// The issuer of the token
    pub iss: String,
    /// The subject (user) identifier
    pub sub: String,
    /// The audience (client) identifier
    pub aud: String,
    /// The client identifier
    pub client_id: String,
    /// The expiration time
    pub exp: i64,
    /// The issued at time
    pub iat: i64,
    /// The not before time
    pub nbf: i64,
    /// The JWT ID for uniqueness
    pub jti: String,
    /// The granted scope
    pub scope: Option<String>,
    /// The user's roles
    pub roles: Option<Vec<String>>,
    /// The user's groups
    pub groups: Option<Vec<String>>,
}

/// Generate a JWT access token from claims.
///
/// In the new architecture, real signing is handled by authenc-crypto's JwtService.
/// This function serializes claims to a base64-encoded JSON payload as a placeholder.
/// Actual JWT signing should be injected via a JwtService in the TokenExchangeService.
/// Decode JWT payload (without signature verification) and return AccessTokenClaims.
/// For full verification use JwtValidator from authenc-crypto.
pub fn verify_jwt_with_validation(token: &str) -> Result<AccessTokenClaims> {
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() != 3 {
        return Err(AuthencError::validation("Invalid JWT format"));
    }
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let payload = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| AuthencError::validation(format!("Failed to decode JWT payload: {e}")))?;
    serde_json::from_slice::<AccessTokenClaims>(&payload)
        .map_err(|e| AuthencError::validation(format!("Failed to parse JWT claims: {e}")))
}

pub fn generate_access_token(
    claims: &AccessTokenClaims,
    additional_claims: Option<&HashMap<String, serde_json::Value>>,
) -> String {
    let mut claims_map = serde_json::to_value(claims)
        .and_then(|v| serde_json::from_value::<HashMap<String, serde_json::Value>>(v))
        .unwrap_or_default();

    if let Some(additional) = additional_claims {
        claims_map.extend(additional.clone());
    }

    // Placeholder: return base64-encoded claims. Real implementation should
    // use authenc_crypto::JwtService injected into TokenExchangeService.
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let claims_json = serde_json::to_string(&claims_map).unwrap_or_default();
    format!(
        "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.{}.placeholder",
        URL_SAFE_NO_PAD.encode(claims_json.as_bytes())
    )
}
