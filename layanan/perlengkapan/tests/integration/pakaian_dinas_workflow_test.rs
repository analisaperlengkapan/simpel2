//! Pakaian Dinas integration tests (#33 / F5-C).
//!
//! Exercises the pakaian-dinas surface end-to-end against a real Postgres:
//!   1. admin master CRUD (jenis → spesifikasi) + RBAC,
//!   2. operator pengajuan create → spesifikasi FK join + satker_terpilih insert,
//!   3. the per-satker validator workflow (Input → SubmitToValidator →
//!      SubmitToPusat → Selesai) with the real state-machine + RBAC guard and
//!      activity logging.
//!
//! The pakaian header/satker status column is `aktivitas_id` (FK →
//! `ms_aktivitas_bmn.kode`), matching the code + FE contract; the squashed
//! baseline was reconciled to it (was `status_kode`) and the pakaian codes
//! 1000-1012 are now seeded. The per-satker workflow row
//! (`pengajuan_pakaian_dinas_satker`) has no API creator yet (no "submit"
//! materializes it from satker_terpilih), so the validator test seeds that one
//! row directly — legitimate setup — then drives the real transition logic.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

fn auth_headers(role: &str, user_id: &str, satker_id: &str) -> Headers {
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
            reqwest::header::HeaderValue::from_str(satker_id).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{}::{}::{}",
                role, user_id, satker_id
            ))
            .unwrap(),
        ),
    ]
}

const ADMIN: &str = "00000000-0000-0000-0000-000000000003";
const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const SATKER: &str = "SKR001";

async fn post(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    body: serde_json::Value,
) -> axum_test::TestResponse {
    let mut req = server.post(path).json(&body);
    for (k, v) in auth_headers(role, user, SATKER) {
        req = req.add_header(k, v);
    }
    req.await
}

#[tokio::test]
async fn test_pakaian_dinas_master_crud_rbac() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // 1. Admin creates a jenis pakaian dinas.
    let res = post(
        &server,
        "/pakaian-dinas/jenis",
        "admin",
        ADMIN,
        json!({"nama": "PDH", "deskripsi": "Pakaian Dinas Harian", "is_active": true}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "create jenis: {:?}", res.text());
    let jenis_id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Admin creates a spesifikasi under that jenis (FK).
    let res = post(
        &server,
        "/pakaian-dinas/spesifikasi",
        "admin",
        ADMIN,
        json!({
            "jenis_pakaian_dinas_id": jenis_id,
            "nama": "Baju PDH",
            "gender": "SEMUA",
            "ukuran_group": "BAJU",
            "is_active": true
        }),
    )
    .await;
    assert_eq!(
        res.status_code(),
        201,
        "create spesifikasi: {:?}",
        res.text()
    );

    // 3. Non-admin is rejected from master CRUD (RBAC enforced server-side).
    let res = post(
        &server,
        "/pakaian-dinas/jenis",
        "operator_satker",
        OPERATOR,
        json!({"nama": "PDL", "is_active": true}),
    )
    .await;
    assert_eq!(
        res.status_code(),
        403,
        "create jenis as operator: {:?}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

/// Operator creates a pengajuan: exercises the spesifikasi FK join, the
/// `aktivitas_id` default (Input/1000, FK → ms_aktivitas_bmn), the new
/// `scope_satker`/`wilayah_id` columns (#19), and the satker_terpilih insert.
#[tokio::test]
async fn test_pakaian_dinas_pengajuan_create() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Seed reference data.
    let res = post(
        &server,
        "/pakaian-dinas/jenis",
        "admin",
        ADMIN,
        json!({"nama": "PDH", "is_active": true}),
    )
    .await;
    let jenis_id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let res = post(
        &server,
        "/pakaian-dinas/spesifikasi",
        "admin",
        ADMIN,
        json!({
        "jenis_pakaian_dinas_id": jenis_id, "nama": "Baju PDH",
        "gender": "SEMUA", "ukuran_group": "BAJU", "is_active": true}),
    )
    .await;
    let spesifikasi_id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = post(
        &server,
        "/pakaian-dinas/pengajuan",
        "operator_satker",
        OPERATOR,
        json!({
        "nama": "Pengajuan PDH 2026", "tahun": 2026,
        "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31",
        "pilihan_satker": "sebagian", "spesifikasi_ids": [spesifikasi_id],
        "satker_ids": ["0200010"]}),
    )
    .await;
    assert_eq!(res.status_code(), 201, "create pengajuan: {:?}", res.text());
    let body = res.json::<serde_json::Value>();
    // Fresh pengajuan starts at Input (1000).
    assert_eq!(
        body["data"]["aktivitas_id"], 1000,
        "initial status: {:?}",
        body
    );

    teardown_test_db(&db_name).await;
}

/// Per-satker validator workflow: drive the real state machine through
/// Input → SubmitToValidator → SubmitToPusat → Selesai, asserting each
/// transition persists and that an illegal (status, action, role) combo is
/// rejected. The `pengajuan_pakaian_dinas_satker` row is seeded directly (no
/// API submit materializes it yet); everything else is the production path
/// (`/pakaian-dinas/validator-action` → determine_next_status →
/// transition_satker_with_activity → activity log, FK → ms_aktivitas_bmn).
#[tokio::test]
async fn test_pakaian_dinas_validator_workflow() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Seed reference data + create a parent pengajuan (operator).
    let r = post(
        &server,
        "/pakaian-dinas/jenis",
        "admin",
        ADMIN,
        json!({"nama": "PDH", "is_active": true}),
    )
    .await;
    let jenis_id = r.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let r = post(
        &server,
        "/pakaian-dinas/spesifikasi",
        "admin",
        ADMIN,
        json!({"jenis_pakaian_dinas_id": jenis_id, "nama": "Baju PDH",
               "gender": "SEMUA", "ukuran_group": "BAJU", "is_active": true}),
    )
    .await;
    let spec_id = r.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let r = post(
        &server,
        "/pakaian-dinas/pengajuan",
        "operator_satker",
        OPERATOR,
        json!({"nama": "Pengajuan PDH 2026", "tahun": 2026,
               "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31",
               "pilihan_satker": "sebagian", "spesifikasi_ids": [spec_id],
               "satker_ids": ["0200010"]}),
    )
    .await;
    assert_eq!(r.status_code(), 201, "create pengajuan: {:?}", r.text());
    let pengajuan_id = r.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Seed one per-satker workflow row at Input (1000). No API path creates it.
    let satker_row_id = Uuid::new_v4();
    {
        let client = db.pool().get().await.unwrap();
        client
            .execute(
                "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker \
                 (id, pengajuan_id, satker_id, aktivitas_id, created_by) \
                 VALUES ($1, $2, $3, 1000, $4)",
                &[
                    &satker_row_id,
                    &Uuid::parse_str(&pengajuan_id).unwrap(),
                    // MySIMKARI kode_satker — the column is varchar(20) (V006/#94),
                    // not a uuid.
                    &"0200010",
                    &Uuid::parse_str(OPERATOR).unwrap(),
                ],
            )
            .await
            .expect("seed pengajuan_pakaian_dinas_satker row");
    }

    // Helper: call validator-action with a given role/aksi.
    let action = |role: &'static str, aksi: &'static str| {
        let server = &server;
        let id = satker_row_id;
        async move {
            post(
                server,
                "/pakaian-dinas/validator-action",
                role,
                "00000000-0000-0000-0000-000000000002",
                json!({"pengajuan_satker_id": id, "aksi": aksi, "komentar": "ok"}),
            )
            .await
        }
    };

    // Illegal: approve at Input by validator_pusat → 400 (state-machine guard).
    let res = action("validator_pusat", "approve").await;
    assert_eq!(
        res.status_code(),
        400,
        "illegal transition should be rejected: {:?}",
        res.text()
    );

    // 1. Pelaksana submits: Input(1000) → SubmitToValidator(1001).
    let res = action("pelaksana", "submit").await;
    assert_eq!(res.status_code(), 200, "submit: {:?}", res.text());
    assert_eq!(
        res.json::<serde_json::Value>()["data"]["aktivitas_id"],
        1001
    );

    // 2. Validator wilayah approves: SubmitToValidator(1001) → SubmitToPusat(1004).
    let res = action("validator_wilayah", "approve").await;
    assert_eq!(res.status_code(), 200, "wilayah approve: {:?}", res.text());
    assert_eq!(
        res.json::<serde_json::Value>()["data"]["aktivitas_id"],
        1004
    );

    // 3. Validator pusat approves: SubmitToPusat(1004) → Selesai(1008).
    let res = action("validator_pusat", "approve").await;
    assert_eq!(res.status_code(), 200, "pusat approve: {:?}", res.text());
    assert_eq!(
        res.json::<serde_json::Value>()["data"]["aktivitas_id"],
        1008
    );

    // Each successful transition logged an activity row (3 total).
    let n: i64 = {
        let client = db.pool().get().await.unwrap();
        client
            .query_one(
                "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas \
                 WHERE pengajuan_satker_id = $1",
                &[&satker_row_id],
            )
            .await
            .unwrap()
            .get(0)
    };
    assert_eq!(n, 3, "three activity rows expected (submit + 2 approvals)");

    teardown_test_db(&db_name).await;
}
