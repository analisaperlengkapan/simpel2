//! SIMPelv2 Layanan Dashboard
//!
//! Microservice untuk dashboard, analytics, dan real-time metrics
//! untuk semua unit kerja dalam superapp SIMPelv2

mod aggregator;
mod analytics;
mod charts;
mod config;
mod error;
mod handlers;
mod models;
mod real_time;

use crate::{config::AppConfig, handlers::create_routes};
use prometheus::Registry;
use std::{net::SocketAddr, time::Duration};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AppConfig::from_env();

    // Setup tracing
    tracing_subscriber::fmt::init();

    // Database connection
    let mut pool_config = deadpool_postgres::Config::new();
    pool_config.url = Some(config.database_url.clone());
    let db = pool_config.create_pool(
        Some(deadpool_postgres::Runtime::Tokio1),
        tokio_postgres::NoTls,
    )?;

    // Redis connection untuk real-time data
    let _redis_client = redis::Client::open(config.redis_url.clone())?;

    // Metrics registry
    let _metrics_registry = Registry::new();

    // Create router
    let app = create_routes(config.clone(), db.clone()).layer(
        tower::ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new())
            .layer({
                #[allow(deprecated)]
                TimeoutLayer::new(Duration::from_secs(30))
            })
            .layer(CorsLayer::permissive()),
    );

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("🚀 Dashboard service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
