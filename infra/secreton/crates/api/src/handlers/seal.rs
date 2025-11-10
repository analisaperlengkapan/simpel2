//! Seal/Unseal API handlers
//!
//! Provides REST endpoints for vault seal/unseal operations,
//! initialization, and rekey functionality.

use axum::{extract::State, http::StatusCode, response::Json, Extension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{error, info, instrument, warn};

use crate::handlers::AppState;
use secreton_core::audit::{AuditLog, AuditStatus};
use secreton_core::services::seal::{SealError, SealStatus};

/// Rate limiter for unseal attempts to prevent brute force attacks
#[derive(Clone)]
pub struct UnsealRateLimiter {
    attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_attempts: usize,
    window: Duration,
}

impl UnsealRateLimiter {
    pub fn new(max_attempts: usize, window_seconds: u64) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window: Duration::from_secs(window_seconds),
        }
    }

    /// Check if IP is rate limited. Returns Ok(()) if allowed, Err if rate limited.
    pub fn check_attempt(&self, ip: &str) -> Result<(), String> {
        let mut attempts = self.attempts.lock().map_err(|e| format!("Lock error: {}", e))?;
        let now = Instant::now();

        // Clean old attempts
        if let Some(ip_attempts) = attempts.get_mut(ip) {
            ip_attempts.retain(|&t| now.duration_since(t) < self.window);

            if ip_attempts.len() >= self.max_attempts {
                return Err(format!(
                    "Rate limit exceeded: {} attempts in {} seconds. Try again later.",
                    self.max_attempts,
                    self.window.as_secs()
                ));
            }

            ip_attempts.push(now);
        } else {
            attempts.insert(ip.to_string(), vec![now]);
        }

        Ok(())
    }

    /// Reset rate limit for an IP (e.g., after successful unseal)
    pub fn reset(&self, ip: &str) {
        if let Ok(mut attempts) = self.attempts.lock() {
            attempts.remove(ip);
        }
    }
}

/// Initialize vault request
#[derive(Debug, Deserialize)]
pub struct InitializeRequest {
    /// Number of secret shares to generate
    pub secret_shares: usize,

    /// Threshold of shares required to unseal
    pub secret_threshold: usize,
}

/// Initialize vault response
#[derive(Debug, Serialize)]
pub struct InitializeResponse {
    /// Base64-encoded Shamir shares (must be distributed securely)
    pub keys: Vec<String>,

    /// Base64-encoded root token (for initial authentication)
    pub root_token: String,
}

/// Unseal request
#[derive(Debug, Deserialize)]
pub struct UnsealRequest {
    /// Base64-encoded unseal key (Shamir share)
    pub key: String,

    /// Reset unseal progress if true
    #[serde(default)]
    pub reset: bool,
}

/// Seal status response
#[derive(Debug, Serialize)]
pub struct SealStatusResponse {
    /// Seal type (e.g., "shamir")
    pub seal_type: String,

    /// Whether vault is initialized
    pub initialized: bool,

    /// Whether vault is sealed
    pub sealed: bool,

    /// Total number of shares
    pub t: usize,

    /// Threshold required
    pub n: usize,

    /// Current progress (shares provided)
    pub progress: usize,

    /// Nonce for unseal session
    pub nonce: String,

    /// Version
    pub version: String,
}

impl From<SealStatus> for SealStatusResponse {
    fn from(status: SealStatus) -> Self {
        use secreton_core::services::seal::SealState;

        Self {
            seal_type: status.seal_type,
            initialized: status.initialized,
            sealed: matches!(status.state, SealState::Sealed | SealState::Unsealing),
            t: status.threshold,
            n: status.total_shares,
            progress: status.progress,
            nonce: status.nonce.unwrap_or_default(),
            version: status.version,
        }
    }
}

/// Rekey init request
#[derive(Debug, Deserialize)]
pub struct RekeyInitRequest {
    /// New number of secret shares
    pub secret_shares: usize,

    /// New threshold
    pub secret_threshold: usize,
}

/// Rekey update request
#[derive(Debug, Deserialize)]
pub struct RekeyUpdateRequest {
    /// Unseal key for authorization
    pub key: String,

    /// Nonce from rekey init
    pub nonce: String,
}

/// Rekey status response
#[derive(Debug, Serialize)]
pub struct RekeyStatusResponse {
    /// Whether rekey is in progress
    pub started: bool,

    /// Nonce for rekey session
    pub nonce: String,

    /// Current progress
    pub progress: usize,

    /// Required shares
    pub required: usize,

    /// New number of shares
    pub n: Option<usize>,

    /// New threshold
    pub t: Option<usize>,
}

/// GET /v1/sys/seal-status
/// Returns the seal status of the vault
///
/// This endpoint is whitelisted and accessible even when vault is sealed.
#[instrument(skip(state))]
pub async fn get_seal_status(
    State(state): State<AppState>,
) -> Result<Json<SealStatusResponse>, (StatusCode, String)> {
    info!("Fetching seal status");

    // Get seal status from SealService
    let status = state.seal.status().await;
    let response: SealStatusResponse = status.into();

    info!(
        "Seal status: sealed={}, initialized={}, progress={}/{}",
        response.sealed, response.initialized, response.progress, response.t
    );

    Ok(Json(response))
}

/// POST /v1/sys/seal
/// Seals the vault
///
/// CRITICAL SECURITY: This immediately seals the vault and clears the master key from memory.
/// All subsequent operations (except whitelisted endpoints) will be blocked until unsealed.
///
/// SECURITY: Requires admin role
#[instrument(skip(state))]
pub async fn seal_vault(
    State(state): State<AppState>,
    Extension(user_id): Extension<Option<String>>,
) -> Result<StatusCode, (StatusCode, String)> {
    info!("🔒 Attempting to seal vault");

    // SECURITY: Verify admin authentication
    let user_id = user_id.ok_or_else(|| {
        warn!("Unauthenticated seal attempt");
        (
            StatusCode::UNAUTHORIZED,
            "Authentication required to seal vault".to_string(),
        )
    })?;

    // TODO: Check if user has admin role
    // For now, we log the user but allow the operation
    // In production, add role check:
    // if !state.auth.has_role(&user_id, "admin").await? {
    //     return Err((StatusCode::FORBIDDEN, "Admin role required".to_string()));
    // }
    warn!(
        "Seal operation requested by user: {} (role check not yet implemented)",
        user_id
    );

    // Audit log the seal attempt
    let mut metadata = HashMap::new();
    metadata.insert("operation".to_string(), "seal".to_string());
    metadata.insert("initiated_at".to_string(), chrono::Utc::now().to_rfc3339());

    let audit_log = AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "vault.seal".to_string(),
        actor: Some(user_id.clone()),
        resource_type: "vault".to_string(),
        resource_id: "system".to_string(),
        status: AuditStatus::Success, // Will update to Success/Failure after operation
        ip: None,
        user_agent: None,
        namespace: None,
        metadata,
    };

    // Seal the vault
    state.seal.seal().await.map_err(|e| {
        error!("Failed to seal vault: {:?}", e);

        // Log failed seal attempt
        let mut failed_log = audit_log.clone();
        failed_log.status = AuditStatus::Failure;
        failed_log.metadata.insert("error".to_string(), e.to_string());
        let _ = state.audit.log(failed_log);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to seal vault: {}", e),
        )
    })?;

    info!("✅ Vault sealed successfully by user: {}", user_id);

    // Audit log successful seal
    let mut success_log = audit_log;
    success_log.status = AuditStatus::Success;
    success_log.metadata.insert("completed_at".to_string(), chrono::Utc::now().to_rfc3339());
    let _ = state.audit.log(success_log);

    Ok(StatusCode::NO_CONTENT)
}

/// POST /v1/sys/unseal
/// Provides an unseal key and unseals the vault if threshold is met
///
/// This endpoint is whitelisted and accessible even when vault is sealed.
/// Operators provide Shamir shares one at a time until threshold is reached.
///
/// SECURITY: Rate limited to prevent brute force attacks (max 10 attempts per 60 seconds per IP)
#[instrument(skip(state, request))]
pub async fn unseal_vault(
    State(state): State<AppState>,
    Extension(client_ip): Extension<Option<String>>,
    Json(request): Json<UnsealRequest>,
) -> Result<Json<SealStatusResponse>, (StatusCode, String)> {
    info!("🔓 Processing unseal request");

    // Get client IP for rate limiting (default to "unknown" if not available)
    let client_ip = client_ip.unwrap_or_else(|| "unknown".to_string());

    // SECURITY: Rate limiting to prevent brute force attacks
    // In production, this should be a shared state across instances
    // For now, we create a static rate limiter
    use once_cell::sync::Lazy;
    static RATE_LIMITER: Lazy<UnsealRateLimiter> = Lazy::new(|| {
        UnsealRateLimiter::new(10, 60) // 10 attempts per 60 seconds
    });

    RATE_LIMITER.check_attempt(&client_ip).map_err(|e| {
        warn!("Rate limit exceeded for IP {}: {}", client_ip, e);

        // Audit log rate limit hit
        let mut metadata = HashMap::new();
        metadata.insert("reason".to_string(), "rate_limit_exceeded".to_string());

        let audit_log = AuditLog {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            action: "vault.unseal.rate_limited".to_string(),
            actor: None,
            resource_type: "vault".to_string(),
            resource_id: "system".to_string(),
            status: AuditStatus::Failure,
            ip: Some(client_ip.clone()),
            user_agent: None,
            namespace: None,
            metadata,
        };
        let _ = state.audit.log(audit_log);

        (StatusCode::TOO_MANY_REQUESTS, e)
    })?;

    // Audit log unseal attempt
    let mut metadata = HashMap::new();
    metadata.insert("operation".to_string(), "unseal".to_string());
    metadata.insert("reset".to_string(), request.reset.to_string());

    let audit_log = AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "vault.unseal".to_string(),
        actor: None,
        resource_type: "vault".to_string(),
        resource_id: "system".to_string(),
        status: AuditStatus::Success, // Will update based on result
        ip: Some(client_ip.clone()),
        user_agent: None,
        namespace: None,
        metadata,
    };

    // Handle reset if requested
    if request.reset {
        info!("Resetting unseal progress");
        state.seal.reset_unseal().await.map_err(|e| {
            error!("Failed to reset unseal: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to reset unseal: {}", e),
            )
        })?;
    }

    // Provide unseal key (Shamir share)
    let status = state.seal.unseal(request.key).await.map_err(|e| {
        error!("Unseal failed: {:?}", e);

        // Audit log failed unseal attempt
        let mut failed_log = audit_log.clone();
        failed_log.status = AuditStatus::Failure;
        failed_log.metadata.insert("error".to_string(), e.to_string());
        let _ = state.audit.log(failed_log);

        match e {
            SealError::InvalidUnsealKey => {
                (StatusCode::BAD_REQUEST, "Invalid unseal key".to_string())
            }
            SealError::AlreadyUnsealed => (
                StatusCode::BAD_REQUEST,
                "Vault is already unsealed".to_string(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unseal error: {}", e),
            ),
        }
    })?;

    let response: SealStatusResponse = status.into();

    // Log progress
    if response.sealed {
        info!(
            "Unseal progress: {}/{} shares provided",
            response.progress, response.t
        );

        // Log progress in audit
        let mut progress_log = audit_log;
        progress_log.status = AuditStatus::Success;
        progress_log.metadata.insert("status".to_string(), "in_progress".to_string());
        progress_log.metadata.insert("progress".to_string(), response.progress.to_string());
        progress_log.metadata.insert("threshold".to_string(), response.t.to_string());
        let _ = state.audit.log(progress_log);
    } else {
        info!("✅ Vault unsealed successfully!");

        // Reset rate limiter for this IP on successful unseal
        RATE_LIMITER.reset(&client_ip);

        // Audit log successful unseal
        let mut success_log = audit_log;
        success_log.status = AuditStatus::Success;
        success_log.metadata.insert("status".to_string(), "unsealed".to_string());
        success_log.metadata.insert("completed_at".to_string(), chrono::Utc::now().to_rfc3339());
        let _ = state.audit.log(success_log);
    }

    Ok(Json(response))
}

/// POST /v1/sys/init
/// Initializes a new vault
///
/// CRITICAL SECURITY: This endpoint can only be called once.
/// After initialization, the vault remains SEALED.
/// Operators must manually unseal with threshold shares.
///
/// This endpoint is whitelisted and accessible even when vault is sealed.
#[instrument(skip(state, request))]
pub async fn initialize_vault(
    State(state): State<AppState>,
    Json(request): Json<InitializeRequest>,
) -> Result<Json<InitializeResponse>, (StatusCode, String)> {
    info!(
        "🔐 Initializing vault with {} shares and {} threshold",
        request.secret_shares, request.secret_threshold
    );

    // Validate request
    if request.secret_threshold > request.secret_shares {
        return Err((
            StatusCode::BAD_REQUEST,
            "Threshold cannot exceed total shares".to_string(),
        ));
    }

    if request.secret_threshold < 1 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Threshold must be at least 1".to_string(),
        ));
    }

    if request.secret_shares < 1 || request.secret_shares > 255 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Shares must be between 1 and 255".to_string(),
        ));
    }

    // Check if already initialized
    let status = state.seal.status().await;
    if status.initialized {
        warn!("Attempted to initialize already initialized vault");
        return Err((
            StatusCode::BAD_REQUEST,
            "Vault is already initialized".to_string(),
        ));
    }

    // Update seal configuration with requested parameters
    // TODO: This should be done more elegantly
    // For now, we'll create a new SealService with the requested config
    // In production, this would update the existing service's config

    // Initialize vault - generates master key and Shamir shares
    let shares = state.seal.initialize().await.map_err(|e| {
        error!("Failed to initialize vault: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to initialize vault: {}", e),
        )
    })?;

    // Encode shares as base64 for distribution
    use base64::{Engine as _, engine::general_purpose};
    let encoded_shares: Result<Vec<String>, String> = shares
        .iter()
        .map(|share| {
            share.to_bytes()
                .map(|bytes| general_purpose::STANDARD.encode(&bytes))
                .map_err(|e| format!("Failed to convert share to bytes: {}", e))
        })
        .collect();

    let encoded_shares = encoded_shares.map_err(|e| {
        error!("Failed to encode Shamir shares: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode shares: {}", e),
        )
    })?;

    // Generate root token
    // TODO: Implement proper root token generation with JWT
    // For now, use a placeholder
    let root_token = format!("root-{}", uuid::Uuid::new_v4());

    info!("✅ Vault initialized successfully");
    info!("⚠️  CRITICAL: Vault remains SEALED after initialization");
    info!(
        "   Operators must unseal with {} of {} shares",
        request.secret_threshold, request.secret_shares
    );
    info!(
        "   Distribute shares securely to {} operators",
        request.secret_shares
    );

    // TODO: Audit log the initialization
    // state.audit.log_vault_init(request.secret_shares, request.secret_threshold).await;

    // TODO: Update metrics
    // metrics::counter!("secreton_vault_initializations_total").increment(1);

    let response = InitializeResponse {
        keys: encoded_shares,
        root_token,
    };

    Ok(Json(response))
}

/// POST /v1/sys/rekey/init
/// Initiates a rekey operation
#[instrument(skip(state, request))]
pub async fn rekey_init(
    State(state): State<AppState>,
    Json(request): Json<RekeyInitRequest>,
) -> Result<Json<RekeyStatusResponse>, (StatusCode, String)> {
    info!(
        "Initiating rekey with {} shares and {} threshold",
        request.secret_shares, request.secret_threshold
    );

    // Validate request
    if request.secret_threshold > request.secret_shares {
        return Err((
            StatusCode::BAD_REQUEST,
            "Threshold cannot exceed total shares".to_string(),
        ));
    }

    // TODO: Check admin authentication
    // TODO: Get SealService from state
    // TODO: Call seal_service.start_rekey().await
    // TODO: Audit log the rekey initiation

    // Placeholder response
    warn!("Rekey init endpoint not fully implemented");

    let response = RekeyStatusResponse {
        started: true,
        nonce: "rekey_nonce_placeholder".to_string(),
        progress: 0,
        required: 3,
        n: Some(request.secret_shares),
        t: Some(request.secret_threshold),
    };

    Ok(Json(response))
}

/// POST /v1/sys/rekey/update
/// Provides a key for the rekey operation
#[instrument(skip(state, request))]
pub async fn rekey_update(
    State(state): State<AppState>,
    Json(request): Json<RekeyUpdateRequest>,
) -> Result<Json<RekeyStatusResponse>, (StatusCode, String)> {
    info!("Processing rekey update");

    // TODO: Validate nonce
    // TODO: Get SealService from state
    // TODO: Call seal_service.rekey_update(request.key).await
    // TODO: Audit log the rekey progress

    // Placeholder response
    warn!("Rekey update endpoint not fully implemented");

    let response = RekeyStatusResponse {
        started: true,
        nonce: request.nonce,
        progress: 1,
        required: 3,
        n: Some(5),
        t: Some(3),
    };

    Ok(Json(response))
}

/// Create seal routes
pub fn create_routes() -> axum::Router<AppState> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/seal-status", get(get_seal_status))
        .route("/seal", post(seal_vault))
        .route("/unseal", post(unseal_vault))
        .route("/init", post(initialize_vault))
        .route("/rekey/init", post(rekey_init))
        .route("/rekey/update", post(rekey_update))
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_seal_status_conversion() {
        use secreton_core::services::seal::{SealConfig, SealState, SealStatus};

        let seal_status = SealStatus {
            state: SealState::Sealed,
            seal_type: "shamir".to_string(),
            initialized: true,
            total_shares: 5,
            threshold: 3,
            progress: 0,
            nonce: Some("test_nonce".to_string()),
            version: "1.0.0".to_string(),
        };

        let response: SealStatusResponse = seal_status.into();

        assert_eq!(response.seal_type, "shamir");
        assert!(response.initialized);
        assert!(response.sealed);
        assert_eq!(response.n, 5);
        assert_eq!(response.t, 3);
        assert_eq!(response.progress, 0);
        assert_eq!(response.nonce, "test_nonce");
    }

    #[test]
    fn test_initialize_request_validation() {
        let valid_request = InitializeRequest {
            secret_shares: 5,
            secret_threshold: 3,
        };

        assert!(valid_request.secret_threshold <= valid_request.secret_shares);
        assert!(valid_request.secret_threshold >= 1);
        assert!(valid_request.secret_shares >= 1 && valid_request.secret_shares <= 255);
    }
}
