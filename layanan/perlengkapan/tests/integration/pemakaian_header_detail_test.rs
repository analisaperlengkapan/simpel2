//! One permohonan = one pegawai holding a LIST of BMN (V014).
//!
//! Until V014 the FIRST asset lived in columns on `izin_pemakaian_bmn` and the
//! rest lived in `pemakaian_bmn_items`. Two shapes for one thing, and every
//! reader only ever learned the first:
//!
//!   * the SK printed one row — `sk_izin_pdf.rs` said so in a comment;
//!   * `find_booking_conflict` read only the parent, so an asset held as an
//!     item looked FREE to the next applicant and could be lent out twice;
//!   * `renew_permit` sent `additional_bmn_items: vec![]`, dropping every
//!     asset but the first at renewal.
//!
//! Each of those is a test below, written so it FAILS against the old shape
//! rather than merely passing against the new one. The suite that shipped
//! beside the old code — `pemakaian_bmn_service_tests.rs`, deleted with this
//! change — asserted hard-coded arrays against themselves and passed all eight
//! of its tests against a build with `bmn_nup` removed from the permit
//! entirely.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const VALIDATOR_SATKER: &str = "00000000-0000-0000-0000-000000000002";
const APPROVER_SATKER: &str = "00000000-0000-0000-0000-000000000003";
const SATKER: &str = "SKR001";

fn auth_headers(
    role: &str,
    user_id: &str,
    satker_id: &str,
) -> Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)> {
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
                "Bearer mock::{role}::{user_id}::{satker_id}"
            ))
            .unwrap(),
        ),
    ]
}

/// A laptop item. Laptops are the least demanding kind — the vehicle and
/// housing CHECKs would otherwise decide what these tests are measuring.
fn laptop(nup: &str, nama: &str) -> Value {
    json!({
        "jenis_bmn": "LAPTOP",
        "bmn_nup": nup,
        "bmn_kode_barang": "3060201003",
        "bmn_nama_barang": nama,
        "bmn_merk": "Dell",
        "serial_number": format!("SN-{nup}")
    })
}

fn create_body(items: Vec<Value>) -> Value {
    json!({
        "pegawai_nip": "198501012010011001",
        "pegawai_nama": "Budi Santoso",
        "pegawai_satker_id": "00000000-0000-0000-0000-000000000001",
        "pegawai_satker_nama": "Kejaksaan Negeri Jakarta Selatan",
        "tanggal_mulai": "2026-01-01",
        "tanggal_selesai": "2026-12-31",
        "keperluan": "Digunakan untuk pelaksanaan tugas kedinasan sehari-hari",
        "items": items
    })
}

async fn post(server: &TestServer, path: &str, role: &str, user: &str, body: Value) -> Value {
    let mut req = server.post(path).json(&body);
    for (k, v) in auth_headers(role, user, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let status = res.status_code();
    let teks = res.text();
    assert!(status.is_success(), "POST {path} -> {status}: {teks}");
    serde_json::from_str(&teks).unwrap()
}

async fn create_permit(server: &TestServer, items: Vec<Value>) -> String {
    post(
        server,
        "/pemakaian-bmn",
        "operator_satker",
        OPERATOR,
        create_body(items),
    )
    .await["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

/// submit → forward → approve, which auto-activates. Distinct users per step:
/// the maker-checker rule refuses a validator or approver who is also the
/// requester.
async fn drive_existing_to_active(server: &TestServer, id: &str) {
    post(
        server,
        &format!("/pemakaian-bmn/{id}/transition"),
        "operator_satker",
        OPERATOR,
        json!({"target_status": "SUBMITTED", "catatan": "Mohon validasi"}),
    )
    .await;
    post(
        server,
        &format!("/pemakaian-bmn/{id}/validator-satker-action"),
        "validator_satker",
        VALIDATOR_SATKER,
        json!({"action": "forward", "expected_version": 1, "catatan": "Diteruskan"}),
    )
    .await;
    post(
        server,
        &format!("/pemakaian-bmn/{id}/approver-satker-action"),
        "approver_satker",
        APPROVER_SATKER,
        json!({"action": "approve", "expected_version": 2, "catatan": "Disetujui"}),
    )
    .await;
}

async fn drive_to_active(server: &TestServer, items: Vec<Value>) -> String {
    let id = create_permit(server, items).await;
    drive_existing_to_active(server, &id).await;
    id
}

async fn get_permit(server: &TestServer, id: &str) -> Value {
    let mut req = server.get(&format!("/pemakaian-bmn/{id}"));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "detail: {}", res.text());
    res.json::<Value>()
}

/// The asset list of a permit, read from the detail response.
///
/// A helper rather than an inline path because `["a"]["b"]` on a wrong path
/// yields `Null`, and `.as_array().unwrap_or_default()` then reports an empty
/// list — so "the field moved" and "the permit has no assets" would look
/// identical. This names which it is. (`IzinPemakaianDetailResponse`
/// `#[serde(flatten)]`s the permit, so its fields sit directly under `data`.)
fn items_of(detail: &Value) -> &Vec<Value> {
    detail["data"]["bmn_items"]
        .as_array()
        .unwrap_or_else(|| panic!("data.bmn_items is not an array; response was: {detail}"))
}

/// Every asset survives the round trip, in the order it was sent.
///
/// Order matters because `urutan` is what numbers the rows on page 2 of the
/// SK; reading it back by `created_at` would be non-deterministic for items
/// written inside one transaction.
#[tokio::test]
async fn every_asset_on_a_permit_comes_back_in_order() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let id = create_permit(
        &server,
        vec![
            laptop("501", "Laptop Pertama"),
            laptop("502", "Laptop Kedua"),
            laptop("503", "Laptop Ketiga"),
        ],
    )
    .await;

    let detail = get_permit(&server, &id).await;
    let items = items_of(&detail);
    assert_eq!(items.len(), 3, "three assets were sent, three must return");
    let urutan: Vec<i64> = items.iter().map(|i| i["urutan"].as_i64().unwrap()).collect();
    assert_eq!(urutan, vec![1, 2, 3], "urutan numbers the SK rows");
    let nup: Vec<&str> = items
        .iter()
        .map(|i| i["bmn_nup"].as_str().unwrap())
        .collect();
    assert_eq!(nup, vec!["501", "502", "503"], "order as submitted");

    teardown_test_db(&db_name).await;
}

/// The header's period is MIN/MAX over the items.
///
/// This pins the one denormalisation V014 keeps. The header dates stay because
/// the expiry scheduler, two dozen indexes and every monitoring query read
/// them; `sync_permit_window` is their only writer, and if it ever stops
/// running this test is what says so.
#[tokio::test]
async fn the_permit_window_tracks_its_items() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let mut awal = laptop("511", "Laptop Awal");
    awal["tanggal_mulai"] = json!("2026-03-01");
    awal["tanggal_selesai"] = json!("2026-06-30");
    let mut akhir = laptop("512", "Laptop Akhir");
    akhir["tanggal_mulai"] = json!("2026-05-01");
    akhir["tanggal_selesai"] = json!("2026-09-30");

    let id = create_permit(&server, vec![awal, akhir]).await;
    let izin = get_permit(&server, &id).await["data"].clone();

    assert_eq!(
        izin["tanggal_mulai"].as_str(),
        Some("2026-03-01"),
        "window opens with the earliest asset, not with the request default"
    );
    assert_eq!(
        izin["tanggal_selesai"].as_str(),
        Some("2026-09-30"),
        "window closes with the latest asset"
    );

    teardown_test_db(&db_name).await;
}

/// An asset held as the SECOND item of an active permit is not available.
///
/// This is the double-lending bug. `find_booking_conflict` read only the
/// parent table, so anything past the first asset was invisible and a second
/// permit over it was accepted with a 201.
#[tokio::test]
async fn an_asset_held_beyond_the_first_is_not_free() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    drive_to_active(
        &server,
        vec![laptop("521", "Laptop Utama"), laptop("522", "Laptop Kedua")],
    )
    .await;

    // The SECOND asset of that permit, requested by a new permohonan.
    let mut req = server
        .post("/pemakaian-bmn")
        .json(&create_body(vec![laptop("522", "Laptop Kedua")]));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let teks = res.text();
    assert_eq!(
        res.status_code(),
        400,
        "an asset already held must not be lent again: {teks}"
    );
    assert!(
        teks.contains("522"),
        "the message must name the asset that clashed: {teks}"
    );

    // The opposite direction, so this is not just "create always fails": an
    // asset nobody holds is still accepted.
    let bebas = create_permit(&server, vec![laptop("529", "Laptop Bebas")]).await;
    assert!(!bebas.is_empty());

    teardown_test_db(&db_name).await;
}

/// Renewal carries the whole list, and supersedes what it renews.
///
/// Two failures in one path: renewal used to send `additional_bmn_items:
/// vec![]`, so a two-asset permit renewed into a one-asset permit and the
/// second asset was left with no permit at all; and nothing revoked the permit
/// being replaced, so both stayed ACTIVE over the same assets.
#[tokio::test]
async fn a_renewal_carries_every_asset_and_supersedes_the_old_permit() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let lama = drive_to_active(
        &server,
        vec![laptop("531", "Laptop A"), laptop("532", "Laptop B")],
    )
    .await;

    let baru = post(
        &server,
        &format!("/pemakaian-bmn/{lama}/renew"),
        "operator_satker",
        OPERATOR,
        json!({
            "tanggal_mulai": "2027-01-01",
            "tanggal_selesai": "2027-12-31",
            "keperluan": "Perpanjangan untuk tugas kedinasan tahun berikutnya"
        }),
    )
    .await["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let detail = get_permit(&server, &baru).await;
    let items = items_of(&detail);
    assert_eq!(
        items.len(),
        2,
        "the renewal must carry BOTH assets, not just the first"
    );
    assert_eq!(
        items[0]["tanggal_selesai"].as_str(),
        Some("2027-12-31"),
        "a renewal is a NEW period — carrying the old dates would produce a \
         renewal that expired before it was signed"
    );

    // A renewal still in DRAFT revokes nothing.
    let sebelum = get_permit(&server, &lama).await["data"]["status"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(sebelum, "ACTIVE", "a renewal still in DRAFT revokes nothing");

    drive_existing_to_active(&server, &baru).await;

    let lama_izin = get_permit(&server, &lama).await["data"].clone();
    assert_eq!(
        lama_izin["status"].as_str(),
        Some("REVOKED"),
        "the permit a renewal replaces is withdrawn the moment the new one is \
         in force — otherwise both are ACTIVE over the same assets"
    );
    assert_eq!(
        lama_izin["superseded_by_id"].as_str(),
        Some(baru.as_str()),
        "and it points at what replaced it"
    );
    assert!(
        lama_izin["catatan_revocation"]
            .as_str()
            .unwrap_or_default()
            .contains("Digantikan"),
        "the note says it was replaced, not merely revoked: {lama_izin}"
    );

    teardown_test_db(&db_name).await;
}

/// A permit has to hold something, and cannot hold the same thing twice.
///
/// The duplicate is a mistake the list makes possible and the booking check
/// cannot catch: neither row exists yet, so neither conflicts with the other.
#[tokio::test]
async fn a_permit_needs_at_least_one_asset_and_no_repeats() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    for (items, penanda, kenapa) in [
        (vec![], "Minimal satu BMN", "a permit with no asset is not a permit"),
        (
            vec![laptop("541", "Laptop A"), laptop("541", "Laptop A")],
            "lebih dari sekali",
            "the same asset twice on one permit",
        ),
    ] {
        let mut req = server.post("/pemakaian-bmn").json(&create_body(items));
        for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
            req = req.add_header(k, v);
        }
        let res = req.await;
        let teks = res.text();
        assert_eq!(res.status_code(), 400, "{kenapa}: {teks}");
        assert!(
            teks.contains(penanda),
            "{kenapa} — message must say which rule: {teks}"
        );
    }

    teardown_test_db(&db_name).await;
}
