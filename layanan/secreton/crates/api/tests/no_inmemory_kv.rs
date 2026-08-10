//! Does a secret written over REST actually go somewhere that survives the
//! process?
//!
//! `/v1/kv` used to say yes and mean no. It was a `HashMap` behind an `RwLock`,
//! built fresh in `main()` on every boot: writes returned `200` with a version
//! number, reads came back correct for as long as the pod lived, and the whole
//! store evaporated on the next restart. Nothing about the API surface said so
//! (#130).
//!
//! A round-trip through the HTTP layer cannot tell the two apart — `store` then
//! `get` passes identically against a HashMap and against Postgres. So these
//! tests reach past the router and read the container's own storage handle,
//! the same technique `encrypted_storage::tests::encryption_at_rest_is_real`
//! uses for the encryption claim.

mod common;

use std::collections::HashMap;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::{token_with_policies, unsealed_state};
use secreton_api::create_api_router;
use tower::ServiceExt;

/// `root` bypasses the policy store, which is unreachable in this fixture by
/// design. Without it every request below would be refused before reaching a
/// handler and the tests would pass for the wrong reason (#773).
fn root_auth() -> String {
    format!("Bearer {}", token_with_policies(&["root"]))
}

/// Hierarchical paths must work on the surviving route.
///
/// Every real caller uses one — `fetch-secrets.sh` reads `simpelv1/app` and
/// `postgres/simpelv2`, `migrate-env-to-secreton.sh` writes `<prefix>/<key>`.
/// The removed `/v1/kv` router matched `{*path}`; if `/v1/secret/data` had kept
/// a single-segment `{path}`, deleting the in-memory store would have left no
/// route capable of addressing any of them, and the 404 would have looked like
/// "no such secret" rather than "no such route".
#[tokio::test]
async fn a_secret_written_over_rest_lands_in_the_shared_backend() {
    const PATH: &str = "simpelv1/app/kv-persistence-probe";

    let state = unsealed_state().await;
    let services = state.services.clone();
    let router = create_api_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/v1/secret/data/{PATH}"))
                .header("authorization", root_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "data": { "api-key": "s3cr3t" } }).to_string(),
                ))
                .expect("build request"),
        )
        .await
        .expect("router responds");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "the write itself must succeed before persistence means anything"
    );

    // The assertion that matters. `secret_storage` is the handle the whole
    // service shares — in production it is Postgres behind `EncryptedStorage`.
    // A router-local map would leave nothing here, and this is the only place
    // that difference is visible.
    let entry = services
        .secret_storage
        .get_by_path(PATH)
        .await
        .expect("shared backend is readable")
        .expect("REST write is not in the shared backend — it went to a process-local store");

    // `EncryptedStorage::get_by_path` decrypts in place, so `encrypted_data`
    // holds plaintext on the way out of the wrapper. Reading it through the
    // *inner* backend instead is what proves the ciphertext claim, and that is
    // already covered by `encrypted_storage::tests::encryption_at_rest_is_real`
    // — the question here is only whether the bytes arrived at all.
    let stored: HashMap<String, String> = serde_json::from_slice(&entry.encrypted_data)
        .expect("stored payload is the JSON the handler wrote");
    assert_eq!(
        stored.get("api-key").map(String::as_str),
        Some("s3cr3t"),
        "the shared backend holds a different value than the one written"
    );
}

/// The legacy in-memory routes must not be mountable again by accident.
///
/// Write-then-read is the discriminator, not the status code on its own: an
/// unmatched path and an empty-but-mounted store both answer `404` to a bare
/// `GET`. Only a `POST` that is *accepted* followed by a `GET` that returns the
/// value proves a store is listening there — so that is the sequence asserted.
#[tokio::test]
async fn the_legacy_in_memory_kv_routes_accept_nothing() {
    const PATH: &str = "/v1/kv/secret/data/legacy-probe";

    let router = create_api_router(unsealed_state().await);

    let write = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(PATH)
                .header("authorization", root_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "data": { "api-key": "s3cr3t" } }).to_string(),
                ))
                .expect("build request"),
        )
        .await
        .expect("router responds");

    assert_eq!(
        write.status(),
        StatusCode::NOT_FOUND,
        "something accepted a write on the legacy KV path — an in-memory secret store is mounted again"
    );

    let read = router
        .oneshot(
            Request::builder()
                .uri(PATH)
                .header("authorization", root_auth())
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("router responds");

    assert_eq!(
        read.status(),
        StatusCode::NOT_FOUND,
        "the legacy KV path served a read; it must not exist"
    );
}
