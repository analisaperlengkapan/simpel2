//! Layer-2 integration tests for the pakaian-dinas ukuran endpoints (#117).
//!
//! These exist because the whole "Isi Ukuran" page was dead in every
//! environment and nothing noticed. The frontend `Ukuran` DTO declared
//! `id`/`size`/`created_at`/`updated_at`; `perlengkapan.ms_ukuran` is a
//! three-column table keyed on `(ukuran, "group")` with none of those. So
//! `resp.json()` failed on every response, the resource resolved to `Err`, and
//! the form branch never rendered — zero `<select>` elements on a page whose
//! entire purpose is three dropdowns.
//!
//! It survived because BOTH available signals were misleading:
//!   - `cargo check` is happy — the two structs live in different crates and
//!     never meet at a type boundary; JSON is the only contract between them.
//!   - the e2e assertion that "passed" (`getByText('Isi Ukuran')`) was matching
//!     the PageLayout DESCRIPTION ("Isi ukuran pakaian dinas Anda…"), because
//!     Playwright's `getByText` is substring + case-insensitive by default. The
//!     card it was meant to prove had rendered never rendered at all.
//!
//! The durable guard is therefore a test that asserts the JSON KEY NAMES the
//! browser will actually destructure, not merely that the handler returns 200.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

/// `mock::<role>::<user_id>::<satker>::<nip>` — the trailing NIP is what the
/// personal-ukuran endpoints resolve the employee from (there is no id in the
/// request body), so it is required for anything past the master list.
const NIP: &str = "199203142014031001";

fn auth(nip: bool) -> reqwest::header::HeaderValue {
    let token = if nip {
        format!("Bearer mock::operator_satker::00000000-0000-0000-0000-000000000003::SKR001::{NIP}")
    } else {
        "Bearer mock::operator_satker::00000000-0000-0000-0000-000000000003::SKR001".to_string()
    };
    reqwest::header::HeaderValue::from_str(&token).unwrap()
}

/// The master list feeds all three `<select>` elements. The keys asserted here
/// are exactly the ones `antarmuka/perlengkapan/src/api/pakaian_dinas.rs::Ukuran`
/// destructures — if either side drifts again, this fails in seconds instead of
/// costing a full stack build to discover.
#[tokio::test]
async fn master_ukuran_serializes_the_keys_the_frontend_reads() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = server
        .get("/pakaian-dinas/ukuran?group=BAJU")
        .add_header(reqwest::header::AUTHORIZATION, auth(false))
        .await;
    assert_eq!(res.status_code(), 200, "master ukuran: {}", res.text());

    let body: serde_json::Value = res.json();
    let items = body["data"].as_array().expect("data array");
    assert!(
        !items.is_empty(),
        "V002 seeds ms_ukuran; an empty list means the seed did not apply"
    );

    let first = &items[0];
    // Present — the three columns that actually exist.
    for key in ["ukuran", "group", "urutan"] {
        assert!(!first[key].is_null(), "missing `{key}` in {first}");
    }
    // Absent — the four the frontend used to demand. Asserting their ABSENCE is
    // the point: a DTO that merely tolerates extra keys would still have hidden
    // the original break, since serde's failure was on the missing ones.
    for key in ["id", "size", "created_at", "updated_at"] {
        assert!(
            first[key].is_null(),
            "`{key}` reappeared in the payload — the frontend DTO does not \
             declare it and the table has no such column: {first}"
        );
    }
    assert_eq!(first["group"], "BAJU", "the ?group filter must be honoured");

    teardown_test_db(&db_name).await;
}

/// Full round-trip on the self-service pair, including the two shapes the
/// frontend had wrong: the response is keyed by `nip` (not a surrogate id), and
/// the request carries three REQUIRED strings (not nullable ones).
#[tokio::test]
async fn personal_ukuran_round_trips_through_the_nip_claim() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Nothing stored yet → `data: null`, not a 404. The frontend models this as
    // `Option<PegawaiPakaianDinas>` and would break on either a 404 or `{}`.
    let res = server
        .get("/pakaian-dinas/ukuran-pakaian-pegawai")
        .add_header(reqwest::header::AUTHORIZATION, auth(true))
        .await;
    assert_eq!(res.status_code(), 200, "empty read: {}", res.text());
    assert!(res.json::<serde_json::Value>()["data"].is_null());

    let res = server
        .post("/pakaian-dinas/ukuran-pakaian-pegawai")
        .add_header(reqwest::header::AUTHORIZATION, auth(true))
        .json(&json!({
            "ukuran_baju": "L",
            "ukuran_celana": "32",
            "ukuran_sepatu": "42",
            "with_hijab": true,
        }))
        .await;
    assert_eq!(res.status_code(), 200, "save: {}", res.text());

    let saved: serde_json::Value = res.json();
    let data = &saved["data"];
    assert_eq!(
        data["nip"], NIP,
        "the employee is keyed by the JWT nip claim"
    );
    assert_eq!(data["ukuran_baju"], "L");
    assert_eq!(data["ukuran_celana"], "32");
    assert_eq!(data["ukuran_sepatu"], "42");
    assert_eq!(data["with_hijab"], true);

    // Re-read: the write is persisted, not just echoed back.
    let res = server
        .get("/pakaian-dinas/ukuran-pakaian-pegawai")
        .add_header(reqwest::header::AUTHORIZATION, auth(true))
        .await;
    assert_eq!(res.json::<serde_json::Value>()["data"]["ukuran_baju"], "L");

    teardown_test_db(&db_name).await;
}

/// A save that omits `with_hijab` must LEAVE IT ALONE.
///
/// This was a live data-loss bug: the request field was `#[serde(default)] bool`
/// and the upsert wrote `with_hijab = EXCLUDED.with_hijab`, so an employee
/// saving their sizes silently cleared the flag the operator wizard had set —
/// and `with_hijab` is read back out by the uniform reports (rendered Y/T in
/// `laporan_report.rs`), so the damage surfaced as wrong report output far from
/// the endpoint that caused it.
#[tokio::test]
async fn saving_sizes_does_not_clear_a_hijab_flag_it_was_not_given() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    for body in [
        json!({"ukuran_baju":"M","ukuran_celana":"30","ukuran_sepatu":"40","with_hijab":true}),
        // Same employee, no `with_hijab` key at all.
        json!({"ukuran_baju":"S","ukuran_celana":"29","ukuran_sepatu":"39"}),
    ] {
        let res = server
            .post("/pakaian-dinas/ukuran-pakaian-pegawai")
            .add_header(reqwest::header::AUTHORIZATION, auth(true))
            .json(&body)
            .await;
        assert_eq!(res.status_code(), 200, "save {body}: {}", res.text());
    }

    let res = server
        .get("/pakaian-dinas/ukuran-pakaian-pegawai")
        .add_header(reqwest::header::AUTHORIZATION, auth(true))
        .await;
    let data = res.json::<serde_json::Value>()["data"].clone();
    assert_eq!(data["ukuran_baju"], "S", "the sizes DID update");
    assert_eq!(
        data["with_hijab"], true,
        "an omitted with_hijab must preserve the stored value, not reset it"
    );

    teardown_test_db(&db_name).await;
}

/// The three sizes are required server-side. The frontend now blocks an empty
/// select before sending, but the server must not depend on that.
#[tokio::test]
async fn personal_ukuran_rejects_an_incomplete_body() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = server
        .post("/pakaian-dinas/ukuran-pakaian-pegawai")
        .add_header(reqwest::header::AUTHORIZATION, auth(true))
        .json(&json!({"ukuran_baju": "", "ukuran_celana": "32", "ukuran_sepatu": "42"}))
        .await;
    assert_eq!(
        res.status_code(),
        400,
        "empty ukuran_baju must be rejected: {}",
        res.text()
    );

    // No NIP in the token → the endpoint cannot identify the employee.
    let res = server
        .get("/pakaian-dinas/ukuran-pakaian-pegawai")
        .add_header(reqwest::header::AUTHORIZATION, auth(false))
        .await;
    assert_eq!(res.status_code(), 400, "missing nip claim: {}", res.text());

    teardown_test_db(&db_name).await;
}
