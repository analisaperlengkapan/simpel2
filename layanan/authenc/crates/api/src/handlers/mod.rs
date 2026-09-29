//! HTTP request handlers
//!
//! Every module here is mounted by `router.rs` (or used by a mounted
//! module — `oidc_jwt` backs the oauth2 id_token path). The #102 audit
//! deleted 17 unrouted modules (device flow, token exchange, SAML/social/
//! SSO federation, consent UI, Ed25519 twins, …): none had a route, an
//! intra-crate consumer, or a frontend/simpelv1 caller.

// Authentication handlers
pub mod active_role;
pub mod auth;
pub mod auth_helpers;
pub mod mfa;
pub mod session;
pub mod webauthn;

// OAuth2/OIDC handlers
pub mod client;
pub mod client_registration;
pub mod jwks;
pub mod oauth2;
pub mod oidc_jwt;
pub mod token_validation;

// CAPTCHA handlers
pub mod captcha;

// Utility handlers
pub mod health;
pub mod metrics;

// Re-export handlers
pub use active_role::switch_active_role_handler;
pub use auth::{
    ChangePasswordRequest, ErrorResponse, LoginRequest, LoginResponse, LogoutRequest,
    PasswordResetConfirmRequest, PasswordResetRequest, RefreshTokenRequest, RefreshTokenResponse,
    UpdateProfileRequest, UserProfileResponse, change_password_handler, get_current_user_handler,
    login_handler, logout_handler, password_reset_confirm_handler, password_reset_request_handler,
    refresh_token_handler, update_profile_handler,
};

pub use auth_helpers::{
    AuthError, extract_bearer_token, extract_user_from_token, verify_self_or_admin,
    verify_token_from_body,
};

pub use oauth2::{
    AuthorizeRequest, AuthorizeResponse, OidcDiscoveryResponse, TokenRequest, TokenResponse,
    UserInfoResponse, authorize_handler, discovery_handler, token_handler, userinfo_handler,
};

pub use session::{
    ListSessionsResponse, SessionInfo, list_sessions_handler, terminate_session_handler,
};

pub use mfa::{
    BackupCodesRequest, BackupCodesResponse, MfaApiService, MfaSetupData, MfaStatusData,
    MfaVerifyRequest, MfaVerifyResponse, RecoveryVerifyRequest, mfa_backup_codes_handler,
    mfa_setup_handler, mfa_status_handler, mfa_verify_handler, mfa_verify_recovery_handler,
    mfa_verify_setup_handler, totp_disable_handler, totp_enable_handler,
    totp_verify_handler as totp_verify_handler_mfa,
};

pub use token_validation::{
    IntrospectRequest, IntrospectResponse, RevokeRequest, RevokeResponse, ValidateTokenRequest,
    ValidateTokenResponse, introspect_handler, revoke_handler, validate_token_handler,
};

pub use webauthn::{
    CredentialResponse, FinishAuthenticationRequest, FinishAuthenticationResponse,
    FinishRegistrationRequest, FinishRegistrationResponse, StartAuthenticationRequest,
    StartAuthenticationResponse, StartRegistrationRequest, StartRegistrationResponse,
    UpdateCredentialRequest, delete_credential_handler, finish_authentication_handler,
    finish_registration_handler, list_credentials_handler, start_authentication_handler,
    start_registration_handler, update_credential_handler,
};

pub use client::{
    ClientResponse, CreateClientRequest, CreateClientResponse, ListClientsQuery,
    ListClientsResponse, UpdateClientRequest, create_client_handler, delete_client_handler,
    get_client_handler, list_clients_handler, update_client_handler,
};

pub use client_registration::{
    ClientRegistrationRequest, ClientRegistrationResponse, ClientUpdateRequest, DcrErrorResponse,
    delete_client_configuration_handler, get_client_configuration_handler, register_client_handler,
    update_client_configuration_handler,
};

#[cfg(feature = "captcha-debug")]
pub use captcha::captcha_debug_answer_handler;
pub use captcha::{captcha_challenge_handler, captcha_image_handler, captcha_verify_handler};
