//! Object-level scoping for kebutuhan-BMN's per-satker surfaces.
//!
//! #93 scoped ONE of them — `get_satker_detail` — and the siblings on the very
//! same `satker/{id}` path kept reading the row unscoped. Measured against
//! deployed staging (rc29), operator_a (0200010):
//!
//! ```text
//! GET /kebutuhan-bmn/satker/{foreign}            -> 404  (#93, working)
//! GET /kebutuhan-bmn/satker/{foreign}/aktivitas  -> 200, another satker's
//!                                                  trail incl. each
//!                                                  validator's id and NIP
//! GET /kebutuhan-bmn/satker/{foreign}/analisis   -> 200, their whole
//!                                                  feasibility case
//! GET .../laporan/download                       -> 200, a 4 973-byte PDF of
//!                                                  another satker's report
//! GET /kebutuhan-bmn/pengajuan/{id}/satker       -> 200, all participants,
//!                                                  including another wilayah
//! ```
//!
//! Every test below asserts both halves. The positive half alone passes just
//! as well with no scope at all — which is exactly how one fixed endpoint sat
//! beside five broken ones without anyone noticing.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

fn auth_headers(role: &str, user_id: &str, satker: &str) -> Headers {
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_str(user_id).unwrap(),
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
                "Bearer mock::{}::{}::{}",
                role, user_id, satker
            ))
            .unwrap(),
        ),
    ]
}

const PUSAT: &str = "00000000-0000-0000-0000-000000000003";
const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";

async fn get(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    satker: &str,
) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in auth_headers(role, user, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

async fn post(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    satker: &str,
    body: serde_json::Value,
) -> axum_test::TestResponse {
    let mut req = server.post(path).json(&body);
    for (k, v) in auth_headers(role, user, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

struct Fixture {
    pengajuan_id: String,
    /// Participation row of SKR001 (KJT01).
    own_id: String,
    /// Participation row of SKR002 (KJT01, different satker).
    foreign_id: String,
    /// Participation row of SKR003 (KJT02 — a different wilayah entirely).
    far_id: String,
}

/// One campaign, three participating satkers across two Kejati — the smallest
/// shape in which the satker tier and the wilayah tier both have a positive
/// AND a negative case to land on.
async fn seed(server: &TestServer, db: &layanan_perlengkapan::shared::db::Database) -> Fixture {
    let r = post(
        server,
        "/kebutuhan-bmn/pengajuan",
        "validator_pusat",
        PUSAT,
        "PUSAT001",
        json!({"nama": "Kebutuhan BMN 2026", "tahun": 2026,
               "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31"}),
    )
    .await;
    assert_eq!(r.status_code(), 201, "create pengajuan: {:?}", r.text());
    // The create response nests the record under `pengajuan` when it returns
    // the detail view and returns it flat otherwise; read both rather than
    // pinning one and finding out in CI.
    let body = r.json::<serde_json::Value>();
    let pengajuan_id = body["data"]["pengajuan"]["id"]
        .as_str()
        .or_else(|| body["data"]["id"].as_str())
        .unwrap_or_else(|| panic!("no pengajuan id in {body:?}"))
        .to_string();

    let mut ids = Vec::new();
    for satker in ["SKR001", "SKR002", "SKR003"] {
        let r = post(
            server,
            &format!("/kebutuhan-bmn/pengajuan/{pengajuan_id}/satker"),
            "validator_pusat",
            PUSAT,
            "PUSAT001",
            json!({"satker_id": satker}),
        )
        .await;
        assert!(
            r.status_code() == 200 || r.status_code() == 201,
            "add satker {satker}: {:?}",
            r.text()
        );
        ids.push(
            r.json::<serde_json::Value>()["data"]["id"]
                .as_str()
                .expect("satker row id")
                .to_string(),
        );
    }

    // One activity row per participant. Without this the trail is empty for
    // everyone, and "scoped out" is indistinguishable from "nothing happened
    // yet" — the negative assertion would pass with no scope at all. Verified:
    // with the scope disabled, the trail test now fails like the others.
    let client = db.pool().get().await.unwrap();
    for id in &ids {
        client
            .execute(
                "INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas \
                 (pengajuan_satker_id, from_status_kode, to_status_kode, user_id, \
                  nip, nama, role, aksi, komentar) \
                 VALUES ($1, 2000, 2002, $2, '19800101000000009', 'Validator Uji', \
                         'validator_wilayah', 'submit', 'catatan uji')",
                &[
                    &uuid::Uuid::parse_str(id).unwrap(),
                    &uuid::Uuid::parse_str(PUSAT).unwrap(),
                ],
            )
            .await
            .expect("seed activity row");
    }

    Fixture {
        pengajuan_id,
        own_id: ids[0].clone(),
        foreign_id: ids[1].clone(),
        far_id: ids[2].clone(),
    }
}

#[tokio::test]
async fn the_participant_list_is_scoped_per_tier() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;
    let path = format!("/kebutuhan-bmn/pengajuan/{}/satker", f.pengajuan_id);

    let all = get(&server, &path, "validator_pusat", PUSAT, "PUSAT001").await;
    assert_eq!(all.status_code(), 200);
    assert_eq!(
        all.json::<serde_json::Value>()["data"]
            .as_array()
            .unwrap()
            .len(),
        3
    );

    let mine = get(&server, &path, "operator_satker", OPERATOR, "SKR001").await;
    assert_eq!(mine.status_code(), 200);
    let rows = mine.json::<serde_json::Value>();
    let rows = rows["data"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "an operator sees only its own row: {rows:?}");
    assert_eq!(rows[0]["satker_id"], "SKR001");

    // KJT01 supervises SKR001 and SKR002; SKR003 sits under KJT02.
    let region = get(&server, &path, "validator_wilayah", PUSAT, "KJT01").await;
    assert_eq!(region.status_code(), 200);
    let rows = region.json::<serde_json::Value>();
    let codes: Vec<String> = rows["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["satker_id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        codes.len(),
        2,
        "wilayah sees its own region only: {codes:?}"
    );
    assert!(!codes.contains(&"SKR003".to_string()), "{codes:?}");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_approval_trail_of_another_satker_is_closed() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    // Positive control: the caller's own trail is reachable.
    let own = get(
        &server,
        &format!("/kebutuhan-bmn/satker/{}/aktivitas", f.own_id),
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await;
    assert_eq!(own.status_code(), 200, "own trail: {:?}", own.text());
    assert_eq!(
        own.json::<serde_json::Value>()["data"]
            .as_array()
            .unwrap()
            .len(),
        1,
        "the positive half must be non-empty, or the negative half proves nothing"
    );

    for (label, id) in [
        ("another satker", &f.foreign_id),
        ("another wilayah", &f.far_id),
    ] {
        let res = get(
            &server,
            &format!("/kebutuhan-bmn/satker/{id}/aktivitas"),
            "operator_satker",
            OPERATOR,
            "SKR001",
        )
        .await;
        assert_eq!(res.status_code(), 200);
        assert!(
            res.json::<serde_json::Value>()["data"]
                .as_array()
                .unwrap()
                .is_empty(),
            "{label}: the trail names every validator who acted, with their NIP"
        );
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_feasibility_analysis_of_another_satker_is_closed() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    let own = get(
        &server,
        &format!("/kebutuhan-bmn/satker/{}/analisis", f.own_id),
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await;
    assert_eq!(own.status_code(), 200, "own analysis: {:?}", own.text());

    let foreign = get(
        &server,
        &format!("/kebutuhan-bmn/satker/{}/analisis", f.foreign_id),
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await;
    // 404, not 403 — see #93.
    assert_eq!(foreign.status_code(), 404, "{:?}", foreign.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_analysis_report_of_another_satker_cannot_be_downloaded() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    for endpoint in ["laporan/preview", "laporan/download"] {
        let own = get(
            &server,
            &format!("/kebutuhan-bmn/satker/{}/{endpoint}", f.own_id),
            "operator_satker",
            OPERATOR,
            "SKR001",
        )
        .await;
        assert_eq!(own.status_code(), 200, "own {endpoint}: {:?}", own.text());

        let foreign = get(
            &server,
            &format!("/kebutuhan-bmn/satker/{}/{endpoint}", f.foreign_id),
            "operator_satker",
            OPERATOR,
            "SKR001",
        )
        .await;
        assert_eq!(
            foreign.status_code(),
            404,
            "{endpoint} returned another satker's document: {:?}",
            foreign.text()
        );
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_barang_cannot_be_added_to_or_removed_from_another_satker() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    let body = json!({"nama": "Kursi Kerja", "jumlah": 2, "kode_barang": "3060201004"});

    // Positive control: the operator may add to its own row.
    let own = post(
        &server,
        &format!("/kebutuhan-bmn/satker/{}/barang", f.own_id),
        "operator_satker",
        OPERATOR,
        "SKR001",
        body.clone(),
    )
    .await;
    assert_eq!(own.status_code(), 201, "own barang: {:?}", own.text());
    let barang_id = own.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // The same call against another satker's row must not land. The status
    // gate would have let it through — both rows are in the same state.
    let foreign = post(
        &server,
        &format!("/kebutuhan-bmn/satker/{}/barang", f.foreign_id),
        "operator_satker",
        OPERATOR,
        "SKR001",
        body,
    )
    .await;
    assert_eq!(
        foreign.status_code(),
        404,
        "cross-satker barang insert: {:?}",
        foreign.text()
    );

    // And a barang reached by its OWN id still consults the owning satker.
    let mut req = server.delete(&format!("/kebutuhan-bmn/barang/{barang_id}"));
    for (k, v) in auth_headers("operator_satker", OPERATOR, "SKR002") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(
        res.status_code(),
        404,
        "another satker deleted a barang by id: {:?}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_validator_cannot_act_on_a_satker_outside_its_region() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    // KJT02 supervises SKR003, not SKR001. Right role, wrong region — and the
    // action stamps a validator identity onto the row before it returns, so a
    // scope placed only at the transition would still have branded it.
    let res = post(
        &server,
        &format!("/kebutuhan-bmn/satker/{}/validator-wilayah", f.own_id),
        "validator_wilayah",
        PUSAT,
        "KJT02",
        json!({"aksi": "forward", "catatan": "diteruskan"}),
    )
    .await;
    assert_eq!(
        res.status_code(),
        404,
        "out-of-region wilayah action: {:?}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}
