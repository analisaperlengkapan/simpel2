use axum::{
    Router,
    routing::{get, post, put},
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod handlers;
mod models;
mod state;

use handlers::{create_case, get_case_by_id, get_cases, update_case};
use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Initialize state
    let state = AppState::new();

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/datun/health", get(health_check))
        .route("/api/v1/datun/status", get(status))
        // Case Management Routes
        .route("/api/v1/datun/cases", get(get_cases).post(create_case))
        .route(
            "/api/v1/datun/cases/:id",
            get(get_case_by_id).put(update_case),
        )
        .with_state(state)
        // Enable CORS for frontend integration
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Datun service listening on {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Datun Service OK"
}

async fn status() -> &'static str {
    "Datun service is running"
}
