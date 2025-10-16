use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/intel/health", get(health_check))
        .route("/api/v1/intel/status", get(status));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("intel service listening on {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "intel Service OK"
}

async fn status() -> &'static str {
    "intel service is running"
}
