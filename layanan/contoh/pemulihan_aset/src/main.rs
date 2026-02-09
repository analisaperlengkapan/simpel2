use axum::{
    Router,
    routing::{get, post},
};
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_postgres::NoTls;
use tower_http::cors::{Any, CorsLayer};

mod db;
mod handlers;
mod model;

use db::DB;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Setup connection pool
    let mut cfg = Config::new();
    cfg.host = Some(env::var("DB_HOST").unwrap_or("localhost".to_string()));
    cfg.port = Some(
        env::var("DB_PORT")
            .unwrap_or("5432".to_string())
            .parse()
            .unwrap(),
    );
    cfg.user = Some(env::var("DB_USER").unwrap_or("postgres".to_string()));
    cfg.password = Some(env::var("DB_PASSWORD").unwrap_or("postgres".to_string()));
    cfg.dbname = Some(env::var("DB_NAME").unwrap_or("simpelv2_pemulihan_aset".to_string()));

    // Use ManagerConfig to set recycling method if needed
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create pool");
    let db = DB::new(pool);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/pemulihan_aset/health", get(health_check))
        .route("/api/v1/pemulihan_aset/status", get(status))
        // Case routes
        .route(
            "/api/v1/pemulihan_aset/cases",
            get(handlers::list_cases).post(handlers::create_case),
        )
        // Asset routes
        .route(
            "/api/v1/pemulihan_aset/cases/:case_id/assets",
            get(handlers::list_assets),
        )
        .route(
            "/api/v1/pemulihan_aset/assets",
            post(handlers::create_asset),
        )
        .layer(cors)
        .with_state(db);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("pemulihan_aset service listening on {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "pemulihan_aset Service OK"
}

async fn status() -> &'static str {
    "pemulihan_aset service is running"
}
