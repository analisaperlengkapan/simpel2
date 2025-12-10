//! TOTP API handlers
//!
//! REST API endpoints for TOTP secrets engine operations.

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::secrets::totp::{
    TotpCodeResponse, TotpKeyCreateRequest, TotpKeyResponse, TotpValidationRequest,
    TotpValidationResponse,
};

use super::AppState;

/// Create TOTP routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/keys", post(create_totp_key))
        .route("/keys", get(list_totp_keys))
        .route("/keys/:key_name", get(get_totp_key))
        .route("/keys/:key_name", delete(delete_totp_key))
        .route("/code/:key_name", post(generate_totp_code))
        .route("/validate/:key_name", post(validate_totp_code))
}

/// Create TOTP key
/// # Endpoint
/// `POST /v1/totp/keys`
/// # Request Body
/// ```json
/// {
///   "name": "user@example.com",
///   "issuer": "SIMKARI",
///   "account_name": "user@example.com",
///   "algorithm": "SHA1",
///   "digits": 6,
///   "period": 30,
///   "skew": 1
/// }
/// ```
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "name": "user@example.com",
///     "issuer": "SIMKARI",
///     "account_name": "user@example.com",
///     "secret": "JBSWY3DPEHPK3PXP",
///     "algorithm": "SHA1",
///     "digits": 6,
///     "period": 30,
///     "qr_code_url": "otpauth://totp/...",
///     "created_at": "2025-11-11T06:13:00Z",
///     "last_validated_at": null
///   }
/// }
/// ```
#[tracing::instrument(skip(state))]
async fn create_totp_key(
    State(state): State<AppState>,
    Json(request): Json<TotpKeyCreateRequest>,
) -> ApiResult<Json<ApiResponse<TotpKeyResponse>>> {
    info!("Creating TOTP key: {}", request.name);

    match state.totp_engine.create_key(request).await {
        Ok(response) => {
            info!("TOTP key created successfully");
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to create TOTP key: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create TOTP key: {}",
                e
            )))
        }
    }
}

/// Get TOTP key
/// # Endpoint
/// `GET /v1/totp/keys/:key_name`
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "name": "user@example.com",
///     "issuer": "SIMKARI",
///     "account_name": "user@example.com",
///     "secret": "JBSWY3DPEHPK3PXP",
///     "algorithm": "SHA1",
///     "digits": 6,
///     "period": 30,
///     "qr_code_url": "otpauth://totp/...",
///     "created_at": "2025-11-11T06:13:00Z",
///     "last_validated_at": null
///   }
/// }
/// ```
#[tracing::instrument(skip(state))]
async fn get_totp_key(
    State(state): State<AppState>,
    Path(key_name): Path<String>,
) -> ApiResult<Json<ApiResponse<TotpKeyResponse>>> {
    info!("Getting TOTP key: {}", key_name);

    match state.totp_engine.get_key(&key_name).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            error!("Failed to get TOTP key: {:?}", e);
            Err(ApiError::not_found(format!("TOTP key not found: {}", e)))
        }
    }
}

/// Delete TOTP key
/// # Endpoint
/// `DELETE /v1/totp/keys/:key_name`
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": null
/// }
/// ```
#[tracing::instrument(skip(state))]
async fn delete_totp_key(
    State(state): State<AppState>,
    Path(key_name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting TOTP key: {}", key_name);

    match state.totp_engine.delete_key(&key_name).await {
        Ok(_) => {
            info!("TOTP key deleted successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to delete TOTP key: {:?}", e);
            Err(ApiError::not_found(format!(
                "Failed to delete TOTP key: {}",
                e
            )))
        }
    }
}

/// List TOTP keys
/// # Endpoint
/// `GET /v1/totp/keys`
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "keys": ["user1@example.com", "user2@example.com"]
///   }
/// }
/// ```
#[tracing::instrument(skip(state))]
async fn list_totp_keys(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<TotpKeyListResponse>>> {
    info!("Listing TOTP keys");

    let keys = state.totp_engine.list_keys().await;
    Ok(Json(ApiResponse::success(TotpKeyListResponse { keys })))
}

/// Generate TOTP code
/// # Endpoint
/// `POST /v1/totp/code/:key_name`
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "code": "123456",
///     "expires_at": "2025-11-11T06:13:30Z"
///   }
/// }
/// ```
#[tracing::instrument(skip(state))]
async fn generate_totp_code(
    State(state): State<AppState>,
    Path(key_name): Path<String>,
) -> ApiResult<Json<ApiResponse<TotpCodeResponse>>> {
    info!("Generating TOTP code for key: {}", key_name);

    match state.totp_engine.generate_code(&key_name).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            error!("Failed to generate TOTP code: {:?}", e);
            Err(ApiError::not_found(format!(
                "Failed to generate TOTP code: {}",
                e
            )))
        }
    }
}

/// Validate TOTP code
/// # Endpoint
/// `POST /v1/totp/validate/:key_name`
/// # Request Body
/// ```json
/// {
///   "key_name": "user@example.com",
///   "code": "123456",
///   "skew": 1
/// }
/// ```
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "valid": true,
///     "timestamp": "2025-11-11T06:13:00Z"
///   }
/// }
/// ```
#[tracing::instrument(skip(state, request), fields(key_name = %key_name))]
async fn validate_totp_code(
    State(state): State<AppState>,
    Path(key_name): Path<String>,
    Json(mut request): Json<TotpValidationRequest>,
) -> ApiResult<Json<ApiResponse<TotpValidationResponse>>> {
    info!("Validating TOTP code for key: {}", key_name);

    // Override key_name from path parameter
    request.key_name = key_name;

    match state.totp_engine.validate_code(request).await {
        Ok(response) => {
            if response.valid {
                info!("TOTP code validation successful");
            } else {
                info!("TOTP code validation failed");
            }
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("TOTP code validation error: {:?}", e);
            Err(ApiError::bad_request(format!(
                "TOTP validation failed: {}",
                e
            )))
        }
    }
}

/// TOTP key list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpKeyListResponse {
    pub keys: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiConfig;
    use crate::services::ServiceContainer;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_totp_key() {
        let config = ApiConfig::default();
        let services = ServiceContainer::new(&config).await.unwrap();
        let app = create_routes().with_state(services.into());

        let request_body = serde_json::json!({
            "name": "test_user",
            "issuer": "SIMKARI",
            "account_name": "test@example.com"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/keys")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_totp_keys() {
        let config = ApiConfig::default();
        let services = ServiceContainer::new(&config).await.unwrap();
        let app = create_routes().with_state(services.into());

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/keys")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
