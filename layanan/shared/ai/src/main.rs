use axum::{
    routing::{get, post},
    Router,
    middleware::from_fn,
    http::Request,
    response::Response,
    body::Body,
};
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use hyper::StatusCode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sentry::{ClientInitGuard, IntoDsn};
use sentry_tracing::layer as sentry_tracing_layer;
use metrics_exporter_prometheus::PrometheusBuilder;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use tower_http::cors::{CorsLayer, Any};
use uuid::Uuid;
use tokio::signal;

#[derive(Debug, Deserialize, Serialize)]
pub struct JwtClaims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

async fn rbac_middleware<B>(req: Request<B>, next: axum::middleware::Next<B>, required_role: &'static str) -> Result<Response, StatusCode> {
    use axum::http::header;
    let auth_header = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
    if let Some(auth) = auth_header {
        if auth.starts_with("Bearer ") {
            let token = &auth[7..];
            let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
            let claims = decode::<JwtClaims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::new(Algorithm::HS256));
            if let Ok(data) = claims {
                if data.claims.role == required_role {
                    // Extend request with claims if needed
                    return Ok(next.run(req).await);
                }
            }
        }
    }
    Err(StatusCode::FORBIDDEN)
}

mod config;
mod error;
mod handlers;
mod llm;
mod ocr;
mod rag;
mod models;
mod supervised;
mod rlhf;
mod active_learning;
mod transfer_learning;
mod hitl;

async fn auth_middleware<B>(req: Request<B>, next: axum::middleware::Next<B>) -> Result<Response, StatusCode> {
    use axum::http::header;
    let api_key = std::env::var("API_KEY").unwrap_or_default();
    let auth_header = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
    let x_api_key = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());
    let valid =
        (x_api_key.is_some() && x_api_key.unwrap() == api_key)
        || (auth_header.is_some() && auth_header.unwrap().starts_with("Bearer ") && auth_header.unwrap()[7..] == api_key);
    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}

struct RateLimiter {
    clients: Mutex<HashMap<String, (u32, Instant)>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self { clients: Mutex::new(HashMap::new()) }
    }
    fn check(&self, ip: &str) -> bool {
        let mut clients = self.clients.lock().unwrap();
        let now = Instant::now();
        let entry = clients.entry(ip.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) > Duration::from_secs(60) {
            *entry = (1, now);
            true
        } else {
            if entry.0 < 100 {
                entry.0 += 1;
                true
            } else {
                false
            }
        }
    }
}

static RATE_LIMITER: once_cell::sync::Lazy<Arc<RateLimiter>> = once_cell::sync::Lazy::new(|| Arc::new(RateLimiter::new()));

async fn rate_limit_middleware<B>(req: Request<B>, next: axum::middleware::Next<B>) -> Result<Response, StatusCode> {
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("127.0.0.1")
        .to_string();
    if !RATE_LIMITER.check(&ip) {
        let mut resp = Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Retry-After", "60")
            .body(Body::from("Rate limit exceeded. Please try again later."))
            .unwrap();
        return Ok(resp);
    }
    Ok(next.run(req).await)
}

async fn request_id_middleware<B>(mut req: Request<B>, next: axum::middleware::Next<B>) -> Result<Response, StatusCode> {
    use axum::http::header;
    let req_id = req.headers().get("x-request-id").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_else(|| Uuid::new_v4().to_string());
    req.extensions_mut().insert(req_id.clone());
    let mut resp = next.run(req).await;
    resp.headers_mut().insert(header::HeaderName::from_static("x-request-id"), req_id.parse().unwrap().into());
    Ok(resp)
}

fn mask_sensitive(data: &str) -> String {
    let mut masked = data.to_string();
    let patterns = ["password", "api_key", "token", "secret"];
    for pat in patterns.iter() {
        let re = regex::Regex::new(&format!(r#"{}"\s*:\s*"[^"]*""#, pat)).unwrap();
        masked = re.replace_all(&masked, format!("{}: \"***\"", pat)).to_string();
    }
    masked
}

async fn audit_log_middleware<B>(req: Request<B>, next: axum::middleware::Next<B>) -> Result<Response, StatusCode> {
    use chrono::Utc;
    let endpoint = req.uri().path().to_string();
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("127.0.0.1")
        .to_string();
    let user = req.headers().get("x-api-key").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let waktu = Utc::now();
    // Mask payload jika ada
    let masked_body = req.extensions().get::<String>().map(|b| mask_sensitive(b)).unwrap_or_default();
    // Log ke DB (async, fire and forget)
    let pool = req.extensions().get::<crate::models::AppState>().map(|s| s.pool.clone());
    if let Some(pool) = pool {
        let endpoint = endpoint.clone();
        let ip = ip.clone();
        let user = user.clone();
        let masked_body = masked_body.clone();
        tokio::spawn(async move {
            let _ = sqlx::query!("INSERT INTO ai_audit_logs (endpoint, ip, user_id, waktu, payload) VALUES ($1, $2, $3, $4, $5)", endpoint, ip, user, waktu, masked_body).execute(&pool).await;
        });
    }
    Ok(next.run(req).await)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Sentry
    let _sentry = sentry::init((
        std::env::var("SENTRY_DSN").unwrap_or_default().into_dsn().ok(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    // Integrasi tracing dengan Sentry
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish()
        .with(sentry_tracing_layer());
    tracing::subscriber::set_global_default(subscriber)?;

    // Prometheus exporter
    let builder = PrometheusBuilder::new();
    let recorder = builder.install_recorder()?;
    let prometheus_handle = recorder.handle();

    info!("🤖 Starting SIMPelv2 AI Service...");

    // Load configuration
    let config = config::Config::load()?;
    info!("📋 Configuration loaded successfully");

    // Initialize database connection
    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    info!("🗄️ Database connection established");

    // Initialize AI models
    let llm_service = llm::LlmService::new(&config).await?;
    let ocr_service = ocr::OcrService::new(&config).await?;
    let rag_service = rag::RagService::new(&config).await?;
    
    info!("🧠 AI models initialized successfully");

    // Initialize job queue
    let job_queue = std::sync::Arc::new(std::sync::Mutex::new(crate::models::JobQueue::new()));

    // Build application state
    let state = models::AppState {
        pool,
        config,
        llm_service,
        ocr_service,
        rag_service,
        job_queue: job_queue.clone(),
    };

    // Background worker untuk job queue (DB persistence) dengan graceful shutdown
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let mut shutdown = signal::ctrl_c();
        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    tracing::info!("Worker received shutdown signal, exiting gracefully.");
                    break;
                }
                _ = async {
                    // Ambil 1 job status 'queued'
                    let rec = sqlx::query!("SELECT id, job_type, payload FROM ai_jobs WHERE status = 'queued' ORDER BY created_at ASC LIMIT 1")
                        .fetch_optional(&pool_clone)
                        .await;
                    if let Ok(Some(job)) = rec {
                        let job_id = job.id;
                        tracing::info!("Processing job from DB: {:?}", job_id);
                        // Simulasi proses
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        let _ = sqlx::query!("UPDATE ai_jobs SET status = 'done', updated_at = NOW() WHERE id = $1", job_id)
                            .execute(&pool_clone)
                            .await;
                    } else {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                } => {}
            }
        }
    });

    // CORS setup
    let cors_origins = std::env::var("CORS_ORIGINS").unwrap_or_else(|_| "*".to_string());
    let allowed_origins = if cors_origins == "*" {
        Any
    } else {
        let origins: Vec<_> = cors_origins.split(',').map(|s| s.trim().parse().unwrap()).collect();
        origins
    };
    let cors = CorsLayer::new().allow_origin(allowed_origins).allow_methods(Any).allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/ai/llm/generate", post(handlers::generate_text))
        .route("/ai/llm/summarize", post(handlers::summarize_text))
        .route("/ai/ocr/extract", post(handlers::extract_text))
        .route("/ai/rag/query", post(handlers::rag_query))
        .route("/ai/rag/ingest", post(handlers::rag_ingest))
        .route("/ai/classify", post(handlers::classify_document))
        .route("/ai/recommend", post(handlers::generate_recommendation))
        .route("/ai/supervised/train", post(handlers::train_model))
        .route("/ai/supervised/predict", post(handlers::predict))
        .route("/ai/rlhf/train", post(handlers::rlhf_train))
        .route("/ai/active/query", post(handlers::active_learning_query))
        .route("/ai/transfer/fine_tune", post(handlers::transfer_learning))
        .route("/ai/hitl/annotate", post(handlers::hitl_annotate))
        .route("/ai/model/upload", post(handlers::upload_model))
        .route("/ai/model/download/:id", get(handlers::download_model))
        .route("/ai/model/register", post(handlers::register_model))
        .route("/ai/model/:id", get(handlers::get_model))
        .route("/ai/model/approve/:id", post(handlers::approve_model))
        .route("/ai/model/rollback/:id", post(handlers::rollback_model))
        .route("/ai/job/enqueue", post(handlers::enqueue_job))
        .route("/ai/job/status/:id", get(handlers::job_status))
        .route("/ai/job/cancel/:id", post(handlers::cancel_job))
        .route("/ai/job/retry/:id", post(handlers::retry_job))
        .route("/ai/explain", post(handlers::explain))
        .with_state(state)
        .layer(cors)
        .layer(from_fn(audit_log_middleware))
        .layer(from_fn(rate_limit_middleware))
        .layer(from_fn(auth_middleware))
        .layer(from_fn(request_id_middleware))
        .route("/metrics", get(|| async move {
            let metrics = prometheus_handle.render();
            ([("Content-Type", "text/plain; version=0.0.4")], metrics)
        }))
        .route("/ai/admin/only", get(|| async { "admin only" }).layer(from_fn_with_state(state.clone(), |req, next| rbac_middleware(req, next, "admin"))))
        .route("/ai/model/monitor", get(handlers::model_monitor));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3002));
    info!("🌐 AI Service listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(signal::ctrl_c())
        .await?;

    Ok(())
} 