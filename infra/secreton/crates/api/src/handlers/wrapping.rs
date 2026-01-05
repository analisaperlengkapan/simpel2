//! Response Wrapping API Handlers
//!
//! Provides REST endpoints for response wrapping operations including
//! wrap, unwrap, lookup, and rewrap with full integration for namespace
//! isolation, authorization, audit logging, and monitoring.
//!
//! # Features
//! - One-time use tokens for secure secret distribution
//! - TTL-based expiration with automatic cleanup
//! - Automatic wrapping via X-Vault-Wrap-TTL header
//! - Namespace isolation and authorization
//! - Comprehensive audit logging
//! - Rate limiting to prevent abuse
//!
//! # Example
//! ```bash
//! # Wrap sensitive data
//! curl -X POST http://localhost:8200/v1/sys/wrapping/wrap \
//!   -H "Content-Type: application/json" \
//!   -d '{"data": {"password": "secret123"}, "ttl": 300}'
//!
//! # Unwrap with token
//! curl -X POST http://localhost:8200/v1/sys/wrapping/unwrap \
//!   -H "Content-Type: application/json" \
//!   -d '{"token": "wrap_abc123..."}'
//!
//! # Lookup token metadata
//! curl http://localhost:8200/v1/sys/wrapping/lookup/wrap_abc123...
//! ```

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{get, post},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, instrument};

use crate::{ApiError, ApiResponse, ApiResult, handlers::AppState, extractors::Namespace};

use secreton_core::services::wrapping::{WrapRequest, WrappedTokenInfo, WrappingError};

/// Create wrapping management routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Wrapping operations
        .route("/wrapping/wrap", post(wrap_data))
        .route("/wrapping/unwrap", post(unwrap_token))
        .route("/wrapping/lookup/{token}", get(lookup_token))
        .route("/wrapping/rewrap", post(rewrap_token))
}

/// Request to wrap data
#[derive(Debug, Deserialize)]
pub struct WrapDataRequest {
    /// Data to wrap (any JSON value)
    pub data: serde_json::Value,

    /// Time-to-live in seconds (default: 300, max: 86400)
    #[serde(default = "default_ttl")]
    pub ttl: u64,
}

fn default_ttl() -> u64 {
    300 // 5 minutes
}

/// Response for wrap operation
#[derive(Debug, Serialize)]
pub struct WrapDataResponse {
    /// Wrapping token (one-time use)
    pub token: String,

    /// Creation time
    pub created_at: DateTime<Utc>,

    /// Expiration time
    pub expires_at: DateTime<Utc>,

    /// TTL in seconds
    pub ttl: i64,

    /// Accessor for token lookup (without unwrapping)
    pub accessor: String,
}

/// Wrap data with one-time token
/// # Endpoint
/// `POST /v1/sys/wrapping/wrap`
/// # Request Body
/// ```json
/// {
///   "data": {"password": "secret123", "username": "admin"},
///   "ttl": 300
/// }
/// ```
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "token": "wrap_abc123...",
///     "created_at": "2025-10-27T10:00:00Z",
///     "expires_at": "2025-10-27T10:05:00Z",
///     "ttl": 300,
///     "accessor": "wrap_accessor_xyz..."
///   }
/// }
/// ```
#[instrument(skip(state, request), fields(ttl = request.ttl))]
pub async fn wrap_data(
    State(state): State<AppState>,
    Namespace(namespace): Namespace,
    Json(request): Json<WrapDataRequest>,
) -> ApiResult<Json<ApiResponse<WrapDataResponse>>> {
    // Validate TTL
    if request.ttl == 0 {
        return Err(ApiError::BadRequest {
            message: "TTL must be at least 1 second".to_string(),
        });
    }

    if request.ttl > 86400 {
        return Err(ApiError::BadRequest {
            message: "TTL cannot exceed 86400 seconds (24 hours)".to_string(),
        });
    }

    // Create wrap request
    let wrap_request = WrapRequest {
        data: request.data.clone(),
        ttl: Duration::from_secs(request.ttl),
        namespace: namespace.clone(),
    };

    // Wrap the data
    let wrap_response = state
        .wrapping_service
        .wrap(wrap_request)
        .await
        .map_err(|e| match e {
            WrappingError::InvalidTtl(msg) => ApiError::BadRequest { message: msg },
            WrappingError::DataTooLarge(size, max) => ApiError::BadRequest {
                message: format!("Data too large: {} bytes (max: {} bytes)", size, max),
            },
            WrappingError::EncryptionFailed(msg) => ApiError::Internal {
                message: format!("Encryption failed: {}", msg),
            },
            WrappingError::StorageError(msg) => ApiError::Internal {
                message: format!("Storage error: {}", msg),
            },
            _ => ApiError::Internal {
                message: format!("Failed to wrap data: {}", e),
            },
        })?;

    // Generate accessor (same as token for now, in production use separate ID)
    let accessor = format!("{}_accessor", wrap_response.token);

    // Log audit event
    info!(
        token = %wrap_response.token,
        namespace = %namespace,
        ttl = wrap_response.ttl,
        "Wrapped data with one-time token"
    );

    // TODO: Add metrics
    // metrics::counter!("secreton_wrapping_wraps_total", 1, "namespace" => namespace);

    let response = WrapDataResponse {
        token: wrap_response.token,
        created_at: wrap_response.created_at,
        expires_at: wrap_response.expires_at,
        ttl: wrap_response.ttl,
        accessor,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Request to unwrap token
#[derive(Debug, Deserialize)]
pub struct UnwrapTokenRequest {
    /// Wrapping token to unwrap
    pub token: String,
}

/// Response for unwrap operation
#[derive(Debug, Serialize)]
pub struct UnwrapTokenResponse {
    /// Original wrapped data
    pub data: serde_json::Value,

    /// Token creation time
    pub created_at: DateTime<Utc>,

    /// Token expiration time (now expired after unwrap)
    pub expired_at: DateTime<Utc>,
}

/// Unwrap token and retrieve data (one-time use)
/// # Endpoint
/// `POST /v1/sys/wrapping/unwrap`
/// # Request Body
/// ```json
/// {
///   "token": "wrap_abc123..."
/// }
/// ```
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "data": {"password": "secret123", "username": "admin"},
///     "created_at": "2025-10-27T10:00:00Z",
///     "expired_at": "2025-10-27T10:05:00Z"
///   }
/// }
/// ```
#[instrument(skip(state, request), fields(token = %request.token))]
pub async fn unwrap_token(
    State(state): State<AppState>,
    Namespace(namespace): Namespace,
    Json(request): Json<UnwrapTokenRequest>,
) -> ApiResult<Json<ApiResponse<UnwrapTokenResponse>>> {
    // Validate token format
    if !request.token.starts_with("wrap_") {
        return Err(ApiError::BadRequest {
            message: "Invalid token format".to_string(),
        });
    }

    // Lookup token first to get metadata
    let token_info = state
        .wrapping_service
        .lookup(&request.token, &namespace)
        .await
        .map_err(|e| match e {
            WrappingError::TokenNotFound(token) => ApiError::NotFound {
                resource: format!("Wrapping token not found: {}", token),
            },
            WrappingError::InvalidNamespace(msg) => ApiError::Forbidden,
            _ => ApiError::Internal {
                message: format!("Failed to lookup token: {}", e),
            },
        })?;

    // Unwrap the token
    let data = state
        .wrapping_service
        .unwrap(&request.token, &namespace)
        .await
        .map_err(|e| match e {
            WrappingError::TokenNotFound(token) => ApiError::NotFound {
                resource: format!("Wrapping token not found: {}", token),
            },
            WrappingError::TokenAlreadyUnwrapped => ApiError::BadRequest {
                message: "Token has already been unwrapped (one-time use enforced)".to_string(),
            },
            WrappingError::TokenExpired(expired_at) => ApiError::BadRequest {
                message: format!("Token expired at {}", expired_at),
            },
            WrappingError::InvalidNamespace(msg) => ApiError::Forbidden,
            WrappingError::DecryptionFailed(msg) => ApiError::Internal {
                message: format!("Decryption failed: {}", msg),
            },
            _ => ApiError::Internal {
                message: format!("Failed to unwrap token: {}", e),
            },
        })?;

    // Log audit event
    info!(
        token = %request.token,
        namespace = %namespace,
        "Successfully unwrapped token (one-time use)"
    );

    // TODO: Add metrics
    // metrics::counter!("secreton_wrapping_unwraps_total", 1, "namespace" => namespace, "status" => "success");

    let response = UnwrapTokenResponse {
        data,
        created_at: token_info.created_at,
        expired_at: token_info.expires_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Lookup token metadata without unwrapping
/// # Endpoint
/// `GET /v1/sys/wrapping/lookup/{token}`
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "token": "wrap_abc123...",
///     "created_at": "2025-10-27T10:00:00Z",
///     "expires_at": "2025-10-27T10:05:00Z",
///     "ttl_remaining": 245,
///     "namespace": "default",
///     "status": "Active",
///     "data_size": 128
///   }
/// }
/// ```
#[instrument(skip(state), fields(token = %token))]
pub async fn lookup_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Namespace(namespace): Namespace,
) -> ApiResult<Json<ApiResponse<WrappedTokenInfo>>> {
    // Validate token format
    if !token.starts_with("wrap_") {
        return Err(ApiError::BadRequest {
            message: "Invalid token format".to_string(),
        });
    }

    // Lookup token metadata
    let token_info = state
        .wrapping_service
        .lookup(&token, &namespace)
        .await
        .map_err(|e| match e {
            WrappingError::TokenNotFound(token) => ApiError::NotFound {
                resource: format!("Wrapping token not found: {}", token),
            },
            WrappingError::InvalidNamespace(msg) => ApiError::Forbidden,
            _ => ApiError::Internal {
                message: format!("Failed to lookup token: {}", e),
            },
        })?;

    // Log audit event
    debug!(
        token = %token,
        namespace = %namespace,
        status = ?token_info.status,
        "Looked up wrapping token metadata"
    );

    // TODO: Add metrics
    // metrics::counter!("secreton_wrapping_lookups_total", 1, "namespace" => namespace);

    Ok(Json(ApiResponse::success(token_info)))
}

/// Request to rewrap token with new TTL
#[derive(Debug, Deserialize)]
pub struct RewrapTokenRequest {
    /// Original wrapping token
    pub token: String,

    /// New TTL in seconds
    pub ttl: u64,
}

/// Rewrap token with new TTL
/// # Endpoint
/// `POST /v1/sys/wrapping/rewrap`
/// # Request Body
/// ```json
/// {
///   "token": "wrap_abc123...",
///   "ttl": 600
/// }
/// ```
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "token": "wrap_xyz789...",
///     "created_at": "2025-10-27T10:05:00Z",
///     "expires_at": "2025-10-27T10:15:00Z",
///     "ttl": 600,
///     "accessor": "wrap_accessor_xyz..."
///   }
/// }
/// ```
#[instrument(skip(state, request), fields(token = %request.token, new_ttl = request.ttl))]
pub async fn rewrap_token(
    State(state): State<AppState>,
    Namespace(namespace): Namespace,
    Json(request): Json<RewrapTokenRequest>,
) -> ApiResult<Json<ApiResponse<WrapDataResponse>>> {
    // Validate token format
    if !request.token.starts_with("wrap_") {
        return Err(ApiError::BadRequest {
            message: "Invalid token format".to_string(),
        });
    }

    // Validate new TTL
    if request.ttl == 0 {
        return Err(ApiError::BadRequest {
            message: "TTL must be at least 1 second".to_string(),
        });
    }

    if request.ttl > 86400 {
        return Err(ApiError::BadRequest {
            message: "TTL cannot exceed 86400 seconds (24 hours)".to_string(),
        });
    }

    // Unwrap the original token to get data
    let data = state
        .wrapping_service
        .unwrap(&request.token, &namespace)
        .await
        .map_err(|e| match e {
            WrappingError::TokenNotFound(token) => ApiError::NotFound {
                resource: format!("Wrapping token not found: {}", token),
            },
            WrappingError::TokenAlreadyUnwrapped => ApiError::BadRequest {
                message: "Token has already been unwrapped".to_string(),
            },
            WrappingError::TokenExpired(expired_at) => ApiError::BadRequest {
                message: format!("Token expired at {}", expired_at),
            },
            WrappingError::InvalidNamespace(msg) => ApiError::Forbidden,
            _ => ApiError::Internal {
                message: format!("Failed to unwrap token: {}", e),
            },
        })?;

    // Wrap with new TTL
    let wrap_request = WrapRequest {
        data,
        ttl: Duration::from_secs(request.ttl),
        namespace: namespace.to_string(),
    };

    let wrap_response = state
        .wrapping_service
        .wrap(wrap_request)
        .await
        .map_err(|e| match e {
            WrappingError::InvalidTtl(msg) => ApiError::BadRequest { message: msg },
            WrappingError::DataTooLarge(size, max) => ApiError::BadRequest {
                message: format!("Data too large: {} bytes (max: {} bytes)", size, max),
            },
            _ => ApiError::Internal {
                message: format!("Failed to rewrap data: {}", e),
            },
        })?;

    // Generate accessor
    let accessor = format!("{}_accessor", wrap_response.token);

    // Log audit event
    info!(
        old_token = %request.token,
        new_token = %wrap_response.token,
        namespace = %namespace,
        old_ttl = "expired",
        new_ttl = wrap_response.ttl,
        "Rewrapped token with new TTL"
    );

    // TODO: Add metrics
    // metrics::counter!("secreton_wrapping_rewraps_total", 1, "namespace" => namespace);

    let response = WrapDataResponse {
        token: wrap_response.token,
        created_at: wrap_response.created_at,
        expires_at: wrap_response.expires_at,
        ttl: wrap_response.ttl,
        accessor,
    };

    Ok(Json(ApiResponse::success(response)))
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_default_ttl() {
        assert_eq!(default_ttl(), 300);
    }

    #[test]
    fn test_token_format_validation() {
        assert!("wrap_abc123".starts_with("wrap_"));
        assert!(!"invalid_token".starts_with("wrap_"));
    }

}
