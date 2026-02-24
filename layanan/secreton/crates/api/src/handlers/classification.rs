//! Classification API Handlers
//!
//! REST API endpoints for data classification management.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, instrument};

use secreton_core::services::classification::{ClassificationLevel, ClassificationService};

/// Shared application state
#[derive(Clone)]
pub struct ClassificationState {
    pub classification_service: Arc<dyn ClassificationService>,
}

/// Request to classify a secret
#[derive(Debug, Deserialize, Serialize)]
pub struct ClassifyRequest {
    /// Classification level
    pub level: ClassificationLevel,
}

/// Response for classification operation
#[derive(Debug, Serialize)]
pub struct ClassifyResponse {
    pub success: bool,
    pub message: String,
}

/// Classify a secret
///
/// POST /v1/secret/classify/{path}
#[instrument(skip(state))]
pub async fn classify_secret(
    State(state): State<ClassificationState>,
    Path(path): Path<String>,
    Json(request): Json<ClassifyRequest>,
) -> Response {
    info!(path = %path, level = ?request.level, "Classifying secret");

    // TODO: Extract user_id from authentication context
    let user_id = "system";

    match state
        .classification_service
        .classify(&path, request.level, user_id)
        .await
    {
        Ok(_) => {
            let response = ClassifyResponse {
                success: true,
                message: format!("Secret classified as {}", request.level),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to classify secret");
            let response = ClassifyResponse {
                success: false,
                message: format!("Classification failed: {}", e),
            };
            (StatusCode::BAD_REQUEST, Json(response)).into_response()
        }
    }
}

/// Get classification report
///
/// GET /v1/classification/report
#[instrument(skip(state))]
pub async fn get_classification_report(State(state): State<ClassificationState>) -> Response {
    info!("Generating classification report");

    match state.classification_service.generate_report().await {
        Ok(report) => {
            info!(
                total_secrets = report.total_secrets,
                total_accesses = report.total_accesses,
                "Classification report generated"
            );
            (StatusCode::OK, Json(report)).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to generate classification report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to generate report: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Get classification for a secret
///
/// GET /v1/secret/classification/{path}
#[instrument(skip(state))]
pub async fn get_secret_classification(
    State(state): State<ClassificationState>,
    Path(path): Path<String>,
) -> Response {
    info!(path = %path, "Getting secret classification");

    match state.classification_service.get_classification(&path).await {
        Ok(Some(metadata)) => (StatusCode::OK, Json(metadata)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Secret not classified"
            })),
        )
            .into_response(),
        Err(e) => {
            error!(error = %e, "Failed to get classification");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get classification: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Check if MFA is required for a secret
///
/// GET /v1/secret/require-mfa/{path}
#[instrument(skip(state))]
pub async fn check_mfa_required(
    State(state): State<ClassificationState>,
    Path(path): Path<String>,
) -> Response {
    info!(path = %path, "Checking MFA requirement");

    match state.classification_service.require_mfa(&path).await {
        Ok(required) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "path": path,
                "mfa_required": required
            })),
        )
            .into_response(),
        Err(e) => {
            error!(error = %e, "Failed to check MFA requirement");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to check MFA requirement: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Get policy violations
///
/// GET /v1/classification/violations?limit=100
#[instrument(skip(state))]
pub async fn get_policy_violations(
    State(state): State<ClassificationState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);

    info!(limit = limit, "Getting policy violations");

    match state.classification_service.get_violations(limit).await {
        Ok(violations) => (StatusCode::OK, Json(violations)).into_response(),
        Err(e) => {
            error!(error = %e, "Failed to get violations");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get violations: {}", e)
                })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use secreton_core::services::classification::InMemoryClassificationService;
    use tower::ServiceExt;

    fn create_test_app() -> Router {
        let service = Arc::new(InMemoryClassificationService::new());
        let state = ClassificationState {
            classification_service: service,
        };

        Router::new()
            .route(
                "/v1/secret/classify/{*path}",
                axum::routing::post(classify_secret),
            )
            .route(
                "/v1/classification/report",
                axum::routing::get(get_classification_report),
            )
            .route(
                "/v1/secret/classification/{*path}",
                axum::routing::get(get_secret_classification),
            )
            .route(
                "/v1/secret/require-mfa/{*path}",
                axum::routing::get(check_mfa_required),
            )
            .route(
                "/v1/classification/violations",
                axum::routing::get(get_policy_violations),
            )
            .with_state(state)
    }

    #[tokio::test]
    async fn test_classify_secret() {
        let app = create_test_app();

        let request = Request::builder()
            .method("POST")
            .uri("/v1/secret/classify/test/secret")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"level":"RAHASIA"}"#))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_classification_report() {
        let app = create_test_app();

        let request = Request::builder()
            .method("GET")
            .uri("/v1/classification/report")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_check_mfa_required() {
        let app = create_test_app();

        let request = Request::builder()
            .method("GET")
            .uri("/v1/secret/require-mfa/test/secret")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
