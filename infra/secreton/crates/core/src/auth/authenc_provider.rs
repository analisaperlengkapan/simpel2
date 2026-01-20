//! Authenc Authentication Provider for Secreton
//!
//! This module provides authentication integration with the Authenc IAM service,
//! enabling Secreton to validate tokens and authenticate users through the
//! SIMKARI super app authentication system.

use crate::error::CoreError;
use crate::utils::base64_encode;
use crate::models::auth::UserInfo;
use crate::resilience::{CircuitBreaker, CircuitBreakerConfig};
use crate::utils::correlation::CorrelationContext;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Post-quantum signature types supported by Authenc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PqSignature {
    /// ML-DSA signature
    MlDsa {
        signature: Vec<u8>,
        public_key: Vec<u8>,
    },
    /// SPHINCS+ signature
    SphincsPlusShake256 {
        signature: Vec<u8>,
        public_key: Vec<u8>,
    },
    /// Hybrid classical + post-quantum signature
    Hybrid {
        classical_signature: Vec<u8>,
        pq_signature: Box<PqSignature>,
    },
}

/// Credentials for user authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// User identifier (NIP or UUID)
    pub user_id: String,
    /// Authentication token from Authenc
    pub token: String,
    /// Optional additional authentication factors
    pub additional_factors: Option<HashMap<String, String>>,
}

/// Authentication result from Authenc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    /// Whether authentication was successful
    pub success: bool,
    /// User information if authenticated
    pub user_info: Option<UserInfo>,
    /// Error message if authentication failed
    pub error_message: Option<String>,
    /// Session duration in seconds
    pub session_duration: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Token validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidation {
    /// Whether token is valid
    pub valid: bool,
    /// Token expiration time
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// User associated with the token
    pub user_info: Option<UserInfo>,
    /// Token scopes/permissions
    pub scopes: Vec<String>,
    /// Error message if validation failed
    pub error_message: Option<String>,
}

/// User information for Authenc authorization checks
///
/// **Note**: This is specific to Indonesian government Authenc integration.
/// For general user operations, use `crate::models::user::User` (the canonical User model).
///
/// This struct contains Authenc-specific fields (NIP, satker_code) that may not
/// be present in other authentication providers.
#[deprecated(
    since = "1.1.0",
    note = "Renamed to AuthencUserInfo to avoid confusion with canonical User model in crate::models::user. Use AuthencUserInfo instead."
)]
pub type User = AuthencUserInfo;

/// User information from Authenc provider (Indonesian government authentication)
///
/// This struct represents user data returned from the Authenc authentication system,
/// which is specific to Indonesian government agencies. It includes government-specific
/// fields like NIP (employee ID number) and satker_code (organizational unit code).
///
/// **Important**: This is NOT the canonical User model. For general user operations,
/// use `crate::models::user::User` instead. This struct is only for Authenc integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthencUserInfo {
    /// User identifier (as String for Authenc compatibility)
    pub id: String,
    /// User's NIP (Nomor Induk Pegawai - Indonesian civil servant ID)
    pub nip: Option<String>,
    /// User's full name
    pub name: String,
    /// User's email address
    pub email: Option<String>,
    /// Satker code (Satuan Kerja - organizational unit code)
    pub satker_code: Option<String>,
    /// User roles (from Authenc)
    pub roles: Vec<String>,
    /// User permissions (from Authenc)
    pub permissions: Vec<String>,
}

/// Token cache entry
#[derive(Debug, Clone)]
struct TokenCacheEntry {
    validation: TokenValidation,
    cached_at: Instant,
    ttl: Duration,
}

impl TokenCacheEntry {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// Token validation cache
#[derive(Debug)]
pub struct TokenCache {
    cache: Arc<RwLock<HashMap<String, TokenCacheEntry>>>,
    default_ttl: Duration,
    max_entries: usize,
}

impl TokenCache {
    /// Create a new token cache
    pub fn new(default_ttl: Duration, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_entries,
        }
    }

    /// Get cached token validation
    pub async fn get(&self, token: &str) -> Option<TokenValidation> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(token)
            && !entry.is_expired()
        {
            return Some(entry.validation.clone());
        }
        None
    }

    /// Cache token validation result
    pub async fn put(&self, token: String, validation: TokenValidation) {
        let mut cache = self.cache.write().await;

        // Remove expired entries and enforce max size
        cache.retain(|_, entry| !entry.is_expired());
        if cache.len() >= self.max_entries {
            // Remove oldest entry
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }

        let entry = TokenCacheEntry {
            validation,
            cached_at: Instant::now(),
            ttl: self.default_ttl,
        };
        cache.insert(token, entry);
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        self.cache.write().await.clear();
    }
}

/// Validation cache for user permissions
pub type ValidationCache = TokenCache;

/// Post-quantum signature validator
#[derive(Debug)]
pub struct PostQuantumValidator {
    /// Supported algorithms
    supported_algorithms: Vec<String>,
}

impl PostQuantumValidator {
    /// Create a new post-quantum validator
    pub fn new() -> Self {
        Self {
            supported_algorithms: vec![
                "ml-dsa".to_string(),
                "sphincs-plus-shake256".to_string(),
                "hybrid-ml-dsa".to_string(),
            ],
        }
    }

    /// Validate a post-quantum signature
    pub async fn validate_signature(
        &self,
        signature: &PqSignature,
        data: &[u8],
    ) -> Result<bool, CoreError> {
        match signature {
            PqSignature::MlDsa {
                signature: sig,
                public_key,
            } => {
                // TODO: Implement ML-DSA signature verification
                // This would use a post-quantum cryptography library
                log::warn!("ML-DSA signature validation not yet implemented");
                Ok(false)
            }
            PqSignature::SphincsPlusShake256 {
                signature: sig,
                public_key,
            } => {
                // TODO: Implement SPHINCS+ signature verification
                log::warn!("SPHINCS+ signature validation not yet implemented");
                Ok(false)
            }
            PqSignature::Hybrid {
                classical_signature,
                pq_signature,
            } => {
                // TODO: Implement hybrid signature verification
                // Verify both classical and post-quantum signatures
                log::warn!("Hybrid signature validation not yet implemented");
                Ok(false)
            }
        }
    }
}

/// Authentication provider trait for pluggable auth backends
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Authenticate a user with credentials
    async fn authenticate_user(
        &self,
        user_id: &str,
        credentials: &Credentials,
    ) -> Result<AuthResult, CoreError>;

    /// Validate an authentication token
    async fn validate_token(&self, token: &str) -> Result<TokenValidation, CoreError>;

    /// Check if user has permission to access a resource
    async fn check_resource_permissions(
        &self,
        user: &User,
        resource_id: &str,
    ) -> Result<bool, CoreError>;

    /// Validate application access to a resource
    async fn validate_application_access(
        &self,
        app_id: &str,
        resource: &str,
    ) -> Result<bool, CoreError>;

    /// Validate post-quantum signature
    async fn validate_pq_signature(
        &self,
        signature: &PqSignature,
        data: &[u8],
    ) -> Result<bool, CoreError>;
}

/// Authenc authentication provider implementation
#[derive(Debug)]
pub struct AuthencAuthProvider {
    /// Authenc service endpoint
    authenc_endpoint: String,
    /// Client certificate for mTLS
    client_cert: Option<String>,
    /// HTTP client for requests
    client: Client,
    /// Token validation cache
    token_cache: TokenCache,
    /// Permission validation cache
    validation_cache: ValidationCache,
    /// Post-quantum signature validator
    pq_validator: PostQuantumValidator,
    /// Request timeout
    request_timeout: Duration,
    /// Circuit breaker for resilience
    circuit_breaker: CircuitBreaker,
    /// Current correlation context (thread-local)
    correlation_context: Arc<RwLock<Option<CorrelationContext>>>,
}

impl AuthencAuthProvider {
    /// Create a new Authenc authentication provider
    pub fn new(authenc_endpoint: String, client_cert: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            authenc_endpoint,
            client_cert,
            client,
            token_cache: TokenCache::new(Duration::from_secs(300), 1000), // 5 min TTL, 1000 entries
            validation_cache: ValidationCache::new(Duration::from_secs(60), 500), // 1 min TTL, 500 entries
            pq_validator: PostQuantumValidator::new(),
            request_timeout: Duration::from_secs(30),
            circuit_breaker: CircuitBreaker::new(),
            correlation_context: Arc::new(RwLock::new(None)),
        }
    }

    /// Create provider with custom configuration
    pub fn with_config(
        authenc_endpoint: String,
        client_cert: Option<String>,
        cache_ttl: Duration,
        max_cache_entries: usize,
        request_timeout: Duration,
    ) -> Self {
        let client = Client::builder()
            .timeout(request_timeout)
            .build()
            .unwrap_or_else(|_| Client::new());

        let circuit_breaker_config = CircuitBreakerConfig {
            max_failures: 5,
            timeout: Duration::from_secs(30),
            reset_timeout: Duration::from_secs(60),
        };

        Self {
            authenc_endpoint,
            client_cert,
            client,
            token_cache: TokenCache::new(cache_ttl, max_cache_entries),
            validation_cache: ValidationCache::new(cache_ttl / 5, max_cache_entries / 2),
            pq_validator: PostQuantumValidator::new(),
            request_timeout,
            circuit_breaker: CircuitBreaker::with_config(circuit_breaker_config),
            correlation_context: Arc::new(RwLock::new(None)),
        }
    }

    /// Set correlation context for subsequent requests
    pub async fn set_correlation_context(&self, context: CorrelationContext) {
        *self.correlation_context.write().await = Some(context);
    }

    /// Get current correlation context
    pub async fn get_correlation_context(&self) -> Option<CorrelationContext> {
        self.correlation_context.read().await.clone()
    }

    /// Clear correlation context
    pub async fn clear_correlation_context(&self) {
        *self.correlation_context.write().await = None;
    }

    /// Execute HTTP request with circuit breaker and error handling
    async fn execute_request<T>(&self, request: reqwest::RequestBuilder) -> Result<T, CoreError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Check circuit breaker
        if !self.circuit_breaker.can_execute().await {
            tracing::warn!("Circuit breaker is open, blocking request to Authenc");
            return Err(CoreError::service_unavailable(
                "Authenc service unavailable (circuit breaker open)",
            ));
        }

        // Execute request
        let result = self.execute_request_internal(request).await;

        // Record result in circuit breaker
        match &result {
            Ok(_) => {
                self.circuit_breaker.record_success().await;
            }
            Err(e) => {
                // Only record failure for network/service errors, not auth errors
                if matches!(e, CoreError::ServiceUnavailable { .. }) {
                    self.circuit_breaker.record_failure().await;
                }
            }
        }

        result
    }

    ///Internal request execution without circuit breaker
    async fn execute_request_internal<T>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, CoreError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Add correlation headers if context is set
        let request = if let Some(ctx) = self.get_correlation_context().await {
            ctx.add_to_headers(request)
        } else {
            // No correlation context, generate one for this request
            let ctx = CorrelationContext::new();
            ctx.add_to_headers(request)
        };

        let response = request
            .send()
            .await
            .map_err(|e| CoreError::network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if status == 401 {
            return Err(CoreError::authentication("Invalid or expired token"));
        } else if status == 403 {
            return Err(CoreError::authorization("Access denied"));
        } else if status == 404 {
            return Err(CoreError::not_found("Resource not found"));
        } else if !status.is_success() {
            return Err(CoreError::service_unavailable(format!("HTTP {}", status)));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| CoreError::validation(format!("Invalid response format: {}", e)))
    }
}

#[async_trait]
impl AuthProvider for AuthencAuthProvider {
    async fn authenticate_user(
        &self,
        user_id: &str,
        credentials: &Credentials,
    ) -> Result<AuthResult, CoreError> {
        let url = format!("{}/v1/auth/authenticate", self.authenc_endpoint);

        let payload = serde_json::json!({
            "user_id": user_id,
            "token": credentials.token,
            "additional_factors": credentials.additional_factors,
            "service": "secreton"
        });

        let request = self.client.post(&url).json(&payload);

        // Add client certificate if available
        if let Some(_cert) = &self.client_cert {
            // TODO: Add mTLS client certificate configuration
            log::debug!("mTLS client certificate configuration not yet implemented");
        }

        #[derive(Deserialize)]
        struct AuthResponse {
            success: bool,
            user_info: Option<UserInfo>,
            error_message: Option<String>,
            session_duration: Option<u64>,
            metadata: Option<HashMap<String, String>>,
        }

        let response: AuthResponse = self.execute_request(request).await?;

        Ok(AuthResult {
            success: response.success,
            user_info: response.user_info,
            error_message: response.error_message,
            session_duration: response.session_duration,
            metadata: response.metadata.unwrap_or_default(),
        })
    }

    async fn validate_token(&self, token: &str) -> Result<TokenValidation, CoreError> {
        // Check cache first
        if let Some(cached_validation) = self.token_cache.get(token).await {
            return Ok(cached_validation);
        }

        let url = format!("{}/v1/auth/validate-token", self.authenc_endpoint);

        let payload = serde_json::json!({
            "token": token,
            "service": "secreton"
        });

        let request = self.client.post(&url).json(&payload);

        #[derive(Deserialize)]
        struct TokenValidationResponse {
            valid: bool,
            expires_at: Option<String>,
            user_info: Option<UserInfo>,
            scopes: Option<Vec<String>>,
            error_message: Option<String>,
        }

        let response: TokenValidationResponse = self.execute_request(request).await?;

        let expires_at = response
            .expires_at
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let validation = TokenValidation {
            valid: response.valid,
            expires_at,
            user_info: response.user_info,
            scopes: response.scopes.unwrap_or_default(),
            error_message: response.error_message,
        };

        // Cache the result if valid
        if validation.valid {
            self.token_cache
                .put(token.to_string(), validation.clone())
                .await;
        }

        Ok(validation)
    }

    async fn check_resource_permissions(
        &self,
        user: &User,
        resource_id: &str,
    ) -> Result<bool, CoreError> {
        let cache_key = format!("{}:{}", user.id, resource_id);

        // Check cache first
        if let Some(cached_result) = self.validation_cache.get(&cache_key).await {
            return Ok(cached_result.valid);
        }

        let url = format!("{}/v1/auth/check-permissions", self.authenc_endpoint);

        let payload = serde_json::json!({
            "user_id": user.id,
            "resource_id": resource_id,
            "operation": "read",
            "context": {
                "satker_code": user.satker_code,
                "roles": user.roles,
                "service": "secreton"
            }
        });

        let request = self.client.post(&url).json(&payload);

        #[derive(Deserialize)]
        struct PermissionResponse {
            allowed: bool,
            reason: Option<String>,
        }

        let response: PermissionResponse = self.execute_request(request).await?;

        // Cache the result
        let validation = TokenValidation {
            valid: response.allowed,
            expires_at: None,
            user_info: None,
            scopes: vec![],
            error_message: response.reason,
        };
        self.validation_cache.put(cache_key, validation).await;

        Ok(response.allowed)
    }

    async fn validate_application_access(
        &self,
        app_id: &str,
        resource: &str,
    ) -> Result<bool, CoreError> {
        let url = format!("{}/v1/auth/validate-app-access", self.authenc_endpoint);

        let payload = serde_json::json!({
            "app_id": app_id,
            "resource": resource,
            "service": "secreton"
        });

        let request = self.client.post(&url).json(&payload);

        #[derive(Deserialize)]
        struct AppAccessResponse {
            allowed: bool,
            reason: Option<String>,
        }

        let response: AppAccessResponse = self.execute_request(request).await?;

        Ok(response.allowed)
    }

    async fn validate_pq_signature(
        &self,
        signature: &PqSignature,
        data: &[u8],
    ) -> Result<bool, CoreError> {
        // First, try local validation
        match self.pq_validator.validate_signature(signature, data).await {
            Ok(true) => return Ok(true),
            Ok(false) => {
                // If local validation fails, try remote validation with Authenc
                log::debug!("Local PQ signature validation failed, trying remote validation");
            }
            Err(e) => {
                log::warn!("Local PQ signature validation error: {}", e);
            }
        }

        // Remote validation with Authenc
        let url = format!("{}/v1/crypto/validate-pq-signature", self.authenc_endpoint);

        let signature_data = match signature {
            PqSignature::MlDsa {
                signature: sig,
                public_key,
            } => {
                serde_json::json!({
                    "algorithm": "ml-dsa",
                    "signature": base64_encode(sig),
                    "public_key": base64_encode(public_key),
                    "data": base64_encode(data)
                })
            }
            PqSignature::SphincsPlusShake256 {
                signature: sig,
                public_key,
            } => {
                serde_json::json!({
                    "algorithm": "sphincs-plus-shake256",
                    "signature": base64_encode(sig),
                    "public_key": base64_encode(public_key),
                    "data": base64_encode(data)
                })
            }
            PqSignature::Hybrid {
                classical_signature,
                pq_signature,
            } => {
                serde_json::json!({
                    "algorithm": "hybrid",
                    "classical_signature": base64_encode(classical_signature),
                    "pq_signature": pq_signature,
                    "data": base64_encode(data)
                })
            }
        };

        let request = self.client.post(&url).json(&signature_data);

        #[derive(Deserialize)]
        struct SignatureValidationResponse {
            valid: bool,
            algorithm: String,
            error_message: Option<String>,
        }

        let response: SignatureValidationResponse = self.execute_request(request).await?;

        Ok(response.valid)
    }
}

impl Default for PostQuantumValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_cache() {
        let cache = TokenCache::new(Duration::from_millis(100), 10);

        let validation = TokenValidation {
            valid: true,
            expires_at: None,
            user_info: None,
            scopes: vec!["read".to_string()],
            error_message: None,
        };

        // Test cache put and get
        cache
            .put("test_token".to_string(), validation.clone())
            .await;
        let cached = cache.get("test_token").await;
        assert!(cached.is_some());
        assert!(cached.unwrap().valid);

        // Test expiration
        tokio::time::sleep(Duration::from_millis(150)).await;
        let expired = cache.get("test_token").await;
        assert!(expired.is_none());
    }

    #[test]
    fn test_pq_signature_types() {
        let ml_dsa_sig = PqSignature::MlDsa {
            signature: vec![1, 2, 3],
            public_key: vec![4, 5, 6],
        };

        match ml_dsa_sig {
            PqSignature::MlDsa {
                signature,
                public_key,
            } => {
                assert_eq!(signature, vec![1, 2, 3]);
                assert_eq!(public_key, vec![4, 5, 6]);
            }
            _ => panic!("Wrong signature type"),
        }
    }

    #[tokio::test]
    async fn test_authenc_provider_creation() {
        let provider = AuthencAuthProvider::new("https://authenc.example.com".to_string(), None);

        assert_eq!(provider.authenc_endpoint, "https://authenc.example.com");
        assert!(provider.client_cert.is_none());
    }
}
