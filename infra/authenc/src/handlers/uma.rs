//! UMA 2.0 HTTP Handlers
//!
//! HTTP endpoints for UMA 2.0 authorization flow:
//! - Permission ticket endpoint
//! - Authorization endpoint (RPT issuance)
//! - RPT introspection
//! - Claims gathering
//! - Resource owner authorization

use crate::app::AppState;
use crate::error::Result;
use crate::handlers::api::auth_bearer::AuthBearer;
use crate::services::uma::{
    AuthorizationContextBuilder, AuthorizationDecision, SubmittedClaims, UmaAuthorizationRequest,
    UmaAuthorizationResponse, UmaPermissionRequest,
};
use axum::{
    Router,
    extract::State,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Permission ticket request (from resource server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionTicketRequest {
    pub permissions: Vec<UmaPermissionRequest>,
}

/// Permission ticket response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionTicketResponse {
    pub ticket: String,
}

/// POST /uma/permission - Request permission ticket
///
/// Called by resource server when client attempts access without RPT
pub async fn request_permission_ticket(
    State(state): State<Arc<AppState>>,
    _auth: AuthBearer, // Require authentication
    Json(req): Json<PermissionTicketRequest>,
) -> Result<Json<PermissionTicketResponse>> {
    // TODO: Extract realm_id and resource_server_id from authenticated client claims
    let realm_id = "default";
    let resource_server_id = "default-resource-server";

    let ticket = state
        .uma_permission_endpoint
        .request_permission_ticket(realm_id, resource_server_id, req.permissions)
        .await?;

    Ok(Json(PermissionTicketResponse { ticket }))
}

/// POST /uma/authorize - Exchange permission ticket for RPT
///
/// Called by client to get RPT (Requesting Party Token)
pub async fn authorize_access(
    State(state): State<Arc<AppState>>,
    auth: AuthBearer, // Extract authenticated user
    Json(req): Json<UmaAuthorizationRequest>,
) -> Result<Json<UmaAuthorizationResponse>> {
    // Extract user/client from authenticated claims
    let subject_id = auth.0.sub.clone();
    // Current JWT Claims only contain `sub` and `exp`, so we reuse `sub` as client identifier
    let client_id = subject_id.clone();
    let realm_id = "default";

    // Build authorization context from HTTP request
    let (subject_id, client_id, attributes, environment) = AuthorizationContextBuilder::new()
        .subject_id(subject_id.clone())
        .client_id(client_id.clone())
        .ip_address("127.0.0.1".to_string()) // TODO: Extract from X-Forwarded-For header
        .user_agent("Mozilla/5.0".to_string()) // TODO: Extract from User-Agent header
        .mfa_completed(false) // TODO: Check from JWT claims
        .trust_score(0.8) // TODO: Calculate from Zero Trust context
        .build()?;

    let response = state
        .uma_permission_endpoint
        .authorize_access(
            realm_id,
            req,
            &subject_id,
            &client_id,
            attributes,
            environment,
        )
        .await?;

    Ok(Json(response))
}

/// POST /uma/introspect - Introspect RPT token
///
/// Called by resource server to validate RPT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionRequest {
    pub token: String,
    pub token_type_hint: Option<String>,
}

pub async fn introspect_rpt(
    State(state): State<Arc<AppState>>,
    _auth: AuthBearer, // Require authentication
    Json(req): Json<IntrospectionRequest>,
) -> Result<Json<crate::services::uma::RptIntrospectionResponse>> {
    // Validate and introspect RPT token
    let introspection = state.uma_rpt_service.introspect(&req.token)?;
    Ok(Json(introspection))
}

/// POST /uma/claims/submit - Submit collected claims
///
/// Called by client after gathering required claims
pub async fn submit_claims(
    State(state): State<Arc<AppState>>,
    _auth: AuthBearer, // Require authentication
    Json(claims): Json<SubmittedClaims>,
) -> Result<Json<crate::services::uma::ClaimsSubmissionResult>> {
    let mut service = state.uma_claims_gathering.lock().await;
    let result = service.submit_claims(claims)?;
    Ok(Json(result))
}

/// GET /uma/claims/gather - Interactive claims gathering page
///
/// Displays form for user to provide required claims
pub async fn gather_claims_page(
    State(_state): State<Arc<AppState>>,
    // TODO: Extract state parameter from query
) -> Result<String> {
    // TODO: Render HTML page for claims gathering
    // In production, this would render a form based on required claims
    Ok("<html><body><h1>Claims Gathering</h1></body></html>".to_string())
}

/// Resource owner authorization endpoints

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationDecisionRequest {
    pub ticket_id: String,
    pub decision: AuthorizationDecision,
    pub reason: Option<String>,
}

/// GET /uma/pending-requests - Get pending authorization requests
///
/// Called by resource owner to see pending access requests
pub async fn get_pending_requests(
    State(state): State<Arc<AppState>>,
    auth: AuthBearer, // Extract authenticated user
) -> Result<Json<Vec<crate::services::uma::ResourceOwnerAuthorizationRequest>>> {
    let owner_id = &auth.0.sub; // Extract from JWT claims
    let realm_id = "default";

    let requests = state
        .uma_resource_owner_auth
        .get_pending_requests(owner_id, realm_id)
        .await?;

    Ok(Json(requests))
}

/// POST /uma/authorize-request - Authorize or deny access request
///
/// Called by resource owner to grant/deny access
pub async fn authorize_request(
    State(state): State<Arc<AppState>>,
    auth: AuthBearer, // Extract authenticated user
    Json(req): Json<AuthorizationDecisionRequest>,
) -> Result<Json<crate::services::uma::ResourceOwnerAuthorizationResponse>> {
    let owner_id = &auth.0.sub; // Extract from JWT claims

    let response = state
        .uma_resource_owner_auth
        .authorize_request(owner_id, &req.ticket_id, req.decision, req.reason)
        .await?;

    Ok(Json(response))
}

/// GET /.well-known/uma2-configuration - UMA 2.0 discovery endpoint
///
/// Returns UMA 2.0 configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UmaConfiguration {
    pub issuer: String,
    pub permission_endpoint: String,
    pub authorization_endpoint: String,
    pub introspection_endpoint: String,
    pub resource_registration_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub grant_types_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub uma_profiles_supported: Vec<String>,
}

pub async fn uma_discovery(// TODO: Extract base URL from config
) -> Result<Json<UmaConfiguration>> {
    let base_url = "https://auth.example.com"; // TODO: Get from config

    Ok(Json(UmaConfiguration {
        issuer: base_url.to_string(),
        permission_endpoint: format!("{}/uma/permission", base_url),
        authorization_endpoint: format!("{}/uma/authorize", base_url),
        introspection_endpoint: format!("{}/uma/introspect", base_url),
        resource_registration_endpoint: format!("{}/uma/resource", base_url),
        token_endpoint: format!("{}/oauth/token", base_url),
        jwks_uri: format!("{}/.well-known/jwks.json", base_url),
        grant_types_supported: vec!["urn:ietf:params:oauth:grant-type:uma-ticket".to_string()],
        response_types_supported: vec!["token".to_string()],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_basic".to_string(),
            "client_secret_post".to_string(),
            "private_key_jwt".to_string(),
        ],
        uma_profiles_supported: vec![],
    }))
}

/// Create UMA routes
///
/// Returns a router with all UMA 2.0 endpoints
pub fn create_uma_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Core UMA endpoints (protected by AuthBearer)
        .route("/uma/permission", post(request_permission_ticket))
        .route("/uma/authorize", post(authorize_access))
        .route("/uma/introspect", post(introspect_rpt))
        // Claims gathering
        .route("/uma/claims/submit", post(submit_claims))
        .route("/uma/claims/gather", get(gather_claims_page))
        // Resource owner authorization
        .route("/uma/pending-requests", get(get_pending_requests))
        .route("/uma/authorize-request", post(authorize_request))
        // Discovery (public endpoint)
        .route("/.well-known/uma2-configuration", get(uma_discovery))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uma_configuration_structure() {
        let config = UmaConfiguration {
            issuer: "https://auth.example.com".to_string(),
            permission_endpoint: "https://auth.example.com/uma/permission".to_string(),
            authorization_endpoint: "https://auth.example.com/uma/authorize".to_string(),
            introspection_endpoint: "https://auth.example.com/uma/introspect".to_string(),
            resource_registration_endpoint: "https://auth.example.com/uma/resource".to_string(),
            token_endpoint: "https://auth.example.com/oauth/token".to_string(),
            jwks_uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
            grant_types_supported: vec!["urn:ietf:params:oauth:grant-type:uma-ticket".to_string()],
            response_types_supported: vec!["token".to_string()],
            token_endpoint_auth_methods_supported: vec!["client_secret_basic".to_string()],
            uma_profiles_supported: vec![],
        };

        // Verify JSON serialization works
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("uma"));
    }
}
