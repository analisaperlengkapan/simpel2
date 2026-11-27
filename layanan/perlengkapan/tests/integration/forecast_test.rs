//! The forecast endpoints against a real database: the series they read, the years they set
//! aside, the notes they give, and the snapshots they keep.
//!
//! The math itself is checked where it can be checked against known answers: the unit tests of
//! `roadmap_sarpras::forecast`. What only a database can show is here: the aggregate over
//! `roadmap_sarpras` that feeds it, and that comparing forecasts compares them with the PREVIOUS
//! ones.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::Value;

const PUSAT: &str = "00000000-0000-0000-0000-000000000003";

async fn get(server: &TestServer, path: &str) -> axum_test::TestResponse {
    server
        .get(path)
        .add_header(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::validator_pusat::{PUSAT}::SKR001"
            ))
            .unwrap(),
        )
        .await
}

/// One plan row for `tahun` (inside its five-year period), `kebutuhan` units.
async fn rencana(
    db: &layanan_perlengkapan::shared::db::Database,
    periode_mulai: i32,
    tahun: i32,
    kebutuhan: i32,
) {
    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "INSERT INTO perlengkapan.roadmap_sarpras
                 (satker_id, periode_mulai, periode_akhir, kode_barang, nama_barang,
                  tahun_rencana, jumlah_kebutuhan, jumlah_terpenuhi, estimasi_anggaran)
             VALUES ($1, $2, $3, 'K1', 'Barang uji', $4, $5, 0, $6)",
            &[
                &uuid::Uuid::new_v4(),
                &periode_mulai,
                &(periode_mulai + 4),
                &tahun,
                &kebutuhan,
                &(kebutuhan as f64 * 1_000_000.0),
            ],
        )
        .await
        .unwrap();
}

/// 2015, 2016, [no 2017], 2018, 2019 and 2020 to 2024: a gap, then seven years in a row.
async fn seed(db: &layanan_perlengkapan::shared::db::Database) {
    for (mulai, tahun, n) in [
        (2015, 2015, 5),
        (2015, 2016, 6),
        (2015, 2018, 9),
        (2015, 2019, 10),
        (2020, 2020, 10),
        (2020, 2021, 14),
        (2020, 2022, 9),
        (2020, 2023, 15),
        (2020, 2024, 11),
    ] {
        rencana(db, mulai, tahun, n).await;
    }
}

#[tokio::test]
async fn the_forecast_runs_on_the_latest_consecutive_years_and_says_what_it_set_aside() {
    let (app, db, db_name) = setup_test_app().await;
    seed(&db).await;
    let server = TestServer::new(app);

    let r = get(&server, "/forecast?method=exponential&horizon=3").await;
    assert_eq!(r.status_code(), 200, "{}", r.text());
    let f = r.json::<Value>();

    // 2017 has no rows: the run that ends at 2024 starts at 2018.
    let historis: Vec<i64> = f["historical"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["tahun"].as_i64().unwrap())
        .collect();
    assert_eq!(historis, (2018..=2024).collect::<Vec<_>>(), "{f}");
    assert!(
        f["catatan"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.as_str().unwrap().contains("2015") && c.as_str().unwrap().contains("2016")),
        "the years set aside are named: {}",
        f["catatan"]
    );

    let p = f["predictions"].as_array().unwrap();
    assert_eq!(
        p.iter()
            .map(|y| y["tahun"].as_i64().unwrap())
            .collect::<Vec<_>>(),
        vec![2025, 2026, 2027]
    );
    for y in p {
        let (lo, mid, hi) = (
            y["confidence_lower"].as_f64().unwrap(),
            y["predicted_kebutuhan"].as_f64().unwrap(),
            y["confidence_upper"].as_f64().unwrap(),
        );
        assert!(lo <= mid && mid <= hi && lo >= 0.0, "{y}");
    }

    // Two years in a row are needed; a lone year is refused rather than extrapolated from nothing.
    teardown_test_db(&db_name).await;
    let (app, db, db_name) = setup_test_app().await;
    rencana(&db, 2020, 2022, 8).await;
    let server = TestServer::new(app);
    let r = get(&server, "/forecast").await;
    assert_eq!(r.status_code(), 400, "{}", r.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_summary_reports_compound_growth_over_the_same_series() {
    let (app, db, db_name) = setup_test_app().await;
    seed(&db).await;
    let server = TestServer::new(app);

    let r = get(&server, "/forecast/summary").await;
    assert_eq!(r.status_code(), 200, "{}", r.text());
    let s = r.json::<Value>();
    assert_eq!(s["total_historical_years"], 7, "{s}");
    // 9 (2018) to 11 (2024) over six steps.
    let esperado = ((11.0_f64 / 9.0).powf(1.0 / 6.0) - 1.0) * 100.0;
    let got = s["avg_yearly_growth_pct"].as_f64().unwrap();
    assert!((got - esperado).abs() < 1e-6, "{got} vs {esperado}");
    assert_eq!(s["trend_direction"], "increasing", "{s}");

    teardown_test_db(&db_name).await;
}

/// Comparing a forecast with the previous ones: the previous ones are what was saved BEFORE this
/// request, the same forecast asked twice is kept once, and refreshing does not pile up copies.
#[tokio::test]
async fn compare_lists_the_previous_forecasts_not_the_one_it_just_made_and_does_not_pile_up_copies()
{
    let (app, db, db_name) = setup_test_app().await;
    seed(&db).await;
    let server = TestServer::new(app);

    let anterior = |v: &Value| v["previous_snapshots"].as_array().unwrap().len();
    let compare = || async {
        let r = get(&server, "/forecast/compare?limit=10").await;
        assert_eq!(r.status_code(), 200, "{}", r.text());
        r.json::<Value>()
    };

    // Nothing was ever saved: nothing is "previous" (it used to list the forecast just made).
    assert_eq!(anterior(&compare().await), 0);
    // That request saved one; a second sees exactly it.
    assert_eq!(anterior(&compare().await), 1);
    // The same forecast again is not saved again, however often the page is refreshed.
    for _ in 0..3 {
        assert_eq!(anterior(&compare().await), 1);
    }
    // A different method is a different forecast, and is kept.
    let r = get(&server, "/forecast?method=exponential").await;
    assert_eq!(r.status_code(), 200, "{}", r.text());
    assert_eq!(anterior(&compare().await), 2);
    // ... once.
    let r = get(&server, "/forecast?method=exponential").await;
    assert_eq!(r.status_code(), 200, "{}", r.text());
    assert_eq!(anterior(&compare().await), 2);

    teardown_test_db(&db_name).await;
}
