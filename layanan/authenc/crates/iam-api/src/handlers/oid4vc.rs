//! OpenID for Verifiable Credentials (OID4VC) HTTP handlers

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

/// Verifiable credential
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifiableCredential {
    pub id: Uuid,
    pub credential_type: String,
    pub issuer: String,
    pub subject_id: Uuid,
    pub claims: serde_json::Value,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Credential issuance request
#[derive(Debug, Deserialize)]
pub struct CredentialIssuanceRequest {
    pub credential_type: String,
    pub subject_id: Uuid,
    pub claims: serde_json::Value,
    pub expires_in_days: Option<u32>,
}

/// Credential presentation request
#[derive(Debug, Deserialize)]
pub struct CredentialPresentationRequest {
    pub credential_ids: Vec<Uuid>,
    pub verifier: String,
    pub challenge: String,
}

/// Credential presentation response
#[derive(Debug, Serialize)]
pub struct CredentialPresentationResponse {
    pub presentation: String,
    pub proof: serde_json::Value,
}

/// POST /api/v1/iam/oid4vc/credentials - Issue verifiable credential
pub async fn issue_credential(
    State(_state): State<Arc<IamApiState>>,
    Json(_request): Json<CredentialIssuanceRequest>,
) -> ApiResult<(StatusCode, Json<VerifiableCredential>)> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "OID4VC not yet implemented. Requires oid4vc_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/oid4vc/credentials - List verifiable credentials
pub async fn list_credentials(
    State(_state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<VerifiableCredential>>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "OID4VC not yet implemented. Requires oid4vc_service in IamApiState.",
    )))
}

/// GET /api/v1/iam/oid4vc/credentials/{id} - Get verifiable credential
pub async fn get_credential(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<VerifiableCredential>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "OID4VC not yet implemented. Requires oid4vc_service in IamApiState.",
    )))
}

/// DELETE /api/v1/iam/oid4vc/credentials/{id} - Revoke verifiable credential
pub async fn revoke_credential(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "OID4VC not yet implemented. Requires oid4vc_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/oid4vc/presentations - Create credential presentation
pub async fn create_presentation(
    State(_state): State<Arc<IamApiState>>,
    Json(_request): Json<CredentialPresentationRequest>,
) -> ApiResult<Json<CredentialPresentationResponse>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "OID4VC not yet implemented. Requires oid4vc_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/oid4vc/presentations/verify - Verify credential presentation
pub async fn verify_presentation(
    State(_state): State<Arc<IamApiState>>,
    Json(_presentation): Json<String>,
) -> ApiResult<Json<serde_json::Value>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "OID4VC not yet implemented. Requires oid4vc_service in IamApiState.",
    )))
}

/// POST /api/v1/iam/oid4vc/credentials/{id}/verify - Verify a specific credential
pub async fn verify_credential(
    State(_state): State<Arc<IamApiState>>,
    Path(_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    Err(crate::error::ApiError(AuthencError::not_implemented(
        "OID4VC not yet implemented. Requires oid4vc_service in IamApiState.",
    )))
}
