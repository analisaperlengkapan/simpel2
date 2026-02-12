//! MySIMKARI data synchronization service
//!
//! This service periodically fetches MySIMKARI pegawai and satker data
//! from the layanan-integrasi gRPC service and auto-provisions user accounts
//! in authenc. Each pegawai gets a user account with NIP as username
//! and NIP as default password.

use crate::database::Database;
use crate::error::AuthencError;
use crate::models::satker::{Satker, SatkerType};
use crate::models::user::CreateUserRequest;
use crate::services::integrasi_client::{IntegrasiClient, IntegrasiClientError};
use crate::services::satker_authorization::SatkerAuthorizationService;
use crate::services::stores::user_store::{UserStore, UserStoreTrait};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

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

/// MySIMKARI data synchronization service
pub struct MysimkariSyncService {
    /// gRPC client for layanan-integrasi
    client: Arc<IntegrasiClient>,
    /// User store for creating/updating users
    user_store: Arc<UserStore>,
    /// Satker authorization service for hierarchy updates
    satker_auth: Arc<SatkerAuthorizationService>,
    /// Database connection for direct queries
    db: Arc<Database>,
    /// Last sync result
    last_result: Arc<RwLock<Option<FullSyncResult>>>,
}

impl MysimkariSyncService {
    /// Create a new MysimkariSyncService
    pub fn new(
        client: Arc<IntegrasiClient>,
        user_store: Arc<UserStore>,
        satker_auth: Arc<SatkerAuthorizationService>,
        db: Arc<Database>,
    ) -> Self {
        Self {
            client,
            user_store,
            satker_auth,
            db,
            last_result: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the background sync scheduler
    pub fn start_scheduler(self: &Arc<Self>, interval_minutes: u64) {
        let service = Arc::clone(self);
        let interval = std::time::Duration::from_secs(interval_minutes * 60);

        info!(
            "Starting MySIMKARI sync scheduler (interval: {} minutes)",
            interval_minutes
        );

        tokio::spawn(async move {
            // Small initial delay to let the service fully start
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            loop {
                info!("MySIMKARI sync scheduler: starting sync cycle");

                match service.sync_all().await {
                    Ok(result) => {
                        info!(
                            "MySIMKARI sync completed in {:.1}s: satker({} created, {} updated), pegawai({} created, {} updated)",
                            result.duration_secs,
                            result.satker.created, result.satker.updated,
                            result.pegawai.created, result.pegawai.updated,
                        );
                    }
                    Err(e) => {
                        error!("MySIMKARI sync failed: {}", e);
                    }
                }

                tokio::time::sleep(interval).await;
            }
        });
    }

    /// Run a complete sync: satker first, then pegawai
    pub async fn sync_all(&self) -> Result<FullSyncResult, AuthencError> {
        let start = std::time::Instant::now();

        // 1. Sync satker data first (needed for hierarchy)
        info!("MySIMKARI sync: starting satker sync");
        let satker_result = self.sync_satker().await?;

        // 2. Sync pegawai data (per satker)
        info!("MySIMKARI sync: starting pegawai sync");
        let pegawai_result = self.sync_pegawai().await?;

        let result = FullSyncResult {
            satker: satker_result,
            pegawai: pegawai_result,
            duration_secs: start.elapsed().as_secs_f64(),
        };

        // Store last result
        *self.last_result.write().await = Some(result.clone());

        Ok(result)
    }

    /// Sync satker data from MySIMKARI via integrasi gRPC
    pub async fn sync_satker(&self) -> Result<SyncResult, AuthencError> {
        let mut result = SyncResult::default();

        // Fetch all satker from integrasi
        let mysimkari_satkers = self
            .client
            .get_mysimkari_satker(None)
            .await
            .map_err(|e| AuthencError::internal(format!("Failed to fetch satker: {}", e)))?;

        info!("Fetched {} satker from MySIMKARI", mysimkari_satkers.len());

        // Convert to authenc Satker model
        let satkers: Vec<Satker> = mysimkari_satkers
            .iter()
            .map(|ms| Self::map_mysimkari_satker_to_satker(ms))
            .collect();

        result.created = satkers.len(); // All treated as upserts

        // Update the satker hierarchy in SatkerAuthorizationService
        self.satker_auth.update_hierarchy(satkers).await;

        info!(
            "Satker hierarchy updated with {} satker records",
            result.created
        );

        Ok(result)
    }

    /// Sync pegawai data: auto-provision user accounts
    pub async fn sync_pegawai(&self) -> Result<SyncResult, AuthencError> {
        let mut result = SyncResult::default();

        // First, get all satker to iterate
        let satkers = self
            .client
            .get_mysimkari_satker(None)
            .await
            .map_err(|e| {
                AuthencError::internal(format!("Failed to fetch satker for pegawai sync: {}", e))
            })?;

        for satker in &satkers {
            match self
                .sync_pegawai_for_satker(&satker.kode_satker, &mut result)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    warn!(
                        "Failed to sync pegawai for satker {}: {}",
                        satker.kode_satker, e
                    );
                    result.errors.push(format!(
                        "Satker {}: {}",
                        satker.kode_satker, e
                    ));
                }
            }
        }

        info!(
            "Pegawai sync complete: {} created, {} updated, {} skipped, {} failed",
            result.created, result.updated, result.skipped, result.failed
        );

        Ok(result)
    }

    /// Sync pegawai for a specific satker
    async fn sync_pegawai_for_satker(
        &self,
        kode_satker: &str,
        result: &mut SyncResult,
    ) -> Result<(), AuthencError> {
        let pegawai_list = self
            .client
            .get_mysimkari_pegawai(kode_satker)
            .await
            .map_err(|e| {
                AuthencError::internal(format!("Failed to fetch pegawai: {}", e))
            })?;

        debug!(
            "Processing {} pegawai for satker {}",
            pegawai_list.len(),
            kode_satker
        );

        for pegawai in &pegawai_list {
            if pegawai.nip.is_empty() {
                result.skipped += 1;
                continue;
            }

            match self.provision_or_update_user(pegawai).await {
                Ok(UserProvisionAction::Created) => result.created += 1,
                Ok(UserProvisionAction::Updated) => result.updated += 1,
                Ok(UserProvisionAction::Skipped) => result.skipped += 1,
                Err(e) => {
                    result.failed += 1;
                    result.errors.push(format!(
                        "NIP {}: {}",
                        pegawai.nip, e
                    ));
                    warn!(
                        "Failed to provision user for NIP {}: {}",
                        pegawai.nip, e
                    );
                }
            }
        }

        Ok(())
    }

    /// Create or update a user account from pegawai data
    ///
    /// - If user doesn't exist: CREATE with NIP as username and NIP as default password
    /// - If user exists: UPDATE nama, jabatan, satker_code (skip password)
    async fn provision_or_update_user(
        &self,
        pegawai: &crate::services::integrasi_client::integrasi_proto::v1::MysimkariPegawai,
    ) -> Result<UserProvisionAction, AuthencError> {
        let nip = &pegawai.nip;

        // Check if user already exists (by username = NIP)
        let existing_user = self.user_store.get_user_by_username(nip).await?;

        if let Some(existing) = existing_user {
            // User exists — check if we need to update
            let needs_update = existing.nama.as_deref() != Some(&pegawai.nama)
                || existing.jabatan.as_deref() != Some(&pegawai.jabatan)
                || existing.satker_code != pegawai.kode_satker;

            if needs_update {
                // Update user fields via direct SQL (nip/nama/jabatan/satker_code)
                self.update_pegawai_fields(
                    &existing.id,
                    &pegawai.nama,
                    &pegawai.jabatan,
                    &pegawai.kode_satker,
                    pegawai.email.as_str(),
                )
                .await?;

                debug!("Updated user {} (NIP: {})", existing.id, nip);
                Ok(UserProvisionAction::Updated)
            } else {
                Ok(UserProvisionAction::Skipped)
            }
        } else {
            // User doesn't exist — create new account
            let email = if pegawai.email.is_empty() {
                format!("{}@kejaksaan.go.id", nip)
            } else {
                pegawai.email.clone()
            };

            let create_request = CreateUserRequest {
                username: nip.clone(),
                email,
                satker_code: pegawai.kode_satker.clone(),
                password: Some(nip.clone()), // Default password = NIP (will be bcrypt hashed by create_user)
                first_name: Some(pegawai.nama.clone()),
                last_name: None,
                nip: Some(nip.clone()),
                nama: Some(pegawai.nama.clone()),
                jabatan: Some(pegawai.jabatan.clone()),
                phone_number: if pegawai.telepon.is_empty() {
                    None
                } else {
                    Some(pegawai.telepon.clone())
                },
                realm_id: None,
                organization_id: None,
                roles: None,
                attributes: Some(serde_json::json!({
                    "source": "mysimkari",
                    "pangkat": pegawai.pangkat,
                    "golongan": pegawai.golongan,
                    "unit_kerja": pegawai.unit_kerja,
                    "status": pegawai.status,
                    "synced_at": Utc::now().to_rfc3339(),
                })),
            };

            let user = self.user_store.add_user(create_request).await?;

            // Set NIP-specific fields and require_password_change via direct SQL
            // because create_user doesn't insert nip/nama/jabatan/satker_code
            self.set_pegawai_fields_after_create(
                &user.id,
                nip,
                &pegawai.nama,
                &pegawai.jabatan,
                &pegawai.kode_satker,
            )
            .await?;

            info!(
                "Created user {} for pegawai {} (NIP: {})",
                user.id, pegawai.nama, nip
            );

            Ok(UserProvisionAction::Created)
        }
    }

    /// Set pegawai-specific fields after user creation
    /// (because create_user doesn't include nip/nama/jabatan/satker_code in INSERT)
    async fn set_pegawai_fields_after_create(
        &self,
        user_id: &Uuid,
        nip: &str,
        nama: &str,
        jabatan: &str,
        satker_code: &str,
    ) -> Result<(), AuthencError> {
        let now = Utc::now();
        let query = r#"
            UPDATE users SET
                nip = $2,
                nama = $3,
                jabatan = $4,
                satker_code = $5,
                require_password_change = true,
                updated_at = $6
            WHERE id = $1
        "#;

        let conn = self.db.get_connection().await?;
        conn.execute(
            query,
            &[user_id, &nip, &nama, &jabatan, &satker_code, &now],
        )
        .await
        .map_err(|e| {
            AuthencError::database(format!(
                "Failed to set pegawai fields for user {}: {}",
                user_id, e
            ))
        })?;

        Ok(())
    }

    /// Update pegawai-specific fields for an existing user
    async fn update_pegawai_fields(
        &self,
        user_id: &Uuid,
        nama: &str,
        jabatan: &str,
        satker_code: &str,
        email: &str,
    ) -> Result<(), AuthencError> {
        let now = Utc::now();
        let query = r#"
            UPDATE users SET
                nama = $2,
                jabatan = $3,
                satker_code = $4,
                email = CASE WHEN $5 = '' THEN email ELSE $5 END,
                updated_at = $6
            WHERE id = $1
        "#;

        let conn = self.db.get_connection().await?;
        conn.execute(
            query,
            &[user_id, &nama, &jabatan, &satker_code, &email, &now],
        )
        .await
        .map_err(|e| {
            AuthencError::database(format!(
                "Failed to update pegawai fields for user {}: {}",
                user_id, e
            ))
        })?;

        Ok(())
    }

    /// Map a MySIMKARI satker proto message to authenc's Satker model
    fn map_mysimkari_satker_to_satker(
        ms: &crate::services::integrasi_client::integrasi_proto::v1::MysimkariSatker,
    ) -> Satker {
        // Determine satker type and level from kode_wilayah
        let (satker_type, level, parent_code) = Self::infer_hierarchy(&ms.kode_satker, &ms.kode_wilayah);

        Satker {
            id: Uuid::new_v4(),
            code: ms.kode_satker.clone(),
            name: ms.nama_satker.clone(),
            description: None,
            parent_code,
            level,
            satker_type,
            active: true,
            attributes: Some(serde_json::json!({
                "alamat": ms.alamat,
                "telepon": ms.telepon,
                "email": ms.email,
                "kode_wilayah": ms.kode_wilayah,
                "nama_wilayah": ms.nama_wilayah,
                "jumlah_pegawai": ms.jumlah_pegawai,
            })),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Infer satker hierarchy (type, level, parent) from kode_satker and kode_wilayah
    ///
    /// Kejaksaan hierarchy pattern:
    /// - Level 0: Kejaksaan Agung (pusat) — kode_satker starts with "01"
    /// - Level 1: Kejaksaan Tinggi (wilayah) — grouped by kode_wilayah
    /// - Level 2: Kejaksaan Negeri (kabupaten/kota)
    /// - Level 3: Cabang Kejaksaan Negeri
    fn infer_hierarchy(
        kode_satker: &str,
        kode_wilayah: &str,
    ) -> (SatkerType, i32, Option<String>) {
        let code_len = kode_satker.len();

        // Simple heuristic based on code patterns
        // This can be refined based on actual MySIMKARI code structure
        if kode_satker.starts_with("01") && code_len <= 4 {
            // Pusat / Kejaksaan Agung
            (SatkerType::Pusat, 0, None)
        } else if !kode_wilayah.is_empty() {
            // Has wilayah code — likely Kejaksaan Tinggi or lower
            if code_len <= 6 {
                // Kejaksaan Tinggi
                (SatkerType::KejaksaanTinggi, 1, Some("01".to_string()))
            } else if code_len <= 9 {
                // Kejaksaan Negeri
                let parent = if kode_wilayah.len() >= 2 {
                    Some(kode_wilayah[..2].to_string())
                } else {
                    None
                };
                (SatkerType::KejaksaanNegeri, 2, parent)
            } else {
                // Cabang
                let parent = if kode_satker.len() >= 9 {
                    Some(kode_satker[..9].to_string())
                } else {
                    None
                };
                (SatkerType::Cabang, 3, parent)
            }
        } else {
            // Default
            (SatkerType::KejaksaanNegeri, 2, None)
        }
    }

    /// Get the last sync result
    pub async fn last_sync_result(&self) -> Option<FullSyncResult> {
        self.last_result.read().await.clone()
    }
}

/// Action taken when provisioning a user
enum UserProvisionAction {
    Created,
    Updated,
    Skipped,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_result_total() {
        let result = SyncResult {
            created: 5,
            updated: 3,
            skipped: 2,
            failed: 1,
            errors: vec![],
        };
        assert_eq!(result.total(), 11);
    }

    #[test]
    fn test_infer_hierarchy_pusat() {
        let (st, level, parent) = MysimkariSyncService::infer_hierarchy("01", "");
        assert_eq!(level, 0);
        assert!(matches!(st, SatkerType::Pusat));
        assert!(parent.is_none());
    }

    #[test]
    fn test_infer_hierarchy_kejaksaan_tinggi() {
        let (st, level, parent) = MysimkariSyncService::infer_hierarchy("010001", "01");
        assert_eq!(level, 1);
        assert!(matches!(st, SatkerType::KejaksaanTinggi));
        assert_eq!(parent, Some("01".to_string()));
    }

    #[test]
    fn test_infer_hierarchy_kejaksaan_negeri() {
        let (st, level, parent) = MysimkariSyncService::infer_hierarchy("010001001", "0100");
        assert_eq!(level, 2);
        assert!(matches!(st, SatkerType::KejaksaanNegeri));
        assert_eq!(parent, Some("01".to_string()));
    }
}
