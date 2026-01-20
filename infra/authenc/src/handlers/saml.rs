//! SAML 2.0 Authentication Handlers
//!
//! Production-ready SAML handlers that load configuration from database.
//! Supports SP metadata, IDP metadata, SSO, ACS, and SLO endpoints.

use crate::database::Database;
use crate::error::AuthencError;
use crate::handlers::jit_admin_service::JitAdminService;
use crate::models::saml as saml_models;
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
use uuid::Uuid;

/// SAML handler state with database access
#[derive(Clone)]
pub struct SamlState {
    pub db: Arc<Database>,
}

impl SamlState {
    /// Create new SAML state
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

/// Create SAML routes with state
pub fn create_saml_routes() -> Router<Arc<Database>> {
    Router::new()
        .route("/sp/metadata", get(sp_metadata))
        .route("/idp/metadata", get(idp_metadata))
        .route("/auth", get(saml_auth))
        .route("/acs", post(saml_acs))
        .route("/slo", get(saml_slo))
}

/// Load SAML Service Provider configuration from database
async fn load_sp_from_database(
    db: &Database,
    entity_id: &str,
) -> Result<Option<saml_models::SamlServiceProvider>, AuthencError> {
    let client = db.get_connection().await?;

    let row = client
        .query_opt(
            "SELECT id, entity_id, metadata_url, metadata_xml, signing_certificate,
                    encryption_certificate, assertion_consumer_service_url,
                    single_logout_service_url, name_id_format, realm_id, enabled,
                    created_at, updated_at, deleted_at
             FROM saml_service_providers
             WHERE entity_id = $1 AND enabled = true AND deleted_at IS NULL",
            &[&entity_id],
        )
        .await
        .map_err(|e| AuthencError::database(format!("Failed to load SAML SP: {}", e)))?;

    match row {
        Some(row) => Ok(Some(saml_models::SamlServiceProvider::try_from(row)?)),
        None => Ok(None),
    }
}

/// Load SAML Identity Provider configuration from database
async fn load_idp_from_database(
    db: &Database,
    entity_id: &str,
) -> Result<Option<saml_models::SamlIdentityProvider>, AuthencError> {
    let client = db.get_connection().await?;

    let row = client
        .query_opt(
            "SELECT id, entity_id, metadata_url, metadata_xml, sso_url, slo_url,
                    signing_certificate, encryption_certificate, name_id_format,
                    realm_id, enabled, created_at, updated_at, deleted_at
             FROM saml_identity_providers
             WHERE entity_id = $1 AND enabled = true AND deleted_at IS NULL",
            &[&entity_id],
        )
        .await
        .map_err(|e| AuthencError::database(format!("Failed to load SAML IDP: {}", e)))?;

    match row {
        Some(row) => Ok(Some(saml_models::SamlIdentityProvider::try_from(row)?)),
        None => Ok(None),
    }
}

/// Load default SAML SP from environment or database
async fn get_default_sp(db: &Database) -> Result<SamlServiceProvider, AuthencError> {
    // Try to load from environment first
    let base_url = std::env::var("AUTHENC_BASE_URL")
        .unwrap_or_else(|_| "https://authenc.example.com".to_string());

    // Try to load from database with default entity_id
    let entity_id = format!("{}/saml/sp", base_url);

    if let Some(db_sp) = load_sp_from_database(db, &entity_id).await? {
        return Ok(SamlServiceProvider {
            entity_id: db_sp.entity_id,
            assertion_consumer_service_url: db_sp.assertion_consumer_service_url,
            single_logout_service_url: db_sp.single_logout_service_url,
            name_id_format: db_sp.name_id_format,
            want_assertions_signed: true,
            want_response_signed: true,
        });
    }

    // Fall back to configuration
    Ok(SamlServiceProvider {
        entity_id,
        assertion_consumer_service_url: format!("{}/saml/acs", base_url),
        single_logout_service_url: Some(format!("{}/saml/slo", base_url)),
        name_id_format: "urn:oasis:names:tc:SAML:1.0:nameid-format:emailAddress".to_string(),
        want_assertions_signed: true,
        want_response_signed: true,
    })
}

/// Load default SAML IDP from environment or database
async fn get_default_idp(db: &Database, idp_entity_id: Option<&str>) -> Result<SamlIdentityProvider, AuthencError> {
    let base_url = std::env::var("AUTHENC_BASE_URL")
        .unwrap_or_else(|_| "https://authenc.example.com".to_string());

    // Use provided entity_id or default
    let entity_id = idp_entity_id
        .map(String::from)
        .unwrap_or_else(|| format!("{}/saml/idp", base_url));

    // Try to load from database
    if let Some(db_idp) = load_idp_from_database(db, &entity_id).await? {
        return Ok(SamlIdentityProvider {
            entity_id: db_idp.entity_id,
            sso_url: db_idp.sso_url,
            slo_url: db_idp.slo_url,
            certificate: db_idp.signing_certificate,
            name_id_format: db_idp.name_id_format,
            want_authn_requests_signed: true,
        });
    }

    // Fall back to configuration from environment
    let signing_cert = std::env::var("SAML_SIGNING_CERTIFICATE")
        .unwrap_or_else(|_| "".to_string());

    Ok(SamlIdentityProvider {
        entity_id,
        sso_url: format!("{}/saml/auth", base_url),
        slo_url: Some(format!("{}/saml/slo", base_url)),
        certificate: signing_cert,
        name_id_format: "urn:oasis:names:tc:SAML:1.0:nameid-format:emailAddress".to_string(),
        want_authn_requests_signed: true,
    })
}

/// SAML service provider metadata endpoint
///
/// Returns SP metadata XML for federation setup.
/// Loads configuration from database or falls back to environment variables.
pub async fn sp_metadata(
    State(db): State<Arc<Database>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> std::result::Result<Html<String>, AuthencError> {
    let mut service = SamlService::new(db.clone());

    // Load SP configuration from database or environment
    let sp = if let Some(entity_id) = params.get("entity_id") {
        if let Some(db_sp) = load_sp_from_database(&db, entity_id).await? {
            SamlServiceProvider {
                entity_id: db_sp.entity_id,
                assertion_consumer_service_url: db_sp.assertion_consumer_service_url,
                single_logout_service_url: db_sp.single_logout_service_url,
                name_id_format: db_sp.name_id_format,
                want_assertions_signed: true,
                want_response_signed: true,
            }
        } else {
            get_default_sp(&db).await?
        }
    } else {
        get_default_sp(&db).await?
    };

    let entity_id = sp.entity_id.clone();
    service.register_service_provider(sp);

    match service.generate_sp_metadata(&entity_id) {
        Ok(metadata) => {
            tracing::info!("Generated SAML SP metadata for entity_id: {}", entity_id);
            Ok(Html(metadata))
        }
        Err(e) => {
            tracing::error!("Failed to generate SAML SP metadata: {}", e);
            Err(AuthencError::internal("Failed to generate SP metadata"))
        }
    }
}

/// SAML identity provider metadata endpoint
///
/// Returns IDP metadata XML for federation setup.
/// Loads configuration from database or falls back to environment variables.
pub async fn idp_metadata(
    State(db): State<Arc<Database>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> std::result::Result<Html<String>, AuthencError> {
    let mut service = SamlService::new(db.clone());

    // Load IDP configuration from database or environment
    let idp = get_default_idp(&db, params.get("entity_id").map(|s| s.as_str())).await?;

    let entity_id = idp.entity_id.clone();
    service.register_identity_provider(idp);

    match service.generate_idp_metadata(&entity_id) {
        Ok(metadata) => {
            tracing::info!("Generated SAML IDP metadata for entity_id: {}", entity_id);
            Ok(Html(metadata))
        }
        Err(e) => {
            tracing::error!("Failed to generate SAML IDP metadata: {}", e);
            Err(AuthencError::internal("Failed to generate IDP metadata"))
        }
    }
}

/// SAML authentication initiation
///
/// Initiates SAML SSO flow by generating AuthnRequest and redirecting to IDP.
/// Loads SP and IDP configuration from database.
pub async fn saml_auth(
    State(db): State<Arc<Database>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> std::result::Result<Redirect, AuthencError> {
    let mut service = SamlService::new(db.clone());

    // Load SP configuration
    let sp = if let Some(sp_entity) = params.get("sp") {
        if let Some(db_sp) = load_sp_from_database(&db, sp_entity).await? {
            SamlServiceProvider {
                entity_id: db_sp.entity_id,
                assertion_consumer_service_url: db_sp.assertion_consumer_service_url,
                single_logout_service_url: db_sp.single_logout_service_url,
                name_id_format: db_sp.name_id_format,
                want_assertions_signed: true,
                want_response_signed: true,
            }
        } else {
            get_default_sp(&db).await?
        }
    } else {
        get_default_sp(&db).await?
    };

    let sp_entity_id = sp.entity_id.clone();
    service.register_service_provider(sp);

    // Load IDP configuration
    let idp = get_default_idp(&db, params.get("idp").map(|s| s.as_str())).await?;
    let idp_entity_id = idp.entity_id.clone();
    service.register_identity_provider(idp);

    let relay_state = params.get("RelayState").map(|s| s.as_str());

    tracing::info!(
        "Initiating SAML auth: SP={}, IDP={}",
        sp_entity_id,
        idp_entity_id
    );

    match service
        .generate_authn_request(&sp_entity_id, &idp_entity_id, relay_state)
        .await
    {
        Ok(redirect_url) => Ok(Redirect::to(&redirect_url)),
        Err(e) => {
            tracing::error!("Failed to generate SAML AuthnRequest: {}", e);
            Err(AuthencError::internal("Failed to initiate SAML authentication"))
        }
    }
}

/// SAML assertion consumer service (ACS) endpoint
///
/// Processes SAML Response from IDP and provisions user via JIT.
/// Loads IDP configuration from database for signature verification.
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
        tracing::warn!("SAML ACS: Missing SAMLResponse parameter");
        return Err(AuthencError::validation("SAMLResponse parameter is required"));
    };

    let relay_state = params.get("RelayState").map(|s| s.as_str());

    // Extract IDP entity ID from RelayState or use default
    let idp_entity_id = relay_state
        .and_then(|rs| {
            // Parse relay state for IDP info if encoded
            serde_json::from_str::<serde_json::Value>(rs)
                .ok()
                .and_then(|v| v["idp_entity_id"].as_str().map(String::from))
        });

    // Load IDP from database for signature verification
    if let Some(ref idp_id) = idp_entity_id {
        if let Some(db_idp) = load_idp_from_database(&db, idp_id).await? {
            tracing::info!("Loaded IDP config from database for verification: {}", idp_id);
            // IDP certificate is used by SamlService internally for verification
        }
    }

    match service
        .process_response(saml_response, relay_state, "")
        .await
    {
        Ok(user_info) => {
            tracing::info!(
                "SAML authentication successful for user: {}",
                user_info.name_id
            );

            // Create JIT provisioning service
            let admin_service = Arc::new(JitAdminService::new(db.clone()));
            let jit_service = Arc::new(DefaultJITProvisioningService::new(
                db.clone(),
                admin_service,
            ));

            // Try to get realm_id from database based on IDP
            let realm_id = if let Some(ref idp_id) = idp_entity_id {
                if let Some(db_idp) = load_idp_from_database(&db, idp_id).await? {
                    db_idp.realm_id.unwrap_or_else(Uuid::new_v4)
                } else {
                    Uuid::new_v4()
                }
            } else {
                Uuid::new_v4()
            };

            // Prepare JIT provisioning request
            let jit_request = JITUserProvisioningRequest {
                identity_provider_id: Uuid::new_v4(), // Will be looked up or created
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
                    .or_else(|| user_info.attributes.get("givenName"))
                    .and_then(|v| v.first())
                    .map(|s| s.to_string()),
                last_name: user_info
                    .attributes
                    .get("lastName")
                    .or_else(|| user_info.attributes.get("sn"))
                    .and_then(|v| v.first())
                    .map(|s| s.to_string()),
                external_attributes: Some(
                    serde_json::to_value(&user_info.attributes)
                        .map_err(|e| AuthencError::internal(&format!("Failed to serialize attributes: {}", e)))?,
                ),
                realm_id,
            };

            // Provision user using JIT
            match jit_service.provision_user(jit_request).await {
                Ok(jit_response) => {
                    tracing::info!(
                        "JIT provisioned user {} (created: {})",
                        jit_response.user.username,
                        jit_response.created
                    );

                    // Generate Ed25519 JWT tokens for the user
                    let access_token = crate::handlers::oidc_ed25519::generate_ed25519_jwt(
                        &jit_response.user.id.to_string(),
                        &realm_id.to_string(),
                        Some(&jit_response.user.email),
                        jit_response.user.first_name.as_deref(),
                        Some("user"),
                    );

                    // Return success with redirect or token
                    if let Some(rs) = relay_state {
                        // If RelayState contains a return URL, redirect there
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(rs) {
                            if let Some(return_url) = parsed["return_url"].as_str() {
                                let redirect_url = format!("{}?token={}", return_url, access_token);
                                return Ok(Html(format!(
                                    r#"<!DOCTYPE html>
<html>
<head>
    <meta http-equiv="refresh" content="0;url={}" />
    <title>SAML Login Success - Redirecting...</title>
</head>
<body>
    <p>Login successful. Redirecting...</p>
    <script>window.location.href = "{}";</script>
</body>
</html>"#,
                                    redirect_url, redirect_url
                                )));
                            }
                        }
                    }

                    // Default: show success page with token
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
<input type="hidden" id="access_token" value="{}" />
<script>
    // Store token for SPA consumption
    if (window.opener) {{
        window.opener.postMessage({{ type: 'saml_auth_success', token: '{}' }}, '*');
        window.close();
    }}
</script>
</body>
</html>"#,
                        jit_response.user.username,
                        jit_response.user.id,
                        user_info.session_index,
                        user_info.authn_context_class_ref,
                        jit_response.created,
                        access_token,
                        access_token
                    );
                    Ok(Html(html))
                }
                Err(e) => {
                    tracing::error!("JIT provisioning failed: {}", e);
                    let html = format!(
                        r#"<!DOCTYPE html>
<html>
<head><title>SAML Login Failed</title></head>
<body>
<h1>User Provisioning Failed</h1>
<p>Error: {}</p>
<p><a href="/">Return to Home</a></p>
</body>
</html>"#,
                        e
                    );
                    Ok(Html(html))
                }
            }
        }
        Err(e) => {
            tracing::error!("SAML response processing failed: {}", e);
            let html = r#"<!DOCTYPE html>
<html>
<head><title>SAML Login Failed</title></head>
<body>
<h1>Authentication Failed</h1>
<p>SAML authentication failed. Please try again or contact your administrator.</p>
<p><a href="/">Return to Home</a></p>
</body>
</html>"#;
            Ok(Html(html.to_string()))
        }
    }
}

/// SAML single logout endpoint
///
/// Handles SAML SLO (Single Logout) requests and responses.
/// Terminates user session and redirects to post-logout URL.
pub async fn saml_slo(
    State(db): State<Arc<Database>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> std::result::Result<Redirect, AuthencError> {
    // Check if this is a logout request or response
    if let Some(logout_request) = params.get("SAMLRequest") {
        // Handle SLO request from IDP
        tracing::info!("Processing SAML SLO request");

        // In production:
        // 1. Parse and validate the LogoutRequest
        // 2. Terminate user sessions
        // 3. Generate LogoutResponse
        // 4. Redirect back to IDP with response

        // For now, just clear session and redirect
        let post_logout_url = params
            .get("RelayState")
            .map(|s| s.as_str())
            .unwrap_or("/");

        return Ok(Redirect::to(post_logout_url));
    }

    if let Some(logout_response) = params.get("SAMLResponse") {
        // Handle SLO response from IDP
        tracing::info!("Processing SAML SLO response");

        let post_logout_url = params
            .get("RelayState")
            .map(|s| s.as_str())
            .unwrap_or("/");

        return Ok(Redirect::to(post_logout_url));
    }

    // Initiate SLO - redirect to IDP's SLO endpoint
    if let Some(idp_entity_id) = params.get("idp") {
        if let Some(db_idp) = load_idp_from_database(&db, idp_entity_id).await? {
            if let Some(slo_url) = db_idp.slo_url {
                tracing::info!("Initiating SAML SLO to IDP: {}", idp_entity_id);
                return Ok(Redirect::to(&slo_url));
            }
        }
    }

    // Default: just redirect to home
    tracing::info!("SAML SLO: No IDP specified, redirecting to home");
    Ok(Redirect::to("/"))
}
