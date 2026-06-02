//! # Layanan Pembinaan Perlengkapan Backend Service
//!
//! Backend microservice for Perlengkapan (asset management) within SIMPEL.
//! Integrates with Authenc (IAM) and Secreton (Secret Manager) via gRPC.

use axum::http::HeaderValue;
use axum::{Router, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use layanan_perlengkapan::shared::{
    cache::CacheManager,
    db::Database,
    grpc::clients::{AuthencClient, IntegrasiClient, SecretonClient},
    middleware,
    rate_limit::{RateLimitConfig, RateLimiter},
};
use layanan_perlengkapan::{
    dashboard, dokumen, kebutuhan_bmn, notifikasi, pakaian_dinas, pemakaian_bmn, penghapusan_bmn,
    roadmap_sarpras, routes, services, workflow,
};
use lib_perlengkapan::contracts::{
    AuditSink, DocumentGenerator, DocumentStorage, NotificationSender,
};

use dashboard::services::DashboardService;
use kebutuhan_bmn::{KebutuhanBmnService, PgKebutuhanBmnRepository};
use pakaian_dinas::{PakaianDinasRepository, PakaianDinasService};
use pemakaian_bmn::PemakaianBmnService;
use roadmap_sarpras::{RoadmapRepository, RoadmapService};
use services::PerlengkapanService;

use layanan_perlengkapan::AppState;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    layanan_perlengkapan::shared::logging::init_structured_logging();

    info!("Starting Layanan Pembinaan Perlengkapan Service");

    // Load configuration from environment
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8093".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");

    // Secreton Integration
    let secreton_url =
        std::env::var("SECRETON_URL").unwrap_or_else(|_| "http://localhost:50051".to_string());
    let mut database_url = std::env::var("DATABASE_URL").ok();

    // Authenc Integration
    let authenc_url =
        std::env::var("AUTHENC_URL").unwrap_or_else(|_| "http://localhost:50052".to_string());

    // Integrasi Integration
    let integrasi_url =
        std::env::var("INTEGRASI_URL").unwrap_or_else(|_| "http://localhost:50053".to_string());

    // Connect to Secreton once and keep the handle around so we can fetch
    // both the DB URL (boot-time) and runtime credentials (SMTP for the
    // notifikasi module). Failure here is non-fatal in dev — we fall back
    // to environment variables — but production should rely on this path.
    let secreton_client: Option<SecretonClient> =
        match SecretonClient::connect(secreton_url.clone()).await {
            Ok(client) => {
                info!("Connected to Secreton at {}", secreton_url);
                Some(client)
            }
            Err(e) => {
                error!(
                    "Failed to connect to Secreton at {}: {}. Falling back to environment variables.",
                    secreton_url, e
                );
                None
            }
        };

    if database_url.is_none()
        && let Some(client) = secreton_client.as_ref()
    {
        match client.get_secret("perlengkapan/db").await {
            Ok(data) => {
                if let Some(url) = data.get("url") {
                    database_url = Some(url.clone());
                    info!("Fetched DATABASE_URL from Secreton");
                }
            }
            Err(e) => error!("Failed to fetch db secret: {:?}", e),
        }
    }

    let database_url = database_url.expect("DATABASE_URL must be set (env or secreton)");

    // Initialize Authenc Client with Retry Logic (optional for dev)
    // `APP_ENV=production` flips Authenc connectivity from "best-effort" to
    // "required". In dev/staging we still allow the dummy fallback so the
    // service is usable when authenc is intentionally offline. In production
    // a missing IdP is a hard failure — a service that silently accepts every
    // token is worse than one that refuses to start.
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    let is_production = app_env.eq_ignore_ascii_case("production");
    let skip_authenc = std::env::var("SKIP_AUTHENC").unwrap_or_default() == "true";

    if skip_authenc && is_production {
        return Err(anyhow::anyhow!(
            "SKIP_AUTHENC=true is forbidden when APP_ENV=production"
        ));
    }

    let authenc_client = if skip_authenc {
        info!("SKIP_AUTHENC=true, using dummy Authenc client (dev mode)");
        AuthencClient::dummy()
    } else {
        info!("Connecting to Authenc at {}", authenc_url);
        let mut retries = 5;
        let mut client = None;
        let mut delay = tokio::time::Duration::from_secs(1);

        while retries > 0 {
            match AuthencClient::connect(authenc_url.clone()).await {
                Ok(c) => {
                    client = Some(c);
                    break;
                }
                Err(e) => {
                    error!(
                        "Failed to connect to Authenc: {}. Retrying in {:?}...",
                        e, delay
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                    retries -= 1;
                }
            }
        }
        match client {
            Some(c) => c,
            None if is_production => {
                return Err(anyhow::anyhow!(
                    "Could not connect to Authenc at {} after retries; refusing to start in production",
                    authenc_url
                ));
            }
            None => {
                error!("Could not connect to Authenc after retries, starting with dummy client");
                AuthencClient::dummy()
            }
        }
    };

    // Initialize Integrasi Client (optional, service continues if unavailable)
    let integrasi_client = {
        info!("Connecting to Integrasi at {}", integrasi_url);
        let mut retries = 3;
        let mut client = None;
        let mut delay = tokio::time::Duration::from_secs(1);

        while retries > 0 {
            match IntegrasiClient::connect(integrasi_url.clone()).await {
                Ok(c) => {
                    client = Some(c);
                    break;
                }
                Err(e) => {
                    error!(
                        "Failed to connect to Integrasi: {}. Retrying in {:?}...",
                        e, delay
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                    retries -= 1;
                }
            }
        }

        if client.is_none() {
            error!(
                "Could not connect to Integrasi after retries, MySIMKARI rekap will be disabled"
            );
        }
        client
    };

    // Initialize database connection
    info!("Connecting to database...");
    let db = Database::new(&database_url).await?;

    // Run migrations:
    // 1. Refinery against the SQL files under `migrations/` (real schema).
    // 2. Legacy hand-coded `CREATE TABLE` statements in
    //    `Database::migrate` — kept for now until every table they create
    //    has a corresponding refinery migration.
    info!("Running refinery migrations...");
    layanan_perlengkapan::migrations::run(db.pool()).await?;
    info!("Running legacy bootstrap tables (Database::migrate)...");
    db.migrate().await?;

    // Add essential indexes for performance optimization
    info!("Adding essential database indexes...");
    layanan_perlengkapan::shared::db_optimization::add_essential_indexes(db.pool()).await?;

    // Create main service with repository wrapper
    let service = PerlengkapanService::new(Arc::new(db.clone()));

    // ── Ports & adapters: dokumen + notifikasi service traits ────────────
    let template_service = Arc::new(dokumen::TemplateService::new());
    let pdf_generator = Arc::new(dokumen::PdfGenerator::new(dokumen::TemplateService::new()));
    let excel_generator = Arc::new(dokumen::excel_generator::ExcelGenerator::new());
    let docx_generator = Arc::new(dokumen::DocxGenerator::new(dokumen::TemplateService::new()));
    let document_storage_for_docs: Arc<dyn DocumentStorage> =
        Arc::new(dokumen::FilesystemStorage::from_env());
    let docs: Arc<dyn DocumentGenerator> = Arc::new(
        dokumen::service::DokumenService::new(
            db.pool().clone(),
            template_service.clone(),
            pdf_generator,
            excel_generator,
            docx_generator,
        )
        .with_storage(document_storage_for_docs.clone()),
    );
    // Try to bring up an EmailService backed by Secreton-fetched SMTP creds.
    // The host/port/from-address still come from `notifikasi::AppConfig`
    // (env), since those aren't secrets and benefit from being grep-able in
    // values.yaml. Production deployments must succeed here; dev profiles
    // can run without an email channel and only the in-app notifications
    // will land.
    let email_service: Option<Arc<notifikasi::email::EmailService>> = {
        let notif_config = notifikasi::config::AppConfig::from_env();
        if notif_config.smtp_host.is_empty() {
            info!("SMTP_HOST unset — skipping EmailService wiring");
            None
        } else if let Some(client) = secreton_client.as_ref() {
            match notifikasi::email::EmailService::new_with_secreton(
                notif_config.clone(),
                db.pool().clone(),
                client,
                "perlengkapan/notifikasi/smtp",
            )
            .await
            {
                Ok(svc) => {
                    info!("EmailService wired with credentials from Secreton");
                    Some(Arc::new(svc))
                }
                Err(e) if is_production => {
                    return Err(anyhow::anyhow!(
                        "EmailService init failed in production: {}",
                        e
                    ));
                }
                Err(e) => {
                    error!("EmailService init failed: {} (continuing without email)", e);
                    None
                }
            }
        } else {
            info!("No Secreton connection — skipping EmailService wiring");
            None
        }
    };

    let mut notifikasi_service = notifikasi::service::NotifikasiService::new(db.pool().clone());
    if let Some(email) = email_service {
        notifikasi_service = notifikasi_service.with_email(email);
    }
    let notifier: Arc<dyn NotificationSender> = Arc::new(notifikasi_service);
    let audit_sink: Arc<dyn AuditSink> = Arc::new(
        layanan_perlengkapan::shared::audit::PgAuditSink::new(db.pool().clone()),
    );
    // Re-use the same storage adapter on AppState so external callers
    // (e.g. handlers that want to stream by storage_key) hit the same
    // backend the generator wrote to.
    let document_storage: Arc<dyn DocumentStorage> = document_storage_for_docs;
    info!("DocumentGenerator + NotificationSender + AuditSink + DocumentStorage ports wired up");

    // Create Pakaian Dinas service
    let pakaian_dinas_repo = PakaianDinasRepository::new(db.pool().clone());
    let mut pakaian_dinas_service = PakaianDinasService::new(pakaian_dinas_repo);
    // Fase 2.4: inject IntegrasiClient (clone) untuk laporan kesegaran sync
    // MySIMKARI di wizard ukuran. Original di-move ke kebutuhan_bmn di bawah.
    if let Some(client) = integrasi_client.clone() {
        pakaian_dinas_service = pakaian_dinas_service.with_integrasi_client(client);
    }
    // #16/#40: audit trail for validator transitions.
    pakaian_dinas_service = pakaian_dinas_service.with_audit_sink(audit_sink.clone());

    // Create Kebutuhan BMN service with workflow engine
    let kebutuhan_bmn_repo = PgKebutuhanBmnRepository::new(db.pool().clone());
    let kebutuhan_bmn_workflow_engine =
        crate::workflow::engine::WorkflowEngine::for_kebutuhan_bmn(db.pool().clone())
            .with_document_generator(docs.clone())
            .with_notification_sender(notifier.clone())
            .with_audit_sink(audit_sink.clone());
    let mut kebutuhan_bmn_service = KebutuhanBmnService::new(
        kebutuhan_bmn_repo,
        authenc_client.clone(),
        kebutuhan_bmn_workflow_engine,
    );
    // #36: keep a clone for AppState so the health endpoint can read live
    // circuit-breaker state. Clones share the same Arc<IntegrasiBreakers>, so
    // this observes exactly what the services trip. Capture before the move.
    let integrasi_client_for_health = integrasi_client.clone();
    if let Some(client) = integrasi_client {
        kebutuhan_bmn_service = kebutuhan_bmn_service.with_integrasi_client(client);
    }

    // Create Dashboard service
    let dashboard_service = DashboardService::new(db.pool().clone());

    // Create Roadmap Sarpras service
    let roadmap_repo = RoadmapRepository::new(db.clone());
    let roadmap_service = RoadmapService::new(roadmap_repo);

    // Create Pemakaian BMN service
    let pemakaian_bmn_repo = pemakaian_bmn::PemakaianBmnRepository::new(db.pool().clone());
    let pemakaian_bmn_workflow_engine =
        crate::workflow::engine::WorkflowEngine::for_pemakaian_bmn(db.pool().clone())
            .with_document_generator(docs.clone())
            .with_notification_sender(notifier.clone())
            .with_audit_sink(audit_sink.clone());
    let pemakaian_bmn_service =
        PemakaianBmnService::new(pemakaian_bmn_repo, pemakaian_bmn_workflow_engine)
            .with_document_generator(docs.clone())
            .with_notification_sender(notifier.clone());

    // Start Pemakaian BMN scheduler for auto-expiry and notifications
    let pemakaian_bmn_scheduler =
        pemakaian_bmn::PemakaianBmnScheduler::new(pemakaian_bmn_service.clone());
    pemakaian_bmn_scheduler.start();
    info!("Pemakaian BMN scheduler started");

    // Start SLA escalation scheduler for workflow monitoring — now wired to
    // the NotificationSender port instead of the deleted gRPC client.
    let sla_scheduler =
        workflow::SlaEscalationScheduler::new(db.pool().clone()).with_notifier(notifier.clone());
    if let Err(e) = sla_scheduler.start() {
        error!("Failed to start SLA escalation scheduler: {}", e);
    } else {
        info!("SLA escalation scheduler started");
    }

    // Create Penghapusan BMN service
    let penghapusan_bmn_workflow_engine =
        crate::workflow::engine::WorkflowEngine::for_penghapusan_bmn(db.pool().clone())
            .with_document_generator(docs.clone())
            .with_notification_sender(notifier.clone())
            .with_audit_sink(audit_sink.clone());
    let penghapusan_bmn_service = Arc::new(
        penghapusan_bmn::PenghapusanBmnService::new(
            db.pool().clone(),
            Arc::new(penghapusan_bmn_workflow_engine),
        )
        .with_document_generator(docs.clone()),
    );
    info!("Penghapusan BMN service initialized");

    // Create broadcast channel for dashboard updates (capacity: 100 messages)
    let (dashboard_tx, _dashboard_rx) = tokio::sync::broadcast::channel(100);

    // Create cache manager
    let cache_manager = Arc::new(CacheManager::new());
    info!("Cache manager initialized");

    // Create rate limiter
    let rate_limit_config = RateLimitConfig::from_env();
    let rate_limiter = Arc::new(RateLimiter::new(rate_limit_config));
    info!(
        "Rate limiter initialized ({}  req/s per user, burst: {})",
        rate_limiter.get_stats().await.config.requests_per_second,
        rate_limiter.get_stats().await.config.burst_size
    );

    // Start rate limiter cleanup task
    let rate_limiter_cleanup = Arc::clone(&rate_limiter);
    tokio::spawn(async move {
        layanan_perlengkapan::shared::rate_limit::cleanup_task(rate_limiter_cleanup).await;
    });

    // Create AppState
    let state = AppState {
        service,
        authenc: authenc_client,
        pakaian_dinas_service,
        kebutuhan_bmn_service,
        pemakaian_bmn_service,
        penghapusan_bmn_service,
        roadmap_service,
        dashboard_service,
        dashboard_updates: dashboard_tx,
        db_pool: db.pool().clone(),
        cache_manager,
        rate_limiter,
        docs,
        notifier,
        audit_sink,
        document_storage,
        boot_time: std::time::Instant::now(),
        integrasi_client: integrasi_client_for_health,
    };

    // Build router
    let app = build_router(state);

    // Start server
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    info!("Perlengkapan service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    let allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());

    let cors = if allowed_origins == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<HeaderValue> = allowed_origins
            .split(',')
            .map(|s| {
                s.trim()
                    .parse::<HeaderValue>()
                    .unwrap_or(HeaderValue::from_static(""))
            })
            .filter(|h| !h.is_empty())
            .collect();

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Health check routes (no auth or rate limiting required)
    let health_routes = Router::new()
        .route(
            "/health",
            get(layanan_perlengkapan::shared::health::health_check),
        )
        .route(
            "/health/ready",
            get(layanan_perlengkapan::shared::health::readiness_check),
        )
        .route(
            "/health/live",
            get(layanan_perlengkapan::shared::health::liveness_check),
        )
        .route("/metrics", get(middleware::metrics::metrics_handler))
        .with_state(Arc::new(state.clone()));

    // API routes with authentication and rate limiting
    let api_routes = routes::create_routes(state.clone());

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .nest("/api/pembinaan/perlengkapan", api_routes)
        .layer(axum::middleware::from_fn(
            middleware::metrics::track_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state.rate_limiter),
            layanan_perlengkapan::shared::rate_limit::rate_limit_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
