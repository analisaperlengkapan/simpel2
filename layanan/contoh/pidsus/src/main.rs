use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod config;
mod handlers;
mod models;
mod repository;

use config::Config;
use handlers::AppState;
use repository::Repository;

#[tokio::main]
async fn main() {
    // Initialize tracing with shared telemetry
    lib_common::telemetry::init_subscriber("info");

    // Load config
    let config = Config::from_env();

    // Initialize DB pool using shared lib_common
    let db_config = lib_common::db::DbConfig {
        url: config.database_url.clone(),
        max_size: config.database_pool_size,
        min_idle: None,
        connection_timeout: None,
        idle_timeout: None,
        max_lifetime: None,
    };
    let pool = lib_common::db::create_postgres_pool(db_config).expect("Failed to create DB pool");
    let repository = Repository::new(pool);
    let app_state = AppState { repository };

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any) // For dev, refine for prod
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/v1/pidsus/health", get(handlers::health_check))
        .route(
            "/api/v1/pidsus/dashboard/stats",
            get(handlers::get_dashboard_stats),
        )
        .route(
            "/api/v1/pidsus/cases",
            get(handlers::list_cases).post(handlers::create_case),
        )
        .route("/api/v1/pidsus/cases/:id", get(handlers::get_case))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    println!("pidsus service listening on {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
