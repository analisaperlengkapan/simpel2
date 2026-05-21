use layanan_perlengkapan::{
    cache_strategy::CacheManager,
    dashboard::services::DashboardService,
    database::Database,
    grpc_clients::AuthencClient,
    kebutuhan_bmn::{KebutuhanBmnService, PgKebutuhanBmnRepository},
    pakaian_dinas::{PakaianDinasRepository, PakaianDinasService},
    pemakaian_bmn::{PemakaianBmnRepository, PemakaianBmnService},
    penghapusan_bmn::PenghapusanBmnService,
    rate_limiting::{RateLimitConfig, RateLimiter},
    roadmap_sarpras::{RoadmapRepository, RoadmapService},
    services::PerlengkapanService,
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
    db.migrate().await.expect("Failed to migrate test DB");

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

    client.execute(
        "CREATE TABLE IF NOT EXISTS integrasi.siman_aset (
            id UUID PRIMARY KEY, kategori_aset VARCHAR, no_aset VARCHAR, ur_sskel VARCHAR, nama VARCHAR, kd_brg VARCHAR, merk VARCHAR, tipe VARCHAR, ur_kondisi VARCHAR, alamat VARCHAR, nama_satker VARCHAR, rph_aset VARCHAR, tgl_perlh VARCHAR, updated_at TIMESTAMPTZ DEFAULT NOW()
        )", &[]).await.unwrap();

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

    let service = PerlengkapanService::new(Arc::new(db.clone()));
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
        PemakaianBmnService::new(pemakaian_bmn_repo, pemakaian_bmn_workflow);

    let penghapusan_bmn_workflow = WorkflowEngine::for_penghapusan_bmn(db.pool().clone());
    let penghapusan_bmn_service = Arc::new(PenghapusanBmnService::new(
        db.pool().clone(),
        Arc::new(penghapusan_bmn_workflow),
    ));

    let (dashboard_tx, _) = tokio::sync::broadcast::channel(100);

    // Ports & adapters: stand up real DokumenService + NotifikasiService so
    // the integration tests exercise the same trait surface production uses.
    let template_service = std::sync::Arc::new(
        layanan_perlengkapan::dokumen::TemplateService::new(),
    );
    let pdf_generator = std::sync::Arc::new(
        layanan_perlengkapan::dokumen::PdfGenerator::new(
            layanan_perlengkapan::dokumen::TemplateService::new(),
        ),
    );
    let excel_generator = std::sync::Arc::new(
        layanan_perlengkapan::dokumen::excel_generator::ExcelGenerator::new(),
    );
    let docs: std::sync::Arc<dyn lib_perlengkapan::contracts::DocumentGenerator> =
        std::sync::Arc::new(layanan_perlengkapan::dokumen::service::DokumenService::new(
            db.pool().clone(),
            template_service,
            pdf_generator,
            excel_generator,
        ));
    let notifier: std::sync::Arc<dyn lib_perlengkapan::contracts::NotificationSender> =
        std::sync::Arc::new(
            layanan_perlengkapan::notifikasi::service::NotifikasiService::new(db.pool().clone()),
        );

    let state = AppState {
        service,
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
    };

    let app = layanan_perlengkapan::routes::create_routes(state);

    (app, db, db_name)
}
