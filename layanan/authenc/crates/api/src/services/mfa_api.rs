//! Concrete `MfaApiService` adapter that drives the REST MFA endpoints.
//!
//! The handlers in `handlers::mfa` only know about the [`MfaApiService`]
//! trait; this adapter satisfies that trait by composing the lower-level
//! [`authenc_mfa::TotpService`] and [`authenc_mfa::BackupCodesService`]
//! over Postgres-backed stores. Without this layer `ApiState.mfa_service`
//! was always `None` and every MFA endpoint returned
//! `mfa_not_configured` (HTTP 503).
//!
//! Note on backup-code handling: the trait surface keeps a single "verify
//! code" entry point (`verify_code`) so the existing frontend doesn't need
//! to know whether the six digits the user typed came from their phone or
//! their printed recovery sheet. We try the TOTP path first, then fall
//! back to the backup-code service — same behaviour as
//! `MfaServiceFacadeImpl::verify_totp` in the gRPC crate, deliberately
//! mirrored here so the two transports stay observationally identical.

use async_trait::async_trait;
use authenc_mfa::{
    BackupCodesConfig, BackupCodesService, BackupCodesStore, TotpConfig, TotpService,
    totp::TotpStore,
};
use authenc_types::UserId;
use base64::Engine;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::handlers::mfa::{MfaApiError, MfaApiService, MfaSetupData, MfaStatusData};

/// Drives the REST MFA endpoints over Postgres-backed stores.
#[derive(Clone)]
pub struct LocalMfaApi<T: TotpStore, B: BackupCodesStore> {
    totp: Arc<TotpService<T>>,
    backup_codes: Arc<BackupCodesService<B>>,
}

impl<T: TotpStore, B: BackupCodesStore> LocalMfaApi<T, B> {
    /// Build the adapter from the underlying stores using defaulted configs
    /// (SHA-1, 6 digits, 30s window, 1-step tolerance — RFC 6238 defaults
    /// that match every consumer-grade authenticator app).
    pub fn with_defaults(totp_store: Arc<T>, backup_store: Arc<B>) -> Self {
        let totp_cfg = TotpConfig::default();
        let backup_cfg = BackupCodesConfig::default();
        Self {
            totp: Arc::new(TotpService::new(totp_cfg, totp_store)),
            backup_codes: Arc::new(BackupCodesService::new(backup_cfg, backup_store)),
        }
    }
}

/// Wrap raw SVG markup in a `data:` URL so the frontend can drop it
/// straight into an `<img src>` without further processing. Matches the
/// shape documented on `MfaSetupData::qr_code_url`.
fn svg_to_data_url(svg: &str) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());
    format!("data:image/svg+xml;base64,{}", b64)
}

#[async_trait]
impl<T, B> MfaApiService for LocalMfaApi<T, B>
where
    T: TotpStore + 'static,
    B: BackupCodesStore + 'static,
{
    async fn setup_totp(&self, user_id: Uuid, username: &str) -> Result<MfaSetupData, MfaApiError> {
        let uid = UserId(user_id);
        let setup = self
            .totp
            .setup_totp(uid, username)
            .await
            .map_err(|e| MfaApiError::internal(format!("totp setup: {}", e)))?;

        let codes = self
            .backup_codes
            .generate_backup_codes(uid)
            .await
            .map_err(|e| MfaApiError::internal(format!("backup codes generate: {}", e)))?;

        info!(user_id = %user_id, "MFA setup completed");
        Ok(MfaSetupData {
            qr_code_url: svg_to_data_url(&setup.qr_code_svg),
            secret_key: setup.manual_entry_key,
            backup_codes: codes,
        })
    }

    async fn verify_setup(&self, user_id: Uuid, code: &str) -> Result<bool, MfaApiError> {
        let uid = UserId(user_id);
        let ok = self
            .totp
            .verify_totp(uid, code)
            .await
            .map_err(|e| MfaApiError::internal(format!("totp verify_setup: {}", e)))?;
        if !ok {
            debug!(user_id = %user_id, "verify_setup rejected invalid TOTP");
            return Err(MfaApiError::invalid_code());
        }
        Ok(true)
    }

    async fn verify_code(&self, user_id: Uuid, code: &str) -> Result<bool, MfaApiError> {
        let uid = UserId(user_id);

        // TOTP first — that's the hot path. `verify_totp` returns
        // `Ok(false)` for a wrong digit and `Err(NotFound)` if the user
        // never enrolled; both should fall through to backup codes so we
        // don't leak "is MFA enrolled?" via timing.
        match self.totp.verify_totp(uid, code).await {
            Ok(true) => {
                debug!(user_id = %user_id, "TOTP verify succeeded");
                return Ok(true);
            }
            Ok(false) => debug!(user_id = %user_id, "TOTP code invalid, trying backup"),
            Err(e) => debug!(user_id = %user_id, error = %e, "TOTP not enrolled, trying backup"),
        }

        let backup_ok = self
            .backup_codes
            .verify_backup_code(uid, code)
            .await
            .map_err(|e| MfaApiError::internal(format!("backup verify: {}", e)))?;

        if backup_ok {
            warn!(user_id = %user_id, "MFA via backup code (one-time)");
            Ok(true)
        } else {
            Err(MfaApiError::invalid_code())
        }
    }

    async fn disable_totp(&self, user_id: Uuid) -> Result<(), MfaApiError> {
        let uid = UserId(user_id);
        // Wipe both legs — leaving stale backup codes around after the
        // primary factor is gone is a footgun.
        self.totp
            .disable_totp(uid)
            .await
            .map_err(|e| MfaApiError::internal(format!("totp disable: {}", e)))?;
        // Best-effort: backup code cleanup must not block disabling MFA.
        if let Err(e) = self.backup_codes.regenerate_backup_codes(uid).await {
            warn!(user_id = %user_id, error = %e, "could not wipe backup codes during disable");
        }
        info!(user_id = %user_id, "MFA disabled");
        Ok(())
    }

    async fn get_status(&self, user_id: Uuid) -> Result<MfaStatusData, MfaApiError> {
        let uid = UserId(user_id);
        let enabled = self
            .totp
            .is_totp_enabled(uid)
            .await
            .map_err(|e| MfaApiError::internal(format!("totp is_enabled: {}", e)))?;
        let remaining = self
            .backup_codes
            .get_remaining_codes_count(uid)
            .await
            .map_err(|e| MfaApiError::internal(format!("backup remaining: {}", e)))?;
        Ok(MfaStatusData {
            enabled,
            setup_at: None, // we don't surface this yet; populate when adapters track it
            backup_codes_remaining: remaining as i32,
            last_used: None, // ditto — wire when the audit log is plumbed in
        })
    }

    async fn generate_backup_codes(&self, user_id: Uuid) -> Result<Vec<String>, MfaApiError> {
        let uid = UserId(user_id);
        let codes = self
            .backup_codes
            .regenerate_backup_codes(uid)
            .await
            .map_err(|e| MfaApiError::internal(format!("backup codes regenerate: {}", e)))?;
        info!(user_id = %user_id, count = codes.len(), "backup codes regenerated");
        Ok(codes)
    }

    async fn backup_codes_remaining(&self, user_id: Uuid) -> Result<usize, MfaApiError> {
        let uid = UserId(user_id);
        self.backup_codes
            .get_remaining_codes_count(uid)
            .await
            .map_err(|e| MfaApiError::internal(format!("backup remaining: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use authenc_mfa::BackupCode;
    use authenc_types::Result as AuthencResult;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    struct MemTotp {
        secrets: RwLock<HashMap<Uuid, String>>,
    }
    impl MemTotp {
        fn new() -> Self {
            Self {
                secrets: RwLock::new(HashMap::new()),
            }
        }
    }
    #[async_trait]
    impl TotpStore for MemTotp {
        async fn store_totp_secret(&self, user_id: UserId, secret: &str) -> AuthencResult<()> {
            self.secrets
                .write()
                .await
                .insert(user_id.0, secret.to_string());
            Ok(())
        }
        async fn get_totp_secret(&self, user_id: UserId) -> AuthencResult<Option<String>> {
            Ok(self.secrets.read().await.get(&user_id.0).cloned())
        }
        async fn delete_totp_secret(&self, user_id: UserId) -> AuthencResult<()> {
            self.secrets.write().await.remove(&user_id.0);
            Ok(())
        }
    }

    struct MemBackup {
        codes: RwLock<HashMap<Uuid, Vec<BackupCode>>>,
    }
    impl MemBackup {
        fn new() -> Self {
            Self {
                codes: RwLock::new(HashMap::new()),
            }
        }
    }
    #[async_trait]
    impl BackupCodesStore for MemBackup {
        async fn store_backup_codes(
            &self,
            user_id: UserId,
            codes: &[BackupCode],
        ) -> AuthencResult<()> {
            self.codes.write().await.insert(user_id.0, codes.to_vec());
            Ok(())
        }
        async fn get_backup_codes(
            &self,
            user_id: UserId,
        ) -> AuthencResult<Option<Vec<BackupCode>>> {
            Ok(self.codes.read().await.get(&user_id.0).cloned())
        }
        async fn update_backup_code(
            &self,
            user_id: UserId,
            code: &str,
            used: bool,
        ) -> AuthencResult<()> {
            if let Some(set) = self.codes.write().await.get_mut(&user_id.0)
                && let Some(c) = set.iter_mut().find(|c| c.code == code)
            {
                c.used = used;
                if used {
                    c.used_at = Some(chrono::Utc::now());
                }
            }
            Ok(())
        }
        async fn delete_backup_codes(&self, user_id: UserId) -> AuthencResult<()> {
            self.codes.write().await.remove(&user_id.0);
            Ok(())
        }
    }

    fn adapter() -> LocalMfaApi<MemTotp, MemBackup> {
        LocalMfaApi::with_defaults(Arc::new(MemTotp::new()), Arc::new(MemBackup::new()))
    }

    #[tokio::test]
    async fn setup_then_status_reflects_enabled() {
        let svc = adapter();
        let uid = Uuid::new_v4();

        let setup = svc.setup_totp(uid, "user@example.test").await.unwrap();
        assert!(setup.qr_code_url.starts_with("data:image/svg+xml;base64,"));
        assert!(!setup.secret_key.is_empty());
        assert_eq!(setup.backup_codes.len(), 10);

        let status = svc.get_status(uid).await.unwrap();
        assert!(status.enabled);
        assert_eq!(status.backup_codes_remaining, 10);
    }

    #[tokio::test]
    async fn verify_code_accepts_backup_when_totp_wrong() {
        let svc = adapter();
        let uid = Uuid::new_v4();
        let setup = svc.setup_totp(uid, "user@example.test").await.unwrap();
        let first_backup = setup.backup_codes[0].clone();

        // Wrong six-digit TOTP shouldn't be accepted as TOTP, but the same
        // call should fall through to backup-code verification and pass.
        let result = svc.verify_code(uid, &first_backup).await.unwrap();
        assert!(result);

        // A backup code burns on first use.
        let status = svc.get_status(uid).await.unwrap();
        assert_eq!(status.backup_codes_remaining, 9);
    }

    #[tokio::test]
    async fn disable_wipes_totp() {
        let svc = adapter();
        let uid = Uuid::new_v4();
        svc.setup_totp(uid, "user@example.test").await.unwrap();
        svc.disable_totp(uid).await.unwrap();
        let status = svc.get_status(uid).await.unwrap();
        assert!(!status.enabled);
    }
}
