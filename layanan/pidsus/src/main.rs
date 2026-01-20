#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::module_inception)]
#![allow(dead_code)]
#![allow(unused_variables)]

use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod config;
mod db;
mod handlers;
mod models;
mod repository;

use config::Config;
use handlers::AppState;
use repository::Repository;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load config
    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
        std::process::exit(1);
    });

    // Initialize DB pool
    let pool = db::create_pool(&config);
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
