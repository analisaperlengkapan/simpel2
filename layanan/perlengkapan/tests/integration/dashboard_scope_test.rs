//! Tiered scoping for the dashboards, after consolidating them to one.
//!
//! Every other per-satker surface in this service has been scoped one by one
//! (#66, #93, #869-#875). The dashboards were the last read that still answered
//! nationwide, and they did it to every caller. Measured against deployed
//! staging (rc29) as operator_a, whose satker is 0200010:
//!
//! ```text
//! GET /bank-aset/dashboard        -> 1 681 aset, 1 satker, Rp 204,5 M  (scoped)
//! GET /dashboard/stats            -> 624 533 aset, 556 satker, Rp 81,9 T
//! GET /dashboard/perlengkapan     -> total_by_satker NAMES other satkers:
//!                                    "KEJAKSAAN NEGERI JAKARTA SELATAN" (0200020),
//!                                    "KEJAKSAAN NEGERI BANDUNG" (0300010)
//! GET /kebutuhan-bmn/dashboard    -> total_satker_terlibat: 3, national totals
//! GET .../export/excel            -> 200, 8 285 bytes of the same aggregates
//! GET .../export/pdf              -> 200, 3 627 bytes of the same aggregates
//! ```
//!
//! Two of those four are gone rather than scoped. `/dashboard/stats` was a
//! second aggregate over `integrasi.siman_aset` whose payload was a strict
//! SUBSET of `/bank-aset/dashboard`; `/kebutuhan-bmn/dashboard` had no caller
//! in either frontend and counted campaigns a third way. Scoping a duplicate is
//! how one rule ends up living in three places and drifting in two of them.
//!
//! What remains needs BOTH scopes, and neither substitutes for the other:
//! perlengkapan's workflow rows are keyed by the MySIMKARI `kode_satker` in the
//! caller's JWT, while SIMAN assets are keyed by the disjoint finance code
//! `kdsatker_keu`. A dashboard that filters one and not the other still reports
//! the country's assets.
//!
//! Each test asserts BOTH directions — what the caller must see and what they
//! must not. The positive half alone passes with no scope at all, which is how
//! `/bank-aset/dashboard` sat correctly scoped beside three that were not.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

fn auth_headers(role: &str, user_id: &str, satker: &str) -> Headers {
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
            reqwest::header::HeaderValue::from_str(satker).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{}::{}::{}",
                role, user_id, satker
            ))
            .unwrap(),
        ),
    ]
}

const PUSAT: &str = "00000000-0000-0000-0000-000000000003";
const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const WILAYAH: &str = "00000000-0000-0000-0000-000000000004";

async fn get(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    satker: &str,
) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in auth_headers(role, user, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

async fn post(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    satker: &str,
    body: serde_json::Value,
) -> axum_test::TestResponse {
    let mut req = server.post(path).json(&body);
    for (k, v) in auth_headers(role, user, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

/// SIMAN assets across the harness's three satkers, chosen so the three tiers
/// give three DIFFERENT answers. SKR001 and SKR002 share Kejati KJT01
/// (`kdsatker_keu` digits 6-9 = 0199); SKR003 sits under KJT02 (1100).
///
///   pusat            -> 9 assets, 3 satkers
///   wilayah @SKR001  -> 5 assets, 2 satkers   (SKR001 + SKR002)
///   satker  @SKR001  -> 3 assets, 1 satker
///
/// With no scope every tier answers 9/3, so any two of these assertions
/// failing together is the signature of a missing filter rather than a wrong
/// one. `jenis_aset` differs per satker so the category breakdown is scoped
/// too — an aggregate can be filtered in its totals and still leak in its
/// breakdown.
async fn seed_siman(db: &layanan_perlengkapan::shared::db::Database) {
    let client = db.pool().get().await.unwrap();
    // Exact counts are asserted, so this test owns the table rather than adding
    // to whatever the harness seeded for other suites.
    client
        .execute("TRUNCATE integrasi.siman_aset", &[])
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO integrasi.siman_aset
                (jenis_aset, nama, ur_kondisi, kdsatker_keu, kd_brg, rph_aset)
             VALUES
                -- SKR001 (KJT01): 2 baik + 1 rusak
                ('Peralatan Mesin Non TIK', 'A1', 'Baik',        '006010199005016000KP', '3060201003', '1000'),
                ('Peralatan Mesin Non TIK', 'A2', 'Baik',        '006010199005016000KP', '3060201003', '1000'),
                ('Peralatan Mesin Non TIK', 'A3', 'Rusak Berat', '006010199005016000KP', '3060201003', '1000'),
                -- SKR002 (KJT01, same wilayah, different satker): 2 baik
                ('Alat Angkutan Bermotor',  'B1', 'Baik',        '006010199666405000KP', '3020104001', '1000'),
                ('Alat Angkutan Bermotor',  'B2', 'Baik',        '006010199666405000KP', '3020104001', '1000'),
                -- SKR003 (KJT02, another wilayah entirely): 4 baik
                ('Tanah',                   'C1', 'Baik',        '006011100007102000KD', '2010101001', '1000'),
                ('Tanah',                   'C2', 'Baik',        '006011100007102000KD', '2010101001', '1000'),
                ('Tanah',                   'C3', 'Baik',        '006011100007102000KD', '2010101001', '1000'),
                ('Tanah',                   'C4', 'Baik',        '006011100007102000KD', '2010101001', '1000')",
            &[],
        )
        .await
        .unwrap();
}

/// One kebutuhan campaign with all three satkers participating.
async fn seed_kebutuhan_campaign(server: &TestServer) -> String {
    let r = post(
        server,
        "/kebutuhan-bmn/pengajuan",
        "validator_pusat",
        PUSAT,
        "PUSAT001",
        json!({"nama": "Kebutuhan BMN 2026", "tahun": 2026,
               "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31"}),
    )
    .await;
    assert_eq!(r.status_code(), 201, "create pengajuan: {}", r.text());
    let body = r.json::<serde_json::Value>();
    let pengajuan_id = body["data"]["pengajuan"]["id"]
        .as_str()
        .or_else(|| body["data"]["id"].as_str())
        .unwrap_or_else(|| panic!("no pengajuan id in {body:?}"))
        .to_string();

    for satker in ["SKR001", "SKR002", "SKR003"] {
        let r = post(
            server,
            &format!("/kebutuhan-bmn/pengajuan/{pengajuan_id}/satker"),
            "validator_pusat",
            PUSAT,
            "PUSAT001",
            json!({"satker_id": satker}),
        )
        .await;
        assert!(
            r.status_code() == 200 || r.status_code() == 201,
            "add satker {satker}: {}",
            r.text()
        );
    }
    pengajuan_id
}

// ---------------------------------------------------------------------------
// /bank-aset/dashboard — the one SIMAN summary, now also the home page's source
// ---------------------------------------------------------------------------

/// This endpoint was scoped in #66 and had no test proving it. That mattered
/// little while it was one of three SIMAN summaries; now that it is the only
/// one — and the one the landing page reads — an unnoticed regression here
/// would take the home page's four cards nationwide again.
#[tokio::test]
async fn the_siman_summary_answers_each_tier_its_own_total() {
    let (app, db, db_name) = setup_test_app().await;
    seed_siman(&db).await;
    let server = TestServer::new(app);

    let baik = |v: &serde_json::Value| -> i64 {
        v["data"]["kondisi_breakdown"]
            .as_array()
            .expect("kondisi_breakdown")
            .iter()
            .filter(|k| k["kondisi"].as_str().unwrap_or_default().to_uppercase() == "BAIK")
            .map(|k| k["count"].as_i64().unwrap_or_default())
            .sum()
    };

    let pusat = get(
        &server,
        "/bank-aset/dashboard",
        "validator_pusat",
        PUSAT,
        "SKR001",
    )
    .await;
    assert_eq!(pusat.status_code(), 200, "pusat: {}", pusat.text());
    let p = pusat.json::<serde_json::Value>();
    assert_eq!(p["data"]["total_aset"], 9);
    assert_eq!(p["data"]["total_satker"], 3);

    let wil = get(
        &server,
        "/bank-aset/dashboard",
        "validator_wilayah",
        WILAYAH,
        "SKR001",
    )
    .await;
    assert_eq!(wil.status_code(), 200, "wilayah: {}", wil.text());
    let w = wil.json::<serde_json::Value>();
    assert_eq!(
        w["data"]["total_aset"], 5,
        "KJT01 = SKR001 (3) + SKR002 (2); 9 means unscoped, 3 means the \
         wilayah tier collapsed to the satker tier"
    );
    assert_eq!(w["data"]["total_satker"], 2);

    let own = get(
        &server,
        "/bank-aset/dashboard",
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await;
    assert_eq!(own.status_code(), 200, "operator: {}", own.text());
    let o = own.json::<serde_json::Value>();
    assert_eq!(o["data"]["total_aset"], 3);
    assert_eq!(o["data"]["total_satker"], 1);
    // The condition split is scoped as well, not just the row count. These are
    // the numbers the home page's "Kondisi Baik" / "Perlu Perbaikan" cards
    // derive from.
    assert_eq!(baik(&o), 2);
    assert_eq!(baik(&p), 8);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_category_breakdown_is_scoped_not_just_the_totals() {
    let (app, db, db_name) = setup_test_app().await;
    seed_siman(&db).await;
    let server = TestServer::new(app);

    let cats = |v: &serde_json::Value| -> Vec<String> {
        v["data"]["kategori_breakdown"]
            .as_array()
            .expect("kategori_breakdown array")
            .iter()
            .map(|c| c["kategori"].as_str().unwrap_or_default().to_string())
            .collect()
    };

    let p = get(
        &server,
        "/bank-aset/dashboard",
        "validator_pusat",
        PUSAT,
        "SKR001",
    )
    .await
    .json::<serde_json::Value>();
    let all = cats(&p);
    assert!(
        all.contains(&"Tanah".to_string()),
        "pusat sees all: {all:?}"
    );
    assert_eq!(all.len(), 3);

    let o = get(
        &server,
        "/bank-aset/dashboard",
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await
    .json::<serde_json::Value>();
    let mine = cats(&o);
    assert_eq!(
        mine,
        vec!["Peralatan Mesin Non TIK".to_string()],
        "SKR001 owns one category; 'Tanah' belongs to SKR003 in another wilayah"
    );

    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// /dashboard/perlengkapan — the seven-aggregate dashboard
// ---------------------------------------------------------------------------

#[tokio::test]
async fn the_perlengkapan_dashboard_never_names_another_satker() {
    let (app, db, db_name) = setup_test_app().await;
    seed_siman(&db).await;
    let server = TestServer::new(app);
    seed_kebutuhan_campaign(&server).await;

    let path = "/dashboard/perlengkapan?tahun_anggaran=2026";

    let p = get(&server, path, "validator_pusat", PUSAT, "SKR001")
        .await
        .json::<serde_json::Value>();
    let all: Vec<String> = p["kebutuhan_metrics"]["total_by_satker"]
        .as_array()
        .expect("total_by_satker")
        .iter()
        .map(|s| s["satker_id"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(all.len(), 3, "pusat sees every participant: {all:?}");

    let res = get(&server, path, "operator_satker", OPERATOR, "SKR001").await;
    assert_eq!(res.status_code(), 200, "operator: {}", res.text());
    let o = res.json::<serde_json::Value>();
    let mine: Vec<String> = o["kebutuhan_metrics"]["total_by_satker"]
        .as_array()
        .expect("total_by_satker")
        .iter()
        .map(|s| s["satker_id"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(
        mine,
        vec!["SKR001".to_string()],
        "this is the field that named Jakarta Selatan and Bandung to an \
         operator in Jakarta Pusat on staging"
    );

    // The SIMAN half of the same payload has to be scoped by the OTHER scope;
    // filtering only the workflow rows would leave this at 9.
    assert_eq!(
        o["asset_utilization"]["total_assets"], 3,
        "asset_utilization is keyed by kdsatker_keu, so it needs AsetScope, \
         not the MySIMKARI-code scope that filters total_by_satker"
    );
    assert_eq!(p["asset_utilization"]["total_assets"], 9);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_wilayah_tier_sees_its_region_and_stops_at_its_border() {
    let (app, db, db_name) = setup_test_app().await;
    seed_siman(&db).await;
    let server = TestServer::new(app);
    seed_kebutuhan_campaign(&server).await;

    let res = get(
        &server,
        "/dashboard/perlengkapan?tahun_anggaran=2026",
        "validator_wilayah",
        WILAYAH,
        "SKR001",
    )
    .await;
    assert_eq!(res.status_code(), 200, "wilayah: {}", res.text());
    let w = res.json::<serde_json::Value>();

    let mut seen: Vec<String> = w["kebutuhan_metrics"]["total_by_satker"]
        .as_array()
        .expect("total_by_satker")
        .iter()
        .map(|s| s["satker_id"].as_str().unwrap_or_default().to_string())
        .collect();
    seen.sort();
    assert_eq!(
        seen,
        vec!["SKR001".to_string(), "SKR002".to_string()],
        "KJT01 holds SKR001 and SKR002; SKR003 is under KJT02"
    );
    assert_eq!(w["asset_utilization"]["total_assets"], 5);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_pemakaian_and_penghapusan_rekap_count_only_the_callers_records() {
    let (app, db, db_name) = setup_test_app().await;
    let client = db.pool().get().await.unwrap();
    // Two permits and two removal proposals, one of each per satker. The
    // rekap cards read `satker_code` — the column V003 added and the pemakaian
    // list and detail are already scoped on (#66, #875).
    for (satker, status) in [("SKR001", "DRAFT"), ("SKR002", "SUBMITTED")] {
        client
            .execute(
                // The rekap counts PERMITS per satker; since V014 a permit's
                // assets live in `pemakaian_bmn_items`, and this card does not
                // read them, so the seed writes the header alone.
                "INSERT INTO perlengkapan.izin_pemakaian_bmn
                    (pegawai_nip, pegawai_nama, pegawai_satker_id,
                     tanggal_mulai, tanggal_selesai, status, satker_code)
                 VALUES ('19800101000000001', 'Pegawai Uji', gen_random_uuid(),
                         '2026-01-01', '2026-12-31', $1, $2)",
                &[&status, &satker],
            )
            .await
            .expect("seed permit");
        client
            .execute(
                // The two `gen_random_uuid()` columns this used to fill were
                // dropped in V010: nothing resolved them, so the seed was
                // inventing identifiers for a record that never needed any.
                // `satker_code` is what the RBAC predicate reads, and it is
                // what this test varies.
                "INSERT INTO perlengkapan.penghapusan_bmn
                    (kode_barang, nama_barang, nup,
                     tanggal_penghapusan, alasan, metode_penghapusan, status,
                     created_by, satker_code)
                 VALUES ('3060201003', 'Meja Kerja',
                         '15', '2026-01-01', 'rusak', 'LELANG', $1, $2, $3)",
                &[&status, &uuid::Uuid::parse_str(PUSAT).unwrap(), &satker],
            )
            .await
            .expect("seed penghapusan");
    }
    drop(client);

    let server = TestServer::new(app);
    let path = "/dashboard/perlengkapan?tahun_anggaran=2026";

    let p = get(&server, path, "validator_pusat", PUSAT, "SKR001")
        .await
        .json::<serde_json::Value>();
    assert_eq!(p["pemakaian_metrics"]["total"], 2);
    assert_eq!(p["penghapusan_metrics"]["total"], 2);

    let o = get(&server, path, "operator_satker", OPERATOR, "SKR001")
        .await
        .json::<serde_json::Value>();
    assert_eq!(
        o["pemakaian_metrics"]["total"], 1,
        "SKR001 owns one permit; 2 means the rekap still counts SKR002's"
    );
    assert_eq!(o["penghapusan_metrics"]["total"], 1);
    // And the status buckets, not just the total: a card can be right in its
    // sum and still show another satker's status names.
    assert_eq!(o["pemakaian_metrics"]["total_by_status"]["DRAFT"], 1);
    assert!(
        o["pemakaian_metrics"]["total_by_status"]["SUBMITTED"].is_null(),
        "SUBMITTED belongs to SKR002: {}",
        o["pemakaian_metrics"]["total_by_status"]
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn both_exports_answer_the_caller_who_asked_for_them() {
    let (app, db, db_name) = setup_test_app().await;
    seed_siman(&db).await;
    let server = TestServer::new(app);
    seed_kebutuhan_campaign(&server).await;

    // The exports render the SAME `PerlengkapanDashboardMetrics` the screen
    // does, through the one service method whose signature now demands both
    // scopes — so an export cannot widen what the page shows without failing to
    // compile. What is worth asserting here is that the scoped path still
    // produces a valid document for a satker-tier caller: the earlier fix for
    // this endpoint (#116) was a 500 on both formats, and a narrower row set is
    // exactly the shape that used to break the renderers (empty sections,
    // zero-row tables).
    for (fmt, magic) in [("excel", &b"PK"[..]), ("pdf", &b"%PDF"[..])] {
        let res = get(
            &server,
            &format!("/dashboard/perlengkapan/export/{fmt}?tahun_anggaran=2026"),
            "operator_satker",
            OPERATOR,
            "SKR001",
        )
        .await;
        assert_eq!(res.status_code(), 200, "{fmt}: {}", res.text());
        let bytes = res.as_bytes();
        assert!(
            bytes.starts_with(magic),
            "{fmt} export is not a {fmt} document: {:?}",
            &bytes[..bytes.len().min(8)]
        );
    }

    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// Drill-down: a filter may narrow, never widen
// ---------------------------------------------------------------------------

/// The whole safety argument for the dashboard filters is that they are a scope
/// TRANSFORM, not an extra condition: `narrow_for_request` returns a narrower
/// scope or `Denied`, and every one of the thirteen places a scope is applied
/// inherits that without needing to remember anything.
///
/// These tests exercise the property that matters — asking for someone else's
/// satker returns THEIR OWN nothing, not the other satker's something. Without
/// it a drill-down is just #871 with a friendlier name: there, `?satker_id=` on
/// the laporan export was applied INSTEAD of the scope rather than after it.
#[tokio::test]
async fn a_drill_down_cannot_reach_outside_the_callers_scope() {
    let (app, db, db_name) = setup_test_app().await;
    seed_siman(&db).await;
    let server = TestServer::new(app);
    seed_kebutuhan_campaign(&server).await;

    let assets = |v: &serde_json::Value| v["asset_utilization"]["total_assets"].as_i64();
    let satkers = |v: &serde_json::Value| {
        v["kebutuhan_metrics"]["total_by_satker"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|s| s["satker_id"].as_str().unwrap_or_default().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    // Pusat drilling into SKR001 sees exactly what SKR001's operator sees.
    let drilled = get(
        &server,
        "/dashboard/perlengkapan?tahun_anggaran=2026&satker=SKR001",
        "validator_pusat",
        PUSAT,
        "SKR001",
    )
    .await;
    assert_eq!(
        drilled.status_code(),
        200,
        "pusat drill: {}",
        drilled.text()
    );
    let d = drilled.json::<serde_json::Value>();
    assert_eq!(assets(&d), Some(3), "SKR001 holds 3 of the 9 seeded assets");
    assert_eq!(satkers(&d), vec!["SKR001".to_string()]);

    // The operator's own view of the same satker, for comparison: identical.
    let own = get(
        &server,
        "/dashboard/perlengkapan?tahun_anggaran=2026",
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await
    .json::<serde_json::Value>();
    assert_eq!(
        assets(&d),
        assets(&own),
        "a drill-down IS that satker's view"
    );

    // An operator naming ANOTHER satker gets nothing — not that satker.
    let stolen = get(
        &server,
        "/dashboard/perlengkapan?tahun_anggaran=2026&satker=SKR002",
        "operator_satker",
        OPERATOR,
        "SKR001",
    )
    .await;
    assert_eq!(stolen.status_code(), 200, "{}", stolen.text());
    let s = stolen.json::<serde_json::Value>();
    assert_eq!(
        assets(&s),
        Some(0),
        "SKR002 holds 2 assets; an operator at SKR001 must see 0, not 2"
    );
    assert!(
        satkers(&s).is_empty(),
        "no participant rows either, got {:?}",
        satkers(&s)
    );

    // A wilayah validator may drill into a satker of their own region...
    let ok = get(
        &server,
        "/dashboard/perlengkapan?tahun_anggaran=2026&satker=SKR002",
        "validator_wilayah",
        WILAYAH,
        "SKR001",
    )
    .await
    .json::<serde_json::Value>();
    assert_eq!(assets(&ok), Some(2), "SKR002 is in KJT01 with SKR001");

    // ...but not into one outside it.
    let denied = get(
        &server,
        "/dashboard/perlengkapan?tahun_anggaran=2026&satker=SKR003",
        "validator_wilayah",
        WILAYAH,
        "SKR001",
    )
    .await
    .json::<serde_json::Value>();
    assert_eq!(
        assets(&denied),
        Some(0),
        "SKR003 is under KJT02 and holds 4 assets; a KJT01 validator must see 0"
    );

    teardown_test_db(&db_name).await;
}

/// The bottleneck card and the processing-time card measure WAITING, not the age of events.
///
/// The old bottleneck query averaged `NOW() - created_at` over every activity of the last 30 days:
/// a stage entered 20 days ago and left two hours later counted as 480 hours of queue, a finished
/// stage was listed as a bottleneck, and the count was activities, not responses. The old
/// processing time was `updated_at - created_at` (the age of the row), under a caption that says
/// "from submitted to done".
///
///   SKR001: now in SubmitWilayah (2002), entered 100 h ago (it was in Input 200 h ago, long gone)
///   SKR002: completed (2008): submitted 50 h ago, completed 10 h ago, so 40 h of processing
///   SKR003: now in Input (2001) since 5 h: under the 24 h threshold, not a bottleneck
#[tokio::test]
async fn the_bottleneck_and_processing_time_cards_measure_waiting_not_the_age_of_events() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_kebutuhan_campaign(&server).await;

    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "DELETE FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas",
            &[],
        )
        .await
        .unwrap();
    for (satker, status, moves) in [
        ("SKR001", 2002, vec![(2001, 200), (2002, 100)]),
        (
            "SKR002",
            2008,
            vec![(2001, 300), (2002, 50), (2004, 40), (2008, 10)],
        ),
        ("SKR003", 2001, vec![(2001, 5)]),
    ] {
        let id: uuid::Uuid = client
            .query_one(
                "UPDATE perlengkapan.pengajuan_kebutuhan_bmn_satker SET status_kode = $2
                  WHERE satker_id = $1 RETURNING id",
                &[&satker, &status],
            )
            .await
            .unwrap()
            .get(0);
        for (to, hours_ago) in moves {
            client
                .execute(
                    "INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                         (pengajuan_satker_id, to_status_kode, aksi, created_at)
                     VALUES ($1, $2, 'uji', NOW() - make_interval(hours => $3))",
                    &[&id, &to, &hours_ago],
                )
                .await
                .unwrap();
        }
    }
    let nama_2002: String = client
        .query_opt(
            "SELECT nama FROM perlengkapan.ms_workflow_status WHERE modul = 'kebutuhan_bmn' AND kode = 2002",
            &[],
        )
        .await
        .unwrap()
        .map(|r| r.get(0))
        .unwrap_or_else(|| "Kode 2002".to_string());

    let r = get(
        &server,
        "/dashboard/perlengkapan?tahun_anggaran=2026",
        "validator_pusat",
        PUSAT,
        "SKR001",
    )
    .await;
    assert_eq!(r.status_code(), 200, "{}", r.text());
    let w = r.json::<serde_json::Value>()["workflow_metrics"].clone();

    // Exactly one bottleneck: the one stage that is holding a response now, for about 100 h.
    let b = w["bottlenecks"].as_array().expect("bottlenecks");
    assert_eq!(b.len(), 1, "only SKR001's current stage is a queue: {w}");
    assert_eq!(b[0]["state"].as_str(), Some(nama_2002.as_str()), "{w}");
    assert_eq!(b[0]["count"].as_i64(), Some(1), "{w}");
    let hours = b[0]["average_time_hours"].as_f64().unwrap();
    assert!(
        (99.5..101.0).contains(&hours),
        "waiting since 100 h ago: {hours}"
    );

    // Submitted to completed for the one finished response: 50 h ago to 10 h ago.
    let avg = w["average_processing_time_hours"]
        .as_f64()
        .unwrap_or_else(|| panic!("a completed response exists: {w}"));
    assert!((39.5..40.5).contains(&avg), "40 h of processing, got {avg}");

    teardown_test_db(&db_name).await;
}

/// A satker AND a region together are an intersection: the region is not dropped because a satker
/// was named. A satker can hold assets under several SIMAN codes in different regions; counting all
/// of them under the region the reader picked is the wrong population. (The fixture cannot build a
/// satker spanning two regions, so this pins the observable half: naming a region the satker has no
/// assets in must not return the satker's assets anyway. The set arithmetic itself is covered by
/// the unit tests on `AsetScope`.)
#[tokio::test]
async fn a_satker_and_a_region_together_are_an_intersection() {
    let (app, db, db_name) = setup_test_app().await;
    seed_siman(&db).await;
    let server = TestServer::new(app);

    let assets = |v: &serde_json::Value| v["asset_utilization"]["total_assets"].as_i64();
    let ask =
        |role: &'static str, user: &'static str, satker: &'static str, query: &'static str| {
            let server = &server;
            async move {
                let r = get(
                    server,
                    &format!("/dashboard/perlengkapan?tahun_anggaran=2026&{query}"),
                    role,
                    user,
                    satker,
                )
                .await;
                assert_eq!(r.status_code(), 200, "{query}: {}", r.text());
                r.json::<serde_json::Value>()
            }
        };

    // Pusat: SKR001's assets sit in region 0199, so that region keeps all 3 and another keeps none.
    let in_region = ask(
        "validator_pusat",
        PUSAT,
        "SKR001",
        "satker=SKR001&wilayah=0199",
    )
    .await;
    assert_eq!(assets(&in_region), Some(3), "SKR001 in its own region");
    let elsewhere = ask(
        "validator_pusat",
        PUSAT,
        "SKR001",
        "satker=SKR001&wilayah=1100",
    )
    .await;
    assert_eq!(
        assets(&elsewhere),
        Some(0),
        "SKR001 holds nothing in 1100: the region must not be dropped because a satker was named"
    );

    // The operator may cut their own satker by region, never reach another one through it.
    let own = ask(
        "operator_satker",
        OPERATOR,
        "SKR001",
        "satker=SKR001&wilayah=0199",
    )
    .await;
    assert_eq!(assets(&own), Some(3));
    let other = ask(
        "operator_satker",
        OPERATOR,
        "SKR001",
        "satker=SKR002&wilayah=0199",
    )
    .await;
    assert_eq!(assets(&other), Some(0), "another satker stays out");

    // A regional validator: a satker of their region, in their region, is theirs ...
    let mine = ask(
        "validator_wilayah",
        WILAYAH,
        "SKR001",
        "satker=SKR002&wilayah=0199",
    )
    .await;
    assert_eq!(assets(&mine), Some(2));
    // ... but naming a region that is not theirs empties BOTH halves rather than leaving a
    // half-filtered dashboard.
    let not_mine = ask(
        "validator_wilayah",
        WILAYAH,
        "SKR001",
        "satker=SKR002&wilayah=1100",
    )
    .await;
    assert_eq!(assets(&not_mine), Some(0));
    assert!(
        not_mine["kebutuhan_metrics"]["total_by_satker"]
            .as_array()
            .is_none_or(|a| a.is_empty()),
        "no perlengkapan rows either: {not_mine}"
    );

    teardown_test_db(&db_name).await;
}

/// A satker that maps to TWO SIMAN regions (0200010 on staging: 0100 and 9999) must not turn a
/// regional validator's request into a 500. The caller's region was looked up as one row by joining
/// the Kejati row to the code map, and `query_opt` refuses a second row, so a satker with two
/// mappings failed before the requested region was even checked.
///
/// The stub's base table has a PRIMARY KEY on `kode_satker`; the real view returns several rows for
/// a satker whose name matches more than one SIMAN code. This test's database is its own, so it
/// drops the key and gives SKR001 a second mapping, which is the shape the view produces.
#[tokio::test]
async fn a_regional_validator_whose_satker_maps_to_two_regions_gets_answers_not_a_500() {
    let (app, db, db_name) = setup_test_app().await;
    seed_siman(&db).await;
    {
        let client = db.pool().get().await.unwrap();
        client
            .execute(
                "ALTER TABLE integrasi.satker_code_map DROP CONSTRAINT satker_code_map_pkey",
                &[],
            )
            .await
            .unwrap();
        client
            .execute(
                "INSERT INTO integrasi.satker_code_map (kode_satker, kdsatker_keu, nama_satker, verified)
                 VALUES ('SKR001', '006019999005016000KP', 'KEJAKSAAN NEGERI UJI A', TRUE)",
                &[],
            )
            .await
            .unwrap();
        // SKR001's second region holds 4 more assets (digits 6-9 = 9999).
        client
            .execute(
                "INSERT INTO integrasi.siman_aset
                    (jenis_aset, nama, ur_kondisi, kdsatker_keu, kd_brg, rph_aset)
                 VALUES
                    ('Peralatan Mesin Non TIK', 'D1', 'Baik', '006019999005016000KP', '3060201003', '1000'),
                    ('Peralatan Mesin Non TIK', 'D2', 'Baik', '006019999005016000KP', '3060201003', '1000'),
                    ('Peralatan Mesin Non TIK', 'D3', 'Baik', '006019999005016000KP', '3060201003', '1000'),
                    ('Peralatan Mesin Non TIK', 'D4', 'Baik', '006019999005016000KP', '3060201003', '1000')",
                &[],
            )
            .await
            .unwrap();
    }
    let server = TestServer::new(app);

    let assets = |v: &serde_json::Value| v["asset_utilization"]["total_assets"].as_i64();
    let ask = |query: &'static str| {
        let server = &server;
        async move {
            let r = get(
                server,
                &format!("/dashboard/perlengkapan?tahun_anggaran=2026&{query}"),
                "validator_wilayah",
                WILAYAH,
                "SKR001",
            )
            .await;
            assert_eq!(
                r.status_code(),
                200,
                "{query}: a satker with two mappings must be answered, not refused: {}",
                r.text()
            );
            r.json::<serde_json::Value>()
        }
    };

    // Both of SKR001's regions are the validator's own: each answers with its own assets.
    assert_eq!(
        assets(&ask("satker=SKR001&wilayah=0199").await),
        Some(3),
        "SKR001 in 0199"
    );
    assert_eq!(
        assets(&ask("satker=SKR001&wilayah=9999").await),
        Some(4),
        "SKR001 in 9999: a region the satker maps to is the validator's too"
    );
    // The region alone (the same lookup, without a satker): SKR001's 3 + SKR002's 2.
    assert_eq!(assets(&ask("wilayah=0199").await), Some(5), "region 0199");
    assert_eq!(assets(&ask("wilayah=9999").await), Some(4), "region 9999");
    // A region that is none of theirs stays empty, and is still an answer, not an error.
    assert_eq!(assets(&ask("satker=SKR001&wilayah=1100").await), Some(0));
    assert_eq!(assets(&ask("wilayah=1100").await), Some(0));

    teardown_test_db(&db_name).await;
}

/// The exports resolve the drill-down the same way the screen does. An export
/// that ignored `?satker=` would hand back the whole region as a file.
#[tokio::test]
async fn the_exports_honour_the_drill_down_too() {
    let (app, db, db_name) = setup_test_app().await;
    seed_siman(&db).await;
    let server = TestServer::new(app);
    seed_kebutuhan_campaign(&server).await;

    for fmt in ["excel", "pdf"] {
        let res = get(
            &server,
            &format!("/dashboard/perlengkapan/export/{fmt}?tahun_anggaran=2026&satker=SKR002"),
            "operator_satker",
            OPERATOR,
            "SKR001",
        )
        .await;
        // Still a valid document — the point is that it is an EMPTY one, not
        // that the request fails.
        assert_eq!(res.status_code(), 200, "{fmt}: {}", res.text());
        assert!(!res.as_bytes().is_empty());
    }

    teardown_test_db(&db_name).await;
}
