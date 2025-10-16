use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/pidsus/health", get(health_check))
        .route("/api/v1/pidsus/status", get(status));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("pidsus service listening on {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "pidsus Service OK"
}

async fn status() -> &'static str {
    "pidsus service is running"
}
