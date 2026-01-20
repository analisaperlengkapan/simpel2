use axum::{
    Router,
    routing::{get, post},
};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_postgres::NoTls;

mod handlers;
mod models;

use handlers::{
    AppState, add_comment, create_perkara, get_perkara_detail, get_timeline, list_perkara,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Database setup from environment variables or default
    let mut pg_config = tokio_postgres::Config::new();
    pg_config.host(&env::var("DB_HOST").unwrap_or_else(|_| "postgres".to_string()));
    pg_config.user(&env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()));
    pg_config.password(&env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string()));
    pg_config.dbname(&env::var("DB_NAME").unwrap_or_else(|_| "simpelv2".to_string()));

    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
    let pool = Pool::builder(mgr).max_size(16).build().unwrap();

    // Run migrations
    if let Ok(client) = pool.get().await {
        let migration_0001 = include_str!("../migrations/0001_create_perkara_table.sql");
        let migration_0002 = include_str!("../migrations/0002_create_timeline_comments.sql");

        if let Err(e) = client.batch_execute(migration_0001).await {
            tracing::error!("Failed to run migration 0001: {}", e);
        } else {
            tracing::info!("Migration 0001 executed successfully");
        }

        if let Err(e) = client.batch_execute(migration_0002).await {
            tracing::error!("Failed to run migration 0002: {}", e);
        } else {
            tracing::info!("Migration 0002 executed successfully");
        }
    } else {
        tracing::error!("Failed to connect to database for migration");
    }

    let state = AppState { pool };

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/pidum/health", get(health_check))
        .route("/api/v1/pidum/status", get(status))
        // Perkara routes
        .route(
            "/api/v1/pidum/perkara",
            get(list_perkara).post(create_perkara),
        )
        .route("/api/v1/pidum/perkara/:id", get(get_perkara_detail))
        .route("/api/v1/pidum/perkara/:id/comments", post(add_comment))
        .route("/api/v1/pidum/perkara/:id/timeline", get(get_timeline))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("pidum service listening on {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "pidum Service OK"
}

async fn status() -> &'static str {
    "pidum service is running"
}
