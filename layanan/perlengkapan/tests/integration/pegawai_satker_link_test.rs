//! An employee's satker resolves through `mysimkari_satker.api_id`.
//!
//! `integrasi.mysimkari_pegawai.satker_id` is TEXT holding the upstream API's
//! UUID, not the MySIMKARI `kode_satker`. Every satker-scoped employee query
//! in the crate read it as if it held the code, so it matched nothing:
//! measured on staging, `WHERE satker_id = '01.01'` returned 0 rows while the
//! resolved join returned 123, and 191 satkers have people in them.
//!
//! Two things kept that invisible, and both are fixed here:
//!
//! 1. the integration fixture seeded `satker_id` with a `kode_satker`, a shape
//!    the real table never has — so the tests ran against a table where the
//!    broken query worked;
//! 2. the roster tests asserted only `200`. An endpoint that answers 200 with
//!    an empty list looks exactly like one that works.
//!
//! So every test below asserts CONTENT, and the last one is a canary on the
//! fixture itself: if someone re-seeds the code into that column, it fails and
//! says why.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;

fn auth_headers(
    role: &str,
    satker: &str,
) -> Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)> {
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_static("00000000-0000-0000-0000-000000000001"),
        ),
        (
            reqwest::header::HeaderName::from_static("x-user-role"),
            reqwest::header::HeaderValue::from_str(role).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("x-satker-id"),
            reqwest::header::HeaderValue::from_str(satker).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{}::00000000-0000-0000-0000-000000000001::{}",
                role, satker
            ))
            .unwrap(),
        ),
    ]
}

async fn get(server: &TestServer, path: &str, role: &str, satker: &str) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in auth_headers(role, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

/// The roster must contain the employee, not merely answer 200.
#[tokio::test]
async fn the_roster_returns_the_satkers_actual_people() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = get(
        &server,
        "/pakaian-dinas/pegawai-satker/SKR001",
        "operator_satker",
        "SKR001",
    )
    .await;
    assert_eq!(res.status_code(), 200);

    let body = res.text();
    assert!(
        body.contains("Pegawai Uji A"),
        "roster answered 200 with nobody in it — the satker link is broken again: {body}"
    );
    assert!(
        !body.contains("Pegawai Uji C"),
        "SKR003's employee must not appear in SKR001's roster: {body}"
    );

    teardown_test_db(&db_name).await;
}

/// Typing a NIP is supposed to identify the employee — that is the whole point
/// of the lookup the request forms depend on.
#[tokio::test]
async fn cek_pegawai_resolves_a_nip_for_the_callers_own_satker() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = get(
        &server,
        "/pemakaian-bmn/cek-pegawai/19800101000000001",
        "operator_satker",
        "SKR001",
    )
    .await;
    assert_eq!(
        res.status_code(),
        200,
        "own employee must resolve: {:?}",
        res.text()
    );
    assert!(res.text().contains("Pegawai Uji A"));

    teardown_test_db(&db_name).await;
}

/// The other direction. Without it the test above passes with no scope at all,
/// which is exactly how the object-level gaps survived.
#[tokio::test]
async fn cek_pegawai_refuses_another_satkers_employee() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // SKR003 belongs to KJT02; SKR001's operator has no business reading it.
    let res = get(
        &server,
        "/pemakaian-bmn/cek-pegawai/19800101000000003",
        "operator_satker",
        "SKR001",
    )
    .await;
    assert_ne!(
        res.status_code(),
        200,
        "an operator read another satker's employee record — personal data: {}",
        res.text()
    );
    assert!(
        !res.text().contains("Pegawai Uji C"),
        "the refusal still leaked the name: {}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

/// Canary on the fixture's SHAPE, not on any query.
///
/// If `satker_id` is ever re-seeded with a `kode_satker`, the broken query
/// starts "working" in tests again and the suite goes quiet about a defect
/// that is total in production. This fails first, and says so.
#[tokio::test]
async fn the_fixture_stores_an_api_id_not_a_kode_satker() {
    let (_app, db, db_name) = setup_test_app().await;
    let client = db.pool().get().await.expect("pool checkout");

    let by_code: i64 = client
        .query_one(
            "SELECT count(*) FROM integrasi.mysimkari_pegawai p
             WHERE p.satker_id IN (SELECT kode_satker FROM integrasi.mysimkari_satker)",
            &[],
        )
        .await
        .expect("query")
        .get(0);
    let by_api_id: i64 = client
        .query_one(
            "SELECT count(*) FROM integrasi.mysimkari_pegawai p
             WHERE p.satker_id IN (SELECT api_id FROM integrasi.mysimkari_satker)",
            &[],
        )
        .await
        .expect("query")
        .get(0);

    assert_eq!(
        by_code, 0,
        "the fixture seeded a kode_satker into `satker_id`. The real table holds \
         mysimkari_satker.api_id (21 325 of 21 328 rows on staging); seeding the \
         code makes the broken query pass here and find nobody in production."
    );
    assert!(
        by_api_id > 0,
        "no employee resolves to a satker at all — the fixture is not the \
         owner's shape in either direction"
    );

    teardown_test_db(&db_name).await;
}
