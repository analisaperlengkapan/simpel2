// Shared integration-test harness. Included via `mod common;` by several test
// binaries; not every binary uses every helper (e.g. SLA tests use only
// setup_test_db), so the standard tests/common allow applies.
#![allow(dead_code)]

use layanan_perlengkapan::shared::{
    cache::CacheManager,
    db::Database,
    grpc::clients::AuthencClient,
    rate_limit::{RateLimitConfig, RateLimiter},
};
use layanan_perlengkapan::{
    dashboard::services::DashboardService,
    export::ExportService,
    kebutuhan_bmn::{KebutuhanBmnService, PgKebutuhanBmnRepository},
    pakaian_dinas::{PakaianDinasRepository, PakaianDinasService},
    pemakaian_bmn::{PemakaianBmnRepository, PemakaianBmnService},
    penghapusan_bmn::PenghapusanBmnService,
    roadmap_sarpras::{RoadmapRepository, RoadmapService},
    state::AppState,
    workflow::engine::WorkflowEngine,
};
use std::sync::Arc;
use tokio_postgres::{Config, NoTls};
use uuid::Uuid;

pub async fn setup_test_db() -> (Database, String) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();
    let base_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://simpel:simpel@localhost:5432/postgres".to_string());

    // Connect to the default database to create a new one
    let config: Config = base_url.parse().expect("Invalid DATABASE_URL");
    let (client, connection) = config
        .connect(NoTls)
        .await
        .expect("Failed to connect to PG");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let db_name = format!("test_db_{}", Uuid::new_v4().simple());
    client
        .execute(&format!("CREATE DATABASE {}", db_name), &[])
        .await
        .expect("Failed to create test database");

    // Create a new URL pointing to the test database
    let mut test_url = config.clone();
    test_url.dbname(&db_name);
    let test_url_str = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.get_user().unwrap_or("simpel"),
        "simpel", // password handling in config is opaque, assume 'simpel' for tests
        config
            .get_hosts()
            .first()
            .map(|h| match h {
                tokio_postgres::config::Host::Tcp(ip) => ip.to_string(),
                tokio_postgres::config::Host::Unix(_) => "localhost".to_string(),
            })
            .unwrap_or_else(|| "localhost".to_string()),
        config.get_ports().first().unwrap_or(&5432),
        db_name
    );

    let db = Database::new(&test_url_str)
        .await
        .expect("Failed to connect to test DB");

    // Cross-schema prerequisites for the squashed baseline (F5-B). perlengkapan
    // V001 has an FK to `authenc.users(id)` and the deploy contract is
    // integrasi → authenc → perlengkapan. These tests isolate perlengkapan, so
    // we stand up the minimal upstream objects (stub) BEFORE applying the
    // baseline — mirroring the deploy order without pulling in the full
    // authenc/integrasi baselines. Without this the baseline aborts (one txn),
    // every perlengkapan table is rolled back, and every test fails on a
    // missing relation. (#33 / F5-C)
    {
        let client = db.pool().get().await.unwrap();
        client
            .batch_execute(
                "CREATE SCHEMA IF NOT EXISTS authenc; \
                 CREATE TABLE IF NOT EXISTS authenc.users (id uuid PRIMARY KEY); \
                 CREATE SCHEMA IF NOT EXISTS integrasi;",
            )
            .await
            .expect("Failed to create cross-schema prerequisites");
    }

    // Execute all SQL migrations from the migrations directory
    if let Ok(entries) = std::fs::read_dir("migrations") {
        let mut paths: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.extension().and_then(|e| e.to_str()) == Some("sql") {
                let sql = std::fs::read_to_string(&path).expect("Failed to read migration sql");
                let client = db.pool().get().await.unwrap();
                if let Err(e) = client.batch_execute(&sql).await {
                    let _ = client.batch_execute("ROLLBACK").await;
                    if let Some(db_err) = e.as_db_error() {
                        tracing::warn!(
                            "Migration error {}: {} - {}",
                            path.display(),
                            db_err.message(),
                            db_err.detail().unwrap_or("")
                        );
                    } else {
                        tracing::warn!("Migration error {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    let client = db.pool().get().await.unwrap();
    // Create integrasi schema for v_siman_summary mocks if needed
    client
        .execute("CREATE SCHEMA IF NOT EXISTS integrasi", &[])
        .await
        .unwrap();
    client
        .execute("CREATE SCHEMA IF NOT EXISTS authenc", &[])
        .await
        .unwrap();

    // Attempt to insert mock users to satisfy any foreign key constraints
    let mock_users = [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
        "00000000-0000-0000-0000-000000000003",
    ];
    for uid_str in mock_users {
        let uid = uuid::Uuid::parse_str(uid_str).unwrap();
        let _ = client
            .execute(
                "INSERT INTO users (id) VALUES ($1) ON CONFLICT DO NOTHING",
                &[&uid],
            )
            .await;
        let _ = client
            .execute(
                "INSERT INTO authenc.users (id) VALUES ($1) ON CONFLICT DO NOTHING",
                &[&uid],
            )
            .await;
    }

    // Second cross-schema stub, and it had drifted the same way the satker one
    // had: `id UUID` where the SoT declares BIGSERIAL, and missing `jenis_aset`
    // (NOT NULL upstream), `kondisi`, `kdsatker_keu`, `kode_barang`, `satker_id`
    // and `nama_barang` entirely — every one of which production code queries.
    // Mirrors `layanan/integrasi/migrations/001_init_schema.sql`. A stub that
    // diverges from the owner's DDL does not merely miss bugs; it certifies them.
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS integrasi.siman_aset (
                id BIGSERIAL PRIMARY KEY,
                jenis_aset TEXT NOT NULL,
                kategori_aset TEXT, no_aset TEXT, ur_sskel TEXT, nama TEXT,
                kd_brg TEXT, merk TEXT, tipe TEXT,
                ur_kondisi TEXT, kondisi TEXT,
                alamat TEXT, nama_satker TEXT, kdsatker_keu TEXT,
                satker_id UUID, nama_barang TEXT, kode_barang TEXT, nup TEXT,
                rph_aset TEXT, tgl_perlh TEXT, raw_data JSONB,
                synced_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW(),
                created_at TIMESTAMPTZ DEFAULT NOW()
            )",
            &[],
        )
        .await
        .unwrap();

    // Cross-schema SoT stub for satker identity (integrasi owns it; perlengkapan
    // reads it). Pakaian-dinas satker queries LEFT JOIN this, and the wilayah
    // resolver filters on `wilayah`.
    //
    // The column names and types MUST mirror `layanan/integrasi/migrations`
    // (`001_init_schema.sql`) — this stub previously declared
    // `id UUID, kode VARCHAR, nama VARCHAR`, none of which the real table has
    // (it is `id BIGSERIAL, kode_satker TEXT, nama_satker TEXT`). That divergence
    // is what made the whole integration suite worthless for #94: the production
    // queries were written to match this FICTION, so they passed here while
    // failing against every real environment. A stub that drifts from the SoT
    // does not merely miss bugs, it actively certifies them.
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS integrasi.mysimkari_satker (
                id BIGSERIAL PRIMARY KEY,
                kode_satker TEXT NOT NULL UNIQUE,
                nama_satker TEXT,
                wilayah TEXT
            )",
            &[],
        )
        .await
        .unwrap();

    // SIMAN assets the penghapusan tests reference by (kode_barang, NUP).
    //
    // These are REQUIRED, not decoration: penghapusan_bmn/services.rs deliberately
    // rejects a create whose asset is absent from SIMAN, because `nilai_perolehan`
    // must come from the authoritative source and never from operator input.
    // Those tests used to pass without any SIMAN row only because the old stub
    // had no `nup` column at all — the lookup errored and degraded to "no value",
    // so the integrity rule silently never ran. With a faithful schema the rule
    // applies, which is the behaviour we actually want asserted.
    //
    // ── SHAPE NOTE (do not tidy this back) ──────────────────────────────────
    // Every value below is written into a column production ACTUALLY fills, and
    // the columns production leaves empty are left NULL here. That asymmetry is
    // the whole point.
    //
    // This fixture used to populate `kategori_aset`, `kode_barang` and `nup`.
    // None of the three is ever written outside our own seed: the SIMAN ingest
    // derives its INSERT column list from the keys of the API payload
    // (`layanan/integrasi/src/db.rs`), and SIMAN sends `no_aset`, `kd_brg`,
    // `nama`, `ur_kondisi` — never those. Census of 624 533 staging rows:
    // `kategori_aset` and `nup` non-empty in 5 rows (our seed), `kode_barang`,
    // `nama_barang` and `kondisi` in 0.
    //
    // A fixture that fills a dead column does not merely miss the bug, it
    // CERTIFIES the broken query — which is exactly how the dashboard read a
    // column nothing writes (#829) and how `WHERE nup = $1` shipped while
    // 404-ing for all 624 533 real assets (#832). Same failure the stub comment
    // above describes, in data rather than in DDL.
    //
    // Formats are copied from staging, not invented: `kd_brg` is ten digits with
    // NO dots (3050201002), NUP carries no leading zeros, `ur_kondisi` reads
    // "Baik" rather than "BAIK", and `jenis_aset` is a real SIMAN taxonomy value.
    // Because `kd_brg` is now populated, the kode_barang-consistency branch in
    // `find_nilai_perolehan` finally executes instead of being skipped for want
    // of a value to compare against.
    client
        .execute(
            // Column list = EXACTLY the columns the SIMAN ingest populates, and
            // the ones it never populates are omitted so they stay NULL. That
            // is not cosmetic: `siman_dead_columns_test` DERIVES the dead-column
            // set by asking this fixture which columns came back empty, so
            // "faithful here" is what makes that guard mean anything. Filling a
            // dead column would silently switch the guard off for that column;
            // dropping a live one would make it reject legitimate queries.
            //
            // Cardinalities the shape is copied from (624 533 staging rows):
            // always set — jenis_aset, no_aset, ur_sskel, kd_brg, nama_satker,
            // kdsatker_keu, rph_aset, tgl_perlh; partly set — merk 483 471,
            // tipe 141 561, alamat 11 757, and `nama` blank-once-trimmed in
            // 138 607 rows (22%). One row each is left empty deliberately,
            // mirroring a sparse column rather than pretending SIMAN fills
            // everything.
            //
            // `ur_sskel` is the nama barang: it is a strict function of
            // `kd_brg` (2 038 codes, 2 038 distinct pairs), so the two rows
            // sharing 3060201003 share its name. `nama`/`merk`/`tipe` are the
            // SIMAN operator's own labels for one item — free text, 64 220
            // distinct values across those same 2 038 codes — so they vary
            // per row here and one is blank.
            "INSERT INTO integrasi.siman_aset
                (jenis_aset, nama, ur_sskel, ur_kondisi, kd_brg, no_aset,
                 rph_aset, tgl_perlh, nama_satker, kdsatker_keu, merk, tipe, alamat)
             VALUES
                ('Peralatan Mesin Non TIK', 'BMN Uji 003/15', 'Meja Kerja', 'Baik', '3060201003', '15', '12500000', '2021-03-11', 'KEJAKSAAN NEGERI UJI', '005001', 'Uji Merk A', 'Tipe-1', 'Jl. Uji No. 1'),
                ('Peralatan Mesin Non TIK', '  ',             'Meja Kerja', 'Baik', '3060201003', '99', '9750000',  '2021-03-11', 'KEJAKSAAN NEGERI UJI', '005001', 'Uji Merk B', NULL,      NULL),
                ('Peralatan Mesin Non TIK', 'BMN Uji 004/16', 'Kursi Kerja','Baik', '3060201004', '16', '4300000',  '2022-07-04', 'KEJAKSAAN NEGERI UJI', '005001', 'Uji Merk C', NULL,      NULL)
             ON CONFLICT DO NOTHING",
            &[],
        )
        .await
        .unwrap();

    (db, db_name)
}

pub async fn teardown_test_db(db_name: &str) {
    let base_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://simpel:simpel@localhost:5432/postgres".to_string());

    let config: Config = base_url.parse().unwrap();
    let (client, connection) = config.connect(NoTls).await.unwrap();

    tokio::spawn(async move {
        let _ = connection.await;
    });

    client
        .execute(
            &format!("DROP DATABASE IF EXISTS {} WITH (FORCE)", db_name),
            &[],
        )
        .await
        .expect("Failed to drop test database");
}

pub async fn setup_test_app() -> (axum::Router, Database, String) {
    let (db, db_name) = setup_test_db().await;

    let export_service = ExportService::new(Arc::new(db.clone()));
    let analisis_service =
        layanan_perlengkapan::analisis::AnalisisService::new(Arc::new(db.clone()));

    // Ports & adapters: stand up real DokumenService + NotifikasiService so the
    // integration tests exercise the same trait surface production uses. Built
    // BEFORE the workflow services so the DocumentGenerator can be wired into
    // pemakaian/penghapusan (konsep SK generation), mirroring main.rs.
    let template_service =
        std::sync::Arc::new(layanan_perlengkapan::dokumen::TemplateService::new());
    let pdf_generator = std::sync::Arc::new(layanan_perlengkapan::dokumen::PdfGenerator::new(
        layanan_perlengkapan::dokumen::TemplateService::new(),
    ));
    let excel_generator =
        std::sync::Arc::new(layanan_perlengkapan::dokumen::excel_generator::ExcelGenerator::new());
    let docx_generator = std::sync::Arc::new(layanan_perlengkapan::dokumen::DocxGenerator::new(
        layanan_perlengkapan::dokumen::TemplateService::new(),
    ));
    let docs: std::sync::Arc<dyn layanan_perlengkapan::contracts::DocumentGenerator> =
        std::sync::Arc::new(layanan_perlengkapan::dokumen::service::DokumenService::new(
            db.pool().clone(),
            template_service,
            pdf_generator,
            excel_generator,
            docx_generator,
        ));
    let notifier: std::sync::Arc<dyn layanan_perlengkapan::contracts::NotificationSender> =
        std::sync::Arc::new(
            layanan_perlengkapan::notifikasi::service::NotifikasiService::new(db.pool().clone()),
        );

    let pakaian_dinas_repo = PakaianDinasRepository::new(db.pool().clone());
    let pakaian_dinas_service = PakaianDinasService::new(pakaian_dinas_repo);

    let kebutuhan_bmn_repo = PgKebutuhanBmnRepository::new(db.pool().clone());
    let kebutuhan_bmn_workflow = WorkflowEngine::for_kebutuhan_bmn(db.pool().clone());
    let kebutuhan_bmn_service = KebutuhanBmnService::new(
        kebutuhan_bmn_repo,
        AuthencClient::dummy(),
        kebutuhan_bmn_workflow,
    );

    let dashboard_service = DashboardService::new(db.pool().clone());
    let roadmap_repo = RoadmapRepository::new(db.clone());
    let roadmap_service = RoadmapService::new(roadmap_repo);

    let pemakaian_bmn_repo = PemakaianBmnRepository::new(db.pool().clone());
    let pemakaian_bmn_workflow = WorkflowEngine::for_pemakaian_bmn(db.pool().clone());
    let pemakaian_bmn_service =
        PemakaianBmnService::new(pemakaian_bmn_repo, pemakaian_bmn_workflow)
            .with_document_generator(docs.clone());

    let penghapusan_bmn_workflow = WorkflowEngine::for_penghapusan_bmn(db.pool().clone())
        .with_document_generator(docs.clone());
    let penghapusan_bmn_service = Arc::new(
        PenghapusanBmnService::new(db.pool().clone(), Arc::new(penghapusan_bmn_workflow))
            .with_document_generator(docs.clone()),
    );

    let (dashboard_tx, _) = tokio::sync::broadcast::channel(100);

    let state = AppState {
        export_service,
        analisis_service,
        authenc: AuthencClient::dummy(),
        pakaian_dinas_service,
        kebutuhan_bmn_service,
        pemakaian_bmn_service,
        penghapusan_bmn_service,
        roadmap_service,
        dashboard_service,
        dashboard_updates: dashboard_tx,
        db_pool: db.pool().clone(),
        cache_manager: Arc::new(CacheManager::new()),
        rate_limiter: Arc::new(RateLimiter::new(RateLimitConfig::from_env())),
        docs,
        notifier,
        audit_sink: std::sync::Arc::new(layanan_perlengkapan::shared::audit::PgAuditSink::new(
            db.pool().clone(),
        )),
        document_storage: std::sync::Arc::new(
            layanan_perlengkapan::dokumen::FilesystemStorage::from_env(),
        ),
        // #36: integrasi client utk circuit-breaker health — tidak dipakai di test.
        integrasi_client: None,
        boot_time: std::time::Instant::now(),
        // #564: body size limit (prod loads from MAX_REQUEST_SIZE_MB); 10 MiB in tests.
        max_request_size: 10 * 1024 * 1024,
    };

    let app = layanan_perlengkapan::routes::create_routes(state);

    (app, db, db_name)
}
