use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::limit::ConcurrencyLimitLayer;
use tower_http::services::ServeDir;
use std::time::Duration;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::trace::TraceLayer;
use tower_http::limit::RateLimitLayer;
use std::num::NonZeroU32;
use metrics_exporter_prometheus::PrometheusBuilder;
use sentry::{ClientInitGuard, IntoDsn};
use sentry_tracing::layer as sentry_tracing_layer;
use crate::middleware::{auth_middleware, rbac_middleware, audit_middleware};

mod auth;
mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod rbac;
mod vault;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Sentry
    let _sentry = sentry::init((
        std::env::var("SENTRY_DSN").unwrap_or_default().into_dsn().ok(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    // Integrasi tracing dengan Sentry
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish()
        .with(sentry_tracing_layer());
    tracing::subscriber::set_global_default(subscriber)?;

    // Prometheus exporter
    let builder = PrometheusBuilder::new();
    let recorder = builder.install_recorder()?;
    let prometheus_handle = recorder.handle();

    info!("🚀 Starting SIMPelv2 Security Service...");

    // Load configuration
    let config = config::Config::load()?;
    info!("📋 Configuration loaded successfully");

    // Initialize database connection
    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    info!("🗄️ Database connection established");

    // Jalankan migrasi otomatis sebelum inisialisasi service
    sqlx::migrate!().run(&pool).await?;
    info!("✅ Database migration completed");

    // Jalankan background task data retention
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        loop {
            // Hapus audit log dan session lebih dari 1 tahun
            let _ = sqlx::query!("DELETE FROM keamanan.audit_logs WHERE timestamp < NOW() - INTERVAL '1 year'")
                .execute(&pool_clone)
                .await;
            let _ = sqlx::query!("DELETE FROM keamanan.sessions WHERE expires_at < NOW() - INTERVAL '1 year'")
                .execute(&pool_clone)
                .await;
            tokio::time::sleep(std::time::Duration::from_secs(86400)).await; // 1 hari
        }
    });

    // Initialize Vault client
    let vault_client = vault::VaultClient::new(&config.vault_url, &config.vault_token)?;
    info!("🔐 Vault client initialized");

    // Build application state
    let state = models::AppState {
        pool,
        vault_client,
        config,
    };

    // Build router
    let login_route = Router::new()
        .route("/auth/login", post(handlers::login))
        .layer(
            ServiceBuilder::new()
                .layer(RateLimitLayer::new(5, Duration::from_secs(60))) // 5 req per 60 detik
                .into_inner()
        );

    let refresh_route = Router::new()
        .route("/auth/refresh", post(handlers::refresh_token))
        .layer(
            ServiceBuilder::new()
                .layer(RateLimitLayer::new(5, Duration::from_secs(60)))
                .into_inner()
        );

    let app = Router::new()
        .route("/health", get(handlers::health))
        .merge(login_route)
        .merge(refresh_route)
        .route("/auth/logout", post(handlers::logout))
        .route("/auth/mfa/setup", post(handlers::setup_mfa))
        .route("/auth/mfa/verify", post(handlers::verify_mfa))
        .route("/users/me", get(handlers::get_current_user).route_layer(middleware::from_fn_with_state(state.clone(), |req, next| rbac_middleware("read:own", req, next))))
        .route("/roles", get(handlers::get_roles).route_layer(middleware::from_fn_with_state(state.clone(), |req, next| rbac_middleware("admin:roles", req, next))))
        .route("/roles", post(handlers::create_role).route_layer(middleware::from_fn_with_state(state.clone(), |req, next| rbac_middleware("admin:roles", req, next))))
        .route("/permissions", get(handlers::get_permissions).route_layer(middleware::from_fn_with_state(state.clone(), |req, next| rbac_middleware("admin:permissions", req, next))))
        .route("/audit/logs", get(handlers::get_audit_logs).route_layer(middleware::from_fn_with_state(state.clone(), |req, next| rbac_middleware("read:audit", req, next))))
        .route("/metrics", get(|| async move {
            let metrics = prometheus_handle.render();
            ([("Content-Type", "text/plain; version=0.0.4")], metrics)
        }))
        .route("/vault/rotate_secret", post(handlers::rotate_vault_secret))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), audit_middleware))
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("🌐 Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
} 