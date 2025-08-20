use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::init();

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/pidsus/health", get(health_check))
        .route("/api/v1/pidsus/status", get(status));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("pidsus service listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health_check() -> &'static str {
    "pidsus Service OK"
}

async fn status() -> &'static str {
    "pidsus service is running"
}
