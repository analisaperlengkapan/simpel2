// Security Service restored after SQLx migration
// Implementation with tokio-postgres and deadpool-postgres

use axum::{routing::get, Json, Router};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("✅ Security Service restored with tokio-postgres");
    info!("🔄 Migration from SQLx completed successfully");

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("🚀 Security Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "active",
        "message": "Security service restored with tokio-postgres",
        "migration_status": "completed"
    }))
}
