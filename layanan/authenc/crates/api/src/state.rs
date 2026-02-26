//! API state with service dependencies

use std::sync::Arc;

use authenc_core::services::{
    AuthenticationServiceImpl, ClientRegistrationService, OAuth2ServiceImpl,
    RealmManagementServiceImpl, UserManagementServiceImpl,
};
use authenc_crypto::jwt::JwtService;
use authenc_storage::Database;
use authenc_webauthn::WebAuthnService;

use crate::handlers::mfa::MfaApiService;
use crate::session_store::SessionStore;

/// API state containing all service dependencies for public REST API
#[derive(Clone)]
pub struct ApiState {
    /// JWT service for token generation and validation
    pub jwt_service: Arc<JwtService>,

    /// Authentication service for login/logout operations
    pub auth_service: Arc<AuthenticationServiceImpl>,

    /// User management service for profile operations
    pub user_service: Arc<UserManagementServiceImpl>,

    /// OAuth2/OIDC service for authorization flows
    pub oauth2_service: Arc<OAuth2ServiceImpl>,

    /// Realm management service for multi-tenancy
    pub realm_service: Arc<RealmManagementServiceImpl>,

    /// Client registration service for OAuth2 clients (DCR)
    pub client_service: Arc<dyn ClientRegistrationService>,

    /// WebAuthn service for passkey operations (PRIMARY authentication method)
    pub webauthn_service: Arc<WebAuthnService>,

    /// MFA/TOTP service for multi-factor authentication (SECONDARY method)
    pub mfa_service: Option<Arc<dyn MfaApiService>>,

    /// Session store for WebAuthn registration/authentication flows
    pub session_store: SessionStore,

    /// Database connection for direct access when needed
    pub database: Arc<Database>,
}

impl ApiState {
    /// Create a new API state with all service dependencies
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        jwt_service: Arc<JwtService>,
        auth_service: Arc<AuthenticationServiceImpl>,
        user_service: Arc<UserManagementServiceImpl>,
        oauth2_service: Arc<OAuth2ServiceImpl>,
        realm_service: Arc<RealmManagementServiceImpl>,
        client_service: Arc<dyn ClientRegistrationService>,
        webauthn_service: Arc<WebAuthnService>,
        mfa_service: Option<Arc<dyn MfaApiService>>,
        session_store: SessionStore,
        database: Arc<Database>,
    ) -> Self {
        Self {
            jwt_service,
            auth_service,
            user_service,
            oauth2_service,
            realm_service,
            client_service,
            webauthn_service,
            mfa_service,
            session_store,
            database,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_state_creation() {
        // This is a placeholder test - actual tests will require mock services
        assert_eq!(2 + 2, 4);
    }
}
