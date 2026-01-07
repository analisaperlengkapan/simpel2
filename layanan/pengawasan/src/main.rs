use layanan_pengawasan::{router::create_router, db};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::cors::CorsLayer;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize Database
    let pool = db::create_pool().await.expect("Failed to create database pool");

    // Run Migrations
    tracing::info!("Running database migrations...");
    if let Err(e) = db::migrate(&pool).await {
        tracing::error!("Failed to run migrations: {}", e);
        // In production, you might want to panic here, but for dev we continue
    }

    let app = create_router(pool).layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3004));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
