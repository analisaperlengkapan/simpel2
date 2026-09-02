//! Object-level scoping for a pakaian-dinas CAMPAIGN's per-satker surfaces.
//!
//! A campaign is nationwide and pusat-authored; a satker's RESPONSE to it is
//! not. The participation row, its approval trail, and the reports built from
//! it are that satker's data. Measured against deployed staging (rc29),
//! operator_a (0200010) against operator_b's satker (0200020):
//!
//! ```text
//! GET /pakaian-dinas/pengajuan/{id}/satker   -> 200, BOTH participants listed
//! GET /pakaian-dinas/satker/{foreign_id}     -> 200, full row
//! GET /pakaian-dinas/satker/{id}/aktivitas   -> 200, the other satker's trail
//!                                               with the validators' NIPs
//! GET /pakaian-dinas/laporan/daftar-pegawai  -> 200, the NATIONAL employee list
//! ```
//!
//! Both halves are asserted throughout. The positive half alone passes just as
//! well with no scope at all.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

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

const ADMIN: &str = "00000000-0000-0000-0000-000000000003";
const PUSAT: &str = "00000000-0000-0000-0000-000000000002";
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
    pengajuan_id: Uuid,
    /// Participation row of SKR001 (KJT01).
    own_id: Uuid,
    /// Participation row of SKR002 (also KJT01, different satker).
    foreign_id: Uuid,
}

/// One campaign, two participating satkers, one employee and one activity row
/// each — the smallest shape in which "scoped" and "unscoped" differ.
async fn seed(server: &TestServer, db: &layanan_perlengkapan::shared::db::Database) -> Fixture {
    seed_at(server, db, 1000).await
}

/// `aktivitas` is the participation row's workflow state. The daftar report
/// reads only APPROVED rows (`aktivitas_id = 1008`), so a report test has to
/// seed them there; the workflow tests want them at Input (1000).
async fn seed_at(
    server: &TestServer,
    db: &layanan_perlengkapan::shared::db::Database,
    aktivitas: i32,
) -> Fixture {
    let r = post(
        server,
        "/pakaian-dinas/jenis",
        "admin",
        ADMIN,
        "PUSAT001",
        json!({"nama": "PDH", "is_active": true}),
    )
    .await;
    assert_eq!(r.status_code(), 201, "create jenis: {:?}", r.text());
    let jenis_id = r.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let r = post(
        server,
        "/pakaian-dinas/spesifikasi",
        "admin",
        ADMIN,
        "PUSAT001",
        json!({"jenis_pakaian_dinas_id": jenis_id, "nama": "Baju PDH",
               "gender": "SEMUA", "ukuran_group": "BAJU", "is_active": true}),
    )
    .await;
    assert_eq!(r.status_code(), 201, "create spesifikasi: {:?}", r.text());
    let spec_id = r.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let r = post(
        server,
        "/pakaian-dinas/pengajuan",
        "validator_pusat",
        PUSAT,
        "PUSAT001",
        json!({"nama": "Pengajuan PDH 2026", "tahun": 2026,
               "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31",
               "pilihan_satker": "sebagian", "spesifikasi_ids": [spec_id],
               "satker_ids": ["SKR001", "SKR002"]}),
    )
    .await;
    assert_eq!(r.status_code(), 201, "create pengajuan: {:?}", r.text());
    let pengajuan_id = Uuid::parse_str(
        r.json::<serde_json::Value>()["data"]["id"]
            .as_str()
            .unwrap(),
    )
    .unwrap();

    // Creating a campaign now materialises one participation row per targeted
    // satker, so these are LOOKED UP rather than inserted. Inserting them again
    // produced two rows per satker — two statuses for the same thing — which is
    // what this assertion caught. Only the status and the child rows are seeded
    // here; the row itself comes from the production path.
    let client = db.pool().get().await.unwrap();
    let mut ids = std::collections::HashMap::new();
    for satker in ["SKR001", "SKR002"] {
        let row = client
            .query_one(
                "SELECT id FROM perlengkapan.pengajuan_pakaian_dinas_satker \
                 WHERE pengajuan_id = $1 AND satker_id = $2",
                &[&pengajuan_id, &satker],
            )
            .await
            .expect("campaign creation must materialise the participation row");
        ids.insert(satker, row.get::<_, Uuid>("id"));
    }
    let own_id = ids["SKR001"];
    let foreign_id = ids["SKR002"];
    for (row_id, satker) in [(own_id, "SKR001"), (foreign_id, "SKR002")] {
        client
            .execute(
                "UPDATE perlengkapan.pengajuan_pakaian_dinas_satker \
                 SET aktivitas_id = $2 WHERE id = $1",
                &[&row_id, &aktivitas],
            )
            .await
            .expect("set participation status");
        client
            .execute(
                "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas \
                 (pengajuan_satker_id, aktivitas_id, komentar, nip, nama, role) \
                 VALUES ($1, 1000, $2, '19800101000000009', 'Validator Uji', 'validator_pusat')",
                &[&row_id, &format!("catatan untuk {satker}")],
            )
            .await
            .expect("seed activity row");
        client
            .execute(
                "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_pegawai \
                 (pengajuan_satker_id, nip, nama, jenis_kelamin) \
                 VALUES ($1, $2, $3, 'L')",
                &[
                    &row_id,
                    &format!(
                        "1980010100000{}",
                        if satker == "SKR001" { "0001" } else { "0002" }
                    ),
                    &format!("Pegawai {satker}"),
                ],
            )
            .await
            .expect("seed participation employee");
    }

    Fixture {
        pengajuan_id,
        own_id,
        foreign_id,
    }
}

#[tokio::test]
async fn the_participant_list_shows_only_the_callers_own_satker() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    let path = format!("/pakaian-dinas/pengajuan/{}/satker", f.pengajuan_id);

    // Pusat authored the campaign and sees every participant.
    let res = get(&server, &path, "validator_pusat", PUSAT, "PUSAT001").await;
    assert_eq!(res.status_code(), 200);
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["data"].as_array().unwrap().len(), 2, "{body:?}");
    assert_eq!(body["total"], 2, "{body:?}");

    // A satker operator sees their own row and no other.
    let res = get(&server, &path, "operator_satker", OPERATOR, "SKR001").await;
    assert_eq!(res.status_code(), 200);
    let body = res.json::<serde_json::Value>();
    let rows = body["data"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{body:?}");
    assert_eq!(rows[0]["satker_id"], "SKR001", "{body:?}");
    // The TOTAL is scoped too. A scoped page beside an unscoped total still
    // tells the caller how many participants they are not being shown.
    assert_eq!(body["total"], 1, "{body:?}");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_participation_row_of_another_satker_is_not_readable() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    // Positive control first: the caller's OWN row is readable.
    let res = get(
        &server,
        &format!("/pakaian-dinas/satker/{}", f.own_id),
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await;
    assert_eq!(res.status_code(), 200, "own row: {:?}", res.text());

    let res = get(
        &server,
        &format!("/pakaian-dinas/satker/{}", f.foreign_id),
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await;
    // 404, not 403 — see #93.
    assert_eq!(res.status_code(), 404, "foreign row: {:?}", res.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_approval_trail_of_another_satker_is_not_readable() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    let own = get(
        &server,
        &format!("/pakaian-dinas/satker/{}/aktivitas", f.own_id),
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await;
    assert_eq!(own.status_code(), 200);
    let body = own.json::<serde_json::Value>();
    assert_eq!(body["data"].as_array().unwrap().len(), 1, "{body:?}");

    let foreign = get(
        &server,
        &format!("/pakaian-dinas/satker/{}/aktivitas", f.foreign_id),
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await;
    assert_eq!(foreign.status_code(), 200);
    let body = foreign.json::<serde_json::Value>();
    assert!(
        body["data"].as_array().unwrap().is_empty(),
        "another satker's trail names the people who acted on it: {body:?}"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_validator_cannot_act_on_a_satker_outside_its_region() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    // KJT02 supervises SKR003, not SKR001. Right role, wrong region.
    let res = post(
        &server,
        "/pakaian-dinas/validator-action",
        "validator_wilayah",
        PUSAT,
        "KJT02",
        json!({"pengajuan_satker_id": f.own_id, "aksi": "approve", "komentar": "disetujui"}),
    )
    .await;
    assert_eq!(
        res.status_code(),
        404,
        "out-of-region approval must be refused: {:?}",
        res.text()
    );

    // And refused means the row did not move: the action writes an activity row
    // stamped with the actor before it returns, so a leak would be visible here.
    let after = get(
        &server,
        &format!("/pakaian-dinas/satker/{}", f.own_id),
        "validator_pusat",
        PUSAT,
        "PUSAT001",
    )
    .await;
    assert_eq!(
        after.json::<serde_json::Value>()["data"]["aktivitas_id"],
        1000
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_daftar_report_is_scoped_and_a_client_filter_can_only_narrow() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed_at(&server, &db, 1008).await;

    let base = format!(
        "/pakaian-dinas/laporan/daftar-pegawai?pengajuan_id={}&page=1&per_page=50",
        f.pengajuan_id
    );

    // Pusat sees both satkers' employees.
    let res = get(&server, &base, "validator_pusat", PUSAT, "PUSAT001").await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["data"].as_array().unwrap().len(), 2, "{body:?}");

    // A satker operator sees only their own.
    let res = get(&server, &base, "operator_satker", OPERATOR, "SKR001").await;
    assert_eq!(res.status_code(), 200);
    let body = res.json::<serde_json::Value>();
    let rows = body["data"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        1,
        "the national list used to come back: {body:?}"
    );
    assert_eq!(rows[0]["nama"], "Pegawai SKR001", "{body:?}");

    // Naming the other satker explicitly narrows to nothing rather than
    // widening to them.
    let res = get(
        &server,
        &format!("{base}&satker_id=SKR002"),
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await;
    assert_eq!(res.status_code(), 200);
    let body = res.json::<serde_json::Value>();
    assert!(
        body["data"].as_array().unwrap().is_empty(),
        "a client filter must only narrow: {body:?}"
    );

    teardown_test_db(&db_name).await;
}

/// A campaign is a pusat instrument. The module's own scope doc has always
/// rested on that — its list-scoping argues no `created_by` escape hatch is
/// needed "because campaigns can only be authored by validator_pusat" — but
/// nothing enforced it: the handler read `claims` for a user id and no more.
#[tokio::test]
async fn only_pusat_may_author_or_delete_a_campaign() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    let res = post(
        &server,
        "/pakaian-dinas/pengajuan",
        "operator_satker",
        OPERATOR,
        "SKR001",
        json!({"nama": "Kampanye milik saya sendiri", "tahun": 2026,
               "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31",
               "pilihan_satker": "semua", "spesifikasi_ids": [Uuid::new_v4()]}),
    )
    .await;
    assert_eq!(
        res.status_code(),
        403,
        "a satker operator must not author a nationwide campaign: {:?}",
        res.text()
    );

    let mut req = server.delete(&format!("/pakaian-dinas/pengajuan/{}", f.pengajuan_id));
    for (k, v) in auth_headers("operator_satker", OPERATOR, "SKR001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(
        res.status_code(),
        403,
        "deleting a campaign cascades every participant's rows: {:?}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}
