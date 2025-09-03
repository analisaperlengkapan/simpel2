use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/datun/health", get(health_check))
        .route("/api/v1/datun/status", get(status));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("datun service listening on {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "datun Service OK"
}

async fn status() -> &'static str {
    "datun service is running"
}
