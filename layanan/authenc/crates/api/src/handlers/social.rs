//! Social login handlers
//!
//! Handles social authentication via OAuth2 providers (Google, GitHub, etc.)

use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// Social login initiation query
#[derive(Debug, Deserialize)]
pub struct SocialLoginQuery {
    pub provider: String,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
}

/// Social login callback query
#[derive(Debug, Deserialize)]
pub struct SocialCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

/// Social provider information
#[derive(Debug, Serialize)]
pub struct SocialProviderInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub icon_url: Option<String>,
}

// ===== Routes =====

/// Create social login routes
pub fn create_social_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/social/providers", get(list_social_providers))
        .route("/social/login", get(social_login))
        .route("/social/callback", get(social_callback))
}

// ===== Handlers =====

/// List available social login providers
async fn list_social_providers(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    let providers: Vec<SocialProviderInfo> = vec![];
    Json(providers)
}

/// Initiate social login
async fn social_login(
    State(_state): State<Arc<ApiState>>,
    Query(_params): Query<SocialLoginQuery>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "not_implemented",
        "message": "Social login (stub)"
    }))
}

/// Handle social login callback
async fn social_callback(
    State(_state): State<Arc<ApiState>>,
    Query(_params): Query<SocialCallbackQuery>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "not_implemented",
        "message": "Social callback (stub)"
    }))
}
