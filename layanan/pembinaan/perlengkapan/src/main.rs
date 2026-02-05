//! # Layanan Pembinaan Perlengkapan Backend Service
//!
//! Backend microservice for Perlengkapan (asset management) within SIMPelv2.
//! Integrates with Authenc (IAM) and Secreton (Secret Manager) via gRPC.

use axum::http::HeaderValue;
use axum::{Router, extract::FromRef, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

mod database;
mod errors;
mod grpc_clients;
mod handlers;
mod kebutuhan_bmn;
mod middleware;
mod models;
mod pakaian_dinas;
mod repository;
mod routes;
mod services;

#[cfg(test)]
mod tests;

use database::Database;
use grpc_clients::{AuthencClient, SecretonClient};
use kebutuhan_bmn::{KebutuhanBmnService, PgKebutuhanBmnRepository};
use pakaian_dinas::{PakaianDinasRepository, PakaianDinasService};
use services::PerlengkapanService;

#[derive(Clone)]
pub struct AppState {
    pub service: PerlengkapanService,
    pub authenc: AuthencClient,
    pub pakaian_dinas_service: PakaianDinasService,
    pub kebutuhan_bmn_service: KebutuhanBmnService,
}

impl FromRef<AppState> for PerlengkapanService {
    fn from_ref(state: &AppState) -> Self {
        state.service.clone()
    }
}

impl FromRef<AppState> for AuthencClient {
    fn from_ref(state: &AppState) -> Self {
        state.authenc.clone()
    }
}

impl FromRef<AppState> for PakaianDinasService {
    fn from_ref(state: &AppState) -> Self {
        state.pakaian_dinas_service.clone()
    }
}

impl FromRef<AppState> for KebutuhanBmnService {
    fn from_ref(state: &AppState) -> Self {
        state.kebutuhan_bmn_service.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,layanan_perlengkapan=debug".into()),
        )
        .init();

    info!("Starting Layanan Pembinaan Perlengkapan Service");

    // Load configuration from environment
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8093".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");

    // Secreton Integration
    let secreton_url =
        std::env::var("SECRETON_URL").unwrap_or_else(|_| "http://localhost:50051".to_string());
    let mut database_url = std::env::var("DATABASE_URL").ok();

    // Authenc Integration
    let authenc_url =
        std::env::var("AUTHENC_URL").unwrap_or_else(|_| "http://localhost:50052".to_string());

    if database_url.is_none() {
        info!("Connecting to Secreton at {}", secreton_url);
        match SecretonClient::connect(secreton_url).await {
            Ok(client) => {
                info!("Connected to Secreton");

                // Fetch DB URL
                match client.get_secret("perlengkapan/db").await {
                    Ok(data) => {
                        if let Some(url) = data.get("url") {
                            database_url = Some(url.clone());
                            info!("Fetched DATABASE_URL from Secreton");
                        }
                    }
                    Err(e) => error!("Failed to fetch db secret: {:?}", e),
                }
            }
            Err(e) => {
                error!(
                    "Failed to connect to Secreton: {}. Falling back to environment variables.",
                    e
                );
            }
        }
    }

    let database_url = database_url.expect("DATABASE_URL must be set (env or secreton)");

    // Initialize Authenc Client with Retry Logic
    info!("Connecting to Authenc at {}", authenc_url);
    let authenc_client = {
        let mut retries = 5;
        let mut client = None;
        let mut delay = tokio::time::Duration::from_secs(1);

        while retries > 0 {
            match AuthencClient::connect(authenc_url.clone()).await {
                Ok(c) => {
                    client = Some(c);
                    break;
                }
                Err(e) => {
                    error!(
                        "Failed to connect to Authenc: {}. Retrying in {:?}...",
                        e, delay
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                    retries -= 1;
                }
            }
        }
        client.expect("Failed to connect to Authenc service after retries")
    };

    // Initialize database connection
    info!("Connecting to database...");
    let db = Database::new(&database_url).await?;

    // Run migrations
    info!("Running database migrations...");
    db.migrate().await?;

    // Create main service with repository wrapper
    let service = PerlengkapanService::new(Arc::new(db.clone()));

    // Create Pakaian Dinas service
    let pakaian_dinas_repo = PakaianDinasRepository::new(db.pool().clone());
    let pakaian_dinas_service = PakaianDinasService::new(pakaian_dinas_repo);

    // Create Kebutuhan BMN service
    let kebutuhan_bmn_repo = PgKebutuhanBmnRepository::new(db.pool().clone());
    let kebutuhan_bmn_service =
        KebutuhanBmnService::new(kebutuhan_bmn_repo, authenc_client.clone());

    // Create AppState
    let state = AppState {
        service,
        authenc: authenc_client,
        pakaian_dinas_service,
        kebutuhan_bmn_service,
    };

    // Build router
    let app = build_router(state);

    // Start server
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    info!("Perlengkapan service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    let allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());

    let cors = if allowed_origins == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<HeaderValue> = allowed_origins
            .split(',')
            .map(|s| {
                s.trim()
                    .parse::<HeaderValue>()
                    .unwrap_or(HeaderValue::from_static(""))
            })
            .filter(|h| !h.is_empty())
            .collect();

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check));

    // API routes with authentication
    // Note: create_routes expects PerlengkapanService, but we pass AppState
    // We need to adjust routes.rs or pass state.service specifically if routes expects service directly.
    // However, typical pattern is router.with_state(state).
    // Let's check routes.rs

    let api_routes = routes::create_routes(state.clone()); // Need to update routes.rs signature

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .nest("/api/pembinaan/perlengkapan", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

// Health check handlers
async fn health_check() -> &'static str {
    "OK"
}

async fn readiness_check() -> &'static str {
    "Ready"
}

async fn liveness_check() -> &'static str {
    "Alive"
}
