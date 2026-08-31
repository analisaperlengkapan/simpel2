//! The spesifikasi list's `jenis` filter, asserted as a filter.
//!
//! `GET /pakaian-dinas/spesifikasi` takes the jenis to narrow by as a query
//! parameter. The backend called that parameter `jenis_id`; both frontend
//! callers have always sent `jenis_pakaian_dinas_id` — the name of the column,
//! and of the field in the create DTO. serde drops query keys it does not
//! know, so the filter deserialised to `None` on every request and the
//! endpoint answered `200 OK` with EVERY jenis's spesifikasi:
//!
//!   - the master drawer (`/pakaian-dinas/jenis/:id/spesifikasi`) listed rows
//!     belonging to other jenis under the one being edited,
//!   - the campaign form's picker offered Toga and Batik rows while the
//!     operator had PDH selected, and a campaign freezes whatever is picked.
//!
//! Nothing could see it: the status is 200, the JSON shape is right, and the
//! two names live in different crates that never meet at a type boundary.
//!
//! So this test does not check the name — a rename would satisfy that while
//! the filter stayed dead. It creates TWO jenis with one spesifikasi each and
//! asserts the response contains the asked-for row and NOT the other one. A
//! filter that is ignored returns both and fails here.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

const ADMIN: &str = "00000000-0000-0000-0000-000000000003";

fn auth() -> reqwest::header::HeaderValue {
    reqwest::header::HeaderValue::from_str(&format!("Bearer mock::admin::{ADMIN}::SKR001")).unwrap()
}

async fn create_jenis(server: &TestServer, nama: &str) -> String {
    let res = server
        .post("/pakaian-dinas/jenis")
        .add_header(reqwest::header::AUTHORIZATION, auth())
        .json(&json!({ "nama": nama, "is_active": true }))
        .await;
    assert_eq!(res.status_code(), 201, "create jenis: {:?}", res.text());
    res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn create_spesifikasi(server: &TestServer, jenis_id: &str, nama: &str) {
    let res = server
        .post("/pakaian-dinas/spesifikasi")
        .add_header(reqwest::header::AUTHORIZATION, auth())
        .json(&json!({
            "jenis_pakaian_dinas_id": jenis_id,
            "nama": nama,
            "gender": "SEMUA",
            "ukuran_group": "BAJU",
            "is_active": true
        }))
        .await;
    assert_eq!(
        res.status_code(),
        201,
        "create spesifikasi: {:?}",
        res.text()
    );
}

#[tokio::test]
async fn spesifikasi_filter_narrows_to_the_asked_for_jenis() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let mine = create_jenis(&server, "E2E Jenis Filter A").await;
    let other = create_jenis(&server, "E2E Jenis Filter B").await;
    create_spesifikasi(&server, &mine, "Spesifikasi Milik A").await;
    create_spesifikasi(&server, &other, "Spesifikasi Milik B").await;

    let res = server
        .get(&format!(
            "/pakaian-dinas/spesifikasi?page=1&per_page=200&jenis_pakaian_dinas_id={mine}"
        ))
        .add_header(reqwest::header::AUTHORIZATION, auth())
        .await;
    assert_eq!(res.status_code(), 200, "list spesifikasi: {:?}", res.text());

    let body: serde_json::Value = res.json();
    let names: Vec<String> = body["data"]
        .as_array()
        .expect("paginated response carries `data`")
        .iter()
        .map(|row| row["nama"].as_str().unwrap_or_default().to_string())
        .collect();

    assert!(
        names.iter().any(|n| n == "Spesifikasi Milik A"),
        "the asked-for jenis's own spesifikasi must be listed: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n == "Spesifikasi Milik B"),
        "a row from ANOTHER jenis came back — the filter is being ignored: {names:?}"
    );

    // And the parameter must be doing the narrowing, not the page size: without
    // it the same request returns both rows. This is the canary in the opposite
    // direction — it fails if the endpoint ever starts filtering by something
    // it was not asked to filter by.
    let all = server
        .get("/pakaian-dinas/spesifikasi?page=1&per_page=200")
        .add_header(reqwest::header::AUTHORIZATION, auth())
        .await;
    assert_eq!(all.status_code(), 200);
    let all_names: Vec<String> = all.json::<serde_json::Value>()["data"]
        .as_array()
        .expect("paginated response carries `data`")
        .iter()
        .map(|row| row["nama"].as_str().unwrap_or_default().to_string())
        .collect();
    for expected in ["Spesifikasi Milik A", "Spesifikasi Milik B"] {
        assert!(
            all_names.iter().any(|n| n == expected),
            "unfiltered list must carry {expected}: {all_names:?}"
        );
    }

    teardown_test_db(&db_name).await;
}
