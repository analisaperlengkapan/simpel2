//! OAuth2 client management HTTP handlers
//!
//! Read-only inspection of `oauth2_clients` for the admin console. Clients
//! are seeded configuration (portal + SSO consumers, `002_seed.sql`) — they
//! are not created or mutated at runtime, so the old create/update/delete/
//! regenerate-secret stubs were removed instead of implemented. The secret
//! hash never leaves the database.

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::IamApiState;
use authenc_types::AuthencError;

/// OAuth2 client response DTO (shape shared with the portal `ClientInfo`)
#[derive(Debug, Serialize)]
pub struct ClientResponse {
    pub id: Uuid,
    pub client_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub client_type: String,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub grant_types: Vec<String>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

fn row_to_client(row: &tokio_postgres::Row) -> ClientResponse {
    ClientResponse {
        id: row.get("id"),
        client_id: row.get("client_id"),
        name: row.get("client_name"),
        description: None,
        client_type: row.get("client_type"),
        redirect_uris: row.get("redirect_uris"),
        scopes: row.get("scopes"),
        grant_types: row.get("grant_types"),
        enabled: row.get("enabled"),
        created_at: row.get("created_at"),
    }
}

const CLIENT_COLUMNS: &str = "id, client_id, client_name, client_type, \
     redirect_uris, scopes, grant_types, enabled, created_at";

/// GET /api/v1/iam/clients - List OAuth2 clients
pub async fn list_clients(
    State(state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<ClientResponse>>> {
    let rows = state
        .database
        .query(
            &format!(
                "SELECT {CLIENT_COLUMNS} FROM oauth2_clients \
                 WHERE deleted_at IS NULL ORDER BY client_id"
            ),
            &[],
        )
        .await
        .map_err(crate::error::ApiError)?;

    Ok(Json(rows.iter().map(row_to_client).collect()))
}

/// GET /api/v1/iam/clients/{id} - Get client details
pub async fn get_client(
    State(state): State<Arc<IamApiState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ClientResponse>> {
    let rows = state
        .database
        .query(
            &format!(
                "SELECT {CLIENT_COLUMNS} FROM oauth2_clients \
                 WHERE id = $1 AND deleted_at IS NULL"
            ),
            &[&id],
        )
        .await
        .map_err(crate::error::ApiError)?;

    match rows.first() {
        Some(row) => Ok(Json(row_to_client(row))),
        None => Err(crate::error::ApiError(AuthencError::not_found("client"))),
    }
}
