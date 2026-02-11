use dokumen::{config::AppConfig, handlers, DocumentScheduler};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = AppConfig::from_env();

    tracing::info!("Starting Document Service on {}:{}", config.server_host, config.server_port);

    // Create database pool
    let db_config = deadpool_postgres::Config {
        user: Some(config.database_url.split('@').next().and_then(|s| s.split(':').nth(1)).unwrap_or("postgres").to_string()),
        password: Some(config.database_url.split('@').next().and_then(|s| s.split(':').nth(2)).unwrap_or("postgres").to_string()),
        host: Some(config.database_url.split('@').nth(1).and_then(|s| s.split(':').next()).unwrap_or("localhost").to_string()),
        port: Some(config.database_url.split('@').nth(1).and_then(|s| s.split(':').nth(1)).and_then(|s| s.split('/').next()).and_then(|s| s.parse().ok()).unwrap_or(5432)),
        dbname: Some(config.database_url.split('/').last().unwrap_or("perlengkapan").to_string()),
        ..Default::default()
    };

    let pool = db_config
        .create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls)?;

    // Test database connection
    let _ = pool.get().await?;
    tracing::info!("Database connection established");

    // Start document scheduler
    let scheduler = Arc::new(DocumentScheduler::new(
        pool.clone(),
        PathBuf::from(&config.archive_storage_path),
    ));
    scheduler.clone().start().await;
    tracing::info!("Document scheduler started");

    // Create handler state
    let state = handlers::HandlerState {
        pool: pool.clone(),
        config: config.clone(),
    };

    // Build router
    let app = handlers::routes(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(config.socket_addr()).await?;
    tracing::info!("Server listening on {}", config.socket_addr());

    axum::serve(listener, app).await?;

    Ok(())
}
