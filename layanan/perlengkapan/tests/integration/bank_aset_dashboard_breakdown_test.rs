//! Every field `/bank-aset/dashboard` returns, from one known table.
//!
//! The five queries that used to build this response each had their own
//! `FROM`, their own `WHERE`, and their own idea of which rows counted. They
//! are now one `GROUPING SETS` pass, which is faster (4 246 ms → 1 469 ms over
//! staging's 624 533 rows) but moves the risk: instead of five queries that
//! could disagree, there is one result set whose rows have to be routed to the
//! right breakdown. A row filed under the wrong grouping set produces a
//! response that is well-formed, plausible, and wrong.
//!
//! So this asserts CONTENT — every total and every breakdown, against counts
//! computed by hand from a seed small enough to check by eye. The two tests
//! that already cover this endpoint assert `total_aset`, `total_satker` and
//! the condition/category splits under scope; what they never look at is
//! `total_nilai_perolehan`, `total_kategori`, `top_satker` or `per_tahun` —
//! exactly the four a misrouted grouping set would corrupt silently.
//!
//! The seed deliberately includes rows that each breakdown must treat
//! differently: an asset whose `tgl_perlh` is not a year, one whose
//! `rph_aset` is not a number, and one with no satker name at all.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;

const PUSAT: &str = "00000000-0000-0000-0000-000000000003";

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

/// Seven assets, three satkers, two categories, three parseable years.
///
/// | asset | satker | kategori | kondisi | rph_aset | tgl_perlh |
/// |---|---|---|---|---|---|
/// | A1 | ALPHA  | Tanah | Baik        | 1000  | 2020-01-01 |
/// | A2 | ALPHA  | Tanah | Baik        | 2000  | 2020-06-01 |
/// | A3 | ALPHA  | Tanah | Rusak Berat | 500   | 2021-01-01 |
/// | B1 | BETA   | Alat Angkutan Bermotor | Baik | 4000 | 2022-01-01 |
/// | B2 | BETA   | Alat Angkutan Bermotor | Baik | `-`  | 2022-01-01 |
/// | C1 | (NULL) | Tanah | `''`        | 1500  | `-`        |
/// | C2 | GAMMA  | Tanah | Baik        | 3000  | `n/a`      |
///
/// `B2.rph_aset` is `'-'` and `C1/C2.tgl_perlh` are absent or unparseable —
/// values SIMAN really sends. Each is skipped by exactly one aggregate and
/// counted by all the others; a rewrite that drops such a row from the wrong
/// breakdown fails a specific assertion below rather than shifting a total by
/// an amount nobody notices.
async fn seed(db: &layanan_perlengkapan::shared::db::Database) {
    let client = db.pool().get().await.unwrap();
    client
        .execute("TRUNCATE integrasi.siman_aset", &[])
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO integrasi.siman_aset
                (jenis_aset, nama, ur_kondisi, kdsatker_keu, nama_satker, kd_brg, rph_aset, tgl_perlh)
             VALUES
                ('Tanah', 'A1', 'Baik',        '006010199005016000KP', 'SATKER ALPHA', '2010101001', '1000', '2020-01-01'),
                ('Tanah', 'A2', 'Baik',        '006010199005016000KP', 'SATKER ALPHA', '2010101001', '2000', '2020-06-01'),
                ('Tanah', 'A3', 'Rusak Berat', '006010199005016000KP', 'SATKER ALPHA', '2010101001', '500',  '2021-01-01'),
                ('Alat Angkutan Bermotor', 'B1', 'Baik', '006010199666405000KP', 'SATKER BETA', '3020104001', '4000', '2022-01-01'),
                ('Alat Angkutan Bermotor', 'B2', 'Baik', '006010199666405000KP', 'SATKER BETA', '3020104001', '-',    '2022-01-01'),
                ('Tanah', 'C1', '',     '006011100007102000KD', NULL,           '2010101001', '1500', NULL),
                ('Tanah', 'C2', 'Baik', '006011100007103000KD', 'SATKER GAMMA', '2010101001', '3000', 'n/a')",
            &[],
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn every_total_and_every_breakdown_agrees_with_the_rows_underneath() {
    let (app, db, db_name) = setup_test_app().await;
    seed(&db).await;
    let server = TestServer::new(app);

    let mut req = server.get("/bank-aset/dashboard");
    for (k, v) in headers() {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let d = res.json::<serde_json::Value>()["data"].clone();

    // ---- totals -----------------------------------------------------------
    assert_eq!(d["total_aset"], 7);
    // 1000+2000+500+4000+0+1500+3000. B2's '-' contributes 0 rather than
    // erroring or dropping the row: the asset still exists, its value is just
    // unknown. This is the field the old code computed three separate times.
    assert_eq!(d["total_nilai_perolehan"], 12000.0);
    // Four distinct kdsatker_keu, counted on the CODE: SATKER GAMMA and the
    // nameless C1 have different codes, so a count by NAME would say 3 (and
    // NULL names would not count at all).
    assert_eq!(
        d["total_satker"], 4,
        "counted on kdsatker_keu; 3 means it regressed to counting names"
    );
    assert_eq!(d["total_kategori"], 2);

    // ---- kondisi ----------------------------------------------------------
    let kondisi = |name: &str| -> i64 {
        d["kondisi_breakdown"]
            .as_array()
            .expect("kondisi_breakdown")
            .iter()
            .find(|k| k["kondisi"] == name)
            .map(|k| k["count"].as_i64().unwrap())
            .unwrap_or(0)
    };
    assert_eq!(kondisi("Baik"), 5);
    assert_eq!(kondisi("Rusak Berat"), 1);
    // C1's condition is `''`, which is absence, not a category of its own.
    assert_eq!(kondisi("TIDAK DIKETAHUI"), 1);
    assert_eq!(d["kondisi_breakdown"].as_array().unwrap().len(), 3);

    // ---- kategori ---------------------------------------------------------
    let kategori = d["kategori_breakdown"].as_array().expect("kategori");
    assert_eq!(kategori.len(), 2);
    // Ordered by count descending: Tanah (5) before Alat Angkutan (2).
    assert_eq!(kategori[0]["kategori"], "Tanah");
    assert_eq!(kategori[0]["count"], 5);
    assert_eq!(kategori[0]["nilai"], 8000.0);
    assert_eq!(kategori[1]["kategori"], "Alat Angkutan Bermotor");
    assert_eq!(kategori[1]["count"], 2);
    assert_eq!(
        kategori[1]["nilai"], 4000.0,
        "B2's unparseable rph_aset adds 0; the asset itself still counts"
    );

    // ---- top satker -------------------------------------------------------
    let satker = d["top_satker"].as_array().expect("top_satker");
    assert_eq!(satker.len(), 4);
    assert_eq!(satker[0]["satker"], "SATKER ALPHA");
    assert_eq!(satker[0]["count"], 3);
    assert_eq!(satker[0]["nilai"], 3500.0);
    assert_eq!(satker[1]["satker"], "SATKER BETA");
    assert_eq!(satker[1]["count"], 2);
    // The nameless satker appears under a label rather than vanishing.
    assert!(
        satker
            .iter()
            .any(|s| s["satker"] == "TIDAK DIKETAHUI" && s["count"] == 1),
        "a NULL nama_satker must still be reported: {satker:?}"
    );

    // ---- per tahun --------------------------------------------------------
    let tahun = d["per_tahun"].as_array().expect("per_tahun");
    // C1 (NULL) and C2 ('n/a') have no parseable year and are excluded HERE
    // while still counting in every total above — the one place the old code
    // expressed with a separate WHERE clause.
    assert_eq!(
        tahun.len(),
        3,
        "only the three parseable years: {tahun:?} (4 means the un-grouped \
         total leaked in as a year; 5 means unparseable dates became a year)"
    );
    assert_eq!(tahun[0]["tahun"], 2022);
    assert_eq!(tahun[0]["count"], 2);
    assert_eq!(tahun[1]["tahun"], 2021);
    assert_eq!(tahun[1]["count"], 1);
    assert_eq!(tahun[2]["tahun"], 2020);
    assert_eq!(tahun[2]["count"], 2);
    assert_eq!(
        tahun
            .iter()
            .map(|t| t["count"].as_i64().unwrap())
            .sum::<i64>(),
        5,
        "7 assets minus the 2 without a parseable year"
    );

    teardown_test_db(&db_name).await;
}
