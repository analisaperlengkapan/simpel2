//! The Laporan Kebutuhan BMN status column, and the filter that pairs with it.
//!
//! Two masters describe this one workflow and they disagree from code 2004
//! down:
//!
//! ```text
//! kode   KebutuhanBmnStatus (writes the column)  ms_aktivitas_bmn (was joined)
//! 2004   SubmitPusat "Diajukan ke Validator Pusat"  ANALISIS_KELAYAKAN
//! 2005   AnalisisKelayakan "Analisis Kelayakan"     PENYUSUNAN_PRIORITAS
//! ```
//!
//! Joining the wrong one did more than print a raw token on screen: it told a
//! Pusat validator that an item merely *queued* for them was already under
//! analysis, and offered "Penyusunan Prioritas" — a step the workflow's own
//! config says does not exist — in the filter.
//!
//! Both assertions matter. Checking only "the label is not the raw code"
//! passes just as well against `ms_aktivitas_bmn.deskripsi`, which is prose
//! but still describes the wrong step.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

const PUSAT: &str = "00000000-0000-0000-0000-000000000003";

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

async fn get(server: &TestServer, path: &str) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in auth_headers("validator_pusat", PUSAT, "PUSAT001") {
        req = req.add_header(k, v);
    }
    req.await
}

/// One campaign, one participant parked in each of the two states the two
/// masters disagree about.
async fn seed(server: &TestServer, db: &layanan_perlengkapan::shared::db::Database) {
    let r = {
        let mut req = server.post("/kebutuhan-bmn/pengajuan").json(&json!({
            "nama": "Kebutuhan BMN 2026", "tahun": 2026,
            "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31"
        }));
        for (k, v) in auth_headers("validator_pusat", PUSAT, "PUSAT001") {
            req = req.add_header(k, v);
        }
        req.await
    };
    assert_eq!(r.status_code(), 201, "create pengajuan: {:?}", r.text());
    let body = r.json::<serde_json::Value>();
    let pengajuan_id = body["data"]["pengajuan"]["id"]
        .as_str()
        .or_else(|| body["data"]["id"].as_str())
        .unwrap_or_else(|| panic!("no pengajuan id in {body:?}"))
        .to_string();

    let mut satker_rows = Vec::new();
    for satker in ["SKR001", "SKR002"] {
        let r = {
            let mut req = server
                .post(&format!("/kebutuhan-bmn/pengajuan/{pengajuan_id}/satker"))
                .json(&json!({ "satker_id": satker }));
            for (k, v) in auth_headers("validator_pusat", PUSAT, "PUSAT001") {
                req = req.add_header(k, v);
            }
            req.await
        };
        assert!(
            r.status_code() == 200 || r.status_code() == 201,
            "add satker {satker}: {:?}",
            r.text()
        );
        satker_rows.push(
            r.json::<serde_json::Value>()["data"]["id"]
                .as_str()
                .expect("satker row id")
                .to_string(),
        );
    }

    let client = db.pool().get().await.unwrap();
    // 2004 = SubmitPusat, 2005 = AnalisisKelayakan — per the enum that writes
    // this column, which is the only definition the workflow actually obeys.
    for (row, kode) in satker_rows.iter().zip([2004_i32, 2005_i32]) {
        let id = uuid::Uuid::parse_str(row).unwrap();
        client
            .execute(
                "UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker \
                 SET status_kode = $2 WHERE id = $1",
                &[&id, &kode],
            )
            .await
            .expect("park the participant in a known state");
        client
            .execute(
                "INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_barang \
                 (pengajuan_satker_id, nama, kode_barang, jumlah, satuan) \
                 VALUES ($1, 'Kursi Kerja', '3050102001', 4, 'Unit')",
                &[&id],
            )
            .await
            .expect("seed a requested item");
    }
}

#[tokio::test]
async fn the_recap_names_the_state_the_item_is_actually_in() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed(&server, &db).await;

    let r = get(&server, "/kebutuhan-bmn/laporan/rekap").await;
    assert_eq!(r.status_code(), 200, "rekap: {:?}", r.text());
    let rows = r.json::<serde_json::Value>()["data"]
        .as_array()
        .expect("rekap rows")
        .clone();
    assert_eq!(rows.len(), 2, "one item per participant: {rows:?}");

    let label_of = |kode: i64| -> String {
        rows.iter()
            .find(|r| r["status_kode"].as_i64() == Some(kode))
            .unwrap_or_else(|| panic!("no row with status_kode {kode}: {rows:?}"))["status_nama"]
            .as_str()
            .unwrap_or_else(|| panic!("status_nama is null for {kode}"))
            .to_string()
    };

    // The whole point: 2004 is the queue for Pusat, not the analysis itself.
    assert_eq!(label_of(2004), "Diajukan ke Validator Pusat");
    assert_eq!(label_of(2005), "Analisis Kelayakan");

    // And no row may carry an internal identifier to the screen.
    for row in &rows {
        let label = row["status_nama"].as_str().unwrap();
        assert!(
            !label.contains('_') && label != label.to_uppercase(),
            "raw identifier in the status column: {label}"
        );
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_filter_offers_exactly_the_states_the_workflow_has() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let _ = &db;

    let r = get(&server, "/kebutuhan-bmn/laporan/status-options").await;
    assert_eq!(r.status_code(), 200, "status options: {:?}", r.text());
    let options = r.json::<serde_json::Value>()["data"]
        .as_array()
        .expect("options")
        .clone();
    assert!(
        !options.is_empty(),
        "the filter would have nothing to offer"
    );

    let by_kode = |kode: i64| -> Option<String> {
        options
            .iter()
            .find(|o| o["kode"].as_i64() == Some(kode))
            .map(|o| o["nama"].as_str().unwrap().to_string())
    };

    // The two the hard-coded frontend list had shifted by one.
    assert_eq!(
        by_kode(2004).as_deref(),
        Some("Diajukan ke Validator Pusat")
    );
    assert_eq!(by_kode(2005).as_deref(), Some("Analisis Kelayakan"));

    // "Penyusunan Prioritas" was offered by the old list; the workflow config
    // says the step was merged away and never existed as its own state.
    for o in &options {
        let nama = o["nama"].as_str().unwrap();
        assert!(
            !nama.eq_ignore_ascii_case("Penyusunan Prioritas"),
            "the filter offers a step this workflow does not have"
        );
        assert!(
            !nama.contains('_') && nama != nama.to_uppercase(),
            "raw identifier in the filter: {nama}"
        );
    }

    // Every state the enum can write must be selectable, or the filter can
    // hide rows the table shows. Derived from the enum, not re-listed here.
    use layanan_perlengkapan::kebutuhan_bmn::models::KebutuhanBmnStatus;
    for kode in 2000..=2010 {
        if let Some(status) = KebutuhanBmnStatus::from_code(kode) {
            assert_eq!(
                by_kode(kode as i64).as_deref(),
                Some(status.label()),
                "filter and enum disagree about {kode}"
            );
        }
    }

    teardown_test_db(&db_name).await;
}

/// The campaign list reads `vw_kebutuhan_bmn_summary`, which joined the same
/// wrong master. On staging every row in this list read a bare `DRAFT`.
#[tokio::test]
async fn the_campaign_list_shows_a_label_not_a_token() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed(&server, &db).await;

    let r = get(&server, "/kebutuhan-bmn/pengajuan?page=1&per_page=10").await;
    assert_eq!(r.status_code(), 200, "list: {:?}", r.text());
    let body = r.json::<serde_json::Value>();
    let rows = body["data"]["data"]
        .as_array()
        .or_else(|| body["data"].as_array())
        .unwrap_or_else(|| panic!("no rows in {body:?}"))
        .clone();
    assert!(!rows.is_empty(), "the seed created a campaign: {rows:?}");

    for row in &rows {
        let label = row["status_nama"]
            .as_str()
            .unwrap_or_else(|| panic!("status_nama missing: {row:?}"));
        assert!(
            !label.contains('_') && label != label.to_uppercase(),
            "raw identifier in the campaign list: {label}"
        );
    }

    // "Draft" reads the same in both masters, so passing on a Draft-only list
    // proves nothing. Move the campaign to 2004 — the first code where the two
    // masters disagree — and pin the label.
    {
        let client = db.pool().get().await.unwrap();
        client
            .execute(
                "UPDATE perlengkapan.pengajuan_kebutuhan_bmn SET status_kode = 2004",
                &[],
            )
            .await
            .expect("park the campaign where the masters disagree");
    }
    let r = get(&server, "/kebutuhan-bmn/pengajuan?page=1&per_page=10").await;
    let body = r.json::<serde_json::Value>();
    let rows = body["data"]["data"]
        .as_array()
        .or_else(|| body["data"].as_array())
        .unwrap()
        .clone();
    assert!(
        rows.iter()
            .all(|r| r["status_nama"] == "Diajukan ke Validator Pusat"),
        "2004 is the queue for Pusat, not the analysis: {rows:?}"
    );

    teardown_test_db(&db_name).await;
}
