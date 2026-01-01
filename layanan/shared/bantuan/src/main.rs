//! SIMPelv2 Layanan Bantuan
//!
//! Service untuk menangani bantuan pengguna, FAQ, ticketing system,
//! dan chatbot berbasis AI.

mod analytics;
mod audit;
mod captcha;
mod chatbot;
mod config;
mod error;
mod export_import;
mod faq;
mod gdpr;
mod handlers;
mod knowledge;
mod models;
mod rate_limit;
mod rbac;
mod ticket;
mod webhook;

use crate::{config::AppConfig, error::AppError, handlers::routes};
use axum::{Router, http::Method};
use prometheus::{Encoder, Registry, TextEncoder};
use std::{net::SocketAddr, time::Duration};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Application state yang dishare ke semua handlers
#[derive(Clone)]
pub struct AppState {
    pub db: deadpool_postgres::Pool,
    pub redis: redis::Client,
    pub config: AppConfig,
    pub metrics_registry: Registry,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Inisialisasi konfigurasi
    let config = AppConfig::from_env();

    // Setup tracing dan logging
    setup_tracing(&config)?;

    // Inisialisasi Sentry untuk error tracking
    let _sentry_guard = setup_sentry(&config);

    // Setup database connection
    let db_pool = setup_database(&config).await?;

    // Setup Redis connection
    let redis_client = setup_redis(&config)?;

    // Setup metrics registry
    let metrics_registry = setup_metrics()?;

    // Create application state
    let state = AppState {
        db: db_pool,
        redis: redis_client,
        config: config.clone(),
        metrics_registry,
    };

    // Create router dengan semua routes
    let app = create_app_router(state).await?;

    // Start server
    start_server(app, &config).await?;
    Ok(())
}

/// Setup tracing dan logging
fn setup_tracing(config: &AppConfig) -> Result<(), AppError> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.log_level))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
                .json(),
        )
        .init();

    tracing::info!("Tracing initialized with level: {}", config.log_level);
    Ok(())
}

/// Setup Sentry untuk error tracking
fn setup_sentry(config: &AppConfig) -> Option<sentry::ClientInitGuard> {
    if let Some(ref dsn) = config.sentry_dsn {
        tracing::info!("Initializing Sentry...");
        Some(sentry::init((
            dsn.as_str(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: 0.1,
                debug: config.log_level == "debug",
                ..Default::default()
            },
        )))
    } else {
        tracing::warn!("Sentry DSN tidak dikonfigurasi");
        None
    }
}

/// Setup database connection pool
async fn setup_database(_config: &AppConfig) -> Result<deadpool_postgres::Pool, AppError> {
    tracing::info!("Connecting to database...");

    let pool_config = deadpool_postgres::Config::new();
    let pool = match pool_config.create_pool(
        Some(deadpool_postgres::Runtime::Tokio1),
        tokio_postgres::NoTls,
    ) {
        Ok(pool) => pool,
        Err(e) => return Err(AppError::PoolConfig(e.to_string())),
    };

    // Test the connection
    let client = pool.get().await?;
    client.simple_query("SELECT 1").await?;

    tracing::info!("Database connected successfully");
    Ok(pool)
}

/// Setup Redis connection
fn setup_redis(config: &AppConfig) -> Result<redis::Client, AppError> {
    tracing::info!("Connecting to Redis...");

    let client = redis::Client::open(config.redis_url.clone())?;

    tracing::info!("Redis client created");
    Ok(client)
}

/// Setup metrics registry
fn setup_metrics() -> Result<Registry, AppError> {
    let registry = Registry::new();

    // Register default metrics
    // Anda bisa menambahkan custom metrics di sini

    tracing::info!("Metrics registry initialized");
    Ok(registry)
}

/// Create application router dengan semua middleware dan routes
async fn create_app_router(state: AppState) -> Result<Router, AppError> {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::HeaderName::from_static("x-api-key"),
        ])
        .allow_credentials(true);

    #[allow(deprecated)]
    // Create main application router
    let app = routes(state.config.clone(), state.db.clone()).layer(
        tower::ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new())
            .layer(TimeoutLayer::new(Duration::from_secs(30)))
            .layer(cors),
    );

    // Add health check and metrics routes
    let health_routes = Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/ready", axum::routing::get(readiness_check))
        .route("/metrics", axum::routing::get(metrics_handler))
        .with_state(state);

    Ok(app.merge(health_routes))
}

/// Start HTTP server
async fn start_server(app: Router, config: &AppConfig) -> Result<(), AppError> {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));

    tracing::info!("🚀 Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Health check handler
async fn health_check() -> &'static str {
    "OK"
}

/// Readiness check handler (bisa check database, Redis, dll)
async fn readiness_check(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<&'static str, AppError> {
    // Check database connection
    let client = state.db.get().await?;
    client.simple_query("SELECT 1").await?;

    // Check Redis connection (optional)
    // let mut conn = state.redis.get_async_connection().await?;
    // redis::cmd("PING").query_async(&mut conn).await?;

    Ok("READY")
}

/// Metrics handler untuk Prometheus
async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<String, AppError> {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    let metric_families = state.metrics_registry.gather();

    encoder.encode(&metric_families, &mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}

/// Graceful shutdown signal
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("🛑 Shutdown signal received");
}
