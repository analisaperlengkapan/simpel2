//! A status must have a code, the code must have a row, and the row must be what
//! the screen prints.
//!
//! The dashboard does not show `status`; it shows the NAME the workflow status
//! master gives the integer `status_kode`. Three ways for that to go wrong, all
//! measured on staging (rc32), all invisible to a suite whose fixtures only ever
//! hold codes that exist:
//!
//! * a row with NO code: a permit is born DRAFT, and the `INSERT` forgot the
//!   denormalised `status_kode`, so the card printed the raw word `DRAFT` beside
//!   the localised others (3 of 7 permits, every one a draft);
//! * a code with NO master row: pemakaian 3010/3011 (the satker approval chain)
//!   and penghapusan 4010/4011 (the Wilayah authority branch) were in the enums
//!   and absent from `ms_workflow_status`, so they would print `Kode 4010`;
//! * a state with NO code: cancelling a usulan changed the text `status` to
//!   `CANCELLED` and left `status_kode` at 4000, so a cancelled usulan was counted
//!   as a Draft ("Draft 9" included 6 of them) and the list printed the raw word.
//!
//! The second is derived, not listed: every code each enum can decode must have a
//! master row, so the next variant added without its row fails here instead of on
//! a screen.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use layanan_perlengkapan::kebutuhan_bmn::models::KebutuhanBmnStatus;
use layanan_perlengkapan::pemakaian_bmn::models::PemakaianBmnStatus;
use layanan_perlengkapan::penghapusan_bmn::models::PenghapusanBmnStatus;
use serde_json::{Value, json};
use uuid::Uuid;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const PUSAT: &str = "00000000-0000-0000-0000-000000000003";
const SATKER: &str = "SKR001";
/// Pegawai milik SATKER di fixture `tests/common.rs`.
const NIP: &str = "19800101000000001";

fn headers(role: &str, user: &str) -> Headers {
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_str(user).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("x-user-role"),
            reqwest::header::HeaderValue::from_str(role).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("x-satker-id"),
            reqwest::header::HeaderValue::from_static(SATKER),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{role}::{user}::{SATKER}"
            ))
            .unwrap(),
        ),
    ]
}

async fn dashboard(server: &TestServer) -> Value {
    let mut req = server.get("/dashboard/perlengkapan?tahun_anggaran=2026");
    for (k, v) in headers("validator_pusat", PUSAT) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    res.json::<Value>()
}

/// The label -> count map a card is drawn from, as sorted `(label, count)`.
fn totals(section: &Value) -> Vec<(String, i64)> {
    let mut v: Vec<(String, i64)> = section["total_by_status"]
        .as_object()
        .unwrap_or_else(|| panic!("no total_by_status in {section}"))
        .iter()
        .map(|(k, n)| (k.clone(), n.as_i64().unwrap()))
        .collect();
    v.sort();
    v
}

// ─── a code with no master row ──────────────────────────────────────────────

/// Every code an enum can decode must exist in `ms_workflow_status` for its
/// module. Derived by scanning the whole code range through `from_code`, so a
/// variant added tomorrow is covered the day it is added.
#[tokio::test]
async fn every_code_the_enums_can_write_has_a_row_in_the_status_master() {
    let (_app, db, db_name) = setup_test_app().await;
    let client = db.pool().get().await.unwrap();

    let scan =
        |decode: &dyn Fn(i32) -> bool| -> Vec<i32> { (0..6000).filter(|c| decode(*c)).collect() };
    let modules: Vec<(&str, Vec<i32>)> = vec![
        (
            "kebutuhan_bmn",
            scan(&|c| KebutuhanBmnStatus::from_code(c).is_some()),
        ),
        (
            "pemakaian_bmn",
            scan(&|c| PemakaianBmnStatus::from_code(c).is_some()),
        ),
        (
            "penghapusan_bmn",
            scan(&|c| PenghapusanBmnStatus::from_code(c).is_some()),
        ),
    ];

    for (modul, codes) in modules {
        assert!(!codes.is_empty(), "{modul}: the scan found no codes at all");
        let rows = client
            .query(
                "SELECT kode FROM perlengkapan.ms_workflow_status WHERE modul = $1",
                &[&modul],
            )
            .await
            .unwrap();
        let have: Vec<i32> = rows.iter().map(|r| r.get("kode")).collect();
        let missing: Vec<i32> = codes
            .iter()
            .copied()
            .filter(|c| !have.contains(c))
            .collect();
        assert!(
            missing.is_empty(),
            "{modul}: the enum can write {missing:?} but ms_workflow_status has no row for \
             them, so the dashboard prints `Kode NNNN`. Add the rows in a migration."
        );
    }

    teardown_test_db(&db_name).await;
}

// ─── a row with no code ─────────────────────────────────────────────────────

#[tokio::test]
async fn a_permit_is_born_with_the_code_of_its_status_and_the_dashboard_names_it() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let mut req = server.post("/pemakaian-bmn").json(&json!({
        "pegawai_nip": NIP,
        "tanggal_mulai": "2031-03-01",
        "tanggal_selesai": "2031-03-10",
        "keperluan": "Digunakan untuk pelaksanaan tugas kedinasan sehari-hari",
        "items": [{
            "jenis_bmn": "LAINNYA",
            "bmn_nup": "1",
            "bmn_kode_barang": "3060201003",
            "bmn_nama_barang": "Barang Uji",
        }],
    }));
    for (k, v) in headers("operator_satker", OPERATOR) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 201, "{}", res.text());
    let id: Uuid = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let client = db.pool().get().await.unwrap();
    let row = client
        .query_one(
            "SELECT status, status_kode FROM perlengkapan.izin_pemakaian_bmn WHERE id = $1",
            &[&id],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, String>("status"), "DRAFT");
    assert_eq!(
        row.get::<_, Option<i32>>("status_kode"),
        Some(PemakaianBmnStatus::Draft.to_code()),
        "a permit that has never transitioned still has to be born with its code"
    );

    // What the card is drawn from: the master's name, never the raw word.
    let d = dashboard(&server).await;
    assert_eq!(
        totals(&d["pemakaian_metrics"]),
        vec![("Draft".to_string(), 1)],
        "the dashboard printed the raw word DRAFT for a permit with no code"
    );

    teardown_test_db(&db_name).await;
}

// ─── a state with no code ───────────────────────────────────────────────────

#[tokio::test]
async fn cancelling_a_penghapusan_moves_its_code_and_stops_it_counting_as_a_draft() {
    let (app, db, db_name) = setup_test_app().await;
    let operator = Uuid::parse_str(OPERATOR).unwrap();
    let client = db.pool().get().await.unwrap();

    // One real Draft and one that will be cancelled; both start at 4000.
    let mut ids = Vec::new();
    for nup in ["15", "16"] {
        let row = client
            .query_one(
                "INSERT INTO perlengkapan.penghapusan_bmn
                    (kode_barang, nama_barang, nup, tanggal_penghapusan, alasan,
                     metode_penghapusan, status, status_kode, created_by, satker_code)
                 VALUES ('3060201003', 'Meja Kerja', $1, '2026-01-01', 'rusak', 'LELANG',
                         'DRAFT', $2, $3, $4)
                 RETURNING id",
                &[
                    &nup,
                    &PenghapusanBmnStatus::Draft.to_code(),
                    &operator,
                    &SATKER,
                ],
            )
            .await
            .unwrap();
        ids.push(row.get::<_, Uuid>("id"));
    }
    let server = TestServer::new(app);

    let before = dashboard(&server).await;
    assert_eq!(
        totals(&before["penghapusan_metrics"]),
        vec![("Draft".to_string(), 2)]
    );

    let mut req = server.delete(&format!("/penghapusan-bmn/{}", ids[1]));
    for (k, v) in headers("operator_satker", OPERATOR) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert!(
        res.status_code().is_success(),
        "cancel: {} {}",
        res.status_code(),
        res.text()
    );

    let row = client
        .query_one(
            "SELECT status, status_kode FROM perlengkapan.penghapusan_bmn WHERE id = $1",
            &[&ids[1]],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, String>("status"), "CANCELLED");
    assert_eq!(
        row.get::<_, Option<i32>>("status_kode"),
        Some(PenghapusanBmnStatus::Cancelled.to_code()),
        "cancelling left the code at 4000, so the usulan kept counting as a Draft"
    );

    let after = dashboard(&server).await;
    assert_eq!(
        totals(&after["penghapusan_metrics"]),
        vec![("Dibatalkan".to_string(), 1), ("Draft".to_string(), 1)],
        "one Draft stays a Draft; the cancelled one is its own bucket, not the raw word"
    );

    // The name the LIST shows comes from the same enum, not a spelling of its own.
    let (label, _tone, _hint) = PenghapusanBmnStatus::describe("CANCELLED");
    assert_eq!(label, "Dibatalkan");

    teardown_test_db(&db_name).await;
}

// ─── the backfills ──────────────────────────────────────────────────────────

/// Rows written before the fix still carry the old shape. The two migrations
/// repair them, touch nothing that was already right, and can be applied again.
#[tokio::test]
async fn the_backfills_repair_old_rows_leave_correct_ones_alone_and_are_idempotent() {
    let (_app, db, db_name) = setup_test_app().await;
    let operator = Uuid::parse_str(OPERATOR).unwrap();
    let client = db.pool().get().await.unwrap();

    // An old permit: DRAFT, no code. A permit that is not a draft, deliberately
    // given a code that disagrees with its text: the backfill must not "fix" it.
    let mut permits = Vec::new();
    for (status, kode) in [("DRAFT", None::<i32>), ("ACTIVE", Some(3004))] {
        let id = Uuid::new_v4();
        client
            .execute(
                "INSERT INTO perlengkapan.izin_pemakaian_bmn
                    (id, pegawai_nip, pegawai_nama, pegawai_jabatan, pegawai_satker_id,
                     pegawai_satker_nama, satker_code, tanggal_mulai, tanggal_selesai, status, status_kode)
                 VALUES ($1, $2, 'Budi', 'Staf', gen_random_uuid(), 'KEJAKSAAN NEGERI UJI A',
                         $3, '2031-01-01', '2031-01-31', $4, $5)",
                &[&id, &NIP, &SATKER, &status, &kode],
            )
            .await
            .unwrap();
        permits.push(id);
    }
    // An old cancelled usulan (still 4000) and a real Draft (4000).
    let mut usulan = Vec::new();
    for (nup, status) in [("21", "CANCELLED"), ("22", "DRAFT")] {
        let row = client
            .query_one(
                "INSERT INTO perlengkapan.penghapusan_bmn
                    (kode_barang, nama_barang, nup, tanggal_penghapusan, alasan,
                     metode_penghapusan, status, status_kode, created_by, satker_code)
                 VALUES ('3060201003', 'Meja Kerja', $1, '2026-01-01', 'rusak', 'LELANG',
                         $2, 4000, $3, $4)
                 RETURNING id",
                &[&nup, &status, &operator, &SATKER],
            )
            .await
            .unwrap();
        usulan.push(row.get::<_, Uuid>("id"));
    }

    let v015 = include_str!("../../migrations/V015__pemakaian_draft_status_kode.sql");
    let v016 = include_str!("../../migrations/V016__workflow_status_master_gaps.sql");
    for round in 1..=2 {
        client
            .batch_execute(v015)
            .await
            .unwrap_or_else(|e| panic!("V015 round {round}: {e}"));
        client
            .batch_execute(v016)
            .await
            .unwrap_or_else(|e| panic!("V016 round {round}: {e}"));

        let code = |table: &str, id: Uuid| {
            let q = format!("SELECT status_kode FROM perlengkapan.{table} WHERE id = $1");
            let client = &client;
            async move {
                client
                    .query_one(&q, &[&id])
                    .await
                    .unwrap()
                    .get::<_, Option<i32>>(0)
            }
        };
        assert_eq!(
            code("izin_pemakaian_bmn", permits[0]).await,
            Some(3000),
            "round {round}: old draft"
        );
        assert_eq!(
            code("izin_pemakaian_bmn", permits[1]).await,
            Some(3004),
            "round {round}: correct row untouched"
        );
        assert_eq!(
            code("penghapusan_bmn", usulan[0]).await,
            Some(4009),
            "round {round}: old cancelled"
        );
        assert_eq!(
            code("penghapusan_bmn", usulan[1]).await,
            Some(4000),
            "round {round}: a real Draft keeps 4000"
        );
    }

    teardown_test_db(&db_name).await;
}
