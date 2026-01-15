//! Authenc gRPC Client
//!
//! Client wrapper for Authenc IAM service providing:
//! - Authentication (login, validate, refresh)
//! - User management
//! - Role management

use super::generated::authenc_v1::{
    authenc_service_client::AuthencServiceClient,
    AuthenticateRequest, AuthenticateResponse,
    ValidateTokenRequest, ValidateTokenResponse,
    RefreshTokenRequest, RefreshTokenResponse,
    CreateUserRequest, CreateUserResponse,
    GetUserRequest, GetUserResponse,
    ListUsersRequest, ListUsersResponse,
    ListRolesRequest, ListRolesResponse,
    EnableMfaRequest, EnableMfaResponse,
    VerifyMfaRequest, VerifyMfaResponse,
    DisableMfaRequest, DisableMfaResponse,
    MfaMethod,
};
use tonic::transport::{Channel, Endpoint};
use tracing::{debug, info};

/// Error type for Authenc client operations
#[derive(Debug, thiserror::Error)]
pub enum AuthencError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
}

/// Authenc gRPC Client wrapper
#[derive(Debug, Clone)]
pub struct AuthencClient {
    client: AuthencServiceClient<Channel>,
}

impl AuthencClient {
    /// Create a new Authenc client
    pub async fn new(endpoint: &str) -> Result<Self, AuthencError> {
        info!("Connecting to Authenc gRPC at {}", endpoint);

        let endpoint = Endpoint::from_shared(endpoint.to_string())
            .map_err(|e| AuthencError::Connection(e.to_string()))?;

        let channel = endpoint
            .connect()
            .await
            .map_err(|e| AuthencError::Connection(e.to_string()))?;

        Ok(Self {
            client: AuthencServiceClient::new(channel),
        })
    }

    /// Authenticate a user with username and password
    pub async fn authenticate(
        &mut self,
        username: &str,
        password: &str,
        mfa_code: Option<&str>,
        captcha_token: Option<&str>,
    ) -> Result<AuthenticateResponse, AuthencError> {
        debug!("Authenticating user: {}", username);

        let request = tonic::Request::new(AuthenticateRequest {
            username: username.to_string(),
            password: password.to_string(),
            mfa_code: mfa_code.map(|s| s.to_string()),
            device_id: None,
            metadata: Default::default(),
            captcha_token: captcha_token.map(|s| s.to_string()),
        });

        let response = self.client.authenticate(request).await?;
        info!("User {} authenticated successfully", username);
        Ok(response.into_inner())
    }

    /// Validate a JWT token
    pub async fn validate_token(
        &mut self,
        token: &str,
        required_scopes: Vec<String>,
    ) -> Result<ValidateTokenResponse, AuthencError> {
        debug!("Validating token");

        let request = tonic::Request::new(ValidateTokenRequest {
            token: token.to_string(),
            required_scopes,
        });

        let response = self.client.validate_token(request).await?;
        Ok(response.into_inner())
    }

    /// Refresh an access token
    pub async fn refresh_token(
        &mut self,
        refresh_token: &str,
    ) -> Result<RefreshTokenResponse, AuthencError> {
        debug!("Refreshing token");

        let request = tonic::Request::new(RefreshTokenRequest {
            refresh_token: refresh_token.to_string(),
        });

        let response = self.client.refresh_token(request).await?;
        Ok(response.into_inner())
    }

    /// Create a new user
    pub async fn create_user(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
        full_name: Option<&str>,
        roles: Vec<String>,
    ) -> Result<CreateUserResponse, AuthencError> {
        debug!("Creating user: {}", username);

        let request = tonic::Request::new(CreateUserRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            full_name: full_name.map(|s| s.to_string()),
            roles,
            metadata: Default::default(),
        });

        let response = self.client.create_user(request).await?;
        info!("User {} created with ID: {}", username, response.get_ref().user_id);
        Ok(response.into_inner())
    }

    /// Get user by ID
    pub async fn get_user(
        &mut self,
        user_id: &str,
    ) -> Result<GetUserResponse, AuthencError> {
        debug!("Getting user: {}", user_id);

        let request = tonic::Request::new(GetUserRequest {
            user_id: user_id.to_string(),
        });

        let response = self.client.get_user(request).await?;
        Ok(response.into_inner())
    }

    /// List users with pagination
    pub async fn list_users(
        &mut self,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Option<&str>,
    ) -> Result<ListUsersResponse, AuthencError> {
        debug!("Listing users");

        let request = tonic::Request::new(ListUsersRequest {
            limit,
            offset,
            filter: filter.map(|s| s.to_string()),
        });

        let response = self.client.list_users(request).await?;
        Ok(response.into_inner())
    }

    /// List available roles
    pub async fn list_roles(
        &mut self,
        user_id: Option<&str>,
    ) -> Result<ListRolesResponse, AuthencError> {
        debug!("Listing roles");

        let request = tonic::Request::new(ListRolesRequest {
            user_id: user_id.map(|s| s.to_string()),
        });

        let response = self.client.list_roles(request).await?;
        Ok(response.into_inner())
    }

    /// Enable MFA for a user
    pub async fn enable_mfa(
        &mut self,
        user_id: &str,
        method: MfaMethod,
    ) -> Result<EnableMfaResponse, AuthencError> {
        debug!("Enabling MFA for user: {}", user_id);

        let request = tonic::Request::new(EnableMfaRequest {
            user_id: user_id.to_string(),
            method: method.into(),
        });

        let response = self.client.enable_mfa(request).await?;
        Ok(response.into_inner())
    }

    /// Verify MFA Setup
    pub async fn verify_mfa(
        &mut self,
        user_id: &str,
        code: &str,
        method: MfaMethod,
    ) -> Result<VerifyMfaResponse, AuthencError> {
        debug!("Verifying MFA for user: {}", user_id);

        let request = tonic::Request::new(VerifyMfaRequest {
            user_id: user_id.to_string(),
            code: code.to_string(),
            method: method.into(),
        });

        let response = self.client.verify_mfa(request).await?;
        Ok(response.into_inner())
    }

    /// Disable MFA for a user
    pub async fn disable_mfa(
        &mut self,
        user_id: &str,
        password: &str,
    ) -> Result<DisableMfaResponse, AuthencError> {
        debug!("Disabling MFA for user: {}", user_id);

        let request = tonic::Request::new(DisableMfaRequest {
            user_id: user_id.to_string(),
            password: password.to_string(),
        });

        let response = self.client.disable_mfa(request).await?;
        Ok(response.into_inner())
    }
}
