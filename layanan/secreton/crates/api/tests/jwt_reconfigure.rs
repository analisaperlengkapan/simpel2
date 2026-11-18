//! The container is built from the bootstrap config, which does not yet carry
//! the application's JWT secret — so `auth.jwt.secret` starts at the
//! compiled-in placeholder `change-this-secret-in-production`. The server then
//! loads the real application config, validates it, and must hand it to the
//! live services before serving; otherwise `validate_secrets` approves one key
//! while `AuthService::validate_token` and the seal handler keep signing and
//! verifying root tokens with the placeholder that is printed in the source
//! tree. Anyone who can reach the HTTP port forges a `token_type:"root"` token
//! and reads every secret.
//!
//! This drives the real `AuthService` (not a mock) through the same two steps
//! the binary performs, and pins both directions: the placeholder-signed token
//! is accepted while the placeholder is in force and rejected once the
//! validated key is adopted, and the validated-key token is accepted.

use std::sync::Arc;

use jsonwebtoken::{Algorithm, EncodingKey, Header};
use secreton_api::config::AuthConfig;
use secreton_api::services::auth::AuthService;
use secreton_crypto::CryptoEngine;
use secreton_storage::MemoryBackend;

/// Mirrors `secreton_api::config::JwtConfig::default()`; kept here so the test
/// fails loudly (a 401) if the default ever changes out from under it.
const PLACEHOLDER: &str = "change-this-secret-in-production";
const ISSUER: &str = "Secreton";
const AUDIENCE: &str = "secreton-api";

fn root_token(secret: &str) -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = serde_json::json!({
        "sub": "00000000-0000-0000-0000-000000000000",
        "iss": ISSUER,
        "aud": AUDIENCE,
        "exp": now + 3600,
        "iat": now,
        "jti": uuid::Uuid::new_v4().to_string(),
        "roles": ["root"],
        "policies": ["root"],
        "token_type": "root",
        "username": "root",
        "email": "root@secreton.local",
        "full_name": null,
        "is_superuser": true,
        "is_active": true,
        "mfa_enabled": false,
        "namespace": "root",
        "satker_code": null,
        "wilayah_code": null,
        "admin_level": null,
    });

    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("sign root token")
}

#[tokio::test]
async fn adopting_the_validated_secret_retires_the_placeholder() {
    let storage = Arc::new(MemoryBackend::new());
    let crypto = Arc::new(CryptoEngine::new());

    let placeholder = AuthConfig::default();
    assert_eq!(placeholder.jwt.secret, PLACEHOLDER);
    let auth = AuthService::new(storage, crypto, &placeholder)
        .await
        .expect("auth service");

    // While the bootstrap placeholder is in force, a placeholder-signed root
    // token is accepted — this is the vulnerable state `reconfigure` closes.
    assert!(
        auth.validate_token(&root_token(PLACEHOLDER)).await.is_ok(),
        "placeholder-signed root token must be accepted before reconfigure"
    );

    // Adopt the validated key (what `ServiceContainer::reconfigure` does).
    let mut validated = placeholder.clone();
    validated.jwt.secret = "ab".repeat(32);
    auth.update_config(validated);

    assert!(
        auth.validate_token(&root_token(PLACEHOLDER)).await.is_err(),
        "placeholder-signed root token must be rejected once the validated secret is adopted"
    );
    assert!(
        auth.validate_token(&root_token(&"ab".repeat(32)))
            .await
            .is_ok(),
        "validated-key token must be accepted after reconfigure"
    );
}
