//! The "Sebaran" map plots one dot per satker and groups the dots by Kejati wilayah, so
//! `/bank-aset/sebaran` has to say WHERE each satker is and WHICH wilayah it belongs to.
//!
//! The coordinates live in MySIMKARI (`mysimkari_satker.lat/long`, TEXT) and SIMAN knows the
//! satker by a different code, so the page reaches them through the integrasi-owned code map.
//! The wilayah is the same definition the filter and the RBAC tier use: digits 6-9 of the SIMAN
//! code, named by the Kejati above the satker. Things that fail silently rather than loudly:
//!   * a satker with no usable coordinates is still listed and counted, just without a place;
//!   * a bad coordinate must not abort the page (the columns are text; a bare cast would raise);
//!   * a satker the map pairs twice must still be listed once, or its assets are counted twice;
//!   * the GPS columns come from the ingest, not a migration: an environment without them must
//!     still answer, not 500.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::Value;

const ADMIN: &str = "00000000-0000-0000-0000-000000000009";

// 20-character finance codes; digits 6-9 are the wilayah.
const A: &str = "006018888010001KD";
const B: &str = "006018888020001KD";
const C: &str = "006017777010001KD";
const D: &str = "006016666010001KD";
const E: &str = "006018888030001KD";

async fn sebaran(server: &TestServer) -> Vec<Value> {
    let res = server
        .get("/bank-aset/sebaran")
        .add_header(
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::admin::{ADMIN}::PUSAT001"
            ))
            .unwrap(),
        )
        .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    res.json::<Value>()["data"]["satker"]
        .as_array()
        .unwrap()
        .clone()
}

async fn seed(db: &layanan_perlengkapan::shared::db::Database) {
    let client = db.pool().get().await.unwrap();
    // MAP01 sits under the Kejati KJT01 (harness hierarchy), so its wilayah has a name; MAP02..05
    // have no parent, so they do not name one themselves and take the name of their wilayah.
    client
        .execute(
            "INSERT INTO integrasi.mysimkari_satker (kode_satker, nama_satker, lat, long, provinsi, api_id, parent_id)
             VALUES ('MAP01', 'KEJARI PETA SATU',   '-6.2',  '106.8', 'DKI Jakarta',    'api-map01', 'api-kjt01'),
                    ('MAP04', 'KEJARI PETA EMPAT',  '3.5',   '98.7',  'Sumatera Utara', 'api-map04', NULL),
                    ('MAP02', 'KEJARI DI SAMUDERA', '0',     '0',     'Tak Dikenal',    'api-map02', NULL),
                    ('MAP03', 'KEJARI HURUF',       'abc',   '106.8', NULL,             'api-map03', NULL),
                    ('MAP05', 'KEJARI TUKAR',       '106.8', '-6.2',  'Terbalik',       'api-map05', NULL)
             ON CONFLICT (kode_satker) DO NOTHING",
            &[],
        )
        .await
        .unwrap();
    // A is paired with TWO MySIMKARI satkers, the verified one must win; E's coordinates are
    // swapped (a longitude in the latitude column), which lies outside Indonesia.
    client
        .execute(
            &format!(
                "INSERT INTO integrasi.satker_code_map (kode_satker, kdsatker_keu, nama_satker, verified)
                 VALUES ('MAP04', '{A}', 'KEJARI PETA EMPAT', FALSE),
                        ('MAP01', '{A}', 'KEJARI PETA SATU',  TRUE),
                        ('MAP02', '{B}', 'KEJARI DI SAMUDERA', TRUE),
                        ('MAP03', '{C}', 'KEJARI HURUF',       TRUE),
                        ('MAP05', '{E}', 'KEJARI TUKAR',       TRUE)"
            ),
            &[],
        )
        .await
        .unwrap();
    // D has no MySIMKARI counterpart at all (a SIMAN-only satker). One of A's assets carries GPS.
    client
        .execute(
            &format!(
                "INSERT INTO integrasi.siman_aset
                    (jenis_aset, nama, ur_kondisi, kdsatker_keu, nama_satker, kd_brg, rph_aset, gps_latitude, gps_longitude)
                 VALUES
                    ('Peralatan Mesin Non TIK', 'Laptop',  'Baik',         '{A}', 'KEJARI PETA SATU',   '3050105039', '1000', '-6.2', '106.8'),
                    ('Peralatan Mesin Non TIK', 'Printer', 'Rusak Ringan', '{A}', 'KEJARI PETA SATU',   '3050105040', '500',  '',     NULL),
                    ('Peralatan Mesin Non TIK', 'Meja',    'Baik',         '{B}', 'KEJARI DI SAMUDERA', '3050201002', '100',  NULL,   NULL),
                    ('Peralatan Mesin Non TIK', 'Kursi',   'Baik',         '{C}', 'KEJARI HURUF',       '3050201003', '100',  NULL,   NULL),
                    ('Peralatan Mesin Non TIK', 'Lemari',  'Baik',         '{D}', 'RS ADHYAKSA',        '3050201004', '100',  NULL,   NULL),
                    ('Peralatan Mesin Non TIK', 'Rak',     'Baik',         '{E}', 'KEJARI TUKAR',       '3050201005', '100',  NULL,   NULL)"
            ),
            &[],
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn each_satker_is_listed_once_with_its_place_its_wilayah_and_its_gps_count() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed(&db).await;

    let rows = sebaran(&server).await;
    let by = |kode: &str| -> &Value {
        let hits: Vec<&Value> = rows.iter().filter(|r| r["kode_satker"] == kode).collect();
        assert_eq!(
            hits.len(),
            1,
            "{kode} must be listed exactly once: {rows:?}"
        );
        hits[0]
    };

    // Two MySIMKARI pairings, one row; its assets counted once; the verified pairing's place.
    let a = by(A);
    assert_eq!(a["total_aset"], 2);
    assert_eq!(a["lintang"], -6.2);
    assert_eq!(a["bujur"], 106.8);
    assert_eq!(a["provinsi"], "DKI Jakarta");
    // One asset has GPS, one has an empty string, one NULL would not count either.
    assert_eq!(a["aset_bergps"], 1);
    // The dashboard drill-down filters by the MySIMKARI code, and the verified pairing wins.
    assert_eq!(a["kode_mysimkari"], "MAP01");

    // Listed and counted, never placed: out of range, not a number, swapped, no counterpart.
    for kode in [B, C, D, E] {
        let r = by(kode);
        assert_eq!(r["total_aset"], 1, "{kode} is still counted: {r}");
        assert!(
            r["lintang"].is_null() && r["bujur"].is_null(),
            "{kode} has no place: {r}"
        );
        assert_eq!(r["aset_bergps"], 0, "{kode}");
    }

    // A pairing whose place is unusable is still a pairing: the satker can be drilled into.
    assert_eq!(by(B)["kode_mysimkari"], "MAP02");
    assert_eq!(by(C)["kode_mysimkari"], "MAP03");
    assert!(
        by(D)["kode_mysimkari"].is_null(),
        "SIMAN-only has no MySIMKARI code"
    );

    // Wilayah = digits 6-9, named by the Kejati above ANY satker of that wilayah: B and E have no
    // parent of their own and still land in the named wilayah 8888; C is a wilayah no Kejati
    // supervises and D a SIMAN-only one, so both are grouped but unnamed.
    for kode in [A, B, E] {
        let r = by(kode);
        assert_eq!(r["wilayah_kode"], "8888", "{kode}: {r}");
        assert_eq!(
            r["wilayah_nama"], "KEJAKSAAN TINGGI UJI SATU",
            "{kode}: {r}"
        );
    }
    assert_eq!(by(C)["wilayah_kode"], "7777");
    assert!(by(C)["wilayah_nama"].is_null());
    assert_eq!(by(D)["wilayah_kode"], "6666");
    assert!(by(D)["wilayah_nama"].is_null());

    // The harness seeds a satker of its own; only this test's five are counted here.
    let mine: Vec<&Value> = rows
        .iter()
        .filter(|r| {
            r["kode_satker"]
                .as_str()
                .is_some_and(|k| [A, B, C, D, E].contains(&k))
        })
        .collect();
    assert_eq!(mine.len(), 5, "no satker dropped or duplicated: {rows:?}");
    let total: i64 = mine.iter().map(|r| r["total_aset"].as_i64().unwrap()).sum();
    assert_eq!(total, 6, "the table's total is the asset count, once each");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_environment_without_the_gps_columns_still_answers() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed(&db).await;
    // The ingest creates a column per payload key; one whose payload never carried GPS has none.
    let client = db.pool().get().await.unwrap();
    client
        .batch_execute(
            "ALTER TABLE integrasi.siman_aset
               DROP COLUMN gps_latitude, DROP COLUMN gps_longitude",
        )
        .await
        .unwrap();
    drop(client);

    let rows = sebaran(&server).await;
    let a = rows
        .iter()
        .find(|r| r["kode_satker"] == A)
        .expect("A listed");
    assert_eq!(a["total_aset"], 2);
    assert_eq!(a["aset_bergps"], 0, "no column, no coordinates: {a}");
    assert_eq!(a["lintang"], -6.2, "the satker's own place is unaffected");

    teardown_test_db(&db_name).await;
}
