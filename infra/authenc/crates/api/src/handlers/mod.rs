//! HTTP request handlers

pub mod auth;
pub mod auth_helpers;
pub mod oauth2;
pub mod session;
pub mod token_validation;
pub mod totp;
pub mod webauthn;

// Re-export handlers
pub use auth::{
    get_current_user_handler, login_handler, logout_handler, refresh_token_handler, ErrorResponse,
    LoginRequest, LoginResponse, LogoutRequest, RefreshTokenRequest, RefreshTokenResponse,
    UserProfileResponse,
};

pub use auth_helpers::{
    extract_bearer_token, extract_user_from_token, verify_self_or_admin, verify_token_from_body,
    AuthError,
};

pub use oauth2::{
    authorize_handler, discovery_handler, token_handler, userinfo_handler, AuthorizeRequest,
    AuthorizeResponse, OidcDiscoveryResponse, TokenRequest, TokenResponse, UserInfoResponse,
};

pub use session::{
    list_sessions_handler, logout_handler as session_logout_handler, ListSessionsResponse,
    SessionInfo,
};

pub use totp::{
    disable_totp_handler, enable_totp_handler, verify_totp_handler, DisableTotpResponse,
    EnableTotpRequest, EnableTotpResponse, VerifyTotpRequest, VerifyTotpResponse,
};

pub use token_validation::{
    introspect_handler, validate_token_handler, IntrospectRequest, IntrospectResponse,
    ValidateTokenRequest, ValidateTokenResponse,
};

pub use webauthn::{
    delete_credential_handler, finish_authentication_handler, finish_registration_handler,
    list_credentials_handler, start_authentication_handler, start_registration_handler,
    update_credential_handler, CredentialResponse, FinishAuthenticationRequest,
    FinishAuthenticationResponse, FinishRegistrationRequest, FinishRegistrationResponse,
    StartAuthenticationRequest, StartAuthenticationResponse, StartRegistrationRequest,
    StartRegistrationResponse, UpdateCredentialRequest,
};


// Handler modules will be added here as implementation progresses
// - auth.rs (login, logout, refresh)
// - token.rs (validate token)
// - oauth2.rs (OAuth2/OIDC endpoints)
// - profile.rs (user profile)
