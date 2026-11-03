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
        "pegawai_nip": "19800101000000001",
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
    let urutan: Vec<i64> = items
        .iter()
        .map(|i| i["urutan"].as_i64().unwrap())
        .collect();
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
    assert_eq!(
        sebelum, "ACTIVE",
        "a renewal still in DRAFT revokes nothing"
    );

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
    assert_eq!(
        lama_izin["revoked_by"].as_str(),
        Some(APPROVER_SATKER),
        "the human whose approval activated the renewal is on the record, as \
         the two-step version stored — the trigger alone cannot know them: {lama_izin}"
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
        (
            vec![],
            "Minimal satu BMN",
            "a permit with no asset is not a permit",
        ),
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

// ---------------------------------------------------------------------------
// Review round on the header/detail change
// ---------------------------------------------------------------------------

async fn put(server: &TestServer, id: &str, body: Value) -> axum_test::TestResponse {
    let mut req = server.put(&format!("/pemakaian-bmn/{id}")).json(&body);
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    req.await
}

async fn renew(server: &TestServer, lama: &str) -> String {
    post(
        server,
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
        .to_string()
}

async fn status_of(server: &TestServer, id: &str) -> String {
    get_permit(server, id).await["data"]["status"]
        .as_str()
        .unwrap()
        .to_string()
}

/// A draft's dates cannot be edited on their own.
///
/// The header's dates are derived from the assets' periods, and the update
/// used to write them straight to the header when the request carried no
/// `items`. The permit then said 2027 while every asset — and page 2 of the SK
/// — still said 2026. The edit is now refused with a sentence, and when the
/// assets ARE sent the new date becomes their default, so both agree.
#[tokio::test]
async fn a_draft_cannot_change_its_dates_without_its_assets() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let id = create_permit(&server, vec![laptop("541", "Laptop A")]).await;

    let res = put(&server, &id, json!({"tanggal_selesai": "2027-12-31"})).await;
    let teks = res.text();
    assert_eq!(
        res.status_code(),
        400,
        "date-only edit must be refused: {teks}"
    );
    assert!(
        teks.contains("daftar BMN"),
        "the message must say what to send instead: {teks}"
    );

    // Nothing moved: the permit and its asset still agree, on the old date.
    let detail = get_permit(&server, &id).await;
    assert_eq!(
        detail["data"]["tanggal_selesai"].as_str(),
        Some("2026-12-31")
    );
    assert_eq!(
        items_of(&detail)[0]["tanggal_selesai"].as_str(),
        Some("2026-12-31"),
        "a refused edit must leave the asset alone"
    );

    // The same date WITH the asset list: accepted, and header and asset move
    // together because the asset takes the date as its default period.
    let res = put(
        &server,
        &id,
        json!({
            "tanggal_selesai": "2027-12-31",
            "items": [laptop("541", "Laptop A")]
        }),
    )
    .await;
    assert_eq!(
        res.status_code(),
        200,
        "date with its assets: {}",
        res.text()
    );
    let detail = get_permit(&server, &id).await;
    assert_eq!(
        detail["data"]["tanggal_selesai"].as_str(),
        Some("2027-12-31")
    );
    assert_eq!(
        items_of(&detail)[0]["tanggal_selesai"].as_str(),
        Some("2027-12-31"),
        "the header's date and the asset's must be the same answer"
    );

    // And an edit that touches neither date still works.
    let res = put(
        &server,
        &id,
        json!({"keperluan": "Keperluan yang sudah diperbarui oleh operator"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "non-date edit: {}", res.text());

    teardown_test_db(&db_name).await;
}

/// Activating a renewal withdraws the permit it renews IN THE SAME STATEMENT.
///
/// Driven by a plain `UPDATE ... SET status = 'ACTIVE'`, not through the
/// application: the previous version withdrew the old permit from a second
/// call after the workflow engine had already committed, so a failure between
/// the two left both ACTIVE. A test that went through the service could not
/// tell "atomic" from "two calls that happened to succeed"; a bare UPDATE can.
#[tokio::test]
async fn activating_a_renewal_withdraws_its_predecessor_in_the_same_statement() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let lama = drive_to_active(
        &server,
        vec![laptop("551", "Laptop A"), laptop("552", "Laptop B")],
    )
    .await;
    let baru = renew(&server, &lama).await;
    assert_eq!(status_of(&server, &lama).await, "ACTIVE", "precondition");
    assert!(
        get_permit(&server, &lama).await["data"]["days_until_expiry"].is_number(),
        "a permit in force counts down to its end date"
    );

    let client = db.pool().get().await.unwrap();
    let baru_uuid = uuid::Uuid::parse_str(&baru).unwrap();
    client
        .execute(
            "UPDATE perlengkapan.izin_pemakaian_bmn SET status = 'ACTIVE' WHERE id = $1",
            &[&baru_uuid],
        )
        .await
        .unwrap();

    let izin = get_permit(&server, &lama).await["data"].clone();
    assert_eq!(
        izin["status"].as_str(),
        Some("REVOKED"),
        "no path to ACTIVE may leave the replaced permit running"
    );
    // A withdrawn permit has no time left to show. The detail page used to read
    // "59 hari lagi" off its end date.
    assert!(
        izin["days_until_expiry"].is_null(),
        "a REVOKED permit still counts down: {izin}"
    );
    assert_eq!(izin["is_expiring_soon"], false);
    assert_eq!(izin["superseded_by_id"].as_str(), Some(baru.as_str()));
    assert_eq!(
        izin["revoked_by_nama"].as_str(),
        Some("system"),
        "the withdrawal is the system's act"
    );
    assert!(
        izin["revoked_by"].is_null(),
        "an activation that bypassed the service names no human — NULL means \
         unknown, not a guess from updated_by: {izin}"
    );
    let kode: i32 = client
        .query_one(
            "SELECT status_kode FROM perlengkapan.izin_pemakaian_bmn WHERE id = $1",
            &[&uuid::Uuid::parse_str(&lama).unwrap()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        kode, 3006,
        "the denormalised status_kode must follow, or the internal-satker gates \
         still see an active permit"
    );

    teardown_test_db(&db_name).await;
}

/// Two renewals of one permit cannot both go live; the loser is rolled back.
///
/// The first activation replaces the permit it renews. The second would be
/// "renewing" something that is already gone, and the old code treated the
/// missing predecessor as a successful no-op — so BOTH renewals ended ACTIVE
/// over the same assets. Checked twice: through the application (a clear 409)
/// and with a bare UPDATE, which is the race the application check cannot see
/// and the trigger must refuse by itself, taking the activation down with it.
#[tokio::test]
async fn a_second_renewal_of_a_superseded_permit_does_not_go_live() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let lama = drive_to_active(&server, vec![laptop("561", "Laptop A")]).await;
    let pertama = renew(&server, &lama).await;
    let kedua = renew(&server, &lama).await;

    drive_existing_to_active(&server, &pertama).await;
    assert_eq!(status_of(&server, &pertama).await, "ACTIVE");
    assert_eq!(status_of(&server, &lama).await, "REVOKED");

    // Through the application: refused with a clear conflict, not a 500.
    post(
        &server,
        &format!("/pemakaian-bmn/{kedua}/transition"),
        "operator_satker",
        OPERATOR,
        json!({"target_status": "SUBMITTED", "catatan": "Mohon validasi"}),
    )
    .await;
    post(
        &server,
        &format!("/pemakaian-bmn/{kedua}/validator-satker-action"),
        "validator_satker",
        VALIDATOR_SATKER,
        json!({"action": "forward", "expected_version": 1, "catatan": "Diteruskan"}),
    )
    .await;
    let mut req = server
        .post(&format!("/pemakaian-bmn/{kedua}/approver-satker-action"))
        .json(&json!({"action": "approve", "expected_version": 2, "catatan": "Disetujui"}));
    for (k, v) in auth_headers("approver_satker", APPROVER_SATKER, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_ne!(
        status_of(&server, &kedua).await,
        "ACTIVE",
        "the second renewal must not be live after approval (response {}: {})",
        res.status_code(),
        res.text()
    );

    // Underneath the application: the trigger refuses the bare UPDATE and the
    // statement leaves no trace.
    let client = db.pool().get().await.unwrap();
    let kedua_uuid = uuid::Uuid::parse_str(&kedua).unwrap();
    let hasil = client
        .execute(
            "UPDATE perlengkapan.izin_pemakaian_bmn SET status = 'ACTIVE' WHERE id = $1",
            &[&kedua_uuid],
        )
        .await;
    let err = hasil.expect_err("a second renewal must not be activatable");
    // `Display` on a tokio_postgres error is just "db error"; the trigger's
    // sentence is on the server-side error object.
    let pesan = err
        .as_db_error()
        .map(|d| d.message().to_string())
        .unwrap_or_else(|| err.to_string());
    assert!(
        pesan.contains("sudah digantikan"),
        "the refusal must say why: {pesan}"
    );
    assert_ne!(
        status_of(&server, &kedua).await,
        "ACTIVE",
        "a refused activation must roll back with the statement"
    );
    assert_eq!(
        status_of(&server, &pertama).await,
        "ACTIVE",
        "and the renewal that won stays in force"
    );

    teardown_test_db(&db_name).await;
}

/// V014 is the EXPAND half of an expand/contract: the columns the previous
/// release reads are still there and still true.
///
/// A permit written by this release must stay readable by the one before it,
/// or `helm rollback` has to be followed by a database restore. The previous
/// release selects the asset columns off the permit row; they are a mirror of
/// asset #1, refreshed after every item write.
#[tokio::test]
async fn the_previous_release_can_still_read_the_permit() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let client = db.pool().get().await.unwrap();

    let id = create_permit(
        &server,
        vec![
            laptop("571", "Laptop Pertama"),
            laptop("572", "Laptop Kedua"),
        ],
    )
    .await;
    let uuid = uuid::Uuid::parse_str(&id).unwrap();

    let baca = || async {
        let r = client
            .query_one(
                "SELECT jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang \
                 FROM perlengkapan.izin_pemakaian_bmn WHERE id = $1",
                &[&uuid],
            )
            .await
            .unwrap();
        // Typed `String`, not `Option<String>`: the previous release decoded
        // these as non-null, and a NULL here is a decode failure there.
        (
            r.get::<_, String>(0),
            r.get::<_, String>(1),
            r.get::<_, String>(2),
            r.get::<_, String>(3),
        )
    };

    let (jenis, nup, kode, nama) = baca().await;
    assert_eq!(
        (jenis.as_str(), nup.as_str(), nama.as_str()),
        ("LAPTOP", "571", "Laptop Pertama"),
        "the mirror is asset #1 (kode {kode})"
    );

    // Reordered: the mirror follows whatever is first now.
    let res = put(
        &server,
        &id,
        json!({"items": [laptop("572", "Laptop Kedua"), laptop("571", "Laptop Pertama")]}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let (_, nup, _, nama) = baca().await;
    assert_eq!(
        (nup.as_str(), nama.as_str()),
        ("572", "Laptop Kedua"),
        "the mirror must follow item #1 when the list is rewritten"
    );

    teardown_test_db(&db_name).await;
}

/// The startup normaliser backfills rows that PREDATE the V014 triggers.
///
/// Legacy items and header-held first assets written before V014 ran have no
/// position, and nothing has fired for them: refinery runs V014 once, so only
/// the normaliser repairs them, before the service serves. The triggers are
/// switched off while those rows are written, because with them on every legacy
/// insert is already normalised and this would test nothing. The end-to-end
/// version, through the real runner, is in `migration_runner_test`; writes that
/// arrive AFTER a pod has started are `legacy_writes_after_startup_are_readable_at_once`.
#[tokio::test]
async fn the_startup_normaliser_backfills_rows_that_predate_the_triggers() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let client = db.pool().get().await.unwrap();

    let sudah = create_permit(
        &server,
        vec![laptop("581", "Laptop A"), laptop("582", "Laptop B")],
    )
    .await;
    let sudah_uuid = uuid::Uuid::parse_str(&sudah).unwrap();

    let snapshot = || async {
        client
            .query_one(
                "SELECT md5(string_agg(i::text, '|' ORDER BY urutan)) \
                 FROM perlengkapan.pemakaian_bmn_items i WHERE izin_pemakaian_id = $1",
                &[&sudah_uuid],
            )
            .await
            .unwrap()
            .get::<_, String>(0)
    };
    let sebelum = snapshot().await;

    // The previous release's shapes, as they stood BEFORE V014 existed: asset
    // on the parent, an extra item with none of the V014 columns, and no
    // trigger to normalise either.
    client
        .batch_execute(
            "ALTER TABLE perlengkapan.izin_pemakaian_bmn \
               DISABLE TRIGGER trg_izin_pemakaian_legacy_item_one; \
             ALTER TABLE perlengkapan.pemakaian_bmn_items \
               DISABLE TRIGGER trg_pemakaian_items_legacy_defaults; \
             INSERT INTO perlengkapan.izin_pemakaian_bmn \
               (id, jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang, no_polisi, no_bpkb, \
                pegawai_nip, pegawai_nama, pegawai_satker_id, tanggal_mulai, tanggal_selesai, status) \
             VALUES ('a0000000-0000-4000-8000-0000000000c1', 'KENDARAAN_BERMOTOR', '583', \
                '3020104001', 'Sedan Lama', 'B 9 ZZ', 'BPKB-9', '198501012010011003', 'Dewi', \
                '00000000-0000-0000-0000-000000000001', '2026-03-01', '2026-09-30', 'DRAFT'); \
             INSERT INTO perlengkapan.pemakaian_bmn_items \
               (izin_pemakaian_id, bmn_nup, bmn_kode_barang, bmn_nama_barang, created_at, updated_at) \
             VALUES ('a0000000-0000-4000-8000-0000000000c1', '584', '3060201003', \
                'Laptop kode lama', now(), now()); \
             ALTER TABLE perlengkapan.izin_pemakaian_bmn \
               ENABLE TRIGGER trg_izin_pemakaian_legacy_item_one; \
             ALTER TABLE perlengkapan.pemakaian_bmn_items \
               ENABLE TRIGGER trg_pemakaian_items_legacy_defaults;",
        )
        .await
        .expect("the previous release's writes must still be accepted by the V014 schema");
    let sisa: i64 = client
        .query_one(
            "SELECT count(*) FROM perlengkapan.pemakaian_bmn_items WHERE urutan IS NULL",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(sisa, 1, "precondition: the legacy item has no position");

    layanan_perlengkapan::migrations::normalize_pemakaian_items(db.pool())
        .await
        .expect("the normaliser must run");

    let ambil = || async {
        client
            .query(
                "SELECT urutan, jenis_bmn, bmn_nup FROM perlengkapan.pemakaian_bmn_items \
                 WHERE izin_pemakaian_id = 'a0000000-0000-4000-8000-0000000000c1' ORDER BY urutan",
                &[],
            )
            .await
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r.get::<_, i32>(0),
                    r.get::<_, String>(1),
                    r.get::<_, String>(2),
                )
            })
            .collect::<Vec<_>>()
    };
    let diharapkan = vec![
        (1, "KENDARAAN_BERMOTOR".to_string(), "583".to_string()),
        (2, "LAINNYA".to_string(), "584".to_string()),
    ];
    assert_eq!(
        ambil().await,
        diharapkan,
        "the parent's asset becomes slot 1 and the old item follows it; the \
         item's kind is LAINNYA because the old schema never recorded one"
    );
    assert_eq!(
        snapshot().await,
        sebelum,
        "a permit this release wrote must be untouched"
    );

    // Idempotent: a start with nothing to repair changes nothing.
    layanan_perlengkapan::migrations::normalize_pemakaian_items(db.pool())
        .await
        .unwrap();
    assert_eq!(ambil().await, diharapkan, "second run must be a no-op");
    assert_eq!(snapshot().await, sebelum);

    teardown_test_db(&db_name).await;
}

/// What the previous release writes for one permit: the first asset on the
/// header, then each extra asset as an item with none of the V014 columns —
/// exactly its own INSERT shapes, in the order it issues them.
fn legacy_sql(id: &str, nup: &str, extras: &[(&str, &str)]) -> String {
    let mut sql = format!(
        "INSERT INTO perlengkapan.izin_pemakaian_bmn \
           (id, jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang, \
            pegawai_nip, pegawai_nama, pegawai_satker_id, pegawai_satker_nama, satker_code, \
            tanggal_mulai, tanggal_selesai, status) \
         VALUES ('{id}', 'LAPTOP', '{nup}', '3060201003', 'Laptop Induk {nup}', \
            '198501012010011009', 'Eko', '00000000-0000-0000-0000-000000000001', \
            'Kejaksaan Negeri Jakarta Selatan', '{SATKER}', \
            '2026-04-01', '2026-10-31', 'DRAFT');"
    );
    for (no, nama) in extras {
        sql.push_str(&format!(
            " INSERT INTO perlengkapan.pemakaian_bmn_items \
                (izin_pemakaian_id, bmn_nup, bmn_kode_barang, bmn_nama_barang, created_at, updated_at) \
              VALUES ('{id}', '{no}', '3060201003', '{nama}', now(), now());"
        ));
    }
    sql
}

/// A legacy write that lands AFTER a new pod has started is readable at once.
///
/// The case the startup normaliser cannot see. During a rolling upgrade old and
/// new pods serve side by side: the new pod has already normalised and begun
/// serving, then an old pod inserts a permit and an extra asset in its own
/// shape. `PemakaianBmnItem::from_row` reads those fields as required, so
/// without a repair AT INSERT every request for that permit would panic until
/// some pod restarted. Driven through the API after the normaliser has run —
/// the order that matters — for both ways it happens: a whole permit written by
/// the old release, and one extra item added to a permit the new release wrote.
#[tokio::test]
async fn legacy_writes_after_startup_are_readable_at_once() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let client = db.pool().get().await.unwrap();

    // "The new pod has started": its startup pass has already run.
    layanan_perlengkapan::migrations::normalize_pemakaian_items(db.pool())
        .await
        .unwrap();

    let baru = create_permit(&server, vec![laptop("611", "Laptop Baru")]).await;

    // An old pod, afterwards: a permit of its own with two extras, and one more
    // asset added to the permit the new release wrote.
    let lama = "a0000000-0000-4000-8000-0000000000e1";
    client
        .batch_execute(&legacy_sql(
            lama,
            "621",
            &[("622", "Laptop Tambahan A"), ("623", "Laptop Tambahan B")],
        ))
        .await
        .expect("the previous release's writes must be accepted");
    client
        .batch_execute(&format!(
            "INSERT INTO perlengkapan.pemakaian_bmn_items \
               (izin_pemakaian_id, bmn_nup, bmn_kode_barang, bmn_nama_barang, created_at, updated_at) \
             VALUES ('{baru}', '612', '3060201003', 'Tambahan dari pod lama', now(), now());"
        ))
        .await
        .expect("an old pod adding an asset to a new permit");

    // The permit the old release wrote: its header asset is slot 1, the extras
    // follow in the order they were inserted, nothing is left NULL.
    let detail = get_permit(&server, lama).await;
    let nup: Vec<&str> = items_of(&detail)
        .iter()
        .map(|i| i["bmn_nup"].as_str().unwrap())
        .collect();
    assert_eq!(nup, vec!["621", "622", "623"], "{detail}");
    let urutan: Vec<i64> = items_of(&detail)
        .iter()
        .map(|i| i["urutan"].as_i64().expect("urutan must not be null"))
        .collect();
    assert_eq!(urutan, vec![1, 2, 3]);
    for i in items_of(&detail) {
        assert!(!i["jenis_bmn"].is_null(), "{i}");
        assert!(!i["tanggal_mulai"].is_null(), "{i}");
        assert!(!i["tanggal_selesai"].is_null(), "{i}");
    }
    assert_eq!(
        items_of(&detail)[0]["jenis_bmn"],
        "LAPTOP",
        "slot 1 keeps the header's kind"
    );
    assert_eq!(
        items_of(&detail)[1]["jenis_bmn"],
        "LAINNYA",
        "an old extra item never recorded a kind; none is invented"
    );
    assert_eq!(
        items_of(&detail)[1]["tanggal_selesai"],
        "2026-10-31",
        "the period defaults to the permit's, as the old release implied"
    );

    // The permit the new release wrote, with an old pod's asset after its own.
    let detail = get_permit(&server, &baru).await;
    let nup: Vec<&str> = items_of(&detail)
        .iter()
        .map(|i| i["bmn_nup"].as_str().unwrap())
        .collect();
    assert_eq!(nup, vec!["611", "612"], "{detail}");

    // Same result as the startup pass would give for the same input — two
    // mechanisms, one meaning. The snapshot is taken BEFORE the pass runs: a
    // comparison of two reads that both follow it could not see the pass
    // rewrite an item, because both reads would carry the rewrite.
    let lama_sebelum = items_of(&get_permit(&server, lama).await).clone();
    let baru_sebelum = items_of(&get_permit(&server, &baru).await).clone();
    assert_eq!(lama_sebelum.len(), 3, "the snapshot must hold the items");
    assert_eq!(baru_sebelum.len(), 2, "the snapshot must hold the items");

    layanan_perlengkapan::migrations::normalize_pemakaian_items(db.pool())
        .await
        .unwrap();

    assert_eq!(
        items_of(&get_permit(&server, lama).await),
        &lama_sebelum,
        "the startup pass must not touch an item the triggers already normalised"
    );
    assert_eq!(
        items_of(&get_permit(&server, &baru).await),
        &baru_sebelum,
        "nor an item written by this release"
    );

    teardown_test_db(&db_name).await;
}

/// Several old pods inserting for one permit at once each get their own
/// position. Without the permit lock they would all compute the same `max + 1`
/// and all but one would die on the `(izin, urutan)` unique constraint — an old
/// pod's request failing because a new release shipped.
#[tokio::test]
async fn concurrent_legacy_inserts_for_one_permit_all_land() {
    let (_app, db, db_name) = setup_test_app().await;
    let lama = "a0000000-0000-4000-8000-0000000000e2";
    {
        let client = db.pool().get().await.unwrap();
        client
            .batch_execute(&legacy_sql(lama, "631", &[]))
            .await
            .unwrap();
    }

    let mut tugas = Vec::new();
    for n in 0..8 {
        let pool = db.pool().clone();
        tugas.push(tokio::spawn(async move {
            let client = pool.get().await.unwrap();
            client
                .batch_execute(&format!(
                    "INSERT INTO perlengkapan.pemakaian_bmn_items \
                       (izin_pemakaian_id, bmn_nup, bmn_kode_barang, bmn_nama_barang, created_at, updated_at) \
                     VALUES ('{lama}', '{}', '3060201003', 'Serentak {n}', now(), now());",
                    640 + n
                ))
                .await
        }));
    }
    for t in tugas {
        t.await
            .unwrap()
            .expect("a legacy insert must not lose the race");
    }

    let client = db.pool().get().await.unwrap();
    let rows = client
        .query(
            "SELECT urutan FROM perlengkapan.pemakaian_bmn_items \
              WHERE izin_pemakaian_id = $1::text::uuid ORDER BY urutan",
            &[&lama],
        )
        .await
        .unwrap();
    let urutan: Vec<i32> = rows.iter().map(|r| r.get(0)).collect();
    assert_eq!(
        urutan,
        (1..=9).collect::<Vec<i32>>(),
        "slot 1 plus eight distinct positions"
    );

    teardown_test_db(&db_name).await;
}

fn mobil(nup: &str) -> Value {
    json!({
        "jenis_bmn": "KENDARAAN_BERMOTOR",
        "bmn_nup": nup,
        "bmn_kode_barang": "3020104001",
        "bmn_nama_barang": "Mini Bus",
        "no_polisi": "B 1234 XY",
        "no_bpkb": "BPKB-1"
    })
}

async fn dashboard(server: &TestServer, query: &str) -> Value {
    let mut req = server.get(&format!("/pemakaian-bmn/monitoring/active-usage{query}"));
    for (k, v) in auth_headers("validator_pusat", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "dashboard{query}: {}", res.text());
    let body = res.json::<Value>();
    body.get("data").cloned().unwrap_or(body)
}

fn share_of<'a>(dash: &'a Value, jenis: &str) -> Option<&'a Value> {
    dash["permits_by_jenis_bmn"]
        .as_array()
        .unwrap_or_else(|| panic!("permits_by_jenis_bmn is not an array: {dash}"))
        .iter()
        .find(|r| r["jenis_bmn"] == jenis)
}

/// The by-kind breakdown counts PERMITS, so a share can never pass 100%, and a
/// `jenis_bmn` filter removes the kinds it excludes.
///
/// Two defects in one query. It counted item rows against a denominator of
/// permits — one permit with two laptops reported LAPTOP = 200% — and with a
/// filter the permit-level EXISTS selected the permit while the join still
/// grouped ALL of its items, so a permit holding a laptop and a car, asked
/// about laptops, answered with a car row as well.
#[tokio::test]
async fn asset_type_shares_stay_within_a_hundred_and_obey_the_kind_filter() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Permit 1: two laptops. Permit 2: a laptop and a car.
    drive_to_active(
        &server,
        vec![laptop("591", "Laptop A"), laptop("592", "Laptop B")],
    )
    .await;
    drive_to_active(&server, vec![laptop("593", "Laptop C"), mobil("594")]).await;

    let dash = dashboard(&server, "").await;
    assert_eq!(dash["total_active_permits"], 2, "two permits: {dash}");

    let laptop_row = share_of(&dash, "LAPTOP").expect("laptops are held");
    assert_eq!(
        laptop_row["count"], 2,
        "both PERMITS hold a laptop; counting the three laptop rows is the bug"
    );
    assert_eq!(laptop_row["percentage"].as_f64(), Some(100.0));
    let mobil_row = share_of(&dash, "KENDARAAN_BERMOTOR").expect("one permit holds a car");
    assert_eq!(mobil_row["count"], 1);
    assert_eq!(mobil_row["percentage"].as_f64(), Some(50.0));
    for r in dash["permits_by_jenis_bmn"].as_array().unwrap() {
        assert!(
            r["percentage"].as_f64().unwrap() <= 100.0,
            "a share of permits cannot exceed 100: {r}"
        );
    }

    // The filter: asked about laptops, no car row.
    let dash = dashboard(&server, "?jenis_bmn=LAPTOP").await;
    assert_eq!(dash["total_active_permits"], 2);
    assert!(
        share_of(&dash, "KENDARAAN_BERMOTOR").is_none(),
        "a permit holding a laptop AND a car must not contribute a car row to \
         a laptop-only query: {dash}"
    );
    assert_eq!(share_of(&dash, "LAPTOP").unwrap()["count"], 2);

    // And the opposite direction, so the filter is not simply "drop everything
    // but laptops": asked about cars, only the car permit remains.
    let dash = dashboard(&server, "?jenis_bmn=KENDARAAN_BERMOTOR").await;
    assert_eq!(dash["total_active_permits"], 1);
    assert!(share_of(&dash, "LAPTOP").is_none(), "{dash}");
    assert_eq!(
        share_of(&dash, "KENDARAAN_BERMOTOR").unwrap()["percentage"].as_f64(),
        Some(100.0)
    );

    teardown_test_db(&db_name).await;
}
