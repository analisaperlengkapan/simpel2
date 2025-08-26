use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::init();

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/pembinaan/health", get(health_check))
        .route("/api/v1/pembinaan/status", get(status));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("pembinaan service listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health_check() -> &'static str {
    "pembinaan Service OK"
}

async fn status() -> &'static str {
    "pembinaan service is running"
}
