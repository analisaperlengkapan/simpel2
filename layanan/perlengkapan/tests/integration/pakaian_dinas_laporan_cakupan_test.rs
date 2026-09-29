//! The laporan's status filter, asserted as a filter — and as valid SQL.
//!
//! Every laporan query counts only satker whose participation reached
//! `Selesai` (1008). That is deliberate: a recap is a procurement figure, and
//! sizes a wilayah validator has not accepted yet should not be ordered
//! against. But it was four bare `1008` literals in four query strings with
//! nothing naming them, and its effect is invisible from outside — mid-campaign
//! the report simply comes back small, and a recap covering 3 of 238 satker
//! renders exactly like a complete one.
//!
//! Two things are asserted here, and neither is the SQL text:
//!
//! 1. **The filter's effect.** A satker at `SubmitToPusat` (1004) is absent
//!    from the recap and a satker at `Selesai` (1008) is present. Asserting
//!    only the second passes just as well with no filter at all.
//! 2. **The query still parses.** Naming the constant meant interpolating it,
//!    and the two raw blocks were `r#"…"#.to_string()` — where `{CONST}` is
//!    literal text, not a value. Postgres rejects that at RUNTIME; `cargo
//!    check` is perfectly happy. A 200 with rows is the only proof.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

const ADMIN: &str = "00000000-0000-0000-0000-000000000003";
const PUSAT: &str = "00000000-0000-0000-0000-000000000002";

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
                "Bearer mock::{role}::{user_id}::{satker}"
            ))
            .unwrap(),
        ),
    ]
}

async fn get(server: &TestServer, path: &str, role: &str, user: &str) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in auth_headers(role, user, "PUSAT001") {
        req = req.add_header(k, v);
    }
    req.await
}

async fn post(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    body: serde_json::Value,
) -> axum_test::TestResponse {
    let mut req = server.post(path).json(&body);
    for (k, v) in auth_headers(role, user, "PUSAT001") {
        req = req.add_header(k, v);
    }
    req.await
}

/// One campaign, two participating satker at DIFFERENT workflow stages, each
/// with one employee carrying one recorded size. The smallest shape in which
/// "counts only Selesai" and "counts everything" give different answers.
struct Fixture {
    pengajuan_id: Uuid,
}

const SATKER_SELESAI: &str = "SKR001";
const SATKER_BELUM: &str = "SKR002";

async fn seed(server: &TestServer, db: &layanan_perlengkapan::shared::db::Database) -> Fixture {
    let r = post(
        server,
        "/pakaian-dinas/jenis",
        "admin",
        ADMIN,
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
        json!({"nama": "Pengajuan PDH 2026", "tahun": 2026,
               "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31",
               "pilihan_satker": "sebagian", "spesifikasi_ids": [spec_id],
               "satker_ids": [SATKER_SELESAI, SATKER_BELUM]}),
    )
    .await;
    assert_eq!(r.status_code(), 201, "create pengajuan: {:?}", r.text());
    let pengajuan_id = Uuid::parse_str(
        r.json::<serde_json::Value>()["data"]["id"]
            .as_str()
            .unwrap(),
    )
    .unwrap();

    let client = db.pool().get().await.unwrap();
    let pakaian_id: Uuid = client
        .query_one(
            "SELECT id FROM perlengkapan.pengajuan_pakaian_dinas_pakaian WHERE pengajuan_id = $1",
            &[&pengajuan_id],
        )
        .await
        .expect("the campaign must carry its clothing item")
        .get("id");

    // No API path materialises the per-satker rows yet, so seed them directly.
    // Everything under test below is the production read path.
    for (satker, aktivitas, nip) in [
        (SATKER_SELESAI, 1008_i32, "198001010000000001"),
        (SATKER_BELUM, 1004_i32, "198001010000000002"),
    ] {
        let row_id = Uuid::new_v4();
        client
            .execute(
                "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker \
                 (id, pengajuan_id, satker_id, aktivitas_id, created_by) \
                 VALUES ($1, $2, $3, $4, $5)",
                &[
                    &row_id,
                    &pengajuan_id,
                    &satker,
                    &aktivitas,
                    &Uuid::parse_str(PUSAT).unwrap(),
                ],
            )
            .await
            .expect("seed participation row");
        let pegawai_id: Uuid = client
            .query_one(
                "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_pegawai \
                 (pengajuan_satker_id, nip, nama, jenis_kelamin) \
                 VALUES ($1, $2, $3, 'L') RETURNING id",
                &[&row_id, &nip, &format!("Pegawai {satker}")],
            )
            .await
            .expect("seed participation employee")
            .get("id");
        client
            .execute(
                "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran \
                 (pengajuan_satker_id, pegawai_id, pakaian_id, ukuran) \
                 VALUES ($1, $2, $3, 'L')",
                &[&row_id, &pegawai_id, &pakaian_id],
            )
            .await
            .expect("seed recorded size");
    }

    Fixture { pengajuan_id }
}

#[tokio::test]
async fn the_recap_counts_finished_satker_and_not_the_unfinished_one() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    let res = get(
        &server,
        &format!(
            "/pakaian-dinas/laporan/rekap-ukuran?pengajuan_id={}",
            f.pengajuan_id
        ),
        "validator_pusat",
        PUSAT,
    )
    .await;
    // A 200 here is itself the proof that the interpolated constant produced a
    // NUMBER and not the literal text `{AKTIVITAS_SELESAI}`, which Postgres
    // would reject as a syntax error long after `cargo check` was satisfied.
    assert_eq!(res.status_code(), 200, "rekap: {:?}", res.text());

    let body = res.json::<serde_json::Value>();
    let rows = body["data"].as_array().expect("data must be an array");
    let total: i64 = rows
        .iter()
        .map(|r| r["jumlah_total"].as_i64().unwrap_or(0))
        .sum();

    // Two employees have a recorded size; only one of them belongs to a satker
    // that reached Selesai. Counting both would mean the status filter is dead;
    // counting neither would mean the query matches nothing at all.
    assert_eq!(
        total, 1,
        "recap must count only the Selesai satker's employee: {body:?}"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_employee_list_answers_and_excludes_the_unfinished_satker() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let f = seed(&server, &db).await;

    let res = get(
        &server,
        &format!(
            "/pakaian-dinas/laporan/daftar-pegawai?pengajuan_id={}&page=1&per_page=50",
            f.pengajuan_id
        ),
        "validator_pusat",
        PUSAT,
    )
    .await;
    assert_eq!(res.status_code(), 200, "daftar: {:?}", res.text());

    let body = res.json::<serde_json::Value>();
    let rows = body["data"].as_array().expect("data must be an array");
    assert_eq!(
        rows.len(),
        1,
        "only the Selesai satker's employee: {body:?}"
    );
    let text = body.to_string();
    assert!(
        text.contains(SATKER_SELESAI),
        "the finished satker must be present: {body:?}"
    );
    assert!(
        !text.contains(SATKER_BELUM),
        "the unfinished satker must not appear: {body:?}"
    );

    teardown_test_db(&db_name).await;
}
