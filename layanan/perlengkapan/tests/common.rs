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
                wilayah TEXT,
                tipe_satker TEXT,
                api_id TEXT,
                parent_id TEXT
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
    // `kdsatker_keu` is TWENTY characters (006010199005016000KP), not the
    // six-digit code this fixture used to carry: `AsetScope` reads the wilayah
    // out of it as `substring(kdsatker_keu FROM 6 FOR 4)`, which on a six-digit
    // value returns a one-character fragment. A short code here would not fail
    // — it would quietly put every satker in a different "wilayah" and let a
    // broken wilayah tier pass.
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
                ('Peralatan Mesin Non TIK', 'BMN Uji 003/15', 'Meja Kerja', 'Baik', '3060201003', '15', '12500000', '2021-03-11', 'KEJAKSAAN NEGERI UJI A', '006010199005016000KP', 'Uji Merk A', 'Tipe-1', 'Jl. Uji No. 1'),
                ('Peralatan Mesin Non TIK', '  ',             'Meja Kerja', 'Baik', '3060201003', '99', '9750000',  '2021-03-11', 'KEJAKSAAN NEGERI UJI A', '006010199005016000KP', 'Uji Merk B', NULL,      NULL),
                ('Peralatan Mesin Non TIK', 'BMN Uji 004/16', 'Kursi Kerja','Baik', '3060201004', '16', '4300000',  '2022-07-04', 'KEJAKSAAN NEGERI UJI A', '006010199005016000KP', 'Uji Merk C', NULL,      NULL)
             ON CONFLICT DO NOTHING",
            &[],
        )
        .await
        .unwrap();

    // ── Cross-schema SoT stub: MySIMKARI kode_satker ↔ SIMAN kdsatker_keu ──
    //
    // `AsetScope` resolves both the satker and the wilayah tier through
    // `integrasi.v_satker_code_map` (integrasi migration 003/004). Without it,
    // every scoped SIMAN read errors — and because the "BMN Tidak Dipakai" card
    // handles SIMAN failure best-effort, the scoped path would silently degrade
    // to `null` and the test would see a plausible answer instead of a broken
    // one. That is the shape of failure this whole fixture exists to prevent.
    //
    // The base table mirrors the owner's DDL. The view is the owner's
    // expression with the auto-derive UNION arm omitted — that arm needs the
    // name-normalisation function chain, and reproducing it here would be a
    // second implementation to drift from. Omitting it only makes the map
    // SMALLER, so a scoping leak still surfaces; a larger stub could hide one.
    // `wilayah_kode` in particular is COPIED, not invented: it is
    // `substring(kdsatker_keu FROM 6 FOR 4)`, and hand-writing a different
    // formula here would certify a broken wilayah tier.
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS integrasi.satker_code_map (
                kode_satker  TEXT PRIMARY KEY,
                kdsatker_keu TEXT,
                nama_satker  TEXT,
                match_method TEXT NOT NULL DEFAULT 'manual',
                verified     BOOLEAN NOT NULL DEFAULT FALSE,
                notes        TEXT,
                created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        )
        .await
        .unwrap();
    client
        .execute(
            "CREATE OR REPLACE VIEW integrasi.v_satker_code_map AS
             SELECT kode_satker,
                    kdsatker_keu,
                    substring(kdsatker_keu FROM 6 FOR 4) AS wilayah_kode,
                    (right(kdsatker_keu, 2) = 'KP')      AS is_pusat,
                    nama_satker,
                    match_method,
                    TRUE AS verified
             FROM integrasi.satker_code_map
             WHERE kdsatker_keu IS NOT NULL",
            &[],
        )
        .await
        .unwrap();

    // Two satkers under ONE Kejati plus a third under another. That combination
    // is what makes the tiers distinguishable: with a single satker, a wilayah
    // filter and a satker filter return the same rows and a broken tier passes.
    //
    // `kdsatker_keu` digits 6-9 carry the wilayah, so SKR001/SKR002 share 0199
    // while SKR003 sits in 1100. The codes are real staging values.
    //
    // The two KEJATI rows and the `parent_id -> api_id` links are REQUIRED:
    // `v_satker_wilayah` derives the tier by climbing that hierarchy, so a
    // fixture without them resolves every satker to NO wilayah and the wilayah
    // tier quietly tests the empty set.
    //
    // `wilayah` is 'II' for ALL of them on purpose. That is the shape of the
    // real column (a JAM grouping spanning many Kejati, never a Kejati name),
    // and it makes this fixture a canary: if a scope ever goes back to reading
    // it, SKR003 becomes visible to SKR001's wilayah validator and the
    // cross-wilayah assertions fail.
    client
        .execute(
            "INSERT INTO integrasi.mysimkari_satker
                 (kode_satker, nama_satker, wilayah, tipe_satker, api_id, parent_id)
             VALUES ('KJT01', 'KEJAKSAAN TINGGI UJI SATU', 'II', 'Kejaksaan Tinggi', 'api-kjt01', NULL),
                    ('KJT02', 'KEJAKSAAN TINGGI UJI DUA',  'II', 'Kejaksaan Tinggi', 'api-kjt02', NULL),
                    ('SKR001', 'KEJAKSAAN NEGERI UJI A', 'II', 'Kejaksaan Negeri', 'api-skr001', 'api-kjt01'),
                    ('SKR002', 'KEJAKSAAN NEGERI UJI B', 'II', 'Kejaksaan Negeri', 'api-skr002', 'api-kjt01'),
                    ('SKR003', 'KEJAKSAAN NEGERI UJI C', 'II', 'Kejaksaan Negeri', 'api-skr003', 'api-kjt02')
             ON CONFLICT (kode_satker) DO NOTHING",
            &[],
        )
        .await
        .unwrap();

    // ── Cross-schema SoT stub: MySIMKARI employees ────────────────────────
    //
    // The pakaian-dinas roster reads and the profile upsert both key off this
    // table: the roster IS this table, and the upsert takes the employee's
    // satker from it rather than from the request body. Columns mirror the
    // owner's DDL (`layanan/integrasi/migrations/001_init_schema.sql`), not a
    // convenient subset — a stub that diverges certifies bugs (#17).
    //
    // Three employees across two Kejati: SKR001 and SKR002 under KJT01, SKR003
    // under KJT02, so both the satker tier and the wilayah tier have a positive
    // AND a negative case to land on.
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS integrasi.mysimkari_pegawai (
                 id BIGSERIAL PRIMARY KEY,
                 nip TEXT NOT NULL UNIQUE,
                 nama TEXT,
                 satker_id TEXT,
                 nama_satker TEXT,
                 jabatan TEXT,
                 jenis_jabatan_terakhir TEXT,
                 eselon TEXT,
                 golpang TEXT,
                 gol_kd TEXT,
                 jk TEXT,
                 agama TEXT,
                 email_dinas TEXT,
                 no_hp TEXT,
                 nrp TEXT,
                 foto TEXT,
                 bidang TEXT,
                 jabat_tmt TEXT,
                 status_pegawai TEXT NOT NULL DEFAULT 'aktif',
                 synced_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                 created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
             );",
        )
        .await
        .unwrap();
    client
        .execute(
            "INSERT INTO integrasi.mysimkari_pegawai (nip, nama, satker_id, nama_satker, jabatan, jk)
             VALUES ('19800101000000001', 'Pegawai Uji A', 'SKR001', 'KEJAKSAAN NEGERI UJI A', 'Staf', 'L'),
                    ('19800101000000002', 'Pegawai Uji B', 'SKR002', 'KEJAKSAAN NEGERI UJI B', 'Staf', 'P'),
                    ('19800101000000003', 'Pegawai Uji C', 'SKR003', 'KEJAKSAAN NEGERI UJI C', 'Staf', 'L')
             ON CONFLICT (nip) DO NOTHING",
            &[],
        )
        .await
        .unwrap();

    // The wilayah tier's single definition. READ FROM THE OWNER'S MIGRATION
    // rather than restated here: a hand-written cross-schema stub is what
    // certified bugs in #17 (the old stub made 14 tests pass and 3 fail; the
    // faithful one inverted that exactly). If 005 changes, these tests change
    // with it or fail loudly — they cannot drift into agreeing with a schema
    // that no longer exists.
    let wilayah_view_sql = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../integrasi/migrations/005_satker_wilayah.sql"
    ))
    .expect("integrasi migration 005 must be readable — it defines the wilayah tier");
    {
        // On a DEDICATED connection, never one from the pool. The file opens
        // with `SET search_path TO integrasi, public`, which is a session
        // setting: a pooled connection carries it back into the pool, and every
        // later borrower of that connection resolves unqualified names in the
        // wrong schema. Nothing here would fail — the queries would just answer
        // about different tables, intermittently, depending on which connection
        // the pool handed out.
        let cfg: Config = test_url_str.parse().unwrap();
        let (mig_client, mig_conn) = cfg.connect(NoTls).await.unwrap();
        let conn_handle = tokio::spawn(async move {
            let _ = mig_conn.await;
        });
        mig_client
            .batch_execute(&wilayah_view_sql)
            .await
            .expect("wilayah view (integrasi 005) must apply to the test schema");
        drop(mig_client);
        let _ = conn_handle.await;
    }
    client
        .execute(
            "INSERT INTO integrasi.satker_code_map (kode_satker, kdsatker_keu, nama_satker, verified)
             VALUES ('SKR001', '006010199005016000KP', 'KEJAKSAAN NEGERI UJI A', TRUE),
                    ('SKR002', '006010199666405000KP', 'KEJAKSAAN NEGERI UJI B', TRUE),
                    ('SKR003', '006011100007102000KD', 'KEJAKSAAN NEGERI UJI C', TRUE)
             ON CONFLICT (kode_satker) DO NOTHING",
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
