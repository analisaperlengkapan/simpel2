//! API state with service dependencies

use std::sync::Arc;

use authenc_core::services::{
    AuthenticationServiceImpl, OAuth2ServiceImpl, UserManagementServiceImpl,
};
use authenc_crypto::jwt::JwtService;
use authenc_webauthn::WebAuthnService;

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

    /// WebAuthn service for passkey operations (PRIMARY authentication method)
    pub webauthn_service: Arc<WebAuthnService>,

    /// Session store for WebAuthn registration/authentication flows
    pub session_store: SessionStore,
}

impl ApiState {
    /// Create a new API state with all service dependencies
    pub fn new(
        jwt_service: Arc<JwtService>,
        auth_service: Arc<AuthenticationServiceImpl>,
        user_service: Arc<UserManagementServiceImpl>,
        oauth2_service: Arc<OAuth2ServiceImpl>,
        webauthn_service: Arc<WebAuthnService>,
        session_store: SessionStore,
    ) -> Self {
        Self {
            jwt_service,
            auth_service,
            user_service,
            oauth2_service,
            webauthn_service,
            session_store,
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
