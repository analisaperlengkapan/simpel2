use axum::{Router, response::Json, routing::get};
use serde_json::{Value, json};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health))
        .route("/", get(root))
        .layer(CorsLayer::permissive());

    // Run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3007));
    println!("Layanan Laporan listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "healthy", "service": "laporan" }))
}

async fn root() -> Json<Value> {
    Json(json!({ "message": "Layanan Laporan API", "version": "0.1.0" }))
}
