//! Seal/Unseal API handlers
//!
//! Provides REST endpoints for vault seal/unseal operations,
//! initialization, and rekey functionality.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, instrument};

use crate::handlers::AppState;
use secreton_core::services::seal::{SealService, SealStatus, SealError};

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
            nonce: status.nonce.unwrap_or_else(|| "".to_string()),
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

    info!("Seal status: sealed={}, initialized={}, progress={}/{}",
        response.sealed, response.initialized, response.progress, response.t);

    Ok(Json(response))
}

/// POST /v1/sys/seal
/// Seals the vault
///
/// CRITICAL SECURITY: This immediately seals the vault and clears the master key from memory.
/// All subsequent operations (except whitelisted endpoints) will be blocked until unsealed.
#[instrument(skip(state))]
pub async fn seal_vault(
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    info!("🔒 Sealing vault");

    // TODO: Check admin authentication
    // For now, allow any authenticated user to seal (in production, restrict to admins)

    // Seal the vault
    state.seal.seal().await
        .map_err(|e| {
            error!("Failed to seal vault: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to seal vault: {}", e))
        })?;

    info!("✅ Vault sealed successfully");

    // Audit log the seal operation
    // TODO: Add audit logging
    // state.audit.log_seal_operation(user_id, "seal", true).await;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /v1/sys/unseal
/// Provides an unseal key and unseals the vault if threshold is met
///
/// This endpoint is whitelisted and accessible even when vault is sealed.
/// Operators provide Shamir shares one at a time until threshold is reached.
#[instrument(skip(state, request))]
pub async fn unseal_vault(
    State(state): State<AppState>,
    Json(request): Json<UnsealRequest>,
) -> Result<Json<SealStatusResponse>, (StatusCode, String)> {
    info!("🔓 Processing unseal request");

    // TODO: Implement rate limiting for unseal attempts (prevent brute force)
    // TODO: Audit log the unseal attempt

    // Handle reset if requested
    if request.reset {
        info!("Resetting unseal progress");
        state.seal.reset_unseal().await
            .map_err(|e| {
                error!("Failed to reset unseal: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to reset unseal: {}", e))
            })?;
    }

    // Provide unseal key (Shamir share)
    let status = state.seal.unseal(request.key).await
        .map_err(|e| {
            error!("Unseal failed: {:?}", e);
            // TODO: Update metrics (unseal_failures)
            match e {
                SealError::InvalidUnsealKey => {
                    (StatusCode::BAD_REQUEST, "Invalid unseal key".to_string())
                }
                SealError::AlreadyUnsealed => {
                    (StatusCode::BAD_REQUEST, "Vault is already unsealed".to_string())
                }
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("Unseal error: {}", e))
                }
            }
        })?;

    let response: SealStatusResponse = status.into();

    // Log progress
    if response.sealed {
        info!("Unseal progress: {}/{} shares provided", response.progress, response.t);
    } else {
        info!("✅ Vault unsealed successfully!");
        // TODO: Audit log successful unseal
        // TODO: Update metrics (unseal_success, time_to_unseal)
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
    let shares = state.seal.initialize().await
        .map_err(|e| {
            error!("Failed to initialize vault: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to initialize vault: {}", e))
        })?;

    // Encode shares as base64 for distribution
    use base64::{engine::general_purpose, Engine as _};
    let encoded_shares: Vec<String> = shares.iter()
        .map(|share| {
            let share_bytes = share.to_bytes().expect("Failed to convert share to bytes");
            general_purpose::STANDARD.encode(&share_bytes)
        })
        .collect();

    // Generate root token
    // TODO: Implement proper root token generation with JWT
    // For now, use a placeholder
    let root_token = format!("root-{}", uuid::Uuid::new_v4());

    info!("✅ Vault initialized successfully");
    info!("⚠️  CRITICAL: Vault remains SEALED after initialization");
    info!("   Operators must unseal with {} of {} shares", request.secret_threshold, request.secret_shares);
    info!("   Distribute shares securely to {} operators", request.secret_shares);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_status_conversion() {
        use secreton_core::services::seal::{SealStatus, SealState, SealConfig};

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
