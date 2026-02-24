//! MySIMKARI data synchronization service — **Stub Implementation**
//!
//! This service periodically fetches MySIMKARI pegawai and satker data
//! from the layanan-integrasi gRPC service and auto-provisions user accounts
//! in authenc. Each pegawai gets a user account with NIP as username
//! and NIP as default password.
//!
//! ## Current Status
//!
//! This module is **stubbed** because:
//! - `IntegrasiClient` is defined in the `layanan-integrasi` binary crate,
//!   not in a shared library crate importable by `authenc-federation`.
//! - `SatkerAuthorizationService` lives in `authenc_core::services::satker_authorization`
//!   but requires additional wiring not yet available.
//! - `UserStore` / `UserStoreTrait` live in `authenc_core::stores`.
//!
//! Once `layanan-integrasi` exposes a library crate with the gRPC client,
//! this module can be fully implemented.

use authenc_core::error::AuthencError;
use authenc_storage::Database;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

// ── Data types ─────────────────────────────────────────────────────────────

/// Sync result statistics
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    /// Number of records created
    pub created: usize,
    /// Number of records updated
    pub updated: usize,
    /// Number of records skipped (no changes)
    pub skipped: usize,
    /// Number of records that failed
    pub failed: usize,
    /// Error messages for failed records
    pub errors: Vec<String>,
}

impl SyncResult {
    pub fn total(&self) -> usize {
        self.created + self.updated + self.skipped + self.failed
    }
}

/// Combined sync result for the full sync operation
#[derive(Debug, Clone, Default)]
pub struct FullSyncResult {
    pub satker: SyncResult,
    pub pegawai: SyncResult,
    pub duration_secs: f64,
}

/// MySIMKARI sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysimkariSyncConfig {
    /// Whether automatic sync is enabled
    pub enabled: bool,
    /// Sync interval in seconds
    pub sync_interval_secs: u64,
    /// Default realm ID for provisioned users
    pub default_realm_id: Uuid,
    /// Whether to auto-create users
    pub auto_create_users: bool,
    /// Whether to auto-update existing users
    pub auto_update_users: bool,
}

impl Default for MysimkariSyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sync_interval_secs: 3600,
            default_realm_id: Uuid::nil(),
            auto_create_users: true,
            auto_update_users: true,
        }
    }
}

/// Sync status
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub is_running: bool,
    pub last_sync_at: Option<chrono::DateTime<Utc>>,
    pub last_result: Option<FullSyncResult>,
    pub next_sync_at: Option<chrono::DateTime<Utc>>,
}

// ── Service ────────────────────────────────────────────────────────────────

/// MySIMKARI data synchronization service
///
/// **Note**: Currently stubbed. See module documentation for details.
pub struct MysimkariSyncService {
    #[allow(dead_code)]
    db: Arc<Database>,
    config: MysimkariSyncConfig,
    last_result: Arc<RwLock<Option<FullSyncResult>>>,
}

impl MysimkariSyncService {
    /// Create a new MySIMKARI sync service
    pub fn new(db: Arc<Database>, config: MysimkariSyncConfig) -> Self {
        Self {
            db,
            config,
            last_result: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current sync status
    pub async fn status(&self) -> SyncStatus {
        let last_result = self.last_result.read().await.clone();
        SyncStatus {
            is_running: false,
            last_sync_at: None,
            last_result,
            next_sync_at: None,
        }
    }

    /// Perform a full synchronization (stub)
    pub async fn full_sync(&self) -> Result<FullSyncResult, AuthencError> {
        if !self.config.enabled {
            warn!("MySIMKARI sync is disabled");
            return Ok(FullSyncResult::default());
        }

        info!("Starting MySIMKARI full sync (stub — no IntegrasiClient available)");
        let start = Utc::now();

        // TODO: Implement once IntegrasiClient is available as a library crate
        let result = FullSyncResult {
            satker: SyncResult::default(),
            pegawai: SyncResult::default(),
            duration_secs: (Utc::now() - start).num_milliseconds() as f64 / 1000.0,
        };

        *self.last_result.write().await = Some(result.clone());
        info!("MySIMKARI full sync completed (stub): {:?}", result);
        Ok(result)
    }

    /// Sync satker hierarchy (stub)
    pub async fn sync_satker(&self) -> Result<SyncResult, AuthencError> {
        warn!("MySIMKARI satker sync not implemented (stub)");
        Ok(SyncResult::default())
    }

    /// Sync pegawai / user provisioning (stub)
    pub async fn sync_pegawai(&self) -> Result<SyncResult, AuthencError> {
        warn!("MySIMKARI pegawai sync not implemented (stub)");
        Ok(SyncResult::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_result_total() {
        let result = SyncResult {
            created: 10,
            updated: 5,
            skipped: 3,
            failed: 2,
            errors: vec!["err1".into()],
        };
        assert_eq!(result.total(), 20);
    }

    #[test]
    fn test_default_config() {
        let config = MysimkariSyncConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.sync_interval_secs, 3600);
    }
}
