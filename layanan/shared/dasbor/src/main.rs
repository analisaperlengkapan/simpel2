//! SIMPelv2 Layanan Dashboard
//! 
//! Microservice untuk dashboard, analytics, dan real-time metrics
//! untuk semua unit kerja dalam superapp SIMPelv2

mod config;
mod error;
mod models;
mod handlers;
mod analytics;
mod charts;
mod real_time;
mod aggregator;

use crate::{
    config::AppConfig,
    error::DashboardError,
    handlers::create_routes,
};
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
    timeout::TimeoutLayer,
};
use std::{net::SocketAddr, time::Duration};
use prometheus::Registry;

/// Application state untuk dashboard service
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: redis::Client,
    pub config: AppConfig,
    pub metrics_registry: Registry,
}

#[tokio::main]
async fn main() -> Result<(), DashboardError> {
    // Load configuration
    let config = AppConfig::from_env()?;
    
    // Setup tracing
    tracing_subscriber::fmt::init();
    
    // Database connection
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;
    
    // Redis connection untuk real-time data
    let redis_client = redis::Client::open(config.redis_url.clone())?;
    
    // Metrics registry
    let metrics_registry = Registry::new();
    
    let state = AppState {
        db,
        redis: redis_client,
        config: config.clone(),
        metrics_registry,
    };
    
    // Create router
    let app = create_routes(state)
        .layer(
            tower::ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(CorsLayer::permissive())
        );
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 Dashboard service listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
