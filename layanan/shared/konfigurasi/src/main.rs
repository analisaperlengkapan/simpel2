use axum::{
    extract::State,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::{info, Level};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub log_level: String,
    pub cors_origins: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_host: "0.0.0.0".to_string(),
            server_port: 8765,
            log_level: "info".to_string(),
            cors_origins: vec!["*".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub category: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub entries: HashMap<String, ConfigEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConfigRequest {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "konfigurasi",
        "timestamp": chrono::Utc::now()
    }))
}

async fn get_configs(State(state): State<AppState>) -> Json<Vec<ConfigEntry>> {
    let configs: Vec<ConfigEntry> = state.entries.values().cloned().collect();
    Json(configs)
}

async fn get_config_by_key(
    axum::extract::Path(key): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ConfigEntry>, axum::http::StatusCode> {
    match state.entries.get(&key) {
        Some(config) => Ok(Json(config.clone())),
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

async fn create_config(
    State(mut state): State<AppState>,
    Json(payload): Json<CreateConfigRequest>,
) -> Result<Json<ConfigEntry>, axum::http::StatusCode> {
    let now = chrono::Utc::now();
    let config = ConfigEntry {
        id: Uuid::new_v4(),
        key: payload.key.clone(),
        value: payload.value,
        description: payload.description,
        category: payload.category,
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    state.entries.insert(payload.key, config.clone());
    Ok(Json(config))
}

async fn update_config(
    axum::extract::Path(key): axum::extract::Path<String>,
    State(mut state): State<AppState>,
    Json(payload): Json<UpdateConfigRequest>,
) -> Result<Json<ConfigEntry>, axum::http::StatusCode> {
    match state.entries.get_mut(&key) {
        Some(config) => {
            if let Some(value) = payload.value {
                config.value = value;
            }
            if let Some(description) = payload.description {
                config.description = Some(description);
            }
            if let Some(is_active) = payload.is_active {
                config.is_active = is_active;
            }
            config.updated_at = chrono::Utc::now();
            Ok(Json(config.clone()))
        }
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

async fn delete_config(
    axum::extract::Path(key): axum::extract::Path<String>,
    State(mut state): State<AppState>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    match state.entries.remove(&key) {
        Some(_) => Ok(Json(
            serde_json::json!({"message": "Config deleted successfully"}),
        )),
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

async fn service_info() -> Html<&'static str> {
    Html(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>SIMPelv2 - Configuration Service</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 40px; }
            .header { color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 10px; }
            .info { background: #ecf0f1; padding: 15px; border-radius: 5px; margin: 20px 0; }
            .endpoint { background: #ffffff; border-left: 4px solid #3498db; padding: 10px; margin: 10px 0; }
        </style>
    </head>
    <body>
        <h1 class="header">SIMPelv2 Configuration Service</h1>
        <div class="info">
            <h3>Service Information</h3>
            <p><strong>Service:</strong> Configuration Management</p>
            <p><strong>Version:</strong> 1.0.0</p>
            <p><strong>Status:</strong> Running</p>
        </div>
        <div class="info">
            <h3>Available Endpoints</h3>
            <div class="endpoint"><strong>GET /health</strong> - Health check</div>
            <div class="endpoint"><strong>GET /configs</strong> - Get all configurations</div>
            <div class="endpoint"><strong>GET /configs/{key}</strong> - Get specific configuration</div>
            <div class="endpoint"><strong>POST /configs</strong> - Create new configuration</div>
            <div class="endpoint"><strong>PUT /configs/{key}</strong> - Update configuration</div>
            <div class="endpoint"><strong>DELETE /configs/{key}</strong> - Delete configuration</div>
        </div>
    </body>
    </html>
    "#,
    )
}

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(service_info))
        .route("/health", get(health_check))
        .route("/configs", get(get_configs).post(create_config))
        .route(
            "/configs/:key",
            get(get_config_by_key)
                .put(update_config)
                .delete(delete_config),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    let config = AppConfig::default();
    let state = AppState {
        config: config.clone(),
        entries: HashMap::new(),
    };

    let app = create_router(state);
    let addr = format!("{}:{}", config.server_host, config.server_port);

    info!("🚀 Configuration service starting on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
