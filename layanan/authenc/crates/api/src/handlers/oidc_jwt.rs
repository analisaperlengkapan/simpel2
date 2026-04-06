//! OIDC ID Token generation and validation

use authenc_types::domain::user::User;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use base64ct::{Base64UrlUnpadded, Encoding};
use ed25519_dalek::{Signature, Signer, SigningKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct OidcIdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub auth_time: i64,
    pub nonce: Option<String>,
    pub preferred_username: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Ed25519JwtHeader {
    pub alg: String,
    pub typ: String,
    pub kid: String,
}

/// Generate an OIDC ID token for a user
///
/// The `signing_key` MUST be the same key used by `JwtService` so that
/// relying parties can verify the ID token using the JWKS endpoint.
pub fn generate_id_token(
    user: &User,
    client_id: &str,
    nonce: Option<String>,
    issuer: &str,
    signing_key: &SigningKey,
) -> String {
    let now = Utc::now().timestamp();

    let header = Ed25519JwtHeader {
        alg: "EdDSA".to_string(),
        typ: "JWT".to_string(),
        kid: "authence-ed25519-key".to_string(),
    };

    let name = match (&user.first_name, &user.last_name) {
        (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
        (Some(f), None) => Some(f.clone()),
        (None, Some(l)) => Some(l.clone()),
        _ => user.nama.clone(),
    };

    let claims = OidcIdTokenClaims {
        iss: issuer.to_string(),
        sub: user.id.to_string(),
        aud: client_id.to_string(),
        exp: now + 3600,
        iat: now,
        auth_time: now, // Simplification
        nonce,
        preferred_username: Some(user.username.clone()),
        email: Some(user.email.clone()),
        email_verified: Some(user.email_verified),
        name,
        given_name: user.first_name.clone(),
        family_name: user.last_name.clone(),
    };

    let header_json = serde_json::to_string(&header).unwrap();
    let claims_json = serde_json::to_string(&claims).unwrap();

    let header_b64 = Base64UrlUnpadded::encode_string(header_json.as_bytes());
    let payload_b64 = Base64UrlUnpadded::encode_string(claims_json.as_bytes());

    let signing_input = format!("{}.{}", header_b64, payload_b64);
    let signature: Signature = signing_key.sign(signing_input.as_bytes());
    let signature_b64 = Base64UrlUnpadded::encode_string(signature.to_bytes().as_ref());

    format!("{}.{}", signing_input, signature_b64)
}
