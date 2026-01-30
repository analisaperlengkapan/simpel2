//! Federation Admin API Handlers
//!
//! Provides REST API endpoints for managing user federation:
//! - Identity provider CRUD operations
//! - Manual sync triggers
//! - Federation statistics and monitoring
//! - User identity link management

use crate::app::AppState;
use crate::error::{AuthencError, Result};
use crate::spi::ldap_federation::LdapFederationConfig;
use crate::spi::social::SocialProviderConfig;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Create federation admin routes
pub fn create_federation_admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Identity provider management
        .route("/identity-providers", get(list_identity_providers))
        .route("/identity-providers", post(create_identity_provider))
        .route("/identity-providers/{id}", get(get_identity_provider))
        .route("/identity-providers/{id}", put(update_identity_provider))
        .route("/identity-providers/{id}", delete(delete_identity_provider))
        // Sync management
        .route("/sync/trigger/{alias}", post(trigger_sync))
        .route("/sync/status", get(get_sync_status))
        .route("/sync/history/{alias}", get(get_sync_history))
        // User identity links
        .route(
            "/users/{user_id}/identity-links",
            get(get_user_identity_links),
        )
        .route(
            "/users/{user_id}/identity-links/{link_id}",
            delete(delete_identity_link),
        )
        // Federation statistics
        .route("/statistics/{alias}", get(get_federation_statistics))
}

/// Request to create identity provider
#[derive(Debug, Deserialize)]
pub struct CreateIdentityProviderRequest {
    pub alias: String,
    pub display_name: String,
    pub provider_type: String, // "ldap", "active_directory", "social"
    pub enabled: Option<bool>,
    pub trust_email: Option<bool>,
    pub store_token: Option<bool>,
    pub link_only: Option<bool>,
    pub config: serde_json::Value,
}

/// Request to update identity provider
#[derive(Debug, Deserialize)]
pub struct UpdateIdentityProviderRequest {
    pub display_name: Option<String>,
    pub enabled: Option<bool>,
    pub trust_email: Option<bool>,
    pub store_token: Option<bool>,
    pub link_only: Option<bool>,
    pub config: Option<serde_json::Value>,
}

/// Query parameters for listing
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub realm_id: Option<Uuid>,
    pub enabled: Option<bool>,
}

/// Query parameters for sync history
#[derive(Debug, Deserialize)]
pub struct SyncHistoryQuery {
    pub realm_id: Uuid,
    pub limit: Option<i64>,
}

/// Response for identity provider
#[derive(Debug, Serialize)]
pub struct IdentityProviderResponse {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub alias: String,
    pub display_name: String,
    pub enabled: bool,
    pub provider_type: String,
    pub trust_email: bool,
    pub store_token: bool,
    pub link_only: bool,
    pub config: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// List identity providers
async fn list_identity_providers(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let realm_id = query.realm_id.unwrap_or_else(|| {
        // Default realm - in production this should come from JWT
        Uuid::nil()
    });

    match state
        .federation_manager
        .list_identity_providers(realm_id)
        .await
    {
        Ok(providers) => {
            let responses: Vec<IdentityProviderResponse> = providers
                .into_iter()
                .filter(|p| query.enabled.is_none() || query.enabled == Some(p.enabled))
                .map(|p| IdentityProviderResponse {
                    id: p.id,
                    realm_id: p.realm_id,
                    alias: p.alias,
                    display_name: p.display_name,
                    enabled: p.enabled,
                    provider_type: match p.provider_type {
                        crate::services::federation_manager::FederationProviderType::Ldap => "ldap".to_string(),
                        crate::services::federation_manager::FederationProviderType::ActiveDirectory => "active_directory".to_string(),
                        crate::services::federation_manager::FederationProviderType::Social => "social".to_string(),
                    },
                    trust_email: p.trust_email,
                    store_token: p.store_token,
                    link_only: p.link_only,
                    config: p.config,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
                .collect();

            Json(responses).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Create identity provider
async fn create_identity_provider(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateIdentityProviderRequest>,
) -> impl IntoResponse {
    // Validate configuration based on provider type
    let validation_result = match request.provider_type.as_str() {
        "ldap" | "active_directory" => validate_ldap_config(&request.config),
        "social" | "oidc" | "oauth2" => validate_social_config(&request.config),
        _ => Err(AuthencError::validation(
            "Invalid provider type".to_string(),
        )),
    };

    if let Err(e) = validation_result {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response();
    }

    let client = match state.database.get_connection().await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
                .into_response();
        }
    };

    let id = Uuid::new_v4();
    let realm_id = Uuid::nil(); // Default realm - should come from JWT
    let provider_type_str = request.provider_type.clone();

    match client
        .execute(
            "INSERT INTO identity_broker_configs
             (id, realm_id, alias, display_name, enabled, provider_type,
              trust_email, store_token, link_only, config, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())",
            &[
                &id,
                &realm_id,
                &request.alias,
                &request.display_name,
                &request.enabled.unwrap_or(true),
                &provider_type_str,
                &request.trust_email.unwrap_or(false),
                &request.store_token.unwrap_or(false),
                &request.link_only.unwrap_or(false),
                &request.config,
            ],
        )
        .await
    {
        Ok(_) => {
            // Reload federation manager to register new provider
            if let Err(e) = state.federation_manager.initialize().await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Provider created but failed to initialize: {}", e)
                    })),
                )
                    .into_response();
            }

            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "id": id,
                    "message": "Identity provider created successfully"
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get identity provider by ID
async fn get_identity_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let client = match state.database.get_connection().await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
                .into_response();
        }
    };

    match client
        .query_opt(
            "SELECT id, realm_id, alias, display_name, enabled, provider_type,
                    trust_email, store_token, link_only, config, created_at, updated_at
             FROM identity_broker_configs
             WHERE id = $1",
            &[&id],
        )
        .await
    {
        Ok(Some(row)) => {
            let response = IdentityProviderResponse {
                id: row.get(0),
                realm_id: row.get(1),
                alias: row.get(2),
                display_name: row.get(3),
                enabled: row.get(4),
                provider_type: row.get(5),
                trust_email: row.get(6),
                store_token: row.get(7),
                link_only: row.get(8),
                config: row.get(9),
                created_at: row.get(10),
                updated_at: row.get(11),
            };
            Json(response).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Identity provider not found"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Update identity provider
async fn update_identity_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateIdentityProviderRequest>,
) -> impl IntoResponse {
    let client = match state.database.get_connection().await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
                .into_response();
        }
    };

    // Simple update - in production use query builder
    match client
        .execute(
            "UPDATE identity_broker_configs
             SET display_name = COALESCE($2, display_name),
                 enabled = COALESCE($3, enabled),
                 trust_email = COALESCE($4, trust_email),
                 store_token = COALESCE($5, store_token),
                 link_only = COALESCE($6, link_only),
                 config = COALESCE($7, config),
                 updated_at = NOW()
             WHERE id = $1",
            &[
                &id,
                &request.display_name,
                &request.enabled,
                &request.trust_email,
                &request.store_token,
                &request.link_only,
                &request.config,
            ],
        )
        .await
    {
        Ok(rows_affected) if rows_affected > 0 => {
            // Reload federation manager
            if let Err(e) = state.federation_manager.initialize().await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Provider updated but failed to reload: {}", e)
                    })),
                )
                    .into_response();
            }

            Json(serde_json::json!({
                "message": "Identity provider updated successfully"
            }))
            .into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Identity provider not found"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Delete identity provider
async fn delete_identity_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let client = match state.database.get_connection().await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
                .into_response();
        }
    };

    match client
        .execute("DELETE FROM identity_broker_configs WHERE id = $1", &[&id])
        .await
    {
        Ok(rows_affected) if rows_affected > 0 => {
            // Reload federation manager
            let _ = state.federation_manager.initialize().await;

            Json(serde_json::json!({
                "message": "Identity provider deleted successfully"
            }))
            .into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Identity provider not found"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Trigger manual sync for a provider
async fn trigger_sync(
    State(state): State<Arc<AppState>>,
    Path(alias): Path<String>,
    Query(query): Query<SyncHistoryQuery>,
) -> impl IntoResponse {
    // Check if sync is already running
    if state.user_sync_service.is_sync_running().await {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Sync is already running"
            })),
        )
            .into_response();
    }

    match state
        .user_sync_service
        .sync_provider(&alias, query.realm_id)
        .await
    {
        Ok(result) => Json(result).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get sync status
async fn get_sync_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let is_running = state.user_sync_service.is_sync_running().await;

    Json(serde_json::json!({
        "is_running": is_running,
        "timestamp": chrono::Utc::now()
    }))
}

/// Get sync history for a provider
async fn get_sync_history(
    State(state): State<Arc<AppState>>,
    Path(alias): Path<String>,
    Query(query): Query<SyncHistoryQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(10);

    match state
        .user_sync_service
        .get_sync_history(&alias, query.realm_id, limit)
        .await
    {
        Ok(history) => Json(history).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get user identity links
async fn get_user_identity_links(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match state
        .federation_manager
        .get_user_identity_links(user_id)
        .await
    {
        Ok(links) => Json(links).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Delete user identity link
async fn delete_identity_link(
    State(state): State<Arc<AppState>>,
    Path((_user_id, link_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let client = match state.database.get_connection().await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
                .into_response();
        }
    };

    match client
        .execute(
            "DELETE FROM federated_identity_links WHERE id = $1",
            &[&link_id],
        )
        .await
    {
        Ok(rows_affected) if rows_affected > 0 => Json(serde_json::json!({
            "message": "Identity link deleted successfully"
        }))
        .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Identity link not found"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get federation statistics
async fn get_federation_statistics(
    State(state): State<Arc<AppState>>,
    Path(alias): Path<String>,
) -> impl IntoResponse {
    let client = match state.database.get_connection().await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
                .into_response();
        }
    };

    // Get statistics
    match client
        .query_one(
            "SELECT COUNT(*) as total_links,
                    COUNT(DISTINCT user_id) as total_users,
                    SUM(authentication_count) as total_authentications
             FROM federated_identity_links
             WHERE identity_provider_alias = $1",
            &[&alias],
        )
        .await
    {
        Ok(row) => {
            let total_links: i64 = row.get(0);
            let total_users: i64 = row.get(1);
            let total_authentications: Option<i64> = row.get(2);

            Json(serde_json::json!({
                "provider_alias": alias,
                "total_links": total_links,
                "total_users": total_users,
                "total_authentications": total_authentications.unwrap_or(0),
                "timestamp": chrono::Utc::now()
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Validate LDAP configuration
fn validate_ldap_config(config: &serde_json::Value) -> Result<()> {
    let _: LdapFederationConfig = serde_json::from_value(config.clone())
        .map_err(|e| AuthencError::validation(format!("Invalid LDAP config: {}", e)))?;
    Ok(())
}

/// Validate social provider configuration
fn validate_social_config(config: &serde_json::Value) -> Result<()> {
    let _: SocialProviderConfig = serde_json::from_value(config.clone())
        .map_err(|e| AuthencError::validation(format!("Invalid social config: {}", e)))?;
    Ok(())
}
