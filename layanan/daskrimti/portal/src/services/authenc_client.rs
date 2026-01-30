//! Authenc gRPC client for IAM integration
//!
//! Provides a full gRPC client implementation for Authenc IAM service
//! with circuit breaker pattern for resilience.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tonic::Request;
use tonic::transport::Channel;
use tracing::info;

use crate::proto::authenc::v1::{
    AuthenticateRequest, CheckPermissionRequest, EnableMfaRequest, GetUserRequest, MfaMethod,
    RefreshTokenRequest, RevokeTokenRequest, TokenType, ValidateTokenRequest, VerifyMfaRequest,
    authenc_service_client::AuthencServiceClient,
};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for handling service failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    failure_threshold: u32,
    timeout: Duration,
    success_threshold: u32,
    success_count: Arc<RwLock<u32>>,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 3,
            success_count: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn can_execute(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                drop(state);
                if let Some(last_failure) = *self.last_failure_time.read().await
                    && last_failure.elapsed() >= self.timeout
                {
                    *self.state.write().await = CircuitBreakerState::HalfOpen;
                    *self.success_count.write().await = 0;
                    return true;
                }
                false
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    pub async fn record_success(&self) {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => {
                *self.failure_count.write().await = 0;
            }
            CircuitBreakerState::HalfOpen => {
                drop(state);
                let mut success_count = self.success_count.write().await;
                *success_count += 1;
                if *success_count >= self.success_threshold {
                    *self.state.write().await = CircuitBreakerState::Closed;
                    *self.failure_count.write().await = 0;
                }
            }
            CircuitBreakerState::Open => {
                *self.state.write().await = CircuitBreakerState::Closed;
                *self.failure_count.write().await = 0;
            }
        }
    }

    pub async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        if *failure_count >= self.failure_threshold {
            *self.state.write().await = CircuitBreakerState::Open;
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for Authenc client
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthencError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Service unavailable: {0}")]
    Unavailable(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<tonic::Status> for AuthencError {
    fn from(status: tonic::Status) -> Self {
        use tonic::Code;
        match status.code() {
            Code::Unauthenticated => {
                AuthencError::AuthenticationFailed(status.message().to_string())
            }
            Code::PermissionDenied => AuthencError::Unauthorized(status.message().to_string()),
            Code::NotFound => AuthencError::NotFound(status.message().to_string()),
            Code::Unavailable | Code::DeadlineExceeded => {
                AuthencError::Unavailable(status.message().to_string())
            }
            Code::InvalidArgument => AuthencError::InvalidRequest(status.message().to_string()),
            _ => AuthencError::Internal(format!("gRPC error: {}", status)),
        }
    }
}

/// Token validation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenValidation {
    pub valid: bool,
    pub user_id: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<i64>,
    pub error: Option<String>,
}

/// User info from Authenc
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub mfa_enabled: bool,
}

/// Authentication result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthResult {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: Option<UserInfo>,
    pub mfa_required: bool,
    pub mfa_setup_required: bool,
}

/// MFA setup result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MfaSetupResult {
    pub secret: String,
    pub qr_code_url: String,
    pub backup_codes: Vec<String>,
}

/// Permission check result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionCheck {
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Client for Authenc IAM service
pub struct AuthencClient {
    client: AuthencServiceClient<Channel>,
    circuit_breaker: CircuitBreaker,
    max_retries: u32,
    request_timeout: Duration,
}

impl AuthencClient {
    /// Create a new Authenc client
    pub async fn new(grpc_url: &str) -> anyhow::Result<Self> {
        info!("Connecting to Authenc at {}", grpc_url);
        let channel = Channel::from_shared(grpc_url.to_string())?.connect_lazy();

        let client = AuthencServiceClient::new(channel);

        Ok(Self {
            client,
            circuit_breaker: CircuitBreaker::new(),
            max_retries: 3,
            request_timeout: Duration::from_secs(90),  // 90 seconds timeout for gRPC requests
        })
    }

    /// Execute a request with circuit breaker and retry logic
    async fn execute_with_resilience<F, Fut, T>(&self, operation: F) -> Result<T, AuthencError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, AuthencError>>,
    {
        if !self.circuit_breaker.can_execute().await {
            return Err(AuthencError::Unavailable(
                "Circuit breaker is open".to_string(),
            ));
        }

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match operation().await {
                Ok(result) => {
                    self.circuit_breaker.record_success().await;
                    return Ok(result);
                }
                Err(err) => {
                    last_error = Some(err.clone());

                    // Don't retry on auth errors
                    if matches!(
                        err,
                        AuthencError::AuthenticationFailed(_) | AuthencError::Unauthorized(_)
                    ) {
                        self.circuit_breaker.record_failure().await;
                        return Err(err);
                    }

                    if attempt == self.max_retries {
                        self.circuit_breaker.record_failure().await;
                    }

                    // Exponential backoff
                    if attempt < self.max_retries {
                        let delay = Duration::from_millis(100 * (2_u64.pow(attempt)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AuthencError::Internal("Unknown error".to_string())))
    }

    /// Authenticate a user with username and password
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
        captcha_token: &str,
        mfa_code: Option<&str>,
    ) -> Result<AuthResult, AuthencError> {
        let username = username.to_string();
        let password = password.to_string();
        let captcha_token = Some(captcha_token.to_string());
        let mfa_code = mfa_code.map(|s| s.to_string());
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let username = username.clone();
            let password = password.clone();
            let captcha_token = captcha_token.clone();
            let mfa_code = mfa_code.clone();
            let mut client = client.clone();

            async move {
                let request = AuthenticateRequest {
                    username,
                    password,
                    mfa_code,
                    device_id: None,
                    metadata: Default::default(),
                    captcha_token,
                };

                let response = client
                    .authenticate(Request::new(request))
                    .await
                    .map_err(AuthencError::from)?;

                let resp = response.into_inner();

                let user = resp.user.map(|u| UserInfo {
                    user_id: u.user_id,
                    username: u.username,
                    email: u.email,
                    full_name: u.full_name,
                    roles: u.roles,
                    is_active: u.is_active,
                    mfa_enabled: u.mfa_enabled,
                });

                Ok(AuthResult {
                    access_token: resp.access_token,
                    refresh_token: resp.refresh_token,
                    token_type: resp.token_type,
                    expires_in: resp.expires_in,
                    user,
                    mfa_required: false,
                    mfa_setup_required: false,
                })
            }
        })
        .await
    }

    /// Validate a JWT token
    pub async fn validate_token(&self, token: &str) -> Result<TokenValidation, AuthencError> {
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let request = ValidateTokenRequest {
                    token,
                    required_scopes: vec![],
                };

                let response = client
                    .validate_token(Request::new(request))
                    .await
                    .map_err(AuthencError::from)?;

                let resp = response.into_inner();

                Ok(TokenValidation {
                    valid: resp.valid,
                    user_id: resp.user_id,
                    scopes: resp.scopes,
                    expires_at: resp.expires_at,
                    error: resp.error,
                })
            }
        })
        .await
    }

    /// Refresh an access token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<(String, String, i64), AuthencError> {
        let refresh_token = refresh_token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let refresh_token = refresh_token.clone();
            let mut client = client.clone();

            async move {
                let request = RefreshTokenRequest { refresh_token };

                let response = client
                    .refresh_token(Request::new(request))
                    .await
                    .map_err(AuthencError::from)?;

                let resp = response.into_inner();

                Ok((resp.access_token, resp.refresh_token, resp.expires_in))
            }
        })
        .await
    }

    /// Revoke a token
    pub async fn revoke_token(&self, token: &str, is_refresh: bool) -> Result<bool, AuthencError> {
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let token_type = if is_refresh {
                    TokenType::RefreshToken as i32
                } else {
                    TokenType::AccessToken as i32
                };

                let request = RevokeTokenRequest { token, token_type };

                let response = client
                    .revoke_token(Request::new(request))
                    .await
                    .map_err(AuthencError::from)?;

                Ok(response.into_inner().success)
            }
        })
        .await
    }

    /// Get user information
    pub async fn get_user(&self, token: &str, user_id: &str) -> Result<UserInfo, AuthencError> {
        let user_id = user_id.to_string();
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let user_id = user_id.clone();
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let request = GetUserRequest {
                    user_id: user_id.clone(),
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client.get_user(req).await.map_err(AuthencError::from)?;

                let user = response
                    .into_inner()
                    .user
                    .ok_or_else(|| AuthencError::NotFound("User not found".to_string()))?;

                Ok(UserInfo {
                    user_id: user.user_id,
                    username: user.username,
                    email: user.email,
                    full_name: user.full_name,
                    roles: user.roles,
                    is_active: user.is_active,
                    mfa_enabled: user.mfa_enabled,
                })
            }
        })
        .await
    }

    /// Get user permissions (extracts roles from user info)
    pub async fn get_user_permissions(
        &self,
        token: &str,
        user_id: &str,
    ) -> Result<Vec<String>, AuthencError> {
        let user = self.get_user(token, user_id).await?;
        Ok(user.roles)
    }

    /// Check if user has permission for a resource/action
    pub async fn check_permission(
        &self,
        token: &str,
        user_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<PermissionCheck, AuthencError> {
        let user_id = user_id.to_string();
        let resource = resource.to_string();
        let action = action.to_string();
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let user_id = user_id.clone();
            let resource = resource.clone();
            let action = action.clone();
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let request = CheckPermissionRequest {
                    user_id,
                    resource,
                    action,
                    context: Default::default(),
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client
                    .check_permission(req)
                    .await
                    .map_err(AuthencError::from)?;

                let resp = response.into_inner();

                Ok(PermissionCheck {
                    allowed: resp.allowed,
                    reason: resp.reason,
                })
            }
        })
        .await
    }

    /// Enable MFA for a user
    pub async fn enable_mfa(
        &self,
        token: &str,
        user_id: &str,
    ) -> Result<MfaSetupResult, AuthencError> {
        let user_id = user_id.to_string();
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let user_id = user_id.clone();
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let request = EnableMfaRequest {
                    user_id,
                    method: MfaMethod::Totp as i32,
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client.enable_mfa(req).await.map_err(AuthencError::from)?;

                let resp = response.into_inner();

                Ok(MfaSetupResult {
                    secret: resp.secret,
                    qr_code_url: resp.qr_code_url,
                    backup_codes: resp.backup_codes,
                })
            }
        })
        .await
    }

    /// Verify MFA code
    pub async fn verify_mfa(
        &self,
        token: &str,
        user_id: &str,
        code: &str,
    ) -> Result<bool, AuthencError> {
        let user_id = user_id.to_string();
        let code = code.to_string();
        let token = token.to_string();
        let client = self.client.clone();

        self.execute_with_resilience(|| {
            let user_id = user_id.clone();
            let code = code.clone();
            let token = token.clone();
            let mut client = client.clone();

            async move {
                let request = VerifyMfaRequest {
                    user_id,
                    code,
                    method: MfaMethod::Totp as i32,
                };

                let mut req = Request::new(request);
                req.metadata_mut().insert(
                    "authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                );

                let response = client.verify_mfa(req).await.map_err(AuthencError::from)?;

                Ok(response.into_inner().valid)
            }
        })
        .await
    }

    /// Generate CAPTCHA challenge via gRPC
    pub async fn generate_captcha(
        &self,
        challenge_type: &str,
        difficulty: u8,
        session_id: &str,
    ) -> Result<crate::handlers::captcha::CaptchaResponse, AuthencError> {
        let challenge_type_str = challenge_type.to_string();
        let session_id_str = session_id.to_string();
        let client = self.client.clone();
        let request_timeout = self.request_timeout;

        self.execute_with_resilience(|| {
            let challenge_type = challenge_type_str.clone();
            let session_id = session_id_str.clone();
            let mut client = client.clone();

            async move {
                // Wrap the gRPC call with explicit timeout
                let result = tokio::time::timeout(request_timeout, async {
                    // Convert string to proto ChallengeType enum
                    let proto_challenge_type = match challenge_type.to_lowercase().as_str() {
                        "audio" => 2,      // ChallengeType::Audio
                        "behavioral" => 3, // ChallengeType::Behavioral
                        "logical" => 4,    // ChallengeType::Logical
                        "hybrid" => 5,     // ChallengeType::Hybrid
                        _ => 1,            // ChallengeType::Visual (default)
                    };

                    use crate::proto::authenc::v1::CaptchaChallengeRequest;

                    let request = CaptchaChallengeRequest {
                        session_id,
                        challenge_type: proto_challenge_type,
                        difficulty: difficulty as u32,
                        metadata: std::collections::HashMap::new(),
                    };

                    let response = client
                        .generate_captcha_challenge(Request::new(request))
                        .await
                        .map_err(AuthencError::from)?;

                    let resp = response.into_inner();

                    // Convert proto response to handler response
                    Ok::<_, AuthencError>(crate::handlers::captcha::CaptchaResponse {
                        challenge_id: resp.challenge_id,
                        challenge_type,
                        challenge_data: resp.challenge_data,
                        difficulty: resp.difficulty as u8,
                        expires_at: resp.expires_at,
                    })
                })
                .await;

                // Handle timeout error
                match result {
                    Ok(response) => response,
                    Err(_) => Err(AuthencError::Unavailable(
                        format!("CAPTCHA generation timed out after {:?}", request_timeout)
                    )),
                }
            }
        })
        .await
    }

    /// Verify CAPTCHA response via gRPC
    pub async fn verify_captcha(
        &self,
        challenge_id: &str,
        answer: &str,
        session_id: &str,
        behavioral_data: Option<serde_json::Value>,
    ) -> Result<Option<String>, AuthencError> {
        let challenge_id_str = challenge_id.to_string();
        let answer_str = answer.to_string();
        let session_id_str = session_id.to_string();
        let behavior_bytes = behavioral_data
            .and_then(|v| serde_json::to_vec(&v).ok())
            .unwrap_or_default();
        let client = self.client.clone();
        let request_timeout = self.request_timeout;

        self.execute_with_resilience(|| {
            let challenge_id = challenge_id_str.clone();
            let answer = answer_str.clone();
            let session_id = session_id_str.clone();
            let behavior = behavior_bytes.clone();
            let mut client = client.clone();

            async move {
                // Wrap the gRPC call with explicit timeout
                let result = tokio::time::timeout(request_timeout, async {
                    use crate::proto::authenc::v1::CaptchaVerificationRequest;

                    let request = CaptchaVerificationRequest {
                        challenge_id,
                        answer,
                        behavioral_data: behavior,
                        session_id,
                    };

                    let response = client
                        .verify_captcha_challenge(Request::new(request))
                        .await
                        .map_err(AuthencError::from)?;

                    let resp = response.into_inner();

                    if resp.success {
                        Ok::<_, AuthencError>(Some(resp.verification_token))
                    } else {
                        Ok(None)
                    }
                })
                .await;

                // Handle timeout error
                match result {
                    Ok(response) => response,
                    Err(_) => Err(AuthencError::Unavailable(
                        format!("CAPTCHA verification timed out after {:?}", request_timeout)
                    )),
                }
            }
        })
        .await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool, AuthencError> {
        Ok(self.circuit_breaker.can_execute().await)
    }
}
