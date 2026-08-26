//! Tiered RBAC data-visibility for the Pemakaian BMN read surface.
//!
//! Stakeholder requirement, verbatim: pemakaian BMN harus bisa dimonitor
//! **dengan batasan tiap role** — pusat semua, wilayah hanya wilayahnya, satker
//! hanya satkernya — menampilkan satker mana, nama barangnya, NUP berapa, nama
//! pegawai yang memakai, dan jangka waktu pemakaiannya.
//!
//! What was there before was a ROLE guard only. `enforce_monitoring_read`
//! decides *whether* a caller may read monitoring at all; nothing decided
//! *which rows*. The satker filter arrived as a query parameter, so omitting it
//! — the default, and what the frontend actually did — returned the national
//! picture to an `operator_satker`. Six read endpoints shared that shape, five
//! of them binding the caller's claims as `_claims`.
//!
//! These tests assert the boundary from the outside, over HTTP, per tier. Two
//! satkers share a wilayah and a third does not, because with a single satker a
//! wilayah tier and a satker tier return the same rows and a broken tier passes.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use uuid::Uuid;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

fn auth_headers(role: &str, user_id: &str, satker_code: &str) -> Headers {
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
            reqwest::header::HeaderValue::from_str(satker_code).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{role}::{user_id}::{satker_code}"
            ))
            .unwrap(),
        ),
    ]
}

const USER: &str = "00000000-0000-0000-0000-000000000001";

/// Satker A and B share wilayah `0199`; C sits in `1100`.
const SATKER_A: &str = "SKR001";
const SATKER_B: &str = "SKR002";
const SATKER_C: &str = "SKR003";

/// Markers unique to each satker's permit, used to prove a response carries
/// only what the caller may see. Asserting on these rather than on row counts
/// is deliberate: a count can match by coincidence, a name cannot.
const PEGAWAI_A: &str = "Andi Pemakai A";
const PEGAWAI_B: &str = "Budi Pemakai B";
const PEGAWAI_C: &str = "Cici Pemakai C";
const NIP_A: &str = "199001010001";
const NIP_B: &str = "199002020002";
const NIP_C: &str = "199003030003";
const NUP_A: &str = "15";
const NUP_B: &str = "16";
const NUP_C: &str = "17";

async fn get(server: &TestServer, path: &str, role: &str, satker: &str) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in auth_headers(role, USER, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

/// One ACTIVE permit per satker, each with its own employee, NUP and item.
///
/// `satker_code` is the authoritative column (migration V003, derived from the
/// caller's claims at create). `pegawai_satker_id` is the legacy
/// client-supplied UUID kept alongside it — deliberately set to values that do
/// NOT correspond to the satker codes, so a query that reaches for the UUID
/// instead of the code produces visibly wrong groupings rather than
/// accidentally-right ones.
async fn seed_permits(db: &layanan_perlengkapan::shared::db::Database) -> (Uuid, Uuid, Uuid) {
    let client = db.pool().get().await.unwrap();
    let ids = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    for (id, satker, pegawai, nip, nup, nama_barang, kode_barang) in [
        (
            ids.0,
            SATKER_A,
            PEGAWAI_A,
            NIP_A,
            NUP_A,
            "Meja Kerja",
            "3060201003",
        ),
        (
            ids.1,
            SATKER_B,
            PEGAWAI_B,
            NIP_B,
            NUP_B,
            "Kursi Kerja",
            "3060201004",
        ),
        (
            ids.2,
            SATKER_C,
            PEGAWAI_C,
            NIP_C,
            NUP_C,
            "Meja Kerja",
            "3060201003",
        ),
    ] {
        client
            .execute(
                "INSERT INTO perlengkapan.izin_pemakaian_bmn
                    (id, nomor_izin, jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang,
                     bmn_merk, bmn_tipe, pegawai_nip, pegawai_nama, pegawai_jabatan,
                     pegawai_satker_id, pegawai_satker_nama, satker_code,
                     tanggal_mulai, tanggal_selesai, status, approved_at)
                 VALUES ($1, $2, 'LAINNYA', $3, $4, $5,
                         'Merk Uji', 'Tipe Uji', $6, $7, 'Staf',
                         gen_random_uuid(), $8, $9,
                         CURRENT_DATE - 10, CURRENT_DATE + 20, 'ACTIVE', NOW())",
                &[
                    &id,
                    &format!("IZIN/{satker}/{nup}"),
                    &nup,
                    &kode_barang,
                    &nama_barang,
                    &nip,
                    &pegawai,
                    &format!("KEJAKSAAN NEGERI UJI {}", &satker[5..]),
                    &satker,
                ],
            )
            .await
            .unwrap();
    }
    ids
}

/// Every marker belonging to a satker, for "must not appear" assertions.
fn markers(satker: &str) -> [&'static str; 3] {
    match satker {
        SATKER_A => [PEGAWAI_A, NIP_A, "IZIN/SKR001"],
        SATKER_B => [PEGAWAI_B, NIP_B, "IZIN/SKR002"],
        _ => [PEGAWAI_C, NIP_C, "IZIN/SKR003"],
    }
}

fn assert_absent(body: &str, satker: &str, ctx: &str) {
    for marker in markers(satker) {
        assert!(
            !body.contains(marker),
            "{ctx}: response leaked {satker} data ({marker}):\n{body}"
        );
    }
}

/// Like [`assert_absent`] but ignores the identifier the caller themselves put
/// in the URL.
///
/// A 404 body echoes the NIP or NUP that was asked for, and the caller already
/// knew it — that is not a disclosure. What must never come back is anything
/// they did NOT supply: the employee's name, the permit number.
fn assert_absent_except_requested(body: &str, satker: &str, requested: &str, ctx: &str) {
    for marker in markers(satker) {
        if marker == requested {
            continue;
        }
        assert!(
            !body.contains(marker),
            "{ctx}: response leaked {satker} data ({marker}):\n{body}"
        );
    }
}

fn assert_present(body: &str, satker: &str, ctx: &str) {
    let marker = markers(satker)[0];
    assert!(
        body.contains(marker),
        "{ctx}: expected {satker} data ({marker}) to be visible:\n{body}"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// The listing the stakeholder asked for
// ───────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn monitoring_listing_is_scoped_per_role() {
    let (app, db, db_name) = setup_test_app().await;
    seed_permits(&db).await;
    let server = TestServer::new(app);
    let path = "/pemakaian-bmn/monitoring/pemakaian";

    // Pusat: everything.
    let res = get(&server, path, "validator_pusat", SATKER_A).await;
    assert_eq!(res.status_code(), 200, "pusat: {}", res.text());
    let body = res.text();
    for satker in [SATKER_A, SATKER_B, SATKER_C] {
        assert_present(&body, satker, "pusat");
    }

    // Wilayah: own wilayah only. A and B share 0199; C does not.
    let res = get(&server, path, "validator_wilayah", SATKER_A).await;
    assert_eq!(res.status_code(), 200, "wilayah: {}", res.text());
    let body = res.text();
    assert_present(&body, SATKER_A, "wilayah");
    assert_present(&body, SATKER_B, "wilayah");
    assert_absent(&body, SATKER_C, "wilayah");

    // Satker: own satker only. This is the case that used to return the lot.
    let res = get(&server, path, "operator_satker", SATKER_A).await;
    assert_eq!(res.status_code(), 200, "satker: {}", res.text());
    let body = res.text();
    assert_present(&body, SATKER_A, "satker");
    assert_absent(&body, SATKER_B, "satker");
    assert_absent(&body, SATKER_C, "satker");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn monitoring_listing_carries_the_columns_the_stakeholder_named() {
    let (app, db, db_name) = setup_test_app().await;
    seed_permits(&db).await;
    let server = TestServer::new(app);

    let res = get(
        &server,
        "/pemakaian-bmn/monitoring/pemakaian",
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    let json: serde_json::Value = res.json();
    let rows = json["data"]["data"].as_array().expect("rows array");
    let row = rows
        .iter()
        .find(|r| r["pegawai_nama"] == PEGAWAI_A)
        .unwrap_or_else(|| panic!("own permit missing: {rows:?}"));

    // satker mana / nama barangnya / nup berapa / nama pegawai / jangka waktu
    assert_eq!(row["satker_code"], SATKER_A);
    assert_eq!(row["satker_nama"], "KEJAKSAAN NEGERI UJI 1");
    assert_eq!(row["nama_barang"], "Meja Kerja");
    assert_eq!(row["nup"], NUP_A);
    assert_eq!(row["kode_barang"], "3060201003");
    assert_eq!(row["pegawai_nama"], PEGAWAI_A);
    assert_eq!(row["pegawai_nip"], NIP_A);
    // Jangka waktu: 10 days elapsed, 20 remaining, 30 total.
    assert_eq!(row["durasi_hari"], 30, "durasi: {row}");
    assert_eq!(row["sisa_hari"], 20, "sisa: {row}");
    // merk/tipe stay in their own field: they are the SIMAN operator's labels,
    // not the standard nama barang, and must never be presented as the latter.
    assert_eq!(row["merk_tipe"], "Merk Uji Tipe Uji");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_client_supplied_satker_code_can_only_narrow_never_widen() {
    let (app, db, db_name) = setup_test_app().await;
    seed_permits(&db).await;
    let server = TestServer::new(app);

    // Satker A asks for satker B explicitly. The old parameter was the ONLY
    // thing deciding visibility, so this is the exact request that leaked.
    let res = get(
        &server,
        &format!("/pemakaian-bmn/monitoring/pemakaian?satker_code={SATKER_B}"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let body = res.text();
    assert_absent(&body, SATKER_B, "narrowing filter");
    // Narrowing still works: asking for someone else's satker yields nothing,
    // not everything and not one's own rows.
    assert_absent(&body, SATKER_A, "narrowing filter");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_caller_without_a_satker_identity_sees_nothing() {
    let (app, db, db_name) = setup_test_app().await;
    seed_permits(&db).await;
    let server = TestServer::new(app);

    // Blank satker claim: fail closed, not fail open.
    let res = get(
        &server,
        "/pemakaian-bmn/monitoring/pemakaian",
        "operator_satker",
        " ",
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let body = res.text();
    for satker in [SATKER_A, SATKER_B, SATKER_C] {
        assert_absent(&body, satker, "no satker identity");
    }

    teardown_test_db(&db_name).await;
}

// ───────────────────────────────────────────────────────────────────────────
// The five endpoints that shared the same hole
// ───────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn monitoring_summary_counts_only_what_the_caller_may_see() {
    let (app, db, db_name) = setup_test_app().await;
    seed_permits(&db).await;
    let server = TestServer::new(app);
    let path = "/pemakaian-bmn/monitoring/summary";

    let pusat: serde_json::Value = get(&server, path, "validator_pusat", SATKER_A).await.json();
    let wilayah: serde_json::Value = get(&server, path, "validator_wilayah", SATKER_A)
        .await
        .json();
    let satker: serde_json::Value = get(&server, path, "operator_satker", SATKER_A).await.json();

    assert_eq!(pusat["data"]["sedang_dipakai"], 3, "pusat: {pusat}");
    assert_eq!(wilayah["data"]["sedang_dipakai"], 2, "wilayah: {wilayah}");
    assert_eq!(satker["data"]["sedang_dipakai"], 1, "satker: {satker}");

    // Expiry window is 30 days; every seeded permit ends in 20.
    assert_eq!(satker["data"]["akan_expired_30d"], 1, "satker: {satker}");

    // The idle card must not be null for a satker caller. It used to be:
    // the code returned `None` whenever any filter was present, and scoping
    // makes a filter always present. `null` here means the SIMAN-side scope
    // failed and was swallowed by the best-effort branch.
    assert!(
        satker["data"]["tidak_dipakai"].is_number(),
        "idle card degraded to null for a scoped caller: {satker}"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn active_usage_dashboard_is_scoped() {
    let (app, db, db_name) = setup_test_app().await;
    seed_permits(&db).await;
    let server = TestServer::new(app);
    let path = "/pemakaian-bmn/monitoring/active-usage";

    let res = get(&server, path, "operator_satker", SATKER_A).await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let body = res.text();
    assert_absent(&body, SATKER_B, "active-usage");
    assert_absent(&body, SATKER_C, "active-usage");

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["data"]["total_active_permits"], 1, "{json}");
    // Grouped on the authoritative code, so exactly one bucket for one satker.
    let by_satker = json["data"]["permits_by_satker"].as_array().unwrap();
    assert_eq!(by_satker.len(), 1, "{json}");
    assert_eq!(by_satker[0]["satker_code"], SATKER_A, "{json}");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn expiring_permits_are_scoped() {
    let (app, db, db_name) = setup_test_app().await;
    seed_permits(&db).await;
    let server = TestServer::new(app);

    let res = get(
        &server,
        "/pemakaian-bmn/expiring?days=30",
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let body = res.text();
    assert_present(&body, SATKER_A, "expiring");
    assert_absent(&body, SATKER_B, "expiring");
    assert_absent(&body, SATKER_C, "expiring");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn bmn_usage_history_is_scoped_and_answers_404_out_of_scope() {
    let (app, db, db_name) = setup_test_app().await;
    seed_permits(&db).await;
    let server = TestServer::new(app);

    // Own asset: visible.
    let res = get(
        &server,
        &format!("/pemakaian-bmn/bmn/{NUP_A}/history"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    assert_present(&res.text(), SATKER_A, "own bmn history");

    // Another satker's asset: 404, not 403. A 403 would confirm the NUP exists
    // somewhere, which is the existence oracle #93 closed elsewhere.
    let res = get(
        &server,
        &format!("/pemakaian-bmn/bmn/{NUP_B}/history"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 404, "{}", res.text());
    assert_absent_except_requested(&res.text(), SATKER_B, NUP_B, "cross-satker bmn history");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn pegawai_usage_history_is_scoped() {
    let (app, db, db_name) = setup_test_app().await;
    seed_permits(&db).await;
    let server = TestServer::new(app);

    let res = get(
        &server,
        &format!("/pemakaian-bmn/pegawai/{NIP_A}/history"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    // Enumerating another satker's employee by NIP alone was the sharpest edge
    // of this hole: it named a person and listed the assets they hold.
    let res = get(
        &server,
        &format!("/pemakaian-bmn/pegawai/{NIP_B}/history"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 404, "{}", res.text());
    assert_absent_except_requested(&res.text(), SATKER_B, NIP_B, "cross-satker pegawai history");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn availability_stays_truthful_but_does_not_name_the_out_of_scope_holder() {
    let (app, db, db_name) = setup_test_app().await;
    seed_permits(&db).await;
    let server = TestServer::new(app);

    // Satker B holds NUP 16. Satker A must still learn it is taken — otherwise
    // the UI would offer to book it and create a double booking — but must not
    // learn who holds it.
    let res = get(
        &server,
        &format!("/pemakaian-bmn/bmn/{NUP_B}/availability"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let json: serde_json::Value = res.json();
    assert_eq!(json["data"]["is_available"], false, "{json}");
    assert!(
        json["data"]["active_permit_holder"].is_null(),
        "named an out-of-scope holder: {json}"
    );
    assert_absent(&res.text(), SATKER_B, "availability");

    // In scope, the holder IS named — the redaction must be scope-driven, not
    // a blanket removal that would make the field useless.
    let res = get(
        &server,
        &format!("/pemakaian-bmn/bmn/{NUP_A}/availability"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    let json: serde_json::Value = res.json();
    assert_eq!(json["data"]["active_permit_holder"], PEGAWAI_A, "{json}");

    teardown_test_db(&db_name).await;
}
