//! DCR (Dynamic Client Registration) Admin HTTP handlers

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::IamApiState;
use authenc_types::AuthencError;

/// Create initial access token request
#[derive(Debug, Deserialize)]
pub struct CreateInitialAccessTokenRequest {
    /// Number of allowed registrations
    #[serde(default = "default_count")]
    pub count: i32,
    /// Expiration time in seconds
    pub expires_in: Option<i32>,
    /// Realm ID
    pub realm_id: Option<Uuid>,
}

fn default_count() -> i32 {
    1
}

/// Initial access token response
#[derive(Debug, Serialize)]
pub struct InitialAccessTokenResponse {
    pub id: Uuid,
    pub token: String,
    pub count: i32,
    pub remaining_count: i32,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Initial access token list item
#[derive(Debug, Serialize)]
pub struct InitialAccessTokenListItem {
    pub id: Uuid,
    pub count: i32,
    pub remaining_count: i32,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Registration policy
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationPolicy {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub allowed_grant_types: Vec<String>,
    pub allowed_response_types: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub require_software_statement: bool,
    pub trusted_issuers: Vec<Uuid>,
}

/// Software statement issuer
#[derive(Debug, Serialize, Deserialize)]
pub struct SoftwareStatementIssuer {
    pub id: Uuid,
    pub name: String,
    pub issuer_url: String,
    pub jwks_uri: String,
    pub trusted: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// POST /api/v1/iam/dcr/initial-access-tokens - Create initial access token
pub async fn create_initial_access_token(
    State(_state): State<Arc<IamApiState>>,
    Json(_request): Json<CreateInitialAccessTokenRequest>,
) -> ApiResult<(StatusCode, Json<InitialAccessTokenResponse>)> {
    // TODO: Implement create initial access token
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/dcr/initial-access-tokens - List initial access tokens
pub async fn list_initial_access_tokens(
    State(_state): State<Arc<IamApiState>>,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<Vec<InitialAccessTokenListItem>>> {
    // TODO: Implement list initial access tokens
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// DELETE /api/v1/iam/dcr/initial-access-tokens/{id} - Revoke initial access token
pub async fn revoke_initial_access_token(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // TODO: Implement revoke initial access token
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/dcr/policies/{realm_id} - Get registration policy
pub async fn get_registration_policy(
    State(_state): State<Arc<IamApiState>>,
    Path(_realm_id): Path<Uuid>,
) -> ApiResult<Json<RegistrationPolicy>> {
    // TODO: Implement get registration policy
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/dcr/policies/{realm_id} - Create registration policy
pub async fn create_registration_policy(
    State(_state): State<Arc<IamApiState>>,
    Path(_realm_id): Path<Uuid>,
    Json(_policy): Json<RegistrationPolicy>,
) -> ApiResult<(StatusCode, Json<RegistrationPolicy>)> {
    // TODO: Implement create registration policy
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// PUT /api/v1/iam/dcr/policies/{realm_id}/{policy_id} - Update registration policy
pub async fn update_registration_policy(
    State(_state): State<Arc<IamApiState>>,
    Path((_realm_id, _policy_id)): Path<(Uuid, Uuid)>,
    Json(_policy): Json<RegistrationPolicy>,
) -> ApiResult<Json<RegistrationPolicy>> {
    // TODO: Implement update registration policy
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// DELETE /api/v1/iam/dcr/policies/{realm_id}/{policy_id} - Delete registration policy
pub async fn delete_registration_policy(
    State(_state): State<Arc<IamApiState>>,
    Path((_realm_id, _policy_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    // TODO: Implement delete registration policy
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/dcr/software-statement-issuers - Create software statement issuer
pub async fn create_software_statement_issuer(
    State(_state): State<Arc<IamApiState>>,
    Json(_issuer): Json<SoftwareStatementIssuer>,
) -> ApiResult<(StatusCode, Json<SoftwareStatementIssuer>)> {
    // TODO: Implement create software statement issuer
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/dcr/software-statement-issuers - List software statement issuers
pub async fn list_software_statement_issuers(
    State(_state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<SoftwareStatementIssuer>>> {
    // TODO: Implement list software statement issuers
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/dcr/software-statement-issuers/{id} - Get software statement issuer
pub async fn get_software_statement_issuer(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<SoftwareStatementIssuer>> {
    // TODO: Implement get software statement issuer
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// PUT /api/v1/iam/dcr/software-statement-issuers/{id} - Update software statement issuer
pub async fn update_software_statement_issuer(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_issuer): Json<SoftwareStatementIssuer>,
) -> ApiResult<Json<SoftwareStatementIssuer>> {
    // TODO: Implement update software statement issuer
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// DELETE /api/v1/iam/dcr/software-statement-issuers/{id} - Delete software statement issuer
pub async fn delete_software_statement_issuer(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // TODO: Implement delete software statement issuer
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/dcr/policies - List DCR policies
pub async fn list_dcr_policies(
    State(_state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<RegistrationPolicy>>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/dcr/policies - Create DCR policy
pub async fn create_dcr_policy(
    State(_state): State<Arc<IamApiState>>,
    Json(_policy): Json<RegistrationPolicy>,
) -> ApiResult<(StatusCode, Json<RegistrationPolicy>)> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/dcr/policies/{id} - Get DCR policy
pub async fn get_dcr_policy(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<RegistrationPolicy>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// PUT /api/v1/iam/dcr/policies/{id} - Update DCR policy
pub async fn update_dcr_policy(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
    Json(_policy): Json<RegistrationPolicy>,
) -> ApiResult<Json<RegistrationPolicy>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}

/// DELETE /api/v1/iam/dcr/policies/{id} - Delete DCR policy
pub async fn delete_dcr_policy(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "DCR admin not yet implemented. Requires dcr_admin_service in IamApiState.",
    )))
}
