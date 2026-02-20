//! Federation/Identity Provider management HTTP handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::IamApiState;
use authenc_types::AuthencError;
use crate::error::ApiResult;

/// Identity provider response DTO
#[derive(Debug, Serialize)]
pub struct IdentityProviderResponse {
    pub id: Uuid,
    pub name: String,
    pub provider_type: String, // "oidc", "saml", "ldap"
    pub enabled: bool,
    pub config: JsonValue,
    pub realm_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create identity provider request
#[derive(Debug, Deserialize)]
pub struct CreateIdentityProviderRequest {
    pub name: String,
    pub provider_type: String,
    pub config: JsonValue,
    pub realm_id: Uuid,
}

/// Update identity provider request
#[derive(Debug, Deserialize)]
pub struct UpdateIdentityProviderRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub config: Option<JsonValue>,
}

/// GET /api/v1/iam/identity-providers - List identity providers
pub async fn list_identity_providers(
    State(_state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<IdentityProviderResponse>>> {
    // TODO: Call federation_service.list_providers() when implemented
    Ok(Json(vec![]))
}

/// POST /api/v1/iam/identity-providers - Create identity provider
pub async fn create_identity_provider(
    State(_state): State<Arc<IamApiState>>,
    Json(_req): Json<CreateIdentityProviderRequest>,
) -> ApiResult<(StatusCode, Json<IdentityProviderResponse>)> {
    // TODO: Call federation_service.create_provider() when implemented
    Err(crate::error::ApiError(AuthencError::NotImplemented("create_identity_provider not yet implemented".to_string()))))
}

/// PUT /api/v1/iam/identity-providers/{id} - Update identity provider
pub async fn update_identity_provider(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_req): Json<UpdateIdentityProviderRequest>,
) -> ApiResult<Json<IdentityProviderResponse>> {
    // TODO: Call federation_service.update_provider() when implemented
    Err(crate::error::ApiError(AuthencError::NotImplemented("update_identity_provider not yet implemented".to_string()))))
}

/// DELETE /api/v1/iam/identity-providers/{id} - Delete identity provider
pub async fn delete_identity_provider(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // TODO: Call federation_service.delete_provider() when implemented
    Err(crate::error::ApiError(AuthencError::NotImplemented("delete_identity_provider not yet implemented".to_string()))))
}
