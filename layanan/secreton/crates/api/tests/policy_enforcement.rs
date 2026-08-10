//! Does the REST boundary actually refuse a caller whose policies do not cover
//! the path it asked for?
//!
//! These tests drive the **real** router from [`create_api_router`], not a
//! hand-assembled one, because the defect they guard against was never in the
//! decision logic — `policy_check_middleware` was correct and complete from the
//! day it was written. The defect was that nothing mounted it. A test that
//! layers the middleware itself would have passed happily for the entire period
//! the boundary stood open (#129).
//!
//! Reference point for every assertion below: a call that gets *through* the
//! middleware reaches `get_secret`, which answers `404` for a path that was
//! never written. So `404` means "authorized and dispatched", and anything else
//! means the boundary stopped it — the same discriminator used by the mTLS
//! suite in `crates/grpc/tests/mtls_enforcement.rs`.
//!
//! No database is involved. Tokens are minted with `token_type: "root"`, which
//! `AuthService::validate_token` accepts without a session lookup; that field
//! governs *session* handling only and has no bearing on the authorization
//! decision under test, which reads `policies`.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use jsonwebtoken::{EncodingKey, Header};
use secreton_api::services::ServiceContainer;
use secreton_api::{ApiState, create_api_router};
use secreton_storage::MemoryBackend;
use tower::ServiceExt;

/// Mirrors `crate::config::JwtConfig::default()` — the config
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
/// test stays outside the crate and exercises it as a consumer would.
fn token_with_policies(policies: &[&str]) -> String {
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
/// so a sealed engine would answer 503 for everything and every assertion below
/// would pass for the wrong reason.
async fn unsealed_state() -> ApiState {
    let storage = Arc::new(MemoryBackend::new());

    // A pool handle that will never connect. Deliberate: the policy store is
    // unreachable in these tests, so the denial path exercised is "policies
    // could not be loaded", and the assertion below insists that still refuses
    // rather than falling open. `dbname` is only needed because deadpool
    // rejects a config without one at construction time.
    let mut cfg = deadpool_postgres::Config::new();
    cfg.dbname = Some("policy_enforcement_no_such_db".to_string());
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
        kv: secreton_api::KVApiState {
            engine: Arc::new(secreton_api::KVEngine::new()),
            metrics: Default::default(),
        },
        pki: secreton_api::PkiApiState::default(),
        services: Arc::new(services),
        metrics: Default::default(),
        prometheus_handle: None,
    }
}

async fn get_secret_as(policies: Option<&[&str]>) -> StatusCode {
    let router = create_api_router(unsealed_state().await);

    let mut builder = Request::builder().uri("/v1/secret/data/never-written");
    if let Some(policies) = policies {
        builder = builder.header(
            "authorization",
            format!("Bearer {}", token_with_policies(policies)),
        );
    }

    router
        .oneshot(builder.body(Body::empty()).expect("build request"))
        .await
        .expect("router responds")
        .status()
}

#[tokio::test]
async fn a_caller_holding_no_relevant_policy_never_reaches_the_secret() {
    let status = get_secret_as(Some(&["some-unrelated-policy"])).await;

    // This is the assertion that goes red if `policy_check_middleware` is
    // unmounted: without it the request sails through to `get_secret` and comes
    // back 404. The specific refusal code depends on whether the policy store
    // is reachable — 403 when it answers "no rule matched", 500 when it cannot
    // be consulted at all — and BOTH are correct, because failing to load
    // policies must never mean "allow".
    assert_ne!(
        status,
        StatusCode::NOT_FOUND,
        "an unauthorized caller reached the secret handler — the boundary is open"
    );
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::INTERNAL_SERVER_ERROR,
        "expected a refusal, got {status}"
    );
}

#[tokio::test]
async fn the_root_policy_reaches_the_handler() {
    // The bootstrap case. A freshly initialised engine holds no policies at
    // all, so if root were resolved through the policy store like any other
    // name, the token minted by /v1/sys/init could not create the first policy
    // — the engine would be permanently locked out of its own configuration.
    let status = get_secret_as(Some(&["root"])).await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "root must be dispatched to the handler, which 404s on an unwritten path; got {status}"
    );
}

#[tokio::test]
async fn an_unauthenticated_caller_is_refused() {
    let status = get_secret_as(None).await;

    assert_ne!(
        status,
        StatusCode::NOT_FOUND,
        "an anonymous caller reached the secret handler"
    );
}

/// Every path the auth middleware lets through unauthenticated must also be
/// exempt from the policy check.
///
/// This is the drift guard. The two lists used to be maintained separately and
/// had already diverged: the policy copy covered `/sys/init`, `/sys/unseal` and
/// `/sys/seal-status` but not `/sys/rekey/init` or `/sys/rekey/update`. Those
/// carry no `RequestContext` — auth waved them past — so the policy check's
/// no-context branch would have answered 401 and made rekey impossible the
/// moment the middleware was mounted. `policy_check_middleware` now calls the
/// same `is_whitelisted` predicate; this test is what keeps a future edit from
/// reintroducing a second copy.
#[tokio::test]
async fn auth_whitelisted_paths_are_not_refused_by_the_policy_check() {
    for path in [
        "/v1/sys/seal-status",
        "/v1/sys/init",
        "/v1/sys/unseal",
        "/v1/sys/rekey/init",
        "/v1/sys/rekey/update",
    ] {
        let router = create_api_router(unsealed_state().await);
        let response = router
            .oneshot(
                Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("router responds");

        // The handler's own answer (405 for a GET on a POST-only route, 200,
        // whatever) is not the point. The point is that authorization did not
        // reject it for having no identity.
        assert_ne!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{path} is auth-whitelisted but the policy check demanded an identity"
        );
        assert_ne!(
            response.status(),
            StatusCode::FORBIDDEN,
            "{path} is auth-whitelisted but the policy check denied it"
        );
    }
}
