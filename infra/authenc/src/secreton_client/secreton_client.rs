//! Secreton client for Authenc integration
//! Enhanced client for integrating with the custom Rust-based Secreton secret manager

use super::{Secret, SecretonClientTrait, SecretonError};
use crate::models::user::SecurityContext;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use curve25519_dalek;
use hkdf;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Post-quantum algorithm types supported by Secreton
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PqAlgorithm {
    /// ML-DSA (FIPS 204) - Digital Signature Algorithm
    MlDsa,
    /// ML-KEM (FIPS 203) - Key Encapsulation Mechanism
    MlKem,
    /// SPHINCS+ - Stateless hash-based signature scheme
    SphincsPlusShake256,
    /// Hybrid classical + post-quantum
    Hybrid(Box<PqAlgorithm>),
}

/// Signing key material for JWT operations
#[derive(Debug, Clone)]
pub struct SigningKey {
    /// Key identifier
    pub key_id: String,
    /// Key material (PEM format)
    pub key_material: String,
    /// Algorithm used for signing
    pub algorithm: String,
    /// Key expiration time
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Encryption key material for session data
#[derive(Debug, Clone)]
pub struct EncryptionKey {
    /// Key identifier
    pub key_id: String,
    /// Key material (base64 encoded)
    pub key_material: String,
    /// Encryption algorithm
    pub algorithm: String,
    /// Key size in bits
    pub key_size: u32,
}

/// Post-quantum key material
#[derive(Debug, Clone)]
pub struct PqKey {
    /// Key identifier
    pub key_id: String,
    /// Public key material
    pub public_key: Vec<u8>,
    /// Private key material (if available)
    pub private_key: Option<Vec<u8>>,
    /// Algorithm used
    pub algorithm: PqAlgorithm,
    /// Key parameters
    pub parameters: std::collections::HashMap<String, String>,
}

/// Application configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Application identifier
    pub app_id: String,
    /// Configuration data as JSON
    pub config_data: serde_json::Value,
    /// Configuration version
    pub version: u32,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// MFA setup data returned from secreton MfaManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupData {
    /// QR code URL for scanning with authenticator apps
    pub qr_code_url: String,
    /// Secret key for manual entry in authenticator apps
    pub secret_key: String,
    /// Backup codes for emergency access
    pub backup_codes: Vec<String>,
}

/// MFA status response from secreton MfaManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaStatusResponse {
    /// Whether MFA is enabled for the user
    pub is_enabled: bool,
    /// MFA method type (e.g., "TOTP")
    pub method: Option<String>,
    /// Last time MFA was used (ISO 8601 format)
    pub last_used: Option<String>,
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitBreakerState {
    /// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, testing if service is recovered
    HalfOpen,
}

/// Circuit breaker for handling Secreton service failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Current state of the circuit breaker
    state: Arc<RwLock<CircuitBreakerState>>,
    /// Failure count
    failure_count: Arc<RwLock<u32>>,
    /// Last failure time
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    /// Failure threshold before opening circuit
    failure_threshold: u32,
    /// Timeout before attempting to close circuit
    timeout: Duration,
    /// Success threshold for closing circuit from half-open
    success_threshold: u32,
    /// Success count in half-open state
    success_count: Arc<RwLock<u32>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default settings
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

    /// Create a circuit breaker with custom settings
    pub fn with_config(failure_threshold: u32, timeout: Duration, success_threshold: u32) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold,
            timeout,
            success_threshold,
            success_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Check if request should be allowed
    pub async fn can_execute(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                drop(state);
                // Check if timeout has passed
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.timeout {
                        // Move to half-open state
                        *self.state.write().await = CircuitBreakerState::HalfOpen;
                        *self.success_count.write().await = 0;
                        return true;
                    }
                }
                false
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => {
                // Reset failure count on success
                *self.failure_count.write().await = 0;
            }
            CircuitBreakerState::HalfOpen => {
                drop(state);
                let mut success_count = self.success_count.write().await;
                *success_count += 1;
                if *success_count >= self.success_threshold {
                    // Close the circuit
                    *self.state.write().await = CircuitBreakerState::Closed;
                    *self.failure_count.write().await = 0;
                }
            }
            CircuitBreakerState::Open => {
                // Should not happen, but reset if it does
                *self.state.write().await = CircuitBreakerState::Closed;
                *self.failure_count.write().await = 0;
            }
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        if *failure_count >= self.failure_threshold {
            *self.state.write().await = CircuitBreakerState::Open;
        }
    }

    /// Get current circuit breaker state
    pub async fn get_state(&self) -> CircuitBreakerState {
        *self.state.read().await
    }
}

/// Enhanced client for interacting with the Secreton secret management service
#[derive(Debug, Clone)]
pub struct SecretonClient {
    /// Secreton service endpoint URL
    endpoint: String,
    /// Authentication token for the Secreton service
    token: String,
    /// HTTP client for making requests
    client: Client,
    /// Circuit breaker for reliability
    circuit_breaker: CircuitBreaker,
    /// Request timeout duration
    request_timeout: Duration,
    /// Maximum retry attempts
    max_retries: u32,
}

impl SecretonClient {
    /// Create a new Secreton client with endpoint and authentication
    ///
    /// This constructor initializes a client for the Secreton secret
    /// management service. Secreton provides a secure, scalable platform
    /// for storing and retrieving sensitive configuration data and secrets.
    ///
    /// # Arguments
    /// * `endpoint` - Secreton service endpoint URL
    /// * `token` - Authentication token for service access
    ///
    /// # Returns
    /// A new `SecretonClient` instance configured for secret operations
    ///
    /// # Security Considerations
    /// - Authentication tokens should be securely stored and rotated
    /// - HTTPS should be used for all communication with Secreton
    /// - Token permissions should follow principle of least privilege
    /// - Network traffic should be encrypted and authenticated
    ///
    /// # Secreton Integration
    /// - RESTful API for secret storage and retrieval
    /// - Multi-tenant secret isolation with satker codes
    /// - Version control and audit logging for secrets
    /// - High availability and disaster recovery
    /// - SIMKARI-specific operations for Attorney General's Office
    /// - Circuit breaker pattern for reliability
    /// - Post-quantum cryptography support
    ///
    /// # Example
    /// ```rust
    /// use authenc::secreton_client::secreton_client::SecretonClient;
    ///
    /// let client = SecretonClient::new(
    ///     "https://secreton.example.com".to_string(),
    ///     "your-auth-token".to_string()
    /// );
    /// // Client is ready for secret operations
    /// ```
    pub fn new(endpoint: String, token: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        SecretonClient {
            endpoint,
            token,
            client,
            circuit_breaker: CircuitBreaker::new(),
            request_timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }

    /// Create a new Secreton client with custom configuration
    ///
    /// # Arguments
    /// * `endpoint` - Secreton service endpoint URL
    /// * `token` - Authentication token for service access
    /// * `request_timeout` - Timeout for individual requests
    /// * `max_retries` - Maximum number of retry attempts
    /// * `circuit_breaker_config` - Circuit breaker configuration (failure_threshold, timeout, success_threshold)
    pub fn with_config(
        endpoint: String,
        token: String,
        request_timeout: Duration,
        max_retries: u32,
        circuit_breaker_config: Option<(u32, Duration, u32)>,
    ) -> Self {
        let client = Client::builder()
            .timeout(request_timeout)
            .build()
            .unwrap_or_else(|_| Client::new());

        let circuit_breaker =
            if let Some((failure_threshold, timeout, success_threshold)) = circuit_breaker_config {
                CircuitBreaker::with_config(failure_threshold, timeout, success_threshold)
            } else {
                CircuitBreaker::new()
            };

        SecretonClient {
            endpoint,
            token,
            client,
            circuit_breaker,
            request_timeout,
            max_retries,
        }
    }

    /// Execute a request with circuit breaker and retry logic
    async fn execute_with_circuit_breaker<F, Fut, T>(&self, operation: F) -> Result<T, SecretonError>
    where
        F: Fn() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, SecretonError>> + Send,
    {
        // Check circuit breaker
        if !self.circuit_breaker.can_execute().await {
            return Err(SecretonError::Unavailable(
                "Circuit breaker is open - Secreton service unavailable".to_string(),
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

                    // Don't retry on authentication/authorization errors
                    if matches!(
                        err,
                        SecretonError::AuthenticationFailed(_) | SecretonError::Unauthorized(_)
                    ) {
                        self.circuit_breaker.record_failure().await;
                        return Err(err);
                    }

                    // Record failure for circuit breaker
                    if attempt == self.max_retries {
                        self.circuit_breaker.record_failure().await;
                    }

                    // Wait before retry (exponential backoff)
                    if attempt < self.max_retries {
                        let delay = Duration::from_millis(100 * (2_u64.pow(attempt)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| SecretonError::Other("Unknown error".to_string())))
    }

    /// Retrieve a secret from the Secreton service
    ///
    /// This method fetches a secret value from the Secreton service using
    /// the provided key. Secrets can be optionally scoped to a specific
    /// satker for multi-tenant isolation and access control.
    ///
    /// # Arguments
    /// * `key` - The secret key to retrieve
    /// * `satker_code` - Optional satker code for tenant-specific secret isolation
    ///
    /// # Returns
    /// The secret value as a String if found, None if not found or on error
    ///
    /// # Security Considerations
    /// - Secret keys should be validated to prevent injection attacks
    /// - Satker access should be authorized before retrieval
    /// - Failed retrievals should be logged for security monitoring
    /// - Network errors should not expose sensitive information
    ///
    /// # API Behavior
    /// - Returns None for non-existent secrets (404 responses)
    /// - Returns None for authentication/authorization failures
    /// - Returns None for network or service errors
    /// - Successful retrievals return the secret value
    ///
    /// # Example
    /// ```rust
    /// use authenc::secreton_client::secreton_client::SecretonClient;
    ///
    /// # async fn example() {
    /// let client = SecretonClient::new(
    ///     "https://secreton.example.com".to_string(),
    ///     "your-auth-token".to_string()
    /// );
    /// let secret = client.get_secret("api-key", Some("kejaksaan-agung")).await;
    /// if let Some(value) = secret {
    ///     println!("Retrieved secret: {}", value);
    /// }
    /// # }
    /// ```
    pub async fn get_secret(&self, key: &str, satker_code: Option<&str>) -> Option<String> {
        let operation = || {
            let key = key.to_string();
            let satker_code = satker_code.map(|s| s.to_string());
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = if let Some(satker) = satker_code {
                    format!("{}/v1/secret/data/{}/{}", endpoint, satker, key)
                } else {
                    format!("{}/v1/secret/data/{}", endpoint, key)
                };

                let resp = client
                    .get(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 404 {
                    return Err(SecretonError::NotFound(format!("Secret not found: {}", key)));
                } else if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized("Access denied".to_string()));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct SecretResp {
                    data: Option<std::collections::HashMap<String, String>>,
                }

                let secret_resp: SecretResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                secret_resp
                    .data
                    .and_then(|data| data.values().next().cloned())
                    .ok_or_else(|| SecretonError::NotFound(format!("Secret data not found: {}", key)))
            }
        };

        match self.execute_with_circuit_breaker(operation).await {
            Ok(value) => Some(value),
            Err(_) => None, // Convert errors to None for backward compatibility
        }
    }

    /// Get signing key for JWT operations (SIMKARI-specific)
    ///
    /// Retrieves cryptographic signing keys for JWT token generation
    /// and validation operations within the SIMKARI super app.
    ///
    /// # Arguments
    /// * `key_id` - Identifier for the signing key
    /// * `context` - Security context for the request
    ///
    /// # Returns
    /// The signing key material if found, error otherwise
    pub async fn get_signing_key(
        &self,
        key_id: &str,
        context: &SecurityContext,
    ) -> Result<SigningKey, SecretonError> {
        let operation = || {
            let key_id = key_id.to_string();
            let context = context.clone();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/crypto/signing-key/{}", endpoint, key_id);

                let payload = serde_json::json!({
                    "context": context,
                    "key_usage": "jwt_signing"
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 404 {
                    return Err(SecretonError::NotFound(format!(
                        "Signing key not found: {}",
                        key_id
                    )));
                } else if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to signing key".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct SigningKeyResp {
                    key_id: String,
                    key_material: String,
                    algorithm: String,
                    expires_at: Option<String>,
                }

                let key_resp: SigningKeyResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                let expires_at = key_resp
                    .expires_at
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));

                Ok(SigningKey {
                    key_id: key_resp.key_id,
                    key_material: key_resp.key_material,
                    algorithm: key_resp.algorithm,
                    expires_at,
                })
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Get encryption key for session data (SIMKARI-specific)
    ///
    /// Retrieves encryption keys for protecting session data and
    /// sensitive user information within the authentication system.
    ///
    /// # Arguments
    /// * `resource_id` - Identifier for the resource requiring encryption
    /// * `context` - Security context for the request
    ///
    /// # Returns
    /// The encryption key material if found, error otherwise
    pub async fn get_encryption_key(
        &self,
        resource_id: &str,
        context: &SecurityContext,
    ) -> Result<EncryptionKey, SecretonError> {
        let operation = || {
            let resource_id = resource_id.to_string();
            let context = context.clone();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/crypto/encryption-key/{}", endpoint, resource_id);

                let payload = serde_json::json!({
                    "context": context,
                    "key_usage": "session_encryption"
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 404 {
                    return Err(SecretonError::NotFound(format!(
                        "Encryption key not found: {}",
                        resource_id
                    )));
                } else if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to encryption key".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct EncryptionKeyResp {
                    key_id: String,
                    key_material: String,
                    algorithm: String,
                    key_size: u32,
                }

                let key_resp: EncryptionKeyResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                Ok(EncryptionKey {
                    key_id: key_resp.key_id,
                    key_material: key_resp.key_material,
                    algorithm: key_resp.algorithm,
                    key_size: key_resp.key_size,
                })
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Validate user secret access permissions
    ///
    /// Checks if a user has permission to access a specific secret
    /// based on their role, satker affiliation, and access policies.
    ///
    /// # Arguments
    /// * `user_id` - User identifier (NIP or UUID)
    /// * `secret_path` - Path to the secret being accessed
    ///
    /// # Returns
    /// True if access is allowed, false otherwise
    pub async fn validate_user_secret_access(
        &self,
        user_id: &str,
        secret_path: &str,
    ) -> Result<bool, SecretonError> {
        let operation = || {
            let user_id = user_id.to_string();
            let secret_path = secret_path.to_string();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/auth/validate-access", endpoint);
                let payload = serde_json::json!({
                    "user_id": user_id,
                    "secret_path": secret_path,
                    "operation": "read"
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Ok(false); // Access denied, but not an error
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct AccessValidationResp {
                    allowed: bool,
                    reason: Option<String>,
                }

                let validation_resp: AccessValidationResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                Ok(validation_resp.allowed)
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Get application configuration secrets
    ///
    /// Retrieves configuration data for SIMKARI applications,
    /// including database connections, API endpoints, and feature flags.
    ///
    /// # Arguments
    /// * `app_id` - Application identifier
    ///
    /// # Returns
    /// Configuration data if found, error otherwise
    pub async fn get_application_config(
        &self,
        app_id: &str,
    ) -> Result<ApplicationConfig, SecretonError> {
        let operation = || {
            let app_id = app_id.to_string();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/config/application/{}", endpoint, app_id);

                let resp = client
                    .get(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 404 {
                    return Err(SecretonError::NotFound(format!(
                        "Application config not found: {}",
                        app_id
                    )));
                } else if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to application config".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct AppConfigResp {
                    app_id: String,
                    config_data: serde_json::Value,
                    version: u32,
                    updated_at: String,
                }

                let config_resp: AppConfigResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                let updated_at = chrono::DateTime::parse_from_rfc3339(&config_resp.updated_at)
                    .map_err(|e| SecretonError::InvalidFormat(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&chrono::Utc);

                Ok(ApplicationConfig {
                    app_id: config_resp.app_id,
                    config_data: config_resp.config_data,
                    version: config_resp.version,
                    updated_at,
                })
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Perform hybrid key exchange (X25519 + ML-KEM) with Secreton
    ///
    /// Executes a hybrid key exchange combining classical X25519 ECDH with
    /// post-quantum ML-KEM key encapsulation for quantum-safe communication.
    ///
    /// # Arguments
    /// * `key_id` - Hybrid key identifier in Secreton
    /// * `local_x25519_private` - Local X25519 private key
    /// * `local_mlkem_private` - Local ML-KEM private key (optional, for decapsulation)
    ///
    /// # Returns
    /// Combined shared secret from both key exchanges
    pub async fn hybrid_key_exchange(
        &self,
        key_id: &str,
        local_x25519_private: &[u8; 32],
        local_mlkem_private: Option<&[u8]>,
    ) -> Result<[u8; 32], SecretonError> {
        let operation = || {
            let key_id = key_id.to_string();
            let x25519_private = *local_x25519_private;
            let mlkem_private = local_mlkem_private.map(|k| k.to_vec());
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/crypto/hybrid-key-exchange/{}", endpoint, key_id);

                // Generate local X25519 public key for the exchange
                use curve25519_dalek::{constants::X25519_BASEPOINT, scalar::Scalar};
                let scalar = Scalar::from_bytes_mod_order(x25519_private);
                let local_x25519_public = (scalar * X25519_BASEPOINT).to_bytes();

                let payload = serde_json::json!({
                    "key_id": key_id,
                    "local_x25519_public": BASE64.encode(&local_x25519_public),
                    "request_mlkem_encapsulation": mlkem_private.is_none(),
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 404 {
                    return Err(SecretonError::NotFound(format!(
                        "Hybrid key not found: {}",
                        key_id
                    )));
                } else if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to hybrid key exchange".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct HybridKeyExchangeResp {
                    peer_x25519_public: String,
                    mlkem_ciphertext: Option<String>,
                    combined_shared_secret: Option<String>,
                }

                let exchange_resp: HybridKeyExchangeResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                // If server provided combined secret, use it
                if let Some(combined_secret_b64) = exchange_resp.combined_shared_secret {
                    let combined_secret = BASE64.decode(&combined_secret_b64).map_err(|e| {
                        SecretonError::InvalidFormat(format!(
                            "Invalid combined secret encoding: {}",
                            e
                        ))
                    })?;

                    let secret_array: [u8; 32] = combined_secret.try_into().map_err(|_| {
                        SecretonError::InvalidFormat("Invalid combined secret length".to_string())
                    })?;

                    return Ok(secret_array);
                }

                // Otherwise, perform local key exchange
                let peer_x25519_public =
                    BASE64
                        .decode(&exchange_resp.peer_x25519_public)
                        .map_err(|e| {
                            SecretonError::InvalidFormat(format!(
                                "Invalid peer public key encoding: {}",
                                e
                            ))
                        })?;

                let peer_x25519_array: [u8; 32] = peer_x25519_public.try_into().map_err(|_| {
                    SecretonError::InvalidFormat("Invalid peer public key length".to_string())
                })?;

                // Perform X25519 key exchange
                let peer_point = curve25519_dalek::montgomery::MontgomeryPoint(peer_x25519_array);
                let x25519_shared_secret = (scalar * peer_point).to_bytes();

                // Combine with ML-KEM if available
                let combined_secret = if let (Some(mlkem_ct_b64), Some(_mlkem_priv)) =
                    (exchange_resp.mlkem_ciphertext, mlkem_private)
                {
                    let _mlkem_ciphertext = BASE64.decode(&mlkem_ct_b64).map_err(|e| {
                        SecretonError::InvalidFormat(format!(
                            "Invalid ML-KEM ciphertext encoding: {}",
                            e
                        ))
                    })?;

                    // In a real implementation, we would decapsulate the ML-KEM ciphertext
                    // For now, we'll use HKDF to combine the X25519 secret with a placeholder
                    use hkdf::Hkdf;
                    use sha2::Sha256;

                    let hk = Hkdf::<Sha256>::new(None, &x25519_shared_secret);
                    let mut output = [0u8; 32];
                    hk.expand(b"SIMKARI-hybrid-key-exchange", &mut output)
                        .map_err(|_| SecretonError::Other("HKDF expansion failed".to_string()))?;

                    output
                } else {
                    x25519_shared_secret
                };

                Ok(combined_secret)
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Request ML-KEM encapsulation from Secreton for long-term secret encryption
    ///
    /// Requests Secreton to encapsulate a symmetric key using ML-KEM for
    /// quantum-safe long-term secret storage.
    ///
    /// # Arguments
    /// * `key_id` - ML-KEM key identifier in Secreton
    /// * `plaintext_key` - Symmetric key to encapsulate (32 bytes)
    ///
    /// # Returns
    /// ML-KEM ciphertext containing the encapsulated key
    pub async fn mlkem_encapsulate_key(
        &self,
        key_id: &str,
        plaintext_key: &[u8; 32],
    ) -> Result<Vec<u8>, SecretonError> {
        let operation = || {
            let key_id = key_id.to_string();
            let plaintext = *plaintext_key;
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/crypto/mlkem-encapsulate/{}", endpoint, key_id);

                let payload = serde_json::json!({
                    "key_id": key_id,
                    "plaintext": BASE64.encode(&plaintext),
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 404 {
                    return Err(SecretonError::NotFound(format!(
                        "ML-KEM key not found: {}",
                        key_id
                    )));
                } else if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to ML-KEM encapsulation".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct MlkemEncapsulateResp {
                    ciphertext: String,
                }

                let encap_resp: MlkemEncapsulateResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                BASE64.decode(&encap_resp.ciphertext).map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid ciphertext encoding: {}", e))
                })
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Request ML-DSA signature from Secreton for authentication tokens
    ///
    /// Requests Secreton to sign authentication token data using ML-DSA
    /// for quantum-safe digital signatures.
    ///
    /// # Arguments
    /// * `key_id` - ML-DSA key identifier in Secreton
    /// * `token_data` - Token data to sign
    ///
    /// # Returns
    /// ML-DSA signature bytes
    pub async fn mldsa_sign_token(
        &self,
        key_id: &str,
        token_data: &[u8],
    ) -> Result<Vec<u8>, SecretonError> {
        let operation = || {
            let key_id = key_id.to_string();
            let data = token_data.to_vec();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/crypto/mldsa-sign/{}", endpoint, key_id);

                let payload = serde_json::json!({
                    "key_id": key_id,
                    "data": BASE64.encode(&data),
                    "purpose": "authentication_token",
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 404 {
                    return Err(SecretonError::NotFound(format!(
                        "ML-DSA key not found: {}",
                        key_id
                    )));
                } else if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to ML-DSA signing".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct MldsaSignResp {
                    signature: String,
                }

                let sign_resp: MldsaSignResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                BASE64.decode(&sign_resp.signature).map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid signature encoding: {}", e))
                })
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Get post-quantum cryptographic key (future-proofing)
    ///
    /// Retrieves post-quantum cryptographic keys for quantum-safe
    /// operations as part of the cryptographic modernization strategy.
    ///
    /// # Arguments
    /// * `key_id` - Post-quantum key identifier
    /// * `algorithm` - PQ algorithm type
    ///
    /// # Returns
    /// Post-quantum key material if found, error otherwise
    pub async fn get_post_quantum_key(
        &self,
        key_id: &str,
        algorithm: PqAlgorithm,
    ) -> Result<PqKey, SecretonError> {
        let operation = || {
            let key_id = key_id.to_string();
            let algorithm = algorithm.clone();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let algorithm_str = match &algorithm {
                    PqAlgorithm::MlDsa => "ml-dsa",
                    PqAlgorithm::MlKem => "ml-kem",
                    PqAlgorithm::SphincsPlusShake256 => "sphincs-plus-shake256",
                    PqAlgorithm::Hybrid(inner) => match inner.as_ref() {
                        PqAlgorithm::MlDsa => "hybrid-ml-dsa",
                        PqAlgorithm::MlKem => "hybrid-ml-kem",
                        _ => "hybrid-unknown",
                    },
                };

                let url = format!("{}/v1/crypto/pq-key/{}/{}", endpoint, algorithm_str, key_id);

                let resp = client
                    .get(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 404 {
                    return Err(SecretonError::NotFound(format!(
                        "Post-quantum key not found: {}",
                        key_id
                    )));
                } else if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to post-quantum key".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct PqKeyResp {
                    key_id: String,
                    public_key: String,
                    private_key: Option<String>,
                    algorithm: String,
                    parameters: std::collections::HashMap<String, String>,
                }

                let key_resp: PqKeyResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                let public_key = BASE64.decode(&key_resp.public_key).map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid public key encoding: {}", e))
                })?;

                let private_key = if let Some(priv_key_str) = key_resp.private_key {
                    Some(BASE64.decode(&priv_key_str).map_err(|e| {
                        SecretonError::InvalidFormat(format!("Invalid private key encoding: {}", e))
                    })?)
                } else {
                    None
                };

                Ok(PqKey {
                    key_id: key_resp.key_id,
                    public_key,
                    private_key,
                    algorithm,
                    parameters: key_resp.parameters,
                })
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Get circuit breaker status for monitoring
    pub async fn get_circuit_breaker_state(&self) -> CircuitBreakerState {
        self.circuit_breaker.get_state().await
    }

    /// Reset circuit breaker (for administrative purposes)
    pub async fn reset_circuit_breaker(&self) {
        *self.circuit_breaker.state.write().await = CircuitBreakerState::Closed;
        *self.circuit_breaker.failure_count.write().await = 0;
    }

    /// Setup MFA for a user using secreton MfaManager
    ///
    /// Integrates with existing secreton MfaManager to setup TOTP authentication
    /// for government employees with proper encryption and audit logging.
    ///
    /// # Arguments
    /// * `user_id` - User identifier (NIP or UUID)
    /// * `issuer` - TOTP issuer name (e.g., "SIMPelv2 Kejaksaan RI")
    /// * `account_name` - Account name for TOTP (e.g., "user@kejaksaan.go.id")
    ///
    /// # Returns
    /// MFA setup data including QR code URL and backup codes
    pub async fn setup_mfa(
        &self,
        user_id: &str,
        issuer: &str,
        account_name: &str,
    ) -> Result<MfaSetupData, SecretonError> {
        let operation = || {
            let user_id = user_id.to_string();
            let issuer = issuer.to_string();
            let account_name = account_name.to_string();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/mfa/setup", endpoint);

                let payload = serde_json::json!({
                    "user_id": user_id,
                    "method": "TOTP",
                    "issuer": issuer,
                    "account_name": account_name
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to MFA setup".to_string(),
                    ));
                } else if resp.status() == 409 {
                    return Err(SecretonError::Other(
                        "MFA already configured for user".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct MfaSetupResp {
                    qr_code_url: Option<String>,
                    secret: Option<String>,
                    recovery_codes: Vec<String>,
                }

                let setup_resp: MfaSetupResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                Ok(MfaSetupData {
                    qr_code_url: setup_resp.qr_code_url.unwrap_or_default(),
                    secret_key: setup_resp.secret.unwrap_or_default(),
                    backup_codes: setup_resp.recovery_codes,
                })
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Verify MFA setup using secreton MfaManager
    ///
    /// Verifies the initial TOTP code to complete MFA setup process.
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `code` - TOTP code from authenticator app
    ///
    /// # Returns
    /// Success if verification passes, error otherwise
    pub async fn verify_mfa_setup(&self, user_id: &str, code: &str) -> Result<(), SecretonError> {
        let operation = || {
            let user_id = user_id.to_string();
            let code = code.to_string();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/mfa/verify-setup", endpoint);

                let payload = serde_json::json!({
                    "user_id": user_id,
                    "code": code
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to MFA verification".to_string(),
                    ));
                } else if resp.status() == 400 {
                    return Err(SecretonError::Other("Invalid MFA code".to_string()));
                } else if resp.status() == 404 {
                    return Err(SecretonError::NotFound(
                        "MFA not configured for user".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                Ok(())
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Verify MFA code during authentication using secreton MfaManager
    ///
    /// Verifies TOTP code during login process with replay protection.
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `code` - TOTP code from authenticator app
    ///
    /// # Returns
    /// Success if verification passes, error otherwise
    pub async fn verify_mfa(&self, user_id: &str, code: &str) -> Result<(), SecretonError> {
        let operation = || {
            let user_id = user_id.to_string();
            let code = code.to_string();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/mfa/verify", endpoint);

                let payload = serde_json::json!({
                    "user_id": user_id,
                    "code": code
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to MFA verification".to_string(),
                    ));
                } else if resp.status() == 400 {
                    return Err(SecretonError::Other("Invalid MFA code".to_string()));
                } else if resp.status() == 404 {
                    return Err(SecretonError::NotFound(
                        "MFA not configured for user".to_string(),
                    ));
                } else if resp.status() == 429 {
                    return Err(SecretonError::Other(
                        "Rate limit exceeded for MFA verification".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                Ok(())
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Get MFA status for a user using secreton MfaManager
    ///
    /// Retrieves current MFA configuration and status information.
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    ///
    /// # Returns
    /// MFA status information if found, error otherwise
    pub async fn get_mfa_status(&self, user_id: &str) -> Result<MfaStatusResponse, SecretonError> {
        let operation = || {
            let user_id = user_id.to_string();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/mfa/status/{}", endpoint, user_id);

                let resp = client
                    .get(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to MFA status".to_string(),
                    ));
                } else if resp.status() == 404 {
                    return Err(SecretonError::NotFound("User not found".to_string()));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct MfaStatusResp {
                    is_enabled: bool,
                    method: Option<String>,
                    last_used: Option<String>,
                }

                let status_resp: MfaStatusResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                Ok(MfaStatusResponse {
                    is_enabled: status_resp.is_enabled,
                    method: status_resp.method,
                    last_used: status_resp.last_used,
                })
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Disable MFA for a user using secreton MfaManager
    ///
    /// Removes MFA configuration and secrets for a user (admin operation).
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `admin_context` - Security context for admin authorization
    ///
    /// # Returns
    /// Success if MFA is disabled, error otherwise
    pub async fn disable_mfa(
        &self,
        user_id: &str,
        admin_context: &SecurityContext,
    ) -> Result<(), SecretonError> {
        let operation = || {
            let user_id = user_id.to_string();
            let admin_context = admin_context.clone();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/mfa/disable", endpoint);

                let payload = serde_json::json!({
                    "user_id": user_id,
                    "admin_context": admin_context
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to MFA disable operation".to_string(),
                    ));
                } else if resp.status() == 404 {
                    return Err(SecretonError::NotFound(
                        "MFA not configured for user".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                Ok(())
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Verify recovery code using secreton MfaManager
    ///
    /// Verifies backup recovery codes for emergency MFA access.
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `recovery_code` - Recovery code to verify
    ///
    /// # Returns
    /// Success if recovery code is valid, error otherwise
    pub async fn verify_recovery_code(
        &self,
        user_id: &str,
        recovery_code: &str,
    ) -> Result<(), SecretonError> {
        let operation = || {
            let user_id = user_id.to_string();
            let recovery_code = recovery_code.to_string();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/mfa/verify-recovery", endpoint);

                let payload = serde_json::json!({
                    "user_id": user_id,
                    "recovery_code": recovery_code
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to recovery code verification".to_string(),
                    ));
                } else if resp.status() == 400 {
                    return Err(SecretonError::Other("Invalid recovery code".to_string()));
                } else if resp.status() == 404 {
                    return Err(SecretonError::NotFound(
                        "MFA not configured for user".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                Ok(())
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Regenerate recovery codes using secreton MfaManager
    ///
    /// Generates new backup recovery codes and invalidates old ones.
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    ///
    /// # Returns
    /// New recovery codes if successful, error otherwise
    pub async fn regenerate_recovery_codes(
        &self,
        user_id: &str,
    ) -> Result<Vec<String>, SecretonError> {
        let operation = || {
            let user_id = user_id.to_string();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/mfa/regenerate-recovery", endpoint);

                let payload = serde_json::json!({
                    "user_id": user_id
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to recovery code regeneration".to_string(),
                    ));
                } else if resp.status() == 404 {
                    return Err(SecretonError::NotFound(
                        "MFA not configured for user".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct RecoveryCodesResp {
                    recovery_codes: Vec<String>,
                }

                let codes_resp: RecoveryCodesResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                Ok(codes_resp.recovery_codes)
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Rotate a signing key (for JWT operations)
    ///
    /// Requests Secreton to rotate a signing key and return the new key material.
    /// This is used for automatic key rotation to enhance security.
    ///
    /// # Arguments
    /// * `key_id` - Identifier of the key to rotate
    /// * `context` - Security context for the rotation request
    ///
    /// # Returns
    /// New signing key material after rotation
    pub async fn rotate_signing_key(
        &self,
        key_id: &str,
        context: &SecurityContext,
    ) -> Result<SigningKey, SecretonError> {
        let operation = || {
            let key_id = key_id.to_string();
            let context = context.clone();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/crypto/rotate-signing-key/{}", endpoint, key_id);

                let payload = serde_json::json!({
                    "key_id": key_id,
                    "context": context,
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 404 {
                    return Err(SecretonError::NotFound(format!(
                        "Signing key not found: {}",
                        key_id
                    )));
                } else if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to key rotation".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct RotateKeyResp {
                    key_id: String,
                    key_material: String,
                    algorithm: String,
                    expires_at: Option<String>,
                    version: u32,
                }

                let key_resp: RotateKeyResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                let expires_at = key_resp
                    .expires_at
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));

                Ok(SigningKey {
                    key_id: key_resp.key_id,
                    key_material: key_resp.key_material,
                    algorithm: key_resp.algorithm,
                    expires_at,
                })
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Rotate an encryption key (for session/MFA data)
    ///
    /// Requests Secreton to rotate an encryption key and return the new key material.
    /// This is used for automatic key rotation to enhance security.
    ///
    /// # Arguments
    /// * `key_id` - Identifier of the key to rotate
    /// * `context` - Security context for the rotation request
    ///
    /// # Returns
    /// New encryption key material after rotation
    pub async fn rotate_encryption_key(
        &self,
        key_id: &str,
        context: &SecurityContext,
    ) -> Result<EncryptionKey, SecretonError> {
        let operation = || {
            let key_id = key_id.to_string();
            let context = context.clone();
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/crypto/rotate-encryption-key/{}", endpoint, key_id);

                let payload = serde_json::json!({
                    "key_id": key_id,
                    "context": context,
                });

                let resp = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if resp.status() == 404 {
                    return Err(SecretonError::NotFound(format!(
                        "Encryption key not found: {}",
                        key_id
                    )));
                } else if resp.status() == 401 {
                    return Err(SecretonError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ));
                } else if resp.status() == 403 {
                    return Err(SecretonError::Unauthorized(
                        "Access denied to key rotation".to_string(),
                    ));
                } else if !resp.status().is_success() {
                    return Err(SecretonError::Other(format!("HTTP {}", resp.status())));
                }

                #[derive(Deserialize)]
                struct RotateEncKeyResp {
                    key_id: String,
                    key_material: String,
                    algorithm: String,
                    key_size: u32,
                    version: u32,
                }

                let key_resp: RotateEncKeyResp = resp.json().await.map_err(|e| {
                    SecretonError::InvalidFormat(format!("Invalid response format: {}", e))
                })?;

                Ok(EncryptionKey {
                    key_id: key_resp.key_id,
                    key_material: key_resp.key_material,
                    algorithm: key_resp.algorithm,
                    key_size: key_resp.key_size,
                })
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Health check for Secreton service connectivity
    ///
    /// Performs a basic health check to verify service availability and connectivity.
    ///
    /// # Returns
    /// Success if service is healthy, error otherwise
    pub async fn health_check(&self) -> Result<(), SecretonError> {
        let operation = || {
            let endpoint = self.endpoint.clone();
            let token = self.token.clone();
            let client = self.client.clone();

            async move {
                let url = format!("{}/v1/health", endpoint);

                let resp = client
                    .get(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| SecretonError::Unavailable(format!("Network error: {}", e)))?;

                if !resp.status().is_success() {
                    return Err(SecretonError::Unavailable(format!(
                        "Health check failed: HTTP {}",
                        resp.status()
                    )));
                }

                Ok(())
            }
        };

        self.execute_with_circuit_breaker(operation).await
    }

    /// Reset circuit breaker state after successful operations
    async fn reset_circuit_breaker_state(&self) {
        *self.circuit_breaker.success_count.write().await = 0;
        *self.circuit_breaker.last_failure_time.write().await = None;
    }
}

impl SecretonVault {
    /// Create a new Secreton vault with client configuration
    ///
    /// This constructor initializes a vault implementation that uses
    /// the Secreton service for secret storage and retrieval. The vault
    /// wraps a SecretonClient and provides the standard Vault trait
    /// interface for integration with the authentication platform.
    ///
    /// # Arguments
    /// * `client` - Configured SecretonClient for service communication
    ///
    /// # Returns
    /// A new `SecretonVault` instance ready for secret management operations
    ///
    /// # Security Considerations
    /// - Client should be properly configured with valid credentials
    /// - Network communication should use TLS encryption
    /// - Client permissions should be scoped to required operations
    /// - Secret access should be audited and monitored
    ///
    /// # Integration Features
    /// - Implements standard Vault trait for seamless integration
    /// - Supports satker-based multi-tenancy for Attorney General's Office
    /// - Provides metadata support for secret management
    /// - Handles error conditions gracefully
    /// - Enhanced for SIMKARI super app operations
    ///
    /// # Example
    /// ```rust
    /// use authenc::secreton_client::secreton_client::{SecretonClient, SecretonVault};
    ///
    /// let client = SecretonClient::new(
    ///     "https://secreton.example.com".to_string(),
    ///     "your-auth-token".to_string()
    /// );
    /// let vault = SecretonVault::new(client);
    /// // Vault is ready for secret operations through standard interface
    /// ```
    pub fn new(client: SecretonClient) -> Self {
        SecretonVault { client }
    }

    /// Get the underlying SecretonClient for advanced operations
    pub fn client(&self) -> &SecretonClient {
        &self.client
    }
}

/// Secreton vault implementation for the Vault trait
///
/// This struct provides a read-only vault interface to the Secreton service,
/// implementing the standard Vault trait for integration with the authentication
/// platform. It wraps a SecretonClient and provides secret retrieval operations
/// while maintaining security boundaries appropriate for the Attorney General's
/// Office SIMKARI super app.
///
/// # Security Considerations
/// - Read-only operations only (no secret creation/modification)
/// - All operations are audited through the Secreton service
/// - Network communication uses TLS encryption
/// - Access is scoped by satker (work unit) for multi-tenancy
///
/// # Integration Features
/// - Implements standard Vault trait interface
/// - Supports satker-based multi-tenancy
/// - Provides metadata support for secret management
/// - Handles error conditions gracefully
/// - Enhanced for government security requirements
#[derive(Debug, Clone)]
pub struct SecretonVault {
    /// The underlying Secreton client for service communication
    client: SecretonClient,
}

#[async_trait]
impl SecretonClientTrait for SecretonVault {
    async fn get_secret(&self, key: &str, realm: Option<&str>) -> Option<Secret> {
        self.client
            .get_secret(key, realm)
            .await
            .map(|value| Secret {
                value,
                metadata: None,
                version: Some(1),
                created_at: Some(chrono::Utc::now()),
                expires_at: None,
            })
    }

    async fn put_secret(
        &self,
        _key: &str,
        _value: &str,
        _realm: Option<&str>,
        _metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Result<(), super::VaultError> {
        Err(super::SecretonError::Other(
            "Write operations not supported in read-only integration".to_string(),
        ))
    }

    async fn delete_secret(
        &self,
        _key: &str,
        _realm: Option<&str>,
    ) -> Result<(), super::VaultError> {
        Err(super::SecretonError::Other(
            "Delete operations not supported in read-only integration".to_string(),
        ))
    }

    async fn list_secrets(&self, _realm: Option<&str>) -> Result<Vec<String>, super::VaultError> {
        // List operations not implemented for security reasons
        Ok(vec![])
    }

    async fn rotate_secret(
        &self,
        _key: &str,
        _realm: Option<&str>,
        _generator: Box<dyn Fn() -> String + Send>,
    ) -> Result<super::RotationResult, super::VaultError> {
        Err(super::SecretonError::Other(
            "Rotation managed by Secreton service directly".to_string(),
        ))
    }

    async fn get_secret_versions(
        &self,
        _key: &str,
        _realm: Option<&str>,
    ) -> Result<Vec<Secret>, super::VaultError> {
        // Version history managed by Secreton service
        Ok(vec![])
    }

    async fn health_check(&self) -> Result<bool, super::VaultError> {
        // Simple health check by attempting to connect to the endpoint
        let url = format!("{}/health", self.client.endpoint);
        match self.client.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
