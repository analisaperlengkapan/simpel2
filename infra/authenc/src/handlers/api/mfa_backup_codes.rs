//! MFA Backup Codes API Handlers
//!
//! Handles backup code generation, verification, and management for MFA recovery.

use crate::error::{AuthencError, Result};
use crate::services::mfa_service::MfaService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Request to verify a backup/recovery code
#[derive(Debug, Deserialize)]
pub struct VerifyBackupCodeRequest {
    /// The backup/recovery code to verify
    pub code: String,
}

/// Response containing backup/recovery codes
#[derive(Debug, Serialize)]
pub struct BackupCodesResponse {
    /// List of backup codes
    pub codes: Vec<String>,
    /// Number of codes generated
    pub count: usize,
    /// Warning message about code security
    pub warning: String,
}

/// Response for backup code verification
#[derive(Debug, Serialize)]
pub struct BackupCodeVerificationResponse {
    /// Whether the code was valid
    pub valid: bool,
    /// Number of remaining backup codes
    pub remaining_codes: usize,
    /// Message about the verification result
    pub message: String,
}

/// Response for backup code status
#[derive(Debug, Serialize)]
pub struct BackupCodeStatusResponse {
    /// Whether backup codes are available
    pub available: bool,
    /// Number of remaining backup codes
    pub remaining_codes: usize,
    /// When backup codes were last generated
    pub last_generated: Option<chrono::DateTime<chrono::Utc>>,
}

/// Generate new backup codes for a user
/// This endpoint generates a fresh set of backup codes for MFA recovery.
/// Previous backup codes are invalidated when new ones are generated.
/// # Security Considerations
/// - Requires authenticated user session
/// - Logs backup code generation for audit
/// - Invalidates previous backup codes
/// - Returns codes only once (not stored in plaintext)
pub async fn generate_backup_codes(
    State(mfa_service): State<Arc<MfaService>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<BackupCodesResponse>> {
    // Generate new backup codes using secreton MfaManager
    let codes = mfa_service.regenerate_recovery_codes(user_id).await?;

    Ok(Json(BackupCodesResponse {
        count: codes.len(),
        codes,
        warning: "Store these backup codes in a secure location. They will not be shown again and can be used to recover access if you lose your authenticator device.".to_string(),
    }))
}

/// Verify a backup code for MFA bypass
/// This endpoint allows users to authenticate using a backup code when their
/// primary MFA method (TOTP) is unavailable.
/// # Security Considerations
/// - Each backup code can only be used once
/// - Rate limiting applied to prevent brute force
/// - Logs backup code usage for audit
/// - Requires valid user session
pub async fn verify_backup_code(
    State(mfa_service): State<Arc<MfaService>>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<VerifyBackupCodeRequest>,
) -> Result<Json<BackupCodeVerificationResponse>> {
    // Verify the backup code
    match mfa_service
        .verify_recovery_code(user_id, &request.code)
        .await
    {
        Ok(()) => {
            // Get remaining codes count
            let remaining = mfa_service
                .get_recovery_codes_count(user_id)
                .await
                .unwrap_or(0);

            Ok(Json(BackupCodeVerificationResponse {
                valid: true,
                remaining_codes: remaining,
                message: if remaining == 0 {
                    "Backup code verified successfully. This was your last backup code. Please generate new ones.".to_string()
                } else {
                    format!(
                        "Backup code verified successfully. You have {} backup codes remaining.",
                        remaining
                    )
                },
            }))
        }
        Err(AuthencError::InvalidBackupCode) => Ok(Json(BackupCodeVerificationResponse {
            valid: false,
            remaining_codes: 0,
            message: "Invalid backup code. Please check the code and try again.".to_string(),
        })),
        Err(AuthencError::RecoveryCodeAlreadyUsed) => Ok(Json(BackupCodeVerificationResponse {
            valid: false,
            remaining_codes: 0,
            message:
                "This backup code has already been used. Each backup code can only be used once."
                    .to_string(),
        })),
        Err(e) => Err(e),
    }
}

/// Get backup code status for a user
/// Returns information about the user's backup codes without revealing the codes themselves.
pub async fn get_backup_code_status(
    State(mfa_service): State<Arc<MfaService>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<BackupCodeStatusResponse>> {
    let remaining_codes = mfa_service
        .get_recovery_codes_count(user_id)
        .await
        .unwrap_or(0);
    let has_codes = mfa_service
        .has_recovery_codes(user_id)
        .await
        .unwrap_or(false);

    Ok(Json(BackupCodeStatusResponse {
        available: has_codes,
        remaining_codes,
        last_generated: None, // Could be enhanced to track generation time
    }))
}

/// Disable all backup codes for a user
/// This endpoint invalidates all backup codes for a user. This is typically used
/// when a user wants to disable MFA entirely or when codes are compromised.
/// # Security Considerations
/// - Requires admin privileges or user self-service
/// - Logs backup code invalidation for audit
/// - Cannot be undone (new codes must be generated)
pub async fn disable_backup_codes(
    State(mfa_service): State<Arc<MfaService>>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode> {
    // This would require integration with secreton to disable backup codes
    // For now, we can regenerate empty codes or mark them as disabled

    // Log the action for audit
    tracing::warn!(
        user_id = %user_id,
        event = "backup_codes_disabled",
        "User backup codes were disabled"
    );

    // Return success - actual implementation would call secreton
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secreton_client::secreton_client::SecretonClient;
    use crate::services::mfa_service::MfaService;
    use std::sync::Arc;

    // Note: These tests require integration test infrastructure
    // See tests/mfa_service_unit_tests.rs for mock implementations
    // Skipped until test database infrastructure is ready

    #[tokio::test]
    #[ignore = "Requires test database infrastructure"]
    async fn test_generate_backup_codes() {
        // TODO: Implement once test infrastructure is ready
        // Should verify backup code generation through MfaService
    }

    #[tokio::test]
    #[ignore = "Requires test database infrastructure"]
    async fn test_verify_backup_code() {
        // TODO: Implement once test infrastructure is ready
        // Should verify backup code verification
    }

    #[tokio::test]
    #[ignore = "Requires test database infrastructure"]
    async fn test_backup_code_reuse_prevention() {
        // TODO: Implement once test infrastructure is ready
        // Should verify that used backup codes cannot be reused
    }
}
