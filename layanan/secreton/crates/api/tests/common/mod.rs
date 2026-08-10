//! Shared fixtures for the API integration tests.
//!
//! Both test binaries here drive the **real** router from `create_api_router`
//! against a real `ServiceContainer`, so both need the same two things: a token
//! the auth middleware will accept, and an unsealed engine. They lived as
//! copies in `policy_enforcement.rs` first; a second copy in
//! `no_inmemory_kv.rs` would have been the third hand-maintained duplicate of a
//! constant that must track `config::JwtConfig::default()`, which is how these
//! drift silently into passing for the wrong reason.
//!
//! Every item here is used by every test binary that includes this module. If
//! you add one that is not, `dead_code` will say so — do not silence it, either
//! use it or drop it.

use std::sync::Arc;

use jsonwebtoken::{EncodingKey, Header};
use secreton_api::ApiState;
use secreton_api::services::ServiceContainer;
use secreton_storage::MemoryBackend;

/// Mirrors `secreton_api::config::JwtConfig::default()` — the config
/// `ServiceContainer::new_mock` installs. Note this is the api crate's
/// `JwtConfig`, not the similarly named one in secreton-core, which carries
/// different defaults. If these drift the tests stop authenticating and fail on
/// a 401, which is loud rather than silent.
const JWT_SECRET: &str = "change-this-secret-in-production";
const JWT_ISSUER: &str = "Secreton";
const JWT_AUDIENCE: &str = "secreton-api";

/// Mint a bearer token carrying `policies`.
///
/// Built as raw JSON rather than through the private `Claims` struct so the
/// tests stay outside the crate and exercise it as a consumer would.
///
/// `token_type: "root"` is accepted by `AuthService::validate_token` without a
/// session lookup; that field governs *session* handling only and has no
/// bearing on the authorization decision, which reads `policies`.
pub fn token_with_policies(policies: &[&str]) -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = serde_json::json!({
        "sub": uuid::Uuid::new_v4().to_string(),
        "iss": JWT_ISSUER,
        "aud": JWT_AUDIENCE,
        "exp": now + 3600,
        "iat": now,
        "jti": uuid::Uuid::new_v4().to_string(),
        "roles": ["reader"],
        "policies": policies,
        "token_type": "root",
        "username": "test-caller",
        "email": "test-caller@example.test",
        "full_name": null,
        "is_superuser": false,
        "is_active": true,
        "mfa_enabled": false,
        "namespace": "default",
        "satker_code": null,
        "wilayah_code": null,
        "admin_level": null,
    });

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .expect("mint test token")
}

/// An unsealed engine on in-memory storage.
///
/// Unsealed matters: `seal_check_middleware` runs *outside* the policy check,
/// so a sealed engine would answer 503 for everything and assertions built on
/// top would pass for the wrong reason.
///
/// The pool handle deliberately points nowhere. The policy store is therefore
/// unreachable, which is the denial path worth exercising: "policies could not
/// be loaded" must still refuse rather than fall open. `dbname` is only needed
/// because deadpool rejects a config without one at construction time.
pub async fn unsealed_state() -> ApiState {
    let storage = Arc::new(MemoryBackend::new());

    let mut cfg = deadpool_postgres::Config::new();
    cfg.dbname = Some("secreton_api_tests_no_such_db".to_string());
    cfg.host = Some("127.0.0.1".to_string());
    cfg.port = Some(1);
    let pool = cfg
        .create_pool(None, tokio_postgres::NoTls)
        .expect("build pool handle");

    let services = ServiceContainer::new_mock(storage, pool);

    // Shamir threshold is 3 of 5 (SealConfig in `new_mock`), so feed three
    // shares. `unseal_with_share` takes raw bytes; the string-taking `unseal`
    // wants base64. The in-crate helper this was adapted from hex-encoded a
    // single share and discarded the error, so it never unsealed anything —
    // worth knowing before trusting any assertion built on top of it.
    let shares = services.seal.initialize().await.expect("initialize seal");
    for share in shares.iter().take(3) {
        let bytes = share.to_bytes().expect("serialize share");
        services
            .seal
            .unseal_with_share(&bytes)
            .await
            .expect("submit unseal share");
    }
    assert!(
        !services.seal.is_sealed().await,
        "engine must be unsealed or every assertion below passes on a 503"
    );

    ApiState {
        transit: secreton_api::TransitApiState {
            engine: Arc::new(secreton_crypto::transit::TransitEngine::new()),
            config: None,
            metrics: Default::default(),
        },
        pki: secreton_api::PkiApiState::default(),
        services: Arc::new(services),
        metrics: Default::default(),
        prometheus_handle: None,
    }
}
