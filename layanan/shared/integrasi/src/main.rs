//! SIMPelv2 Layanan Integrasi
//! 
//! Service mesh untuk komunikasi antar microservices dalam superapp.
//! Menangani service discovery, load balancing, dan message passing.

mod config;
mod error;
mod models;
mod handlers;
mod service_discovery;
mod load_balancer;
mod circuit_breaker;
mod message_queue;
mod api_gateway;

use crate::{
    config::AppConfig,
    error::IntegrationError,
    handlers::create_routes,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
    // timeout::TimeoutLayer,  // Disabled - feature not available
};
use std::{net::SocketAddr, time::Duration};
use prometheus::Registry;

/// Application state untuk integration service
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: redis::Client,
    pub config: AppConfig,
    pub metrics_registry: Registry,
    pub service_registry: service_discovery::ServiceRegistry,
}

#[tokio::main]
async fn main() -> Result<(), IntegrationError> {
    // Load configuration
    let config = AppConfig::from_env()?;
    
    // Setup tracing
    tracing_subscriber::fmt::init();
    tracing::info!("🔗 Starting SIMPelv2 Integration Service");
    
    // Database connection
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    
    // Redis connection untuk caching dan pub/sub
    let redis_client = redis::Client::open(config.redis_url.clone())?;
    
    // Service registry untuk service discovery
    let service_registry = service_discovery::ServiceRegistry::new();
    
    // Metrics registry
    let metrics_registry = Registry::new();
    
    let state = AppState {
        db,
        redis: redis_client,
        config: config.clone(),
        metrics_registry,
        service_registry,
    };
    
    // Create router dengan routes untuk:
    // - Service registration/discovery
    // - Load balancing
    // - Message queue management
    // - Health checks
    let app = create_routes()
        .layer(
            tower::ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                // .layer(TimeoutLayer::new(Duration::from_secs(60))) // Disabled - feature not available
                .layer(CorsLayer::permissive())
        );
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 Integration service listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
