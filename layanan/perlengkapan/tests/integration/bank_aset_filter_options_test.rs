//! `/bank-aset/filter-options` on the data shape staging actually has.
//!
//! The satker dropdown groups assets by the MySIMKARI `kode_satker` they map
//! to. 65 satkers on staging are known to SIMAN and not to MySIMKARI
//! (`match_method = 'siman_only'`), and for those the map's `kode_satker` is
//! NULL. The query inner-joined the map and grouped on that column, so those
//! satkers became ONE group whose `value` was NULL — and the row was read as a
//! `String`. The read panics, `panic = "abort"` turns the panic into a dead
//! process, and every request in flight died with it. rc31 shipped that: any
//! user opening Bank Aset killed the backend, and the suite that was green in
//! CI never noticed because its fixture had no such satker.
//!
//! The fixture here has one, deliberately, next to two that map. What the
//! endpoint must do with it is stated in the code's own comment: a satker with
//! no MySIMKARI identity "is therefore not selectable — it remains visible in
//! the unfiltered totals". So the assertions are about what it DOES do, not
//! just that it survives: the code dropdown leaves it out, while the name
//! dropdown and the wilayah dropdown still show it.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;

const PUSAT: &str = "00000000-0000-0000-0000-000000000003";

/// Maps (SKR001, SKR002 in `common.rs`) to these two codes.
const KD_SKR001: &str = "006010199005016000KP";
const KD_SKR002: &str = "006010199666405000KP";
/// In no map row at all: a SIMAN satker MySIMKARI has never heard of. Digits
/// 6-9 are `1100`, a wilayah none of the mapped satkers is in.
const KD_SIMAN_ONLY: &str = "006011100007103000KD";

fn headers() -> Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)> {
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_static(PUSAT),
        ),
        (
            reqwest::header::HeaderName::from_static("x-user-role"),
            reqwest::header::HeaderValue::from_static("validator_pusat"),
        ),
        (
            reqwest::header::HeaderName::from_static("x-satker-id"),
            reqwest::header::HeaderValue::from_static("SKR001"),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::validator_pusat::{PUSAT}::SKR001"
            ))
            .unwrap(),
        ),
    ]
}

/// Five assets: two at SKR001, one at SKR002, two at the SIMAN-only satker.
async fn seed(db: &layanan_perlengkapan::shared::db::Database) {
    let client = db.pool().get().await.unwrap();
    client
        .execute("TRUNCATE integrasi.siman_aset", &[])
        .await
        .unwrap();
    client
        .execute(
            &format!(
                "INSERT INTO integrasi.siman_aset
                    (jenis_aset, nama, ur_kondisi, kdsatker_keu, nama_satker, kd_brg, rph_aset, tgl_perlh)
                 VALUES
                    ('Tanah', 'A1', 'Baik', '{KD_SKR001}',    'KEJAKSAAN NEGERI UJI A',            '2010101001', '1000', '2020-01-01'),
                    ('Tanah', 'A2', 'Baik', '{KD_SKR001}',    'KEJAKSAAN NEGERI UJI A',            '2010101001', '2000', '2020-06-01'),
                    ('Tanah', 'B1', 'Baik', '{KD_SKR002}',    'KEJAKSAAN NEGERI UJI B',            '2010101001', '4000', '2022-01-01'),
                    ('Tanah', 'S1', 'Baik', '{KD_SIMAN_ONLY}', 'KEJAKSAAN NEGERI HANYA ADA DI SIMAN', '2010101001', '3000', '2021-01-01'),
                    ('Tanah', 'S2', 'Baik', '{KD_SIMAN_ONLY}', 'KEJAKSAAN NEGERI HANYA ADA DI SIMAN', '2010101001', '500',  '2021-06-01')"
            ),
            &[],
        )
        .await
        .unwrap();
}

/// `value -> count` of one option list, as the dropdown would show it.
fn counts(list: &serde_json::Value) -> Vec<(String, i64)> {
    let mut v: Vec<(String, i64)> = list
        .as_array()
        .unwrap_or_else(|| panic!("option list is not an array: {list}"))
        .iter()
        .map(|o| {
            (
                o["value"].as_str().expect("option value").to_string(),
                o["count"].as_i64().expect("option count"),
            )
        })
        .collect();
    v.sort();
    v
}

#[tokio::test]
async fn a_satker_with_no_mysimkari_identity_is_left_out_of_the_code_dropdown_instead_of_killing_the_process()
 {
    let (app, db, db_name) = setup_test_app().await;
    seed(&db).await;
    let server = TestServer::new(app);

    let mut req = server.get("/bank-aset/filter-options");
    for (k, v) in headers() {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(
        res.status_code(),
        200,
        "filter-options must answer for a SIMAN-only satker: {}",
        res.text()
    );
    let d = res.json::<serde_json::Value>()["data"].clone();

    // By CODE: only the two satkers that have a MySIMKARI identity. The
    // SIMAN-only one (2 assets) is not selectable, so the counts add to 3, and
    // no option carries an empty value a dropdown could not select.
    assert_eq!(
        counts(&d["satker_kode"]),
        vec![("SKR001".to_string(), 2), ("SKR002".to_string(), 1)],
        "the code dropdown offers exactly the satkers MySIMKARI knows"
    );
    for o in d["satker_kode"].as_array().unwrap() {
        assert!(
            !o["value"].as_str().unwrap_or("").is_empty(),
            "an option with no value cannot be selected: {o}"
        );
    }

    // By NAME and by WILAYAH the SIMAN-only satker is still there: it "remains
    // visible in the unfiltered totals", and its region must not vanish from
    // the list just because MySIMKARI cannot name its satker.
    assert!(
        counts(&d["satker"]).contains(&("KEJAKSAAN NEGERI HANYA ADA DI SIMAN".to_string(), 2)),
        "the name dropdown still lists the SIMAN-only satker: {}",
        d["satker"]
    );
    assert_eq!(
        counts(&d["wilayah"]),
        vec![("0199".to_string(), 3), ("1100".to_string(), 2)],
        "the SIMAN-only satker's wilayah stays selectable"
    );

    teardown_test_db(&db_name).await;
}
