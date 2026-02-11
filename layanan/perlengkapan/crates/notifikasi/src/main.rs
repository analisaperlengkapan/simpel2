#[allow(unused)]
mod audit;
mod config;
#[allow(unused)]
mod email;
#[allow(unused)]
mod error;
mod grpc_service;
mod handlers;
mod in_app;
#[allow(unused)]
mod models;
mod preferences;
#[allow(unused)]
mod push;
#[allow(unused)]
mod queue;
mod queue_processor;
mod scheduler;
#[allow(unused)]
mod security;
mod sms;
#[allow(unused)]
mod template;
mod websocket;
#[allow(unused)]
mod whatsapp;

use crate::config::AppConfig;
use crate::email::EmailService;
use crate::grpc_service::NotificationServiceImpl;
use crate::push::PushService;
use crate::queue_processor::QueueProcessor;
use crate::security::RateLimitState;
use crate::sms::SmsService;
use axum::{Router, http::Method};
use dashmap::DashMap;
use deadpool_postgres::{Config, Runtime};
use prometheus::{Encoder, Registry, TextEncoder};
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = AppConfig::from_env();

    // Logging & tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Notifikasi service");

    // DB pool
    let mut cfg = Config::new();
    cfg.url = Some(config.database_url.clone());
    let pool = cfg.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;

    // Redis
    let _redis = redis::Client::open(config.redis_url.clone())?;

    // Prometheus registry
    let registry = Registry::new();

    // Initialize notification services
    let email_service = Arc::new(EmailService::new(config.clone(), pool.clone()));
    let sms_service = Arc::new(SmsService::new(config.clone(), pool.clone()));
    let push_service = Arc::new(PushService::new(config.clone(), pool.clone()));

    // Initialize queue processor
    let queue_processor = Arc::new(QueueProcessor::new(
        Arc::new(config.clone()),
        pool.clone(),
        email_service.clone(),
        sms_service.clone(),
        push_service.clone(),
    ));

    // Start queue processor in background
    let queue_processor_handle = queue_processor.clone();
    tokio::spawn(async move {
        queue_processor_handle.start().await;
    });

    tracing::info!("Notification queue processor started");

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::HeaderName::from_static("x-api-key"),
        ]);

    // Rate limit state
    let rate_limit_state: RateLimitState = Arc::new(DashMap::new());

    // WebSocket state
    let ws_state = websocket::WsState::new(pool.clone());

    // HTTP Router
    let app = handlers::routes(
        config.clone(),
        pool.clone(),
        rate_limit_state.clone(),
        ws_state.clone(),
    )
    .layer(cors)
    .layer(TraceLayer::new_for_http());

    // Health & metrics
    let metrics_route = Router::new().route(
        "/metrics",
        axum::routing::get(|| async move {
            let encoder = TextEncoder::new();
            let mut buffer = Vec::new();
            let mf = registry.gather();
            encoder.encode(&mf, &mut buffer).unwrap();
            String::from_utf8(buffer).unwrap()
        }),
    );
    let app = app.merge(metrics_route);

    // gRPC server
    let grpc_addr = format!("{}:{}", config.server_host, config.grpc_port)
        .parse::<SocketAddr>()?;

    let grpc_service = NotificationServiceImpl::new(pool.clone());
    let grpc_server = grpc_service.into_server();

    tracing::info!("gRPC server listening on {}", grpc_addr);

    // Start gRPC server in background
    tokio::spawn(async move {
        if let Err(e) = Server::builder()
            .add_service(grpc_server)
            .serve(grpc_addr)
            .await
        {
            tracing::error!("gRPC server error: {}", e);
        }
    });

    // HTTP server startup log
    tracing::info!(
        "HTTP server listening on {}:{}",
        config.server_host,
        config.server_port
    );

    // Run HTTP server
    let addr = SocketAddr::new(config.server_host.parse()?, config.server_port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = signal::ctrl_c();
    #[cfg(unix)]
    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    #[cfg(unix)]
    let terminate = terminate_signal.recv();
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::warn!("Shutdown signal received, shutting down gracefully...");
}
