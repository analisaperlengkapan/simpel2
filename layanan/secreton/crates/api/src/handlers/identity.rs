//! Identity Secrets Engine API handlers
//!
//! Provides OIDC Provider endpoints and entity/group management

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::secrets::identity::{
    OidcDiscovery, OidcProviderConfig, TokenIntrospection, TokenRequest, TokenResponse, UserInfo,
};

use super::AppState;

/// Create Identity/OIDC routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // OIDC Provider endpoints
        .route("/.well-known/openid-configuration", get(oidc_discovery))
        .route("/.well-known/jwks.json", get(oidc_jwks))
        .route("/oidc/token", post(oidc_token))
        .route("/oidc/userinfo", get(oidc_userinfo))
        .route("/oidc/introspect", post(oidc_introspect))
        // Configuration
        .route("/config", post(configure_oidc))
        .route("/config", get(get_oidc_config))
        // Entity management (delegated to existing IdentityService)
        .route("/entity", post(create_entity))
        .route("/entity/:id", get(get_entity))
        .route("/entity/:id", delete(delete_entity))
        .route("/entity", get(list_entities))
}

/// OIDC Discovery endpoint
#[tracing::instrument(skip(state))]
async fn oidc_discovery(State(state): State<AppState>) -> ApiResult<Json<OidcDiscovery>> {
    info!("OIDC discovery request");

    let discovery = state.identity_engine.get_discovery().await;
    Ok(Json(discovery))
}

/// JWKS endpoint
#[tracing::instrument(skip(state))]
async fn oidc_jwks(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    info!("JWKS request");

    let jwks = state.identity_engine.get_jwks().await;
    Ok(Json(serde_json::to_value(jwks).unwrap()))
}

/// Token endpoint
#[tracing::instrument(skip(state, request))]
async fn oidc_token(
    State(state): State<AppState>,
    Json(request): Json<TokenRequest>,
) -> ApiResult<Json<ApiResponse<TokenResponse>>> {
    info!("Token request for grant_type: {}", request.grant_type);

    match state.identity_engine.generate_token(request).await {
        Ok(response) => {
            info!("Token generated successfully");
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to generate token: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Token generation failed: {}",
                e
            )))
        }
    }
}

/// UserInfo endpoint
#[tracing::instrument(skip(state))]
async fn oidc_userinfo(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> ApiResult<Json<ApiResponse<UserInfo>>> {
    info!("UserInfo request");

    // Extract bearer token from Authorization header
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::authentication("Missing or invalid Authorization header"))?;

    match state.identity_engine.get_userinfo(token).await {
        Ok(userinfo) => Ok(Json(ApiResponse::success(userinfo))),
        Err(e) => {
            error!("Failed to get userinfo: {:?}", e);
            Err(ApiError::authentication(format!("UserInfo failed: {}", e)))
        }
    }
}

/// Token introspection endpoint
#[tracing::instrument(skip(state))]
async fn oidc_introspect(
    State(state): State<AppState>,
    Json(request): Json<IntrospectRequest>,
) -> ApiResult<Json<TokenIntrospection>> {
    info!("Token introspection request");

    let result = state.identity_engine.introspect_token(&request.token).await;
    Ok(Json(result))
}

/// Configure OIDC provider
#[tracing::instrument(skip(state, request))]
async fn configure_oidc(
    State(state): State<AppState>,
    Json(request): Json<OidcProviderConfig>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Configuring OIDC provider");

    match state.identity_engine.configure(request).await {
        Ok(_) => {
            info!("OIDC provider configured successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to configure OIDC: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Configuration failed: {}",
                e
            )))
        }
    }
}

/// Get OIDC configuration
#[tracing::instrument(skip(state))]
async fn get_oidc_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<OidcConfigResponse>>> {
    info!("Getting OIDC configuration");

    // Return non-sensitive config info
    let discovery = state.identity_engine.get_discovery().await;

    let response = OidcConfigResponse {
        issuer: discovery.issuer,
        scopes_supported: discovery.scopes_supported,
        response_types_supported: discovery.response_types_supported,
        grant_types_supported: discovery.grant_types_supported,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Create entity (delegates to IdentityService)
#[tracing::instrument(skip(state))]
async fn create_entity(
    State(state): State<AppState>,
    Json(request): Json<CreateEntityRequest>,
) -> ApiResult<Json<ApiResponse<EntityResponse>>> {
    info!("Creating entity: {}", request.name);

    let entity = state.identity_service.create_entity(request.name).await;

    let response = EntityResponse {
        id: entity.id,
        name: entity.name,
        policies: entity.policies,
        metadata: entity
            .metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect(),
        created_at: entity.created_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get entity
#[tracing::instrument(skip(state))]
async fn get_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ApiResponse<EntityResponse>>> {
    info!("Getting entity: {}", id);

    match state.identity_service.get_entity(&id).await {
        Ok(entity) => {
            let response = EntityResponse {
                id: entity.id,
                name: entity.name,
                policies: entity.policies,
                metadata: entity
                    .metadata
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .collect(),
                created_at: entity.created_at,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get entity: {:?}", e);
            Err(ApiError::not_found(format!("Entity not found: {}", e)))
        }
    }
}

/// Delete entity
#[tracing::instrument(skip(state))]
async fn delete_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting entity: {}", id);

    match state.identity_service.delete_entity(&id).await {
        Ok(_) => {
            info!("Entity deleted successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to delete entity: {:?}", e);
            Err(ApiError::not_found(format!(
                "Failed to delete entity: {}",
                e
            )))
        }
    }
}

/// List entities
#[tracing::instrument(skip(state))]
async fn list_entities(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<EntityResponse>>>> {
    info!("Listing entities");

    let entities = state.identity_service.list_entities().await;

    let response: Vec<EntityResponse> = entities
        .into_iter()
        .map(|entity| EntityResponse {
            id: entity.id,
            name: entity.name,
            policies: entity.policies,
            metadata: entity
                .metadata
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect(),
            created_at: entity.created_at,
        })
        .collect();

    Ok(Json(ApiResponse::success(response)))
}

/// Request/Response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfigResponse {
    pub issuer: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResponse {
    pub id: String,
    pub name: String,
    pub policies: Vec<String>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
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
    #[ignore = "Requires database/infrastructure"]
    async fn test_oidc_discovery() {
        let config = ApiConfig::default();
        let services = ServiceContainer::new(&config).await.unwrap();
        let app = create_routes().with_state(services.into());

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/.well-known/openid-configuration")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
