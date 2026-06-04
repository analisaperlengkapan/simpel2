//! API state with service dependencies

use std::sync::Arc;

use authenc_core::services::{
    AuthenticationServiceImpl, ClientRegistrationService, OAuth2ServiceImpl,
    RealmManagementServiceImpl, UserManagementServiceImpl,
};
use authenc_crypto::jwt::JwtService;
use authenc_federation::IntegrasiGrpcClient;
use authenc_storage::{Database, PostgresRevocationStore};
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

    /// Token revocation list (F2H). Checked by `validate`/`introspect` after JWT
    /// verification so access tokens can be invalidated before their `exp`
    /// (logout, role change, account deactivation).
    pub revocation_store: Arc<PostgresRevocationStore>,

    /// CAPTCHA service for generating/verifying challenges
    pub captcha_service: Arc<authenc_core::services::CaptchaService>,

    /// gRPC client for layanan-integrasi (pegawai/satker data)
    pub integrasi_client: Option<Arc<IntegrasiGrpcClient>>,
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
        revocation_store: Arc<PostgresRevocationStore>,
        captcha_service: Arc<authenc_core::services::CaptchaService>,
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
            revocation_store,
            captcha_service,
            integrasi_client: None,
        }
    }

    /// Set integrasi gRPC client
    pub fn with_integrasi_client(mut self, client: Arc<IntegrasiGrpcClient>) -> Self {
        self.integrasi_client = Some(client);
        self
    }

    /// Attach the REST MFA adapter so `handlers::mfa` stops returning
    /// `mfa_not_configured`. Wired from `main.rs` once the Postgres-backed
    /// `TotpStore` and `BackupCodesStore` are available.
    pub fn with_mfa_service(mut self, mfa: Arc<dyn MfaApiService>) -> Self {
        self.mfa_service = Some(mfa);
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_api_state_creation() {
        // This is a placeholder test - actual tests will require mock services
        assert_eq!(2 + 2, 4);
    }
}
