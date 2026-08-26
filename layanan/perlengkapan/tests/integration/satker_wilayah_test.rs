//! The wilayah tier has ONE definition, and these tests are what keeps it that
//! way.
//!
//! Background: `validator_wilayah` scoping grew two independent notions of
//! "wilayah" and neither was the domain's. The MySIMKARI-side scopes compared
//! `integrasi.mysimkari_satker.wilayah`, which — measured against the real
//! 2026-06-17 staging snapshot, not assumed — holds `I`/`II`/`III`, the
//! JAM-level supervision grouping. `I` alone spans 15 Kejaksaan Tinggi and 238
//! satkers, so a validator at Kejati Kepulauan Riau was reading Sumatera Utara
//! and Kalimantan Selatan rows. Meanwhile `AsetScope` used SIMAN's regional
//! code, which IS Kejati-grained. One monitoring screen therefore counted two
//! different populations side by side.
//!
//! Nothing could go red about any of that, because the e2e fixture invented
//! province names for `wilayah` — a value space the source never returns — so
//! every wilayah assertion passed against a world that does not exist.

use crate::common::{setup_test_db, teardown_test_db};

/// Every satker resolves to the Kejaksaan Tinggi above it, and the tier is
/// derived by climbing the hierarchy rather than by reading `wilayah`.
///
/// SKR001/SKR002 sit under KJT01, SKR003 under KJT02. A Kejati resolves to
/// itself. All five rows share `wilayah = 'II'`, so a scope that regressed to
/// that column would put all three Kejari in one group and fail here.
#[tokio::test]
async fn wilayah_resolves_to_the_kejati_above_a_satker() {
    let (db, db_name) = setup_test_db().await;
    let client = db.pool().get().await.unwrap();

    let rows = client
        .query(
            "SELECT kode_satker, wilayah_code, wilayah_nama
             FROM integrasi.v_satker_wilayah ORDER BY kode_satker",
            &[],
        )
        .await
        .unwrap();

    let got: Vec<(String, String)> = rows
        .iter()
        .map(|r| (r.get::<_, String>(0), r.get::<_, String>(1)))
        .collect();

    assert_eq!(
        got,
        vec![
            ("KJT01".to_string(), "KJT01".to_string()),
            ("KJT02".to_string(), "KJT02".to_string()),
            ("SKR001".to_string(), "KJT01".to_string()),
            ("SKR002".to_string(), "KJT01".to_string()),
            ("SKR003".to_string(), "KJT02".to_string()),
        ],
        "wilayah must be the supervising Kejati, climbed via parent_id -> api_id"
    );

    // The premise that makes this test meaningful: the column the tier used to
    // read cannot distinguish these satkers at all.
    let distinct_wilayah: i64 = client
        .query_one(
            "SELECT count(DISTINCT wilayah) FROM integrasi.mysimkari_satker",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        distinct_wilayah, 1,
        "fixture must keep every satker in ONE mysimkari wilayah, so a scope \
         that reads that column groups satkers it must not group"
    );

    teardown_test_db(&db_name).await;
}

/// The two spellings of the tier agree.
///
/// `AsetScope` filters SIMAN assets by `substring(kdsatker_keu, 6, 4)` while
/// every MySIMKARI-side scope climbs `v_satker_wilayah`. That is safe only
/// while the two are the same partition — on staging they are a bijection
/// across all 488 satkers carrying both. This test is the reason that premise
/// is allowed to stay a premise; without it a divergence would show up only as
/// two different populations counted next to each other on the monitoring
/// summary, with no error anywhere.
#[tokio::test]
async fn wilayah_kode_and_kejati_are_the_same_partition() {
    let (db, db_name) = setup_test_db().await;
    let client = db.pool().get().await.unwrap();

    let split: i64 = client
        .query_one(
            "SELECT count(*) FROM (
                 SELECT w.wilayah_code
                 FROM integrasi.v_satker_wilayah w
                 JOIN integrasi.v_satker_code_map m ON m.kode_satker = w.kode_satker
                 WHERE m.wilayah_kode IS NOT NULL
                 GROUP BY w.wilayah_code
                 HAVING count(DISTINCT m.wilayah_kode) > 1
             ) x",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        split, 0,
        "a Kejati must not span two SIMAN wilayah_kode — AsetScope would then \
         show a narrower region than every other scope on the same screen"
    );

    let shared: i64 = client
        .query_one(
            "SELECT count(*) FROM (
                 SELECT m.wilayah_kode
                 FROM integrasi.v_satker_wilayah w
                 JOIN integrasi.v_satker_code_map m ON m.kode_satker = w.kode_satker
                 WHERE m.wilayah_kode IS NOT NULL
                 GROUP BY m.wilayah_kode
                 HAVING count(DISTINCT w.wilayah_code) > 1
             ) x",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        shared, 0,
        "a SIMAN wilayah_kode must not cover two Kejati — AsetScope would then \
         show assets from outside the caller's wilayah"
    );

    // Non-vacuity: the join above must actually match something, or both counts
    // are trivially zero and this test asserts nothing.
    let paired: i64 = client
        .query_one(
            "SELECT count(*) FROM integrasi.v_satker_wilayah w
             JOIN integrasi.v_satker_code_map m ON m.kode_satker = w.kode_satker
             WHERE m.wilayah_kode IS NOT NULL",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert!(
        paired >= 3,
        "fixture must map at least the three Kejari into SIMAN, got {paired}"
    );

    teardown_test_db(&db_name).await;
}

/// A satker with no Kejati above it gets no wilayah, and the tier fails closed
/// rather than opening up.
///
/// Kejaksaan Agung is the case in production: it has no parent Kejati, so it
/// has no row in the view. If `push_condition` compared against a NULL and the
/// SQL used a form where NULL matched, a pusat unit holding the
/// `validator_wilayah` role would read the whole country.
#[tokio::test]
async fn a_satker_without_a_kejati_has_no_wilayah_and_sees_nothing() {
    let (db, db_name) = setup_test_db().await;
    let client = db.pool().get().await.unwrap();

    client
        .execute(
            "INSERT INTO integrasi.mysimkari_satker
                 (kode_satker, nama_satker, wilayah, tipe_satker, api_id, parent_id)
             VALUES ('PST01', 'KEJAKSAAN AGUNG UJI', 'II', 'Kejaksaan Agung', 'api-pst01', NULL)
             ON CONFLICT (kode_satker) DO NOTHING",
            &[],
        )
        .await
        .unwrap();

    let present: i64 = client
        .query_one(
            "SELECT count(*) FROM integrasi.v_satker_wilayah WHERE kode_satker = 'PST01'",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        present, 0,
        "a satker with no Kejati above it has no wilayah"
    );

    // And the predicate that tier produces selects nothing, rather than
    // everything, when the caller is that satker.
    let visible: i64 = client
        .query_one(
            "SELECT count(*) FROM integrasi.mysimkari_satker t
             WHERE t.kode_satker IN (
                 SELECT s.kode_satker FROM integrasi.v_satker_wilayah s
                 WHERE s.wilayah_code = (
                     SELECT w.wilayah_code FROM integrasi.v_satker_wilayah w
                     WHERE w.kode_satker = 'PST01'
                 )
             )",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        visible, 0,
        "the wilayah predicate must fail closed for a caller with no wilayah"
    );

    teardown_test_db(&db_name).await;
}

/// The Kejati list the campaign dropdown is built from names Kejati, not `I`.
///
/// The endpoint has always been called `list_wilayah_kejati` and its docstring
/// always claimed Kejaksaan Tinggi names; it returned
/// `DISTINCT mysimkari_satker.wilayah` instead. Operators picked from that list
/// and the value they picked became `wilayah_id` on the campaign, so a wrong
/// list wrote wrong targeting into the record.
#[tokio::test]
async fn the_wilayah_dropdown_lists_kejati() {
    let (db, db_name) = setup_test_db().await;
    let client = db.pool().get().await.unwrap();

    let rows = client
        .query(
            "SELECT DISTINCT wilayah_code, wilayah_nama
             FROM integrasi.v_satker_wilayah ORDER BY wilayah_nama",
            &[],
        )
        .await
        .unwrap();

    let listed: Vec<(String, String)> = rows
        .iter()
        .map(|r| (r.get::<_, String>(0), r.get::<_, String>(1)))
        .collect();

    assert_eq!(
        listed,
        vec![
            // Ordered by nama, so "…UJI DUA" precedes "…UJI SATU".
            ("KJT02".to_string(), "KEJAKSAAN TINGGI UJI DUA".to_string()),
            ("KJT01".to_string(), "KEJAKSAAN TINGGI UJI SATU".to_string()),
        ],
        "the dropdown must offer Kejati (code + name), one entry per Kejati"
    );

    for (_, nama) in &listed {
        assert!(
            nama.to_uppercase().contains("TINGGI"),
            "dropdown offered {nama:?}, which is not a Kejaksaan Tinggi"
        );
    }

    teardown_test_db(&db_name).await;
}
