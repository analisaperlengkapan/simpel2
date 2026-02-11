//! # Pemakaian BMN Services
//!
//! Business logic layer for BMN usage permit system.
//! Integrates with workflow engine for approval processes.
//!
//! Requirements: REQ-P001 through REQ-P016, REQ-W004, REQ-W005

use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::workflow::engine::{WorkflowEngine, TransitionRequest};

use super::models::*;
use super::repository::PemakaianBmnRepository;

/// Service for BMN usage permit business logic
#[derive(Clone)]
pub struct PemakaianBmnService {
    repository: Arc<PemakaianBmnRepository>,
    workflow_engine: Arc<WorkflowEngine>,
    notifikasi_client: Option<Arc<tokio::sync::Mutex<crate::workflow::NotifikasiClient>>>,
    dokumen_client: Option<Arc<tokio::sync::Mutex<crate::workflow::DokumenClient>>>,
}

impl PemakaianBmnService {
    /// Create a new service instance with workflow engine
    ///
    /// Requirements: REQ-P004
    pub fn new(repository: PemakaianBmnRepository, workflow_engine: WorkflowEngine) -> Self {
        Self {
            repository: Arc::new(repository),
            workflow_engine: Arc::new(workflow_engine),
            notifikasi_client: None,
            dokumen_client: None,
        }
    }

    /// Set the notifikasi service client
    pub fn with_notifikasi_client(
        mut self,
        client: Arc<tokio::sync::Mutex<crate::workflow::NotifikasiClient>>,
    ) -> Self {
        self.notifikasi_client = Some(client);
        self
    }

    /// Set the dokumen service client
    pub fn with_dokumen_client(
        mut self,
        client: Arc<tokio::sync::Mutex<crate::workflow::DokumenClient>>,
    ) -> Self {
        self.dokumen_client = Some(client);
        self
    }

    /// Get permit with allowed transitions
    ///
    /// Requirements: REQ-P001
    pub async fn get_permit_detail(&self, id: Uuid) -> AppResult<IzinPemakaianDetailResponse> {
        let izin = self.repository.get_by_id(id).await?;

        // Get allowed transitions from workflow engine
        let current_status = PemakaianBmnStatus::from_state_name(&izin.status)
            .unwrap_or(PemakaianBmnStatus::Draft);
        let allowed_transitions = self.get_allowed_transitions(current_status);

        // Calculate expiry info
        let (days_until_expiry, is_expiring_soon) = {
            let today = chrono::Utc::now().date_naive();
            let days = (izin.tanggal_selesai - today).num_days();
            let expiring = days >= 0 && days <= 30;
            (Some(days), expiring)
        };

        Ok(IzinPemakaianDetailResponse {
            izin,
            allowed_transitions,
            days_until_expiry,
            is_expiring_soon,
        })
    }

    /// Transition permit to a new status using workflow engine
    ///
    /// This method uses the centralized workflow engine for:
    /// - Transition validation
    /// - Role-based authorization
    /// - Audit logging
    /// - Activity tracking
    ///
    /// Requirements: REQ-P004, REQ-W004, REQ-W005
    pub async fn transition_permit_status(
        &self,
        id: Uuid,
        request: WorkflowTransitionRequest,
        user_id: Uuid,
    ) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id).await?;
        let current_status = PemakaianBmnStatus::from_state_name(&current.status)
            .ok_or_else(|| AppError::BadRequest(format!("Unknown current status: {}", current.status)))?;
        let target_status = PemakaianBmnStatus::from_state_name(&request.target_status)
            .ok_or_else(|| AppError::BadRequest(format!("Invalid target status: {}", request.target_status)))?;

        // Convert status codes to state names for workflow engine
        let from_state = current_status.to_state_name();
        let to_state = target_status.to_state_name();

        // Validate transition using workflow engine
        if !self
            .workflow_engine
            .config()
            .is_valid_transition(from_state, to_state)
        {
            return Err(AppError::BadRequest(format!(
                "Cannot transition from {} to {}",
                current_status.label(),
                target_status.label()
            )));
        }

        // Some transitions require comment
        let requires_comment = matches!(
            target_status,
            PemakaianBmnStatus::Rejected | PemakaianBmnStatus::Revoked
        );

        if requires_comment && request.catatan.is_none() {
            return Err(AppError::BadRequest(
                "Catatan diperlukan untuk transisi ini".to_string(),
            ));
        }

        info!(
            "Transitioning permit {} from {} to {} via workflow engine",
            id,
            current_status.label(),
            target_status.label()
        );

        // Use workflow engine for transition
        let transition_request = TransitionRequest {
            entity_id: id,
            from_state: from_state.to_string(),
            to_state: to_state.to_string(),
            user_id,
            catatan: request.catatan.clone(),
            ip_address: "0.0.0.0".to_string(), // TODO: Get from request context
        };

        // Execute transition through workflow engine
        let _transition_result = self
            .workflow_engine
            .transition(transition_request)
            .await
            .map_err(|e| AppError::Internal(format!("Workflow transition failed: {}", e)))?;

        // Get updated permit
        let updated = self.repository.get_by_id(id).await?;

        // Special handling for APPROVED -> ACTIVE transition
        if target_status == PemakaianBmnStatus::Approved {
            info!("Permit {} approved, can now be activated", id);
        }

        // Special handling for ACTIVE status
        if target_status == PemakaianBmnStatus::Active {
            info!("Permit {} is now active", id);
            // TODO: Schedule expiry notification
        }

        Ok(updated)
    }

    /// Get allowed next states for a permit using workflow engine
    ///
    /// Requirements: REQ-P004
    pub fn get_allowed_transitions(
        &self,
        current_status: PemakaianBmnStatus,
    ) -> Vec<WorkflowTransitionInfo> {
        let current_state = current_status.to_state_name();
        let next_states = self.workflow_engine.get_next_states(current_state);

        next_states
            .into_iter()
            .filter_map(|state_name| {
                PemakaianBmnStatus::from_state_name(&state_name).map(|status| {
                    WorkflowTransitionInfo {
                        status: status.to_state_name().to_string(),
                        label: status.label().to_string(),
                        requires_comment: matches!(
                            status,
                            PemakaianBmnStatus::Rejected | PemakaianBmnStatus::Revoked
                        ),
                    }
                })
            })
            .collect()
    }

    /// Create a new permit request
    ///
    /// Requirements: REQ-P001, REQ-P002, REQ-P003
    pub async fn create_permit(
        &self,
        request: CreateIzinPemakaianRequest,
        user_id: Uuid,
        user_nama: String,
    ) -> AppResult<IzinPemakaianBmn> {
        // Validate request
        request.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

        // Validate BMN type-specific fields
        self.validate_bmn_type_fields(&request)?;

        // Check BMN availability (REQ-P002, REQ-P003)
        let availability = self.repository.check_bmn_availability(&request.bmn_nup).await?;
        if !availability.is_available {
            return Err(AppError::BadRequest(format!(
                "BMN {} sedang digunakan oleh {} hingga {}",
                request.bmn_nup,
                availability.active_permit_holder.unwrap_or_default(),
                availability.active_permit_expires.map(|d| d.to_string()).unwrap_or_default()
            )));
        }

        // Validate date range
        if request.tanggal_selesai <= request.tanggal_mulai {
            return Err(AppError::BadRequest(
                "Tanggal selesai harus lebih besar dari tanggal mulai".to_string(),
            ));
        }

        info!(
            "Creating new permit for BMN {} by pegawai {}",
            request.bmn_nup, request.pegawai_nip
        );

        // Create permit
        let permit = self.repository.create(request, user_id, user_nama).await?;

        Ok(permit)
    }

    /// Update a permit (only in DRAFT status)
    ///
    /// Requirements: REQ-P001
    pub async fn update_permit(
        &self,
        id: Uuid,
        request: UpdateIzinPemakaianRequest,
        user_id: Uuid,
        user_nama: String,
    ) -> AppResult<IzinPemakaianBmn> {
        // Validate request
        request.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

        // Validate date range if both dates provided
        if let (Some(start), Some(end)) = (request.tanggal_mulai, request.tanggal_selesai) {
            if end <= start {
                return Err(AppError::BadRequest(
                    "Tanggal selesai harus lebih besar dari tanggal mulai".to_string(),
                ));
            }
        }

        info!("Updating permit {} by user {}", id, user_id);

        let permit = self.repository.update(id, request, user_id, user_nama).await?;

        Ok(permit)
    }

    /// List permits with pagination and filters
    ///
    /// Requirements: REQ-P001, REQ-P011
    pub async fn list_permits(&self, query: ListPermitsQuery) -> AppResult<PaginatedPermitsResponse> {
        self.repository.list(query).await
    }

    /// Activate a permit (generate permit number and set to ACTIVE)
    ///
    /// Requirements: REQ-P005, REQ-P006
    pub async fn activate_permit(&self, id: Uuid, user_id: Uuid) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id).await?;

        // Can only activate APPROVED permits
        if current.status != "APPROVED" {
            return Err(AppError::BadRequest(
                "Hanya izin dengan status APPROVED yang dapat diaktifkan".to_string(),
            ));
        }

        // Generate permit number (REQ-P005)
        let nomor_izin = self.repository.generate_permit_number(id).await?;
        info!("Generated permit number {} for permit {}", nomor_izin, id);

        // Transition to ACTIVE
        let transition_request = WorkflowTransitionRequest {
            target_status: "ACTIVE".to_string(),
            catatan: Some(format!("Izin diaktifkan dengan nomor {}", nomor_izin)),
        };

        let mut permit = self.transition_permit_status(id, transition_request, user_id).await?;

        // Generate permit document (REQ-P006)
        if let Some(dokumen_client) = &self.dokumen_client {
            info!("Generating permit document for permit {}", id);

            match self.generate_permit_document(&permit, dokumen_client).await {
                Ok((document_id, document_url)) => {
                    info!(
                        "Successfully generated document {} for permit {}",
                        document_id, id
                    );

                    // Update permit with document reference
                    permit = self.repository
                        .update_document_fields(id, document_id, document_url)
                        .await?;
                }
                Err(e) => {
                    warn!(
                        "Failed to generate document for permit {}: {}. Manual generation required.",
                        id, e
                    );
                    // Don't fail activation if document generation fails
                    // Document can be generated manually later
                }
            }
        } else {
            warn!("Dokumen client not configured, skipping document generation for permit {}", id);
        }

        Ok(permit)
    }

    /// Generate permit document via dokumen service
    ///
    /// Requirements: REQ-P006, REQ-D002, REQ-D004
    async fn generate_permit_document(
        &self,
        permit: &IzinPemakaianBmn,
        dokumen_client: &Arc<tokio::sync::Mutex<crate::workflow::DokumenClient>>,
    ) -> AppResult<(Uuid, String)> {
        // Prepare document data
        let document_data = serde_json::json!({
            "nomor_izin": permit.nomor_izin,
            "pegawai_nip": permit.pegawai_nip,
            "pegawai_nama": permit.pegawai_nama,
            "pegawai_jabatan": permit.pegawai_jabatan,
            "pegawai_satker_nama": permit.pegawai_satker_nama,
            "jenis_bmn": permit.jenis_bmn,
            "bmn_nup": permit.bmn_nup,
            "bmn_kode_barang": permit.bmn_kode_barang,
            "bmn_nama_barang": permit.bmn_nama_barang,
            "bmn_merk": permit.bmn_merk,
            "bmn_tahun_perolehan": permit.bmn_tahun_perolehan,
            "no_polisi": permit.no_polisi,
            "no_bpkb": permit.no_bpkb,
            "no_rangka": permit.no_rangka,
            "no_mesin": permit.no_mesin,
            "alamat": permit.alamat,
            "luas_tanah": permit.luas_tanah,
            "luas_bangunan": permit.luas_bangunan,
            "serial_number": permit.serial_number,
            "tanggal_mulai": permit.tanggal_mulai.to_string(),
            "tanggal_selesai": permit.tanggal_selesai.to_string(),
            "keperluan": permit.keperluan,
            "lokasi_pemakaian": permit.lokasi_pemakaian,
            "approved_by_nama": permit.approved_by_nama,
            "approved_at": permit.approved_at.map(|d| d.to_rfc3339()),
        });

        let metadata = serde_json::json!({
            "entity_type": "pemakaian_bmn",
            "entity_id": permit.id.to_string(),
            "document_type": "surat_izin_pemakaian",
        });

        // TODO: Get template_id from configuration
        // For now, use a placeholder UUID
        let template_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
            .map_err(|e| AppError::Internal(format!("Invalid template UUID: {}", e)))?;

        // Call dokumen service with retry logic
        let mut retry_count = 0;
        let max_retries = 3;

        loop {
            let mut client_guard = dokumen_client.lock().await;

            match client_guard
                .generate_document(
                    template_id,
                    document_data.clone(),
                    Some("pdf".to_string()),
                    Some(metadata.clone()),
                )
                .await
            {
                Ok(response) => {
                    let result = crate::workflow::DocumentGenerationResult::try_from(response)
                        .map_err(|e| AppError::Internal(format!("Invalid document response: {}", e)))?;

                    return Ok((result.document_id, result.download_url));
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(AppError::Internal(format!(
                            "Document generation failed after {} retries: {}",
                            max_retries, e
                        )));
                    }

                    warn!(
                        "Document generation attempt {} failed: {}. Retrying...",
                        retry_count, e
                    );

                    // Wait before retry (exponential backoff)
                    tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(retry_count))).await;
                }
            }
        }
    }

    /// Revoke a permit
    ///
    /// Requirements: REQ-P009
    pub async fn revoke_permit(
        &self,
        id: Uuid,
        request: RevokePermitRequest,
        user_id: Uuid,
        user_nama: String,
    ) -> AppResult<IzinPemakaianBmn> {
        // Validate request
        request.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

        let current = self.repository.get_by_id(id).await?;

        // Can only revoke ACTIVE permits
        if current.status != "ACTIVE" {
            return Err(AppError::BadRequest(
                "Hanya izin dengan status ACTIVE yang dapat dicabut".to_string(),
            ));
        }

        info!("Revoking permit {} by user {}", id, user_id);

        // Update status to REVOKED
        let permit = self
            .repository
            .update_status(id, "REVOKED", user_id, user_nama, Some(request.alasan))
            .await?;

        Ok(permit)
    }

    /// Renew a permit (create new permit based on existing one)
    ///
    /// Requirements: REQ-P008
    pub async fn renew_permit(
        &self,
        id: Uuid,
        request: RenewPermitRequest,
        user_id: Uuid,
        user_nama: String,
    ) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id).await?;

        // Can renew ACTIVE or EXPIRED permits
        if !matches!(current.status.as_str(), "ACTIVE" | "EXPIRED") {
            return Err(AppError::BadRequest(
                "Hanya izin dengan status ACTIVE atau EXPIRED yang dapat diperpanjang".to_string(),
            ));
        }

        // Validate date range
        if request.tanggal_selesai <= request.tanggal_mulai {
            return Err(AppError::BadRequest(
                "Tanggal selesai harus lebih besar dari tanggal mulai".to_string(),
            ));
        }

        // Check BMN availability
        let availability = self.repository.check_bmn_availability(&current.bmn_nup).await?;
        if !availability.is_available && availability.active_permit_id != Some(id) {
            return Err(AppError::BadRequest(format!(
                "BMN {} sedang digunakan oleh {} hingga {}",
                current.bmn_nup,
                availability.active_permit_holder.unwrap_or_default(),
                availability.active_permit_expires.map(|d| d.to_string()).unwrap_or_default()
            )));
        }

        info!("Renewing permit {} by user {}", id, user_id);

        // Create new permit as renewal
        let create_request = CreateIzinPemakaianRequest {
            pegawai_nip: current.pegawai_nip.clone(),
            pegawai_nama: current.pegawai_nama.clone(),
            pegawai_satker_id: current.pegawai_satker_id,
            pegawai_satker_nama: current.pegawai_satker_nama.clone(),
            pegawai_jabatan: current.pegawai_jabatan.clone(),
            jenis_bmn: current.jenis_bmn.clone(),
            bmn_nup: current.bmn_nup.clone(),
            bmn_kode_barang: current.bmn_kode_barang.clone(),
            bmn_nama_barang: current.bmn_nama_barang.clone(),
            bmn_merk: current.bmn_merk.clone(),
            bmn_tahun_perolehan: current.bmn_tahun_perolehan,
            no_polisi: current.no_polisi.clone(),
            no_bpkb: current.no_bpkb.clone(),
            no_stnk: current.no_stnk.clone(),
            no_rangka: current.no_rangka.clone(),
            no_mesin: current.no_mesin.clone(),
            alamat: current.alamat.clone(),
            luas_tanah: current.luas_tanah,
            luas_bangunan: current.luas_bangunan,
            serial_number: current.serial_number.clone(),
            spesifikasi: current.spesifikasi.clone(),
            tanggal_mulai: request.tanggal_mulai,
            tanggal_selesai: request.tanggal_selesai,
            keperluan: request.keperluan,
            lokasi_pemakaian: current.lokasi_pemakaian.clone(),
            file_pendukung: current.file_pendukung.clone(),
            is_renewal: Some(true),
            previous_permit_id: Some(id),
        };

        let new_permit = self.repository.create(create_request, user_id, user_nama).await?;

        Ok(new_permit)
    }

    /// Check if a BMN is available for new permit (no active permit exists)
    ///
    /// Requirements: REQ-P002, REQ-P003
    pub async fn check_bmn_availability(&self, bmn_nup: &str) -> AppResult<BmnAvailabilityResponse> {
        self.repository.check_bmn_availability(bmn_nup).await
    }

    /// Get BMN usage history
    ///
    /// Requirements: REQ-P012
    pub async fn get_bmn_usage_history(&self, bmn_nup: &str) -> AppResult<BmnUsageStats> {
        self.repository.get_bmn_usage_history(bmn_nup).await
    }

    /// Get pegawai usage history
    ///
    /// Requirements: REQ-P012
    pub async fn get_pegawai_usage_history(&self, pegawai_nip: &str) -> AppResult<PegawaiUsageStats> {
        self.repository.get_pegawai_usage_history(pegawai_nip).await
    }

    /// Get permits expiring soon (for notifications)
    ///
    /// Requirements: REQ-P007
    pub async fn get_expiring_permits(&self, days_threshold: i32) -> AppResult<Vec<IzinPemakaianBmn>> {
        self.repository.get_expiring_permits(days_threshold).await
    }

    /// Auto-expire permits that have passed their end date
    ///
    /// This should be called by a scheduled job
    /// Requirements: REQ-P010
    pub async fn auto_expire_permits(&self) -> AppResult<usize> {
        info!("Running auto-expire job for permits");
        let count = self.repository.auto_expire_permits().await?;
        info!("Auto-expired {} permits", count);
        Ok(count)
    }

    /// Send expiry reminder notification for a permit
    ///
    /// Requirements: REQ-P007, REQ-N008
    pub async fn send_expiry_reminder(
        &self,
        permit: &IzinPemakaianBmn,
        days_remaining: i32,
    ) -> AppResult<()> {
        if let Some(client) = &self.notifikasi_client {
            let mut client = client.lock().await;

            // Create notification data
            let notification_type = crate::workflow::WorkflowNotificationType::WorkflowTransition {
                entity_type: "pemakaian_bmn".to_string(),
                entity_id: permit.id.to_string(),
                from_state: "ACTIVE".to_string(),
                to_state: format!("EXPIRING_IN_{}_DAYS", days_remaining),
                transition_by: "system".to_string(),
                catatan: Some(format!(
                    "Izin pemakaian BMN {} untuk {} akan berakhir dalam {} hari (tanggal: {}). Nomor izin: {}. Silakan perpanjang jika masih diperlukan.",
                    permit.bmn_nama_barang,
                    permit.pegawai_nama,
                    days_remaining,
                    permit.tanggal_selesai,
                    permit.nomor_izin.as_deref().unwrap_or("N/A")
                )),
            };

            // Send notification to permit holder
            match client
                .send_notification(
                    permit.created_by,
                    notification_type,
                    crate::workflow::NotificationPriority::High,
                )
                .await
            {
                Ok(_) => {
                    info!(
                        "Sent H-{} expiry reminder for permit {} to user {}",
                        days_remaining, permit.id, permit.created_by
                    );
                    // Record success metric
                    crate::metrics::permit_expiry_reminders_sent_total()
                        .with_label_values(&[&days_remaining.to_string(), "success"])
                        .inc();
                }
                Err(e) => {
                    warn!(
                        "Failed to send expiry reminder for permit {}: {}",
                        permit.id, e
                    );
                    // Record error metric
                    crate::metrics::permit_expiry_reminders_sent_total()
                        .with_label_values(&[&days_remaining.to_string(), "error"])
                        .inc();
                    crate::metrics::permit_expiry_reminder_errors_total()
                        .with_label_values(&["notification_failed"])
                        .inc();
                    // Don't fail the job if notification fails
                }
            }
        } else {
            warn!("Notification client not configured, skipping expiry reminder");
        }

        Ok(())
    }

    /// Send expiry notification for an expired permit
    ///
    /// Requirements: REQ-P007, REQ-N008
    pub async fn send_expiry_notification(&self, permit: &IzinPemakaianBmn) -> AppResult<()> {
        if let Some(client) = &self.notifikasi_client {
            let mut client = client.lock().await;

            // Create notification data
            let notification_type = crate::workflow::WorkflowNotificationType::WorkflowTransition {
                entity_type: "pemakaian_bmn".to_string(),
                entity_id: permit.id.to_string(),
                from_state: "ACTIVE".to_string(),
                to_state: "EXPIRED".to_string(),
                transition_by: "system".to_string(),
                catatan: Some(format!(
                    "Izin pemakaian BMN {} untuk {} telah berakhir pada tanggal {}. Nomor izin: {}. BMN harus segera dikembalikan atau izin diperpanjang.",
                    permit.bmn_nama_barang,
                    permit.pegawai_nama,
                    permit.tanggal_selesai,
                    permit.nomor_izin.as_deref().unwrap_or("N/A")
                )),
            };

            // Send notification to permit holder
            match client
                .send_notification(
                    permit.created_by,
                    notification_type,
                    crate::workflow::NotificationPriority::Urgent,
                )
                .await
            {
                Ok(_) => {
                    info!(
                        "Sent expiry notification for permit {} to user {}",
                        permit.id, permit.created_by
                    );
                    // Record success metric
                    crate::metrics::permit_expiry_notifications_sent_total()
                        .with_label_values(&["success"])
                        .inc();
                }
                Err(e) => {
                    warn!(
                        "Failed to send expiry notification for permit {}: {}",
                        permit.id, e
                    );
                    // Record error metric
                    crate::metrics::permit_expiry_notifications_sent_total()
                        .with_label_values(&["error"])
                        .inc();
                    crate::metrics::permit_expiry_reminder_errors_total()
                        .with_label_values(&["notification_failed"])
                        .inc();
                    // Don't fail the job if notification fails
                }
            }
        } else {
            warn!("Notification client not configured, skipping expiry notification");
        }

        Ok(())
    }

    // ========================================================================
    // Monitoring Dashboard Methods
    // ========================================================================

    /// Get active usage monitoring dashboard data
    ///
    /// Requirements: REQ-P011
    pub async fn get_active_usage_dashboard(
        &self,
        query: MonitoringDashboardQuery,
    ) -> AppResult<ActiveUsageMonitoringDashboard> {
        info!("Fetching active usage monitoring dashboard");
        self.repository.get_active_usage_dashboard(query).await
    }

    /// Get BMN utilization report
    ///
    /// Requirements: REQ-P013
    pub async fn get_bmn_utilization_report(
        &self,
        query: MonitoringDashboardQuery,
    ) -> AppResult<BmnUtilizationReport> {
        info!("Generating BMN utilization report");
        self.repository.get_bmn_utilization_report(query).await
    }

    /// Validate BMN type-specific required fields
    ///
    /// Requirements: REQ-P001
    fn validate_bmn_type_fields(&self, request: &CreateIzinPemakaianRequest) -> AppResult<()> {
        let jenis_bmn = JenisBmn::from_str(&request.jenis_bmn)
            .ok_or_else(|| AppError::BadRequest(format!("Invalid jenis_bmn: {}", request.jenis_bmn)))?;

        match jenis_bmn {
            JenisBmn::KendaraanBermotor => {
                if request.no_polisi.is_none() {
                    return Err(AppError::BadRequest("Nomor polisi harus diisi untuk kendaraan bermotor".to_string()));
                }
            }
            JenisBmn::RumahNegara => {
                if request.alamat.is_none() {
                    return Err(AppError::BadRequest("Alamat harus diisi untuk rumah negara".to_string()));
                }
                if request.luas_tanah.is_none() {
                    return Err(AppError::BadRequest("Luas tanah harus diisi untuk rumah negara".to_string()));
                }
                if request.luas_bangunan.is_none() {
                    return Err(AppError::BadRequest("Luas bangunan harus diisi untuk rumah negara".to_string()));
                }
            }
            JenisBmn::Laptop => {
                if request.serial_number.is_none() {
                    return Err(AppError::BadRequest("Serial number harus diisi untuk laptop".to_string()));
                }
            }
            JenisBmn::Lainnya => {
                // No specific validation for other types
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_transitions() {
        let draft = PemakaianBmnStatus::Draft;
        assert!(draft.can_transition_to(PemakaianBmnStatus::Submitted));
        assert!(!draft.can_transition_to(PemakaianBmnStatus::Active));
    }
}
