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

use layanan_perlengkapan::{
    cache_strategy, dashboard, database, database_optimization, dokumen, grpc_clients,
    health, kebutuhan_bmn, logging, middleware, notifikasi, pakaian_dinas, pemakaian_bmn,
    penghapusan_bmn, rate_limiting, roadmap_sarpras, routes, services, workflow,
};
use lib_perlengkapan::contracts::{DocumentGenerator, NotificationSender};

use cache_strategy::CacheManager;
use dashboard::services::DashboardService;
use database::Database;
use grpc_clients::{AuthencClient, IntegrasiClient, SecretonClient};
use kebutuhan_bmn::{KebutuhanBmnService, PgKebutuhanBmnRepository};
use pakaian_dinas::{PakaianDinasRepository, PakaianDinasService};
use pemakaian_bmn::PemakaianBmnService;
use rate_limiting::{RateLimitConfig, RateLimiter};
use roadmap_sarpras::{RoadmapRepository, RoadmapService};
use services::PerlengkapanService;

use layanan_perlengkapan::AppState;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    logging::init_structured_logging();

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

    if database_url.is_none() {
        info!("Connecting to Secreton at {}", secreton_url);
        match SecretonClient::connect(secreton_url).await {
            Ok(client) => {
                info!("Connected to Secreton");

                // Fetch DB URL
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
            Err(e) => {
                error!(
                    "Failed to connect to Secreton: {}. Falling back to environment variables.",
                    e
                );
            }
        }
    }

    let database_url = database_url.expect("DATABASE_URL must be set (env or secreton)");

    // Initialize Authenc Client with Retry Logic (optional for dev)
    let skip_authenc = std::env::var("SKIP_AUTHENC").unwrap_or_default() == "true";
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
    database_optimization::add_essential_indexes(db.pool()).await?;

    // Create main service with repository wrapper
    let service = PerlengkapanService::new(Arc::new(db.clone()));

    // ── Ports & adapters: dokumen + notifikasi service traits ────────────
    let template_service = Arc::new(dokumen::TemplateService::new());
    let pdf_generator =
        Arc::new(dokumen::PdfGenerator::new(dokumen::TemplateService::new()));
    let excel_generator = Arc::new(dokumen::excel_generator::ExcelGenerator::new());
    let docx_generator =
        Arc::new(dokumen::DocxGenerator::new(dokumen::TemplateService::new()));
    let docs: Arc<dyn DocumentGenerator> = Arc::new(dokumen::service::DokumenService::new(
        db.pool().clone(),
        template_service.clone(),
        pdf_generator,
        excel_generator,
        docx_generator,
    ));
    let notifier: Arc<dyn NotificationSender> =
        Arc::new(notifikasi::service::NotifikasiService::new(db.pool().clone()));
    info!("DocumentGenerator + NotificationSender ports wired up");

    // Create Pakaian Dinas service
    let pakaian_dinas_repo = PakaianDinasRepository::new(db.pool().clone());
    let pakaian_dinas_service = PakaianDinasService::new(pakaian_dinas_repo);

    // Create Kebutuhan BMN service with workflow engine
    let kebutuhan_bmn_repo = PgKebutuhanBmnRepository::new(db.pool().clone());
    let kebutuhan_bmn_workflow_engine =
        crate::workflow::engine::WorkflowEngine::for_kebutuhan_bmn(db.pool().clone())
            .with_document_generator(docs.clone())
            .with_notification_sender(notifier.clone());
    let mut kebutuhan_bmn_service = KebutuhanBmnService::new(
        kebutuhan_bmn_repo,
        authenc_client.clone(),
        kebutuhan_bmn_workflow_engine,
    );
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
            .with_notification_sender(notifier.clone());
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
    let sla_scheduler = workflow::SlaEscalationScheduler::new(db.pool().clone())
        .with_notifier(notifier.clone());
    if let Err(e) = sla_scheduler.start() {
        error!("Failed to start SLA escalation scheduler: {}", e);
    } else {
        info!("SLA escalation scheduler started");
    }

    // Create Penghapusan BMN service
    let penghapusan_bmn_workflow_engine =
        crate::workflow::engine::WorkflowEngine::for_penghapusan_bmn(db.pool().clone())
            .with_document_generator(docs.clone())
            .with_notification_sender(notifier.clone());
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
        rate_limiting::cleanup_task(rate_limiter_cleanup).await;
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
        boot_time: std::time::Instant::now(),
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
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
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
            rate_limiting::rate_limit_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
