use crate::database::Database;
use crate::error::AuthencError;
use crate::handlers::federation::jit_admin_service::JitAdminService;
use crate::models::user::JITUserProvisioningRequest;
use crate::services::federation::jit_provisioning::{
    DefaultJITProvisioningService, JITProvisioningService,
};
use crate::services::saml::{SamlIdentityProvider, SamlService, SamlServiceProvider};
use axum::{
    Router,
    extract::{Query, State},
    response::{Html, Redirect},
    routing::{get, post},
};
use std::sync::Arc;

/// Create SAML routes
pub fn create_saml_routes() -> Router<Arc<Database>> {
    Router::new()
        .route("/sp/metadata", get(sp_metadata))
        .route("/idp/metadata", get(idp_metadata))
        .route("/auth", get(saml_auth))
        .route("/acs", post(saml_acs))
        .route("/slo", get(saml_slo))
}

/// SAML service provider metadata endpoint
pub async fn sp_metadata(
    State(db): State<Arc<Database>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> std::result::Result<Html<String>, AuthencError> {
    let mut service = SamlService::new(db);

    // In production, load from configuration
    let sp = SamlServiceProvider {
        entity_id: "https://authenc.example.com/saml/sp".to_string(),
        assertion_consumer_service_url: "https://authenc.example.com/saml/acs".to_string(),
        single_logout_service_url: Some("https://authenc.example.com/saml/slo".to_string()),
        name_id_format: "urn:oasis:names:tc:SAML:1.0:nameid-format:emailAddress".to_string(),
        want_assertions_signed: true,
        want_response_signed: true,
    };
    service.register_service_provider(sp);

    let default_entity_id = "https://authenc.example.com/saml/sp".to_string();
    let entity_id = params.get("entity_id").unwrap_or(&default_entity_id);

    match service.generate_sp_metadata(entity_id) {
        Ok(metadata) => Ok(Html(metadata)),
        Err(_) => Err(AuthencError::internal("Internal server error")),
    }
}

/// SAML identity provider metadata endpoint
pub async fn idp_metadata(
    State(db): State<Arc<Database>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> std::result::Result<Html<String>, AuthencError> {
    let mut service = SamlService::new(db);

    // In production, load from configuration
    let idp = SamlIdentityProvider {
        entity_id: "https://authenc.example.com/saml/idp".to_string(),
        sso_url: "https://authenc.example.com/saml/auth".to_string(),
        slo_url: Some("https://authenc.example.com/saml/slo".to_string()),
        certificate: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...".to_string(), // Placeholder
        name_id_format: "urn:oasis:names:tc:SAML:1.0:nameid-format:emailAddress".to_string(),
        want_authn_requests_signed: true,
    };
    service.register_identity_provider(idp);

    let default_entity_id = "https://authenc.example.com/saml/idp".to_string();
    let entity_id = params.get("entity_id").unwrap_or(&default_entity_id);

    match service.generate_idp_metadata(entity_id) {
        Ok(metadata) => Ok(Html(metadata)),
        Err(_) => Err(AuthencError::internal("Internal server error")),
    }
}

/// SAML authentication initiation
pub async fn saml_auth(
    State(db): State<Arc<Database>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> std::result::Result<Redirect, AuthencError> {
    let mut service = SamlService::new(db);

    // Register service provider
    let sp = SamlServiceProvider {
        entity_id: "https://authenc.example.com/saml/sp".to_string(),
        assertion_consumer_service_url: "https://authenc.example.com/saml/acs".to_string(),
        single_logout_service_url: Some("https://authenc.example.com/saml/slo".to_string()),
        name_id_format: "urn:oasis:names:tc:SAML:1.0:nameid-format:emailAddress".to_string(),
        want_assertions_signed: true,
        want_response_signed: true,
    };
    service.register_service_provider(sp);

    // Register identity provider
    let idp = SamlIdentityProvider {
        entity_id: "https://idp.example.com/saml/idp".to_string(),
        sso_url: "https://idp.example.com/saml/auth".to_string(),
        slo_url: Some("https://idp.example.com/saml/slo".to_string()),
        certificate: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...".to_string(),
        name_id_format: "urn:oasis:names:tc:SAML:1.0:nameid-format:emailAddress".to_string(),
        want_authn_requests_signed: true,
    };
    service.register_identity_provider(idp);

    let default_sp_entity_id = "https://authenc.example.com/saml/sp".to_string();
    let sp_entity_id = params.get("sp").unwrap_or(&default_sp_entity_id);

    let default_idp_entity_id = "https://idp.example.com/saml/idp".to_string();
    let idp_entity_id = params.get("idp").unwrap_or(&default_idp_entity_id);

    let relay_state = params.get("RelayState").map(|s| s.as_str());

    match service
        .generate_authn_request(sp_entity_id, idp_entity_id, relay_state)
        .await
    {
        Ok(redirect_url) => Ok(Redirect::to(&redirect_url)),
        Err(_) => Err(AuthencError::internal("Internal server error")),
    }
}

/// SAML assertion consumer service (ACS) endpoint
pub async fn saml_acs(
    State(db): State<Arc<Database>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    _body: String,
) -> std::result::Result<Html<String>, AuthencError> {
    let service = SamlService::new(db.clone());

    // Extract SAMLResponse from form data or query parameters
    let saml_response = if let Some(response) = params.get("SAMLResponse") {
        response
    } else {
        // In production, parse from form body
        return Err(AuthencError::validation("Bad request"));
    };

    let relay_state = params.get("RelayState").map(|s| s.as_str());

    match service
        .process_response(saml_response, relay_state, "")
        .await
    {
        Ok(user_info) => {
            // Create JIT provisioning service
            let admin_service = Arc::new(JitAdminService::new(db.clone()));
            let jit_service = Arc::new(DefaultJITProvisioningService::new(
                db.clone(),
                admin_service,
            ));

            // Prepare JIT provisioning request
            let jit_request = JITUserProvisioningRequest {
                identity_provider_id: uuid::Uuid::new_v4(), // TODO: Get from SAML configuration
                external_id: user_info.name_id.clone(),
                external_username: user_info
                    .attributes
                    .get("username")
                    .and_then(|v| v.first())
                    .map(|s| s.to_string()),
                external_email: user_info
                    .attributes
                    .get("email")
                    .and_then(|v| v.first())
                    .map(|s| s.to_string()),
                first_name: user_info
                    .attributes
                    .get("firstName")
                    .and_then(|v| v.first())
                    .map(|s| s.to_string()),
                last_name: user_info
                    .attributes
                    .get("lastName")
                    .and_then(|v| v.first())
                    .map(|s| s.to_string()),
                external_attributes: Some(
                    serde_json::to_value(&user_info.attributes)
                        .map_err(|_| AuthencError::internal("Failed to serialize attributes"))?,
                ),
                realm_id: uuid::Uuid::new_v4(), // TODO: Get from SAML configuration
            };

            // Provision user using JIT
            match jit_service.provision_user(jit_request).await {
                Ok(jit_response) => {
                    // In production, create session and redirect to application
                    let html = format!(
                        r#"<!DOCTYPE html>
<html>
<head><title>SAML Login Success</title></head>
<body>
<h1>Login Successful</h1>
<p>Welcome, {}!</p>
<p>User ID: {}</p>
<p>Session Index: {}</p>
<p>Authentication Context: {}</p>
<p>JIT Provisioned: {}</p>
<pre>{:?}</pre>
</body>
</html>"#,
                        jit_response.user.username,
                        jit_response.user.id,
                        user_info.session_index,
                        user_info.authn_context_class_ref,
                        jit_response.created,
                        user_info.attributes
                    );
                    Ok(Html(html))
                }
                Err(e) => {
                    let html = format!(
                        r#"<!DOCTYPE html>
<html>
<head><title>SAML Login Failed</title></head>
<body>
<h1>JIT Provisioning Failed</h1>
<p>Error: {}</p>
</body>
</html>"#,
                        e
                    );
                    Ok(Html(html))
                }
            }
        }
        Err(_) => {
            let html = r#"<!DOCTYPE html>
<html>
<head><title>SAML Login Failed</title></head>
<body>
<h1>Login Failed</h1>
<p>SAML authentication failed. Please try again.</p>
</body>
</html>"#;
            Ok(Html(html.to_string()))
        }
    }
}

/// SAML single logout endpoint
pub async fn saml_slo(
    State(_db): State<Arc<Database>>,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> std::result::Result<Redirect, AuthencError> {
    // In production, implement SAML logout
    // For now, redirect to home page
    Ok(Redirect::to("/"))
}
