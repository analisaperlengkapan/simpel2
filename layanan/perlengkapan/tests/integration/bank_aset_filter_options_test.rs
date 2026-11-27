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

/// GET `/bank-aset/filter-options{query}` as pusat and return `data`.
async fn opsi(server: &TestServer, query: &str) -> serde_json::Value {
    let mut req = server.get(&format!("/bank-aset/filter-options{query}"));
    for (k, v) in headers() {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "{query}: {}", res.text());
    res.json::<serde_json::Value>()["data"].clone()
}

/// The cascade the dashboard drills through: a region narrows the satker list, but a choice must
/// never collapse the list it was made from.
#[tokio::test]
async fn a_chosen_wilayah_narrows_satker_but_not_the_wilayah_list_and_satker_carry_their_wilayah() {
    let (app, db, db_name) = setup_test_app().await;
    seed(&db).await;
    let server = TestServer::new(app);

    // Every satker option says which wilayah it is in, so the client can narrow without asking.
    let semua = opsi(&server, "").await;
    for o in semua["satker_kode"].as_array().unwrap() {
        assert_eq!(
            o["induk"].as_str(),
            Some("0199"),
            "SKR001 and SKR002 both sit in wilayah 0199: {o}"
        );
    }
    // Only satker options carry it.
    assert!(
        semua["wilayah"][0].get("induk").is_none(),
        "{}",
        semua["wilayah"]
    );

    // Choosing the region leaves the region list whole (so another region can be chosen straight
    // away) and narrows the satker list to it.
    let w1100 = opsi(&server, "?wilayah=1100").await;
    assert_eq!(
        counts(&w1100["wilayah"]),
        vec![("0199".to_string(), 3), ("1100".to_string(), 2)],
        "a chosen wilayah must not collapse the wilayah list"
    );
    assert!(
        w1100["satker_kode"].as_array().unwrap().is_empty(),
        "no MySIMKARI-known satker sits in 1100: {}",
        w1100["satker_kode"]
    );
    let w0199 = opsi(&server, "?wilayah=0199").await;
    assert_eq!(
        counts(&w0199["satker_kode"]),
        vec![("SKR001".to_string(), 2), ("SKR002".to_string(), 1)]
    );

    // Choosing a satker leaves the satker list and the wilayah list whole.
    let s1 = opsi(&server, "?satker_kode=SKR001").await;
    assert_eq!(
        counts(&s1["satker_kode"]),
        vec![("SKR001".to_string(), 2), ("SKR002".to_string(), 1)],
        "a chosen satker must not collapse the satker list"
    );
    assert_eq!(
        counts(&s1["wilayah"]),
        vec![("0199".to_string(), 3), ("1100".to_string(), 2)],
        "a chosen satker must not narrow the wilayah list"
    );

    teardown_test_db(&db_name).await;
}

/// `facet` computes only the lists asked for; the rest are empty, and a name that is not a list
/// is refused instead of producing an empty dropdown that reads as "no data".
#[tokio::test]
async fn facet_limits_the_lists_and_an_unknown_name_is_refused() {
    let (app, db, db_name) = setup_test_app().await;
    seed(&db).await;
    let server = TestServer::new(app);

    let d = opsi(&server, "?facet=wilayah,satker_kode").await;
    assert_eq!(d["wilayah"].as_array().unwrap().len(), 2, "{d}");
    assert_eq!(d["satker_kode"].as_array().unwrap().len(), 2, "{d}");
    for lain in ["jenis", "kategori", "kondisi", "satker"] {
        assert!(
            d[lain].as_array().unwrap().is_empty(),
            "{lain} was not asked for: {}",
            d[lain]
        );
    }
    // Asking for nothing in particular still means everything.
    let semua = opsi(&server, "?facet=").await;
    assert!(!semua["jenis"].as_array().unwrap().is_empty());

    let mut req = server.get("/bank-aset/filter-options?facet=wilayah,bogus");
    for (k, v) in headers() {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 400, "{}", res.text());
    assert!(res.text().contains("bogus"), "{}", res.text());

    teardown_test_db(&db_name).await;
}
