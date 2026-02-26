//! Client Policy management HTTP handlers

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::IamApiState;
use authenc_types::AuthencError;

/// Client policy
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
}

/// Client policy profile
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientPolicyProfile {
    pub id: Uuid,
    pub name: String,
    pub policies: Vec<Uuid>,
}

/// GET /api/v1/iam/client-policies - List client policies
pub async fn list_client_policies(
    State(_state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<ClientPolicy>>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Client policy management not yet implemented. Requires client_policy_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/client-policies - Create client policy
pub async fn create_client_policy(
    State(_state): State<Arc<IamApiState>>,
    Json(_policy): Json<ClientPolicy>,
) -> ApiResult<(StatusCode, Json<ClientPolicy>)> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Client policy management not yet implemented. Requires client_policy_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/client-policies/{id} - Get client policy
pub async fn get_client_policy(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<ClientPolicy>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Client policy management not yet implemented. Requires client_policy_service in IamApiState.",
    )))
}

/// PUT /api/v1/iam/client-policies/{id} - Update client policy
pub async fn update_client_policy(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_policy): Json<ClientPolicy>,
) -> ApiResult<Json<ClientPolicy>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Client policy management not yet implemented. Requires client_policy_service in IamApiState.",
    )))
}

/// DELETE /api/v1/iam/client-policies/{id} - Delete client policy
pub async fn delete_client_policy(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Client policy management not yet implemented. Requires client_policy_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/clients/{client_id}/policies/{policy_id} - Assign policy to client
pub async fn assign_policy_to_client(
    State(_state): State<Arc<IamApiState>>,
    Path((_client_id, _policy_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Client policy management not yet implemented. Requires client_policy_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/clients/{client_id}/policies - Assign client policy (single path param)
pub async fn assign_client_policy(
    State(_state): State<Arc<IamApiState>>,
    Path(_client_id): Path<Uuid>,
    Json(_body): Json<serde_json::Value>,
) -> ApiResult<StatusCode> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Client policy management not yet implemented. Requires client_policy_service in IamApiState.",
    )))
}

/// DELETE /api/v1/iam/clients/{client_id}/policies/{policy_id} - Unassign client policy
pub async fn unassign_client_policy(
    State(_state): State<Arc<IamApiState>>,
    Path((_client_id, _policy_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "Client policy management not yet implemented. Requires client_policy_service in IamApiState.",
    )))
}
