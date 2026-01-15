// Re-export axum router for convenience
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

// Database
use crate::app::AppState;

// Sub-modules (Grouped by Feature)
pub mod admin;
pub mod api;
pub mod federation;
pub mod internal;
pub mod oidc;
pub mod security;

// Re-exports
pub use internal::health::create_health_routes;

/// Create the main application router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    // For backward compatibility, extract database from state where needed
    // TODO: Gradually migrate handlers to use AppState directly everywhere
    let db_state = state.database.clone();

    // Create OAuth2 stores
    let oauth2_stores = Arc::new(oidc::oauth2_comprehensive::OAuth2Stores::new());

    // Create combined OAuth2 state
    let oauth2_state = Arc::new(oidc::oauth2_comprehensive::OAuth2AppState {
        database: db_state.clone(),
        oauth2_stores,
        consent_store: state.consent_store.clone(),
    });

    // Create OAuth2 test router without authentication
    let oauth2_test_router = Router::new()
        .route(
            "/oauth2/authorize/test",
            get(oidc::oauth2_comprehensive::test_oauth2_authorize),
        )
        .route(
            "/oauth2/token/test",
            post(oidc::oauth2_comprehensive::test_oauth2_token),
        )
        .with_state(oauth2_state.clone());

    // Create OAuth2 router with combined state
    let oauth2_router = Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(oidc::oauth2_comprehensive::oauth2_discovery),
        )
        .route(
            "/oauth2/token",
            post(oidc::oauth2_comprehensive::oauth2_token),
        )
        .route(
            "/oauth2/introspect",
            post(oidc::oauth2_comprehensive::oauth2_introspect),
        )
        .route(
            "/oauth2/revoke",
            post(oidc::oauth2_comprehensive::oauth2_revoke),
        )
        .route("/oauth2/jwks", get(oidc::oauth2_comprehensive::oauth2_jwks))
        .route(
            "/oauth2/userinfo",
            get(oidc::oauth2_comprehensive::oauth2_userinfo),
        )
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(crate::middleware::auth_middleware_axum::AuthState {
                jwt_secret: state.config.security.jwt_secret.clone(),
            }),
            crate::middleware::auth_middleware_axum::auth_middleware,
        ))
        .with_state(oauth2_state.clone());

    let mut router = Router::new()
        .route("/health", get(internal::health::health))
        .route("/ready", get(internal::health::ready))
        .route("/live", get(internal::health::live))
        .route("/metrics", get(internal::metrics::metrics))
        .route(
            "/health/metrics",
            get(internal::metrics::health_with_metrics),
        )
        // OAuth2 authorization endpoint (accessible without auth)
        .nest(
            "/oauth2",
            Router::new()
                .route(
                    "/authorize",
                    get(oidc::oauth2_comprehensive::oauth2_authorize),
                )
                .with_state(oauth2_state.clone()),
        )
        // Legacy OIDC Endpoints with Ed25519 security
        .route(
            "/.well-known/openid_configuration",
            get(oidc::oidc_ed25519::oidc_discovery_ed25519),
        )
        .route(
            "/oidc/authorize",
            get(oidc::oidc_ed25519::oidc_authorize_ed25519),
        )
        .route("/oidc/token", post(oidc::oidc_ed25519::oidc_token_ed25519))
        .route(
            "/oidc/refresh",
            post(oidc::oidc_ed25519::oidc_refresh_ed25519),
        )
        .route(
            "/oidc/revoke",
            post(oidc::oidc_ed25519::oidc_revoke_ed25519),
        )
        .route("/oidc/jwks", get(oidc::oidc_ed25519::oidc_jwks_ed25519))
        .route(
            "/oidc/userinfo",
            get(oidc::oidc_ed25519::oidc_userinfo_ed25519),
        )
        // SSO logout route requires different state, so nest it
        .nest(
            "/oidc",
            Router::new()
                .route("/logout", get(oidc::oidc_sso::oidc_logout_with_sso))
                .with_state(oidc::oidc_sso::SsoState::new(
                    crate::config::SsoCookieConfig::default(),
                )),
        )
        // Merge OAuth2 test router (without auth)
        .merge(oauth2_test_router)
        // Merge OAuth2 router (with auth for token/userinfo endpoints)
        .merge(oauth2_router)
        // OAuth 2.0 Token Exchange endpoint (RFC 8693)
        .route(
            "/oauth2/token/exchange",
            post(oidc::token_exchange::token_exchange_endpoint),
        )
        .route(
            "/.well-known/oauth-token-exchange",
            get(oidc::token_exchange::token_exchange_metadata),
        )
        // Test consent routes (without auth)
        .merge(security::consent_ui::create_test_consent_routes().with_state(state.clone()))
        // Consent UI routes (with auth)
        .merge(
            security::consent_ui::create_consent_routes()
                .layer(axum::middleware::from_fn_with_state(
                    Arc::new(crate::middleware::auth_middleware_axum::AuthState {
                        jwt_secret: state.config.security.jwt_secret.clone(),
                    }),
                    crate::middleware::auth_middleware_axum::auth_middleware,
                ))
                .with_state(state.clone()),
        )
        // OAuth 2.0 Dynamic Client Registration (RFC 7591/7592)
        .nest(
            "/oauth2",
            admin::client_registration::create_client_registration_routes()
                .with_state(state.clone()),
        )
        // DCR Admin API
        .nest(
            "/api/v1/admin/dcr",
            admin::dcr_admin::create_dcr_admin_routes().with_state(state.clone()),
        )
        // Advanced Services API routes
        // Social login routes
        .nest(
            "/api/v1/auth/social",
            federation::social::create_social_routes().with_state(state.clone()),
        )
        // Federated login routes for LDAP/AD and social authentication
        .nest(
            "/api/v1/auth/federated",
            federation::federated_login::create_federated_login_routes().with_state(state.clone()),
        )
        // SSO (Single Sign-On) routes
        .merge(federation::sso::create_sso_router().with_state(state.clone()))
        // SPI-based federation routes for enterprise providers
        .nest(
            "/api/v1/auth/federation",
            federation::spi_federation::create_federation_routes().with_state(state.clone()),
        )
        // Federation admin routes for identity provider management
        .nest(
            "/api/v1/admin/federation",
            federation::federation_admin::create_federation_admin_routes()
                .with_state(state.clone()),
        )
        // SPI management routes for enterprise features
        .nest(
            "/api/v1/admin/spi",
            admin::spi_management::create_spi_management_routes().with_state(state.clone()),
        )
        // MFA administration routes
        .nest(
            "/api/v1/admin/mfa",
            api::mfa_admin::create_mfa_admin_routes().with_state(state.clone()),
        )
        // MFA management routes for comprehensive admin operations
        .nest(
            "/api/v1/admin/mfa-management",
            api::mfa_management::create_mfa_management_routes().with_state(state.clone()),
        )
        // MFA troubleshooting routes for diagnostic and self-service tools
        .nest(
            "/api/v1/mfa-troubleshooting",
            api::mfa_troubleshooting::create_mfa_troubleshooting_routes().with_state(state.clone()),
        )
        // CAPTCHA routes for challenge generation and validation with security monitoring
        .nest(
            "/api/v1",
            api::captcha::create_captcha_routes()
                .layer(axum::middleware::from_fn_with_state(
                    Arc::new(
                        crate::middleware::security_monitoring_axum::SecurityMonitoringState::new(
                            crate::middleware::security_monitoring_axum::SecurityMonitoringConfig {
                                enabled: true,
                                suspicious_threshold_rpm: 50,
                                monitored_paths: vec![
                                    "/captcha/validate".to_string(),
                                    "/captcha/challenge".to_string(),
                                ],
                                log_auth_attempts: true,
                                log_authz_failures: true,
                            },
                            Some(state.audit_log_store.clone()),
                        ),
                    ),
                    crate::middleware::security_monitoring_axum::security_monitoring_middleware,
                ))
                .with_state(state.clone()),
        )
        // API routes for realms, users, roles, permissions
        .nest(
            "/api/v1/auth",
            api::realm::create_realm_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::user::create_user_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::user_role::create_user_role_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::user_permission::create_user_permission_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::role::create_role_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::permission::create_permission_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::client::create_client_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::service_account::create_service_account_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::audit::create_audit_routes().with_state(state.audit_log_store.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::auth::create_auth_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::auth_flow::create_auth_flow_routes().with_state(state.clone()),
        )
        // WebAuthn/FIDO2 passwordless authentication routes
        .nest(
            "/api/v1/auth/webauthn",
            security::webauthn::create_webauthn_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1",
            api::events::create_event_routes().with_state(state.clone()),
        )
        // Session 5: Event Listener System API
        .nest(
            "/api/v1/realms",
            api::event_listeners::create_event_listener_routes().with_state(state.clone()),
        )
        // Session 5: Protocol Mapper API
        .nest(
            "/api/v1/realms",
            api::protocol_mappers::create_protocol_mapper_routes().with_state(state.clone()),
        )
        // Session 5: Custom Authenticator API
        .nest(
            "/api/v1/realms",
            api::authenticators::create_authenticator_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::permission_check::create_permission_check_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::resource::create_resource_routes().with_state((
                state.resource_store.clone(),
                state.permission_ticket_store.clone(),
                state.scope_store.clone(),
                state.user_store.clone(),
            )),
        )
        .nest(
            "/api/v1/auth",
            api::resources::create_resources_routes().with_state((
                state.resource_store.clone(),
                state.permission_ticket_store.clone(),
            )),
        )
        .nest(
            "/api/v1/auth",
            api::account::create_account_routes().with_state((
                state.user_store.clone(),
                state.session_store.clone(),
                state.oidc_client_store.clone(),
                state.totp_store.clone(),
                state.audit_log_store.clone(),
                state.social_account_store.clone(),
            )),
        )
        .nest(
            "/api/v1/auth",
            api::account::create_consent_routes().with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            api::account_credentials::create_account_credentials_routes().with_state(
                api::account_credentials::AccountCredentialsState {
                    user_store: state.user_store.clone(),
                    totp_store: state.totp_store.clone(),
                },
            ),
        )
        .nest(
            "/oid4vc",
            oidc::oid4vc::create_oid4vc_router().with_state(state.clone()),
        )
        .nest(
            "/vp",
            oidc::oid4vc::create_vp_router().with_state(state.clone()),
        )
        // UMA 2.0 (User-Managed Access) fine-grained authorization
        .merge(security::uma::create_uma_routes().with_state(state.clone()))
        // JWKS endpoint at standard location
        .nest(
            "/.well-known",
            Router::new()
                .route("/jwks.json", get(oidc::jwks::jwks_endpoint))
                .with_state(state.clone()),
        );

    // No static file serving - authenc is a backend microservice only

    // Admin Console UI routes
    #[cfg(feature = "admin_console")]
    {
        router = router.nest(
            "/admin/console",
            crate::admin_console::create_admin_console_routes(state.clone()),
        );
    }

    router.with_state(state)
}

#[cfg(test)]
mod tests {

    use axum::response::IntoResponse;

    use http_body_util::BodyExt;
    use serde_json::Value;

    #[tokio::test]
    async fn test_health_endpoint() {
        use crate::handlers::internal::health::health;

        let response = health().await;

        // Convert response to JSON for testing
        let json_response = response.into_response();
        let body = json_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "healthy");
        assert!(json["version"].is_string());
        assert!(json["timestamp"].is_string());
    }
}
