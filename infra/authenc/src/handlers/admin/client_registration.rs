use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::app::AppState;
use crate::error::AuthencError;
use crate::models::client_registration::{
    ClientRegistrationError, ClientRegistrationRequest, ClientRegistrationResponse,
    ClientUpdateRequest,
};
// Use v2 trait which supports initial_access_token parameter
use crate::services::client_registration_v2::ClientRegistrationService;

/// Extract realm_id from request headers or context
/// In production, this should be extracted from JWT token claims or URL path
fn extract_realm_id(headers: &HeaderMap) -> Option<Uuid> {
    headers
        .get("x-realm-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
}

/// Extract initial access token from Authorization header
fn extract_initial_access_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|auth| auth.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(auth[7..].to_string())
            } else {
                None
            }
        })
}

/// Extract registration access token from Authorization header
fn extract_registration_token(headers: &HeaderMap) -> Option<String> {
    extract_initial_access_token(headers)
}

/// Create client registration routes (RFC 7591/7592)
pub fn create_client_registration_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register_client))
        .route("/register/{client_id}", get(get_client_configuration))
        .route("/register/{client_id}", put(update_client_configuration))
        .route("/register/{client_id}", delete(delete_client_registration))
}

/// Register a new OAuth 2.0 client (RFC 7591)
async fn register_client(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<ClientRegistrationRequest>,
) -> Result<
    (StatusCode, Json<ClientRegistrationResponse>),
    (StatusCode, Json<ClientRegistrationError>),
> {
    // Create client registration service
    use crate::services::client_registration_v2::ProductionClientRegistrationService;

    let realm_id = extract_realm_id(&headers);
    let registration_service = ProductionClientRegistrationService::new(
        state.database.clone(),
        realm_id,
        "/api/v1/oauth2/register".to_string(),
    );

    let initial_access_token = extract_initial_access_token(&headers);

    // Parse software statement from request if present
    let software_statement =
        if let Some(sw_stmt_jwt) = request.additional_metadata.get("software_statement") {
            let mut claims = std::collections::HashMap::new();
            claims.insert("software_statement".to_string(), sw_stmt_jwt.clone());
            Some(crate::models::client_registration::SoftwareStatement {
                software_id: request
                    .additional_metadata
                    .get("software_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                software_version: request
                    .additional_metadata
                    .get("software_version")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                client_metadata: claims,
            })
        } else {
            None
        };

    // Register client
    match registration_service
        .register_client(request, software_statement, initial_access_token)
        .await
    {
        Ok(response) => Ok((StatusCode::CREATED, Json(response))),
        Err(err) => {
            let error_response = match err {
                AuthencError::ValidationError { message } => ClientRegistrationError {
                    error: "invalid_client_metadata".to_string(),
                    error_description: Some(message),
                },
                AuthencError::ConfigurationError { message } => ClientRegistrationError {
                    error: "invalid_request".to_string(),
                    error_description: Some(message),
                },
                AuthencError::AuthenticationFailed => ClientRegistrationError {
                    error: "invalid_token".to_string(),
                    error_description: Some("Invalid initial access token".to_string()),
                },
                _ => ClientRegistrationError {
                    error: "invalid_request".to_string(),
                    error_description: Some("Client registration failed".to_string()),
                },
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Get client configuration (RFC 7592)
async fn get_client_configuration(
    State(state): State<Arc<AppState>>,
    Path(client_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ClientRegistrationResponse>, (StatusCode, Json<ClientRegistrationError>)> {
    // Extract registration access token from Authorization header
    let registration_token = match extract_registration_token(&headers) {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ClientRegistrationError {
                    error: "invalid_token".to_string(),
                    error_description: Some(
                        "Missing or invalid registration access token".to_string(),
                    ),
                }),
            ));
        }
    };

    // Create client registration service
    use crate::services::client_registration_v2::ProductionClientRegistrationService;

    let realm_id = extract_realm_id(&headers);
    let registration_service = ProductionClientRegistrationService::new(
        state.database.clone(),
        realm_id,
        "/api/v1/oauth2/register".to_string(),
    );

    // Get client configuration
    match registration_service
        .get_client_configuration(&client_id, &registration_token)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(err) => {
            let error_response = match err {
                AuthencError::AuthenticationFailed => ClientRegistrationError {
                    error: "invalid_token".to_string(),
                    error_description: Some("Invalid registration access token".to_string()),
                },
                AuthencError::ResourceNotFound { .. } => ClientRegistrationError {
                    error: "invalid_client_id".to_string(),
                    error_description: Some("Client not found".to_string()),
                },
                _ => ClientRegistrationError {
                    error: "invalid_request".to_string(),
                    error_description: Some("Failed to get client configuration".to_string()),
                },
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Update client configuration (RFC 7592)
async fn update_client_configuration(
    State(state): State<Arc<AppState>>,
    Path(client_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<ClientUpdateRequest>,
) -> Result<Json<ClientRegistrationResponse>, (StatusCode, Json<ClientRegistrationError>)> {
    // Extract registration access token from Authorization header
    let registration_token = match extract_registration_token(&headers) {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ClientRegistrationError {
                    error: "invalid_token".to_string(),
                    error_description: Some(
                        "Missing or invalid registration access token".to_string(),
                    ),
                }),
            ));
        }
    };

    // Create client registration service
    use crate::services::client_registration_v2::ProductionClientRegistrationService;

    let realm_id = extract_realm_id(&headers);
    let registration_service = ProductionClientRegistrationService::new(
        state.database.clone(),
        realm_id,
        "/api/v1/oauth2/register".to_string(),
    );

    // Update client configuration
    match registration_service
        .update_client_configuration(&client_id, &registration_token, request)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(err) => {
            let error_response = match err {
                AuthencError::AuthenticationFailed => ClientRegistrationError {
                    error: "invalid_token".to_string(),
                    error_description: Some("Invalid registration access token".to_string()),
                },
                AuthencError::ValidationError { message } => ClientRegistrationError {
                    error: "invalid_client_metadata".to_string(),
                    error_description: Some(message),
                },
                AuthencError::ResourceNotFound { .. } => ClientRegistrationError {
                    error: "invalid_client_id".to_string(),
                    error_description: Some("Client not found".to_string()),
                },
                _ => ClientRegistrationError {
                    error: "invalid_request".to_string(),
                    error_description: Some("Failed to update client configuration".to_string()),
                },
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Delete client registration (RFC 7592)
async fn delete_client_registration(
    State(state): State<Arc<AppState>>,
    Path(client_id): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ClientRegistrationError>)> {
    // Extract registration access token from Authorization header
    let registration_token = match extract_registration_token(&headers) {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ClientRegistrationError {
                    error: "invalid_token".to_string(),
                    error_description: Some(
                        "Missing or invalid registration access token".to_string(),
                    ),
                }),
            ));
        }
    };

    // Create client registration service
    use crate::services::client_registration_v2::ProductionClientRegistrationService;

    let realm_id = extract_realm_id(&headers);
    let registration_service = ProductionClientRegistrationService::new(
        state.database.clone(),
        realm_id,
        "/api/v1/oauth2/register".to_string(),
    );

    // Delete client registration
    match registration_service
        .delete_client_registration(&client_id, &registration_token)
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(err) => {
            let error_response = match err {
                AuthencError::AuthenticationFailed => ClientRegistrationError {
                    error: "invalid_token".to_string(),
                    error_description: Some("Invalid registration access token".to_string()),
                },
                AuthencError::ResourceNotFound { .. } => ClientRegistrationError {
                    error: "invalid_client_id".to_string(),
                    error_description: Some("Client not found".to_string()),
                },
                _ => ClientRegistrationError {
                    error: "invalid_request".to_string(),
                    error_description: Some("Failed to delete client registration".to_string()),
                },
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}
