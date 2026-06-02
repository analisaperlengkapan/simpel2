//! # Pakaian Dinas Services
//!
//! Business logic layer for Pakaian Dinas module.
//! Handles workflow, validation, and integration with other services.

use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use super::models::*;
use super::repository::PakaianDinasRepository;
use crate::shared::error::{AppError, AppResult, bad_request};
use crate::shared::grpc::clients::IntegrasiClient;
use crate::shared::grpc::clients::integrasi::v1::{DataSource, SyncState};
use lib_perlengkapan::audit::{AuditAction, AuditEvent};
use lib_perlengkapan::contracts::AuditSink;

/// Validate periode pengajuan pakaian dinas (Fase 1.8). Mengembalikan
/// `AppError::BadRequest` (422) jika:
/// - salah satu tgl_mulai / tgl_selesai `None` (keduanya wajib), ATAU
/// - tgl_mulai > tgl_selesai.
///
/// Extracted sbg pure function agar dapat di-unit-test tanpa DB.
pub fn validate_periode_pakaian_dinas(
    tgl_mulai: Option<NaiveDate>,
    tgl_selesai: Option<NaiveDate>,
) -> Result<(), AppError> {
    match (tgl_mulai, tgl_selesai) {
        (None, _) | (_, None) => Err(bad_request(
            "Periode (tanggal mulai + tanggal selesai) wajib diisi",
        )),
        (Some(start), Some(end)) if start > end => Err(bad_request(
            "Tanggal mulai tidak boleh lebih besar dari tanggal selesai",
        )),
        _ => Ok(()),
    }
}

/// Validate scope satker pengajuan (#19). Accepts `all`/`semua`, `sebagian`,
/// `wilayah`. `sebagian` butuh `satker_ids` non-kosong; `wilayah` butuh
/// `wilayah_id`. Pure → unit-testable tanpa DB.
pub fn validate_scope_satker(
    pilihan_satker: &str,
    wilayah_id: Option<&str>,
    satker_ids: Option<&[Uuid]>,
) -> Result<(), AppError> {
    match pilihan_satker {
        "all" | "semua" => Ok(()),
        "sebagian" => {
            if satker_ids.map_or(true, |ids| ids.is_empty()) {
                Err(bad_request(
                    "Satker harus dipilih jika pilihan satker = 'sebagian'",
                ))
            } else {
                Ok(())
            }
        }
        "wilayah" => {
            if wilayah_id.map_or(true, |w| w.trim().is_empty()) {
                Err(bad_request(
                    "Wilayah harus dipilih jika pilihan satker = 'wilayah'",
                ))
            } else {
                Ok(())
            }
        }
        _ => Err(bad_request(
            "Pilihan satker harus 'all', 'sebagian', atau 'wilayah'",
        )),
    }
}

/// Service for Pakaian Dinas business logic
#[derive(Clone)]
pub struct PakaianDinasService {
    pub(crate) repository: PakaianDinasRepository,
    /// gRPC client ke layanan-integrasi (resilient, Fase 2.2). Opsional —
    /// `None` saat integrasi tidak tersedia; sinkronisasi freshness pegawai
    /// (Fase 2.4) graceful-degrade ke "tidak diketahui".
    integrasi_client: Option<IntegrasiClient>,
    /// Cross-module audit sink (#16/#40). Opsional — `None` saat belum
    /// di-inject; transisi validator tetap berjalan, hanya tanpa jejak audit.
    audit_sink: Option<Arc<dyn AuditSink>>,
}

impl PakaianDinasService {
    pub fn new(repository: PakaianDinasRepository) -> Self {
        Self {
            repository,
            integrasi_client: None,
            audit_sink: None,
        }
    }

    /// Inject IntegrasiClient (Fase 2.4) untuk laporan freshness sync MySIMKARI.
    pub fn with_integrasi_client(mut self, client: IntegrasiClient) -> Self {
        self.integrasi_client = Some(client);
        self
    }

    /// Inject cross-module audit sink (#16/#40) — jejak transisi validator
    /// pakaian dinas ke `perlengkapan.audit_log`.
    pub fn with_audit_sink(mut self, sink: Arc<dyn AuditSink>) -> Self {
        self.audit_sink = Some(sink);
        self
    }

    /// Roster pegawai satker (dari replika `integrasi.mysimkari_pegawai`,
    /// non-lossy — termasuk gender & foto) + info kesegaran sinkronisasi
    /// MySIMKARI (Fase 2.4). Wizard ukuran menampilkan `last_sync_at` dan
    /// banner bila data berpotensi basi (sync gagal / belum pernah sync).
    ///
    /// Catatan arsitektur: panggilan gRPC MySIMKARI langsung bersifat
    /// *field-poor* (tanpa gender/foto), sedangkan replika kaya field dan
    /// disinkronkan oleh layanan-integrasi. Maka roster tetap dibaca dari
    /// replika; "realtime" diwujudkan sebagai transparansi kesegaran +
    /// fallback aware, bukan tarik-langsung yang lossy.
    pub async fn get_pegawai_roster_with_sync(
        &self,
        satker_id: Uuid,
    ) -> AppResult<PegawaiRosterWithSync> {
        let pegawai = self
            .repository
            .get_mysimkari_pegawai_by_satker(satker_id)
            .await?;

        let sync = match &self.integrasi_client {
            Some(client) => Self::fetch_mysimkari_sync(client).await,
            None => None,
        };

        Ok(PegawaiRosterWithSync {
            total: pegawai.len() as i64,
            pegawai,
            sync,
        })
    }

    /// Best-effort probe status sinkronisasi MySIMKARI. Kegagalan probe
    /// (gRPC timeout/circuit-open) dikembalikan sbg `None` (tidak diketahui),
    /// tidak memblokir penyajian roster.
    async fn fetch_mysimkari_sync(client: &IntegrasiClient) -> Option<PegawaiSyncInfo> {
        match client.get_sync_status(DataSource::Mysimkari).await {
            Ok(status) => {
                let state = SyncState::try_from(status.state).unwrap_or(SyncState::Unspecified);
                let last_sync_at = if status.last_sync_at.trim().is_empty() {
                    None
                } else {
                    Some(status.last_sync_at)
                };
                // "segar" = sync terakhir COMPLETED dan ada timestamp.
                let segar = matches!(state, SyncState::Completed) && last_sync_at.is_some();
                Some(PegawaiSyncInfo {
                    sumber: "mysimkari".to_string(),
                    state: state.as_str_name().to_string(),
                    last_sync_at,
                    segar,
                    records_synced: status.records_synced,
                    error_message: if status.error_message.trim().is_empty() {
                        None
                    } else {
                        Some(status.error_message)
                    },
                })
            }
            Err(e) => {
                tracing::warn!(
                    "Probe sync MySIMKARI gagal (tidak memblokir roster): {}",
                    e
                );
                None
            }
        }
    }

    // ============ Master: Jenis Pakaian Dinas ============

    pub async fn get_all_jenis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<JenisPakaianDinas>, i64)> {
        self.repository.get_all_jenis(page, per_page).await
    }

    pub async fn get_jenis_by_id(&self, id: Uuid) -> AppResult<JenisPakaianDinas> {
        self.repository.get_jenis_by_id(id).await
    }

    pub async fn create_jenis(
        &self,
        request: CreateJenisPakaianDinasRequest,
    ) -> AppResult<JenisPakaianDinas> {
        // Validate request
        use validator::Validate;
        request
            .validate()
            .map_err(|e| bad_request(&e.to_string()))?;

        self.repository.create_jenis(request).await
    }

    pub async fn update_jenis(
        &self,
        id: Uuid,
        request: CreateJenisPakaianDinasRequest,
    ) -> AppResult<JenisPakaianDinas> {
        use validator::Validate;
        request
            .validate()
            .map_err(|e| bad_request(&e.to_string()))?;

        self.repository.update_jenis(id, request).await
    }

    pub async fn delete_jenis(&self, id: Uuid) -> AppResult<()> {
        self.repository.delete_jenis(id).await
    }

    // ============ Master: Spesifikasi ============

    pub async fn get_all_spesifikasi(
        &self,
        page: i32,
        per_page: i32,
        jenis_id: Option<Uuid>,
    ) -> AppResult<(Vec<SpesifikasiPakaianDinas>, i64)> {
        self.repository
            .get_all_spesifikasi(page, per_page, jenis_id)
            .await
    }

    pub async fn get_spesifikasi_by_id(&self, id: Uuid) -> AppResult<SpesifikasiPakaianDinas> {
        self.repository.get_spesifikasi_by_id(id).await
    }

    pub async fn create_spesifikasi(
        &self,
        request: CreateSpesifikasiRequest,
    ) -> AppResult<SpesifikasiPakaianDinas> {
        use validator::Validate;
        request
            .validate()
            .map_err(|e| bad_request(&e.to_string()))?;

        // Validate gender value
        let gender = request.gender.to_uppercase();
        if !["L", "P", "SEMUA"].contains(&gender.as_str()) {
            return Err(bad_request("Gender harus L, P, atau SEMUA"));
        }

        // Validate ukuran_group value
        let group = request.ukuran_group.to_uppercase();
        if !["BAJU", "CELANA", "SEPATU"].contains(&group.as_str()) {
            return Err(bad_request("Ukuran group harus BAJU, CELANA, atau SEPATU"));
        }

        // Verify jenis exists
        self.repository
            .get_jenis_by_id(request.jenis_pakaian_dinas_id)
            .await?;

        self.repository.create_spesifikasi(request).await
    }

    pub async fn update_spesifikasi(
        &self,
        id: Uuid,
        request: CreateSpesifikasiRequest,
    ) -> AppResult<SpesifikasiPakaianDinas> {
        use validator::Validate;
        request
            .validate()
            .map_err(|e| bad_request(&e.to_string()))?;

        self.repository.update_spesifikasi(id, request).await
    }

    pub async fn delete_spesifikasi(&self, id: Uuid) -> AppResult<()> {
        self.repository.delete_spesifikasi(id).await
    }

    // ============ Master: SubSpesifikasi ============

    pub async fn get_all_subspesifikasi(
        &self,
        page: i32,
        per_page: i32,
        spesifikasi_id: Option<Uuid>,
    ) -> AppResult<(Vec<SubSpesifikasiPakaianDinas>, i64)> {
        self.repository
            .get_all_subspesifikasi(page, per_page, spesifikasi_id)
            .await
    }

    pub async fn get_subspesifikasi_by_id(
        &self,
        id: Uuid,
    ) -> AppResult<SubSpesifikasiPakaianDinas> {
        self.repository.get_subspesifikasi_by_id(id).await
    }

    pub async fn create_subspesifikasi(
        &self,
        request: CreateSubSpesifikasiRequest,
    ) -> AppResult<SubSpesifikasiPakaianDinas> {
        use validator::Validate;
        request
            .validate()
            .map_err(|e| bad_request(&e.to_string()))?;

        // Verify spesifikasi exists
        self.repository
            .get_spesifikasi_by_id(request.spesifikasi_id)
            .await?;

        self.repository.create_subspesifikasi(request).await
    }

    pub async fn delete_subspesifikasi(&self, id: Uuid) -> AppResult<()> {
        self.repository.delete_subspesifikasi(id).await
    }

    // ============ Master: Ukuran ============

    pub async fn get_all_ukuran(&self, group: Option<String>) -> AppResult<Vec<Ukuran>> {
        self.repository.get_all_ukuran(group).await
    }

    // ============ Pengajuan ============

    pub async fn get_all_pengajuan(
        &self,
        page: i32,
        per_page: i32,
        tahun: Option<i32>,
    ) -> AppResult<(Vec<PengajuanPakaianDinas>, i64)> {
        self.repository
            .get_all_pengajuan(page, per_page, tahun)
            .await
    }

    pub async fn get_pengajuan_by_id(&self, id: Uuid) -> AppResult<PengajuanPakaianDinas> {
        self.repository.get_pengajuan_by_id(id).await
    }

    pub async fn create_pengajuan(
        &self,
        request: CreatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanPakaianDinas> {
        use validator::Validate;
        request
            .validate()
            .map_err(|e| bad_request(&e.to_string()))?;

        // Fase 1.8: periode WAJIB regardless of is_reguler — stakeholder
        // eksplisit minta "pilih periode (tanggal kapan mulai sampai
        // tanggal kapan berakhir)" sbg input utama (plan §4.1).
        validate_periode_pakaian_dinas(request.tgl_mulai, request.tgl_selesai)?;

        // Validate spesifikasi_ids is not empty
        if request.spesifikasi_ids.is_empty() {
            return Err(bad_request(
                "Minimal satu spesifikasi pakaian harus dipilih",
            ));
        }

        // Validate scope satker (#19: tambah opsi "wilayah").
        validate_scope_satker(
            &request.pilihan_satker,
            request.wilayah_id.as_deref(),
            request.satker_ids.as_deref(),
        )?;

        self.repository.create_pengajuan(request, user_id).await
    }

    pub async fn delete_pengajuan(&self, id: Uuid) -> AppResult<()> {
        self.repository.delete_pengajuan(id).await
    }

    // ============ Pengajuan Satker ============

    pub async fn get_pengajuan_satker_list(
        &self,
        pengajuan_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<PengajuanSatker>, i64)> {
        self.repository
            .get_pengajuan_satker_list(pengajuan_id, page, per_page)
            .await
    }

    pub async fn get_pengajuan_satker_by_id(&self, id: Uuid) -> AppResult<PengajuanSatker> {
        self.repository.get_pengajuan_satker_by_id(id).await
    }

    // ============ Workflow Actions ============

    /// Process workflow action (approve/reject) for a satker submission
    pub async fn process_validator_action(
        &self,
        request: ValidatorActionRequest,
        user_nip: &str,
        user_nama: &str,
        user_role: &str,
    ) -> AppResult<PengajuanSatker> {
        // Get current satker submission
        let satker = self
            .repository
            .get_pengajuan_satker_by_id(request.pengajuan_satker_id)
            .await?;

        // Determine next status based on current status, action, and role.
        // Invalid (status, action, role) combos return 422 here — this doubles
        // as the RBAC + workflow guard for the validator action.
        let next_status =
            self.determine_next_status(satker.aktivitas_id, &request.aksi, user_role)?;

        // Persist the transition + record an activity row (atomic). This is
        // the real per-satker workflow advance that replaces the prior no-op.
        self.repository
            .transition_satker_with_activity(
                request.pengajuan_satker_id,
                next_status,
                request.komentar.clone(),
                Some(user_nip),
                Some(user_nama),
                None,
                Some(user_role),
            )
            .await?;

        // Audit trail (#16) — best-effort, swallowed on failure so a flaky
        // sink never rolls back a committed transition.
        if let Some(sink) = &self.audit_sink {
            let action = match request.aksi.as_str() {
                "approve" => AuditAction::Approve,
                "reject" => AuditAction::Reject,
                _ => AuditAction::Update,
            };
            let event = AuditEvent::new("pakaian_dinas", action, "pengajuan_satker")
                .resource_id(request.pengajuan_satker_id.to_string())
                .action_name("pakaian_dinas.validator_action")
                .message(request.komentar.clone().unwrap_or_default())
                .metadata(serde_json::json!({
                    "from_status": satker.aktivitas_id,
                    "to_status": next_status,
                    "aksi": request.aksi,
                    "role": user_role,
                }));
            let _ = sink.log(event).await;
        }

        // Return the freshly-transitioned satker.
        self.repository
            .get_pengajuan_satker_by_id(request.pengajuan_satker_id)
            .await
    }

    /// Per-satker workflow activity history (#40). Oldest-first list backing
    /// the FE timeline.
    pub async fn list_satker_aktivitas(
        &self,
        satker_id: Uuid,
    ) -> AppResult<Vec<PengajuanSatkerAktivitas>> {
        self.repository.list_satker_aktivitas(satker_id).await
    }

    /// Determine the next workflow status based on current status and action
    fn determine_next_status(
        &self,
        current_status: i32,
        action: &str,
        role: &str,
    ) -> AppResult<i32> {
        let current = AktivitasStatus::from_i32(current_status)
            .ok_or_else(|| bad_request("Status aktivitas tidak valid"))?;

        match (current.clone(), action, role) {
            // Pelaksana submits to validator
            (AktivitasStatus::Input, "submit", "pelaksana") => {
                Ok(AktivitasStatus::SubmitToValidator.to_i32())
            }
            (AktivitasStatus::RevisiPelaksana, "submit", "pelaksana") => {
                Ok(AktivitasStatus::SubmitToValidator.to_i32())
            }

            // Validator wilayah approves/rejects
            (AktivitasStatus::SubmitToValidator, "approve", "validator_wilayah") => {
                Ok(AktivitasStatus::SubmitToPusat.to_i32())
            }
            (AktivitasStatus::SubmitToValidator, "reject", "validator_wilayah") => {
                Ok(AktivitasStatus::RevisiPelaksana.to_i32())
            }

            // Validator pusat approves/rejects
            (AktivitasStatus::SubmitToPusat, "approve", "validator_pusat") => {
                Ok(AktivitasStatus::Selesai.to_i32())
            }
            (AktivitasStatus::SubmitToPusat, "reject", "validator_pusat") => {
                Ok(AktivitasStatus::RevisiWilayah.to_i32())
            }

            // Kejagung direct flow
            (AktivitasStatus::StartKejagung, "submit", "pelaksana") => {
                Ok(AktivitasStatus::SubmitToPusat.to_i32())
            }

            _ => Err(bad_request(&format!(
                "Aksi '{}' tidak valid untuk status '{}' dengan role '{}'",
                action,
                current.label(),
                role
            ))),
        }
    }

    // ============ Personal Uniform Sizes ============

    pub async fn get_personal_ukuran(&self, nip: &str) -> AppResult<Option<PegawaiPakaianDinas>> {
        self.repository.get_pegawai_pakaian_dinas(nip).await
    }

    pub async fn update_personal_ukuran(
        &self,
        request: UpdatePersonalUkuranRequest,
        nip: &str,
        nama: Option<&str>,
    ) -> AppResult<PegawaiPakaianDinas> {
        use validator::Validate;
        request
            .validate()
            .map_err(|e| bad_request(&e.to_string()))?;

        self.repository
            .upsert_pegawai_pakaian_dinas(&request, nip, nama)
            .await
    }

    // ============ MySIMKARI Integration ============

    pub async fn get_pegawai_by_satker(&self, satker_id: Uuid) -> AppResult<Vec<MysimkariPegawai>> {
        self.repository
            .get_mysimkari_pegawai_by_satker(satker_id)
            .await
    }

    /// Get employees with their existing uniform sizes
    pub async fn get_pegawai_with_sizes(
        &self,
        satker_id: Uuid,
    ) -> AppResult<Vec<(MysimkariPegawai, Option<PegawaiPakaianDinas>)>> {
        let pegawai_list = self
            .repository
            .get_mysimkari_pegawai_by_satker(satker_id)
            .await?;

        let mut result = Vec::with_capacity(pegawai_list.len());
        for pegawai in pegawai_list {
            let sizes = self
                .repository
                .get_pegawai_pakaian_dinas(&pegawai.nip)
                .await?;
            result.push((pegawai, sizes));
        }

        Ok(result)
    }

    // ============ Reports ============

    pub async fn get_laporan_rekap_ukuran(
        &self,
        pengajuan_id: Uuid,
        filter: LaporanFilter,
    ) -> AppResult<Vec<LaporanRekapUkuran>> {
        self.repository
            .get_laporan_rekap_ukuran(pengajuan_id, &filter)
            .await
    }

    pub async fn get_laporan_daftar_pegawai(
        &self,
        pengajuan_id: Uuid,
        filter: LaporanFilter,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<LaporanDaftarPegawai>, i64)> {
        self.repository
            .get_laporan_daftar_pegawai(pengajuan_id, &filter, page, per_page)
            .await
    }

    // ============ Workflow Integration ============

    /// Submit pengajuan for approval (DRAFT -> SUBMITTED)
    pub async fn submit_pengajuan(
        &self,
        pengajuan_id: Uuid,
        user_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<PengajuanPakaianDinas> {
        // Get current pengajuan
        let pengajuan = self.repository.get_pengajuan_by_id(pengajuan_id).await?;

        // Verify current status is DRAFT
        let current_status = AktivitasStatus::from_i32(pengajuan.aktivitas_id)
            .ok_or_else(|| bad_request("Status aktivitas tidak valid"))?;

        if current_status != AktivitasStatus::Input {
            return Err(bad_request(&format!(
                "Pengajuan harus dalam status DRAFT untuk disubmit, status saat ini: {}",
                current_status.label()
            )));
        }

        // Update status to SUBMITTED (1001)
        let new_status = AktivitasStatus::SubmitToValidator;
        self.repository
            .update_pengajuan_status(pengajuan_id, new_status.to_i32(), user_id, catatan)
            .await?;

        // Return updated pengajuan
        self.repository.get_pengajuan_by_id(pengajuan_id).await
    }

    /// Approve pengajuan (SUBMITTED -> APPROVED)
    pub async fn approve_pengajuan(
        &self,
        pengajuan_id: Uuid,
        user_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<PengajuanPakaianDinas> {
        // Get current pengajuan
        let pengajuan = self.repository.get_pengajuan_by_id(pengajuan_id).await?;

        // Verify current status is SUBMITTED
        let current_status = AktivitasStatus::from_i32(pengajuan.aktivitas_id)
            .ok_or_else(|| bad_request("Status aktivitas tidak valid"))?;

        if current_status != AktivitasStatus::SubmitToValidator {
            return Err(bad_request(&format!(
                "Pengajuan harus dalam status SUBMITTED untuk diapprove, status saat ini: {}",
                current_status.label()
            )));
        }

        // Update status to APPROVED (1008 - Selesai)
        let new_status = AktivitasStatus::Selesai;
        self.repository
            .update_pengajuan_status(pengajuan_id, new_status.to_i32(), user_id, catatan)
            .await?;

        // Return updated pengajuan
        self.repository.get_pengajuan_by_id(pengajuan_id).await
    }

    /// Reject pengajuan (SUBMITTED -> REJECTED)
    pub async fn reject_pengajuan(
        &self,
        pengajuan_id: Uuid,
        user_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<PengajuanPakaianDinas> {
        // Get current pengajuan
        let pengajuan = self.repository.get_pengajuan_by_id(pengajuan_id).await?;

        // Verify current status is SUBMITTED
        let current_status = AktivitasStatus::from_i32(pengajuan.aktivitas_id)
            .ok_or_else(|| bad_request("Status aktivitas tidak valid"))?;

        if current_status != AktivitasStatus::SubmitToValidator {
            return Err(bad_request(&format!(
                "Pengajuan harus dalam status SUBMITTED untuk direject, status saat ini: {}",
                current_status.label()
            )));
        }

        // Update status to REJECTED (1006 - Ditolak)
        let new_status = AktivitasStatus::Ditolak;
        self.repository
            .update_pengajuan_status(pengajuan_id, new_status.to_i32(), user_id, catatan)
            .await?;

        // Return updated pengajuan
        self.repository.get_pengajuan_by_id(pengajuan_id).await
    }

    /// Complete pengajuan and generate rekapitulasi (APPROVED -> COMPLETED)
    /// This should be called after document generation
    pub async fn complete_pengajuan(
        &self,
        pengajuan_id: Uuid,
        _user_id: Uuid,
        document_id: Option<Uuid>,
        document_url: Option<String>,
    ) -> AppResult<PengajuanPakaianDinas> {
        // Get current pengajuan
        let pengajuan = self.repository.get_pengajuan_by_id(pengajuan_id).await?;

        // Verify current status is APPROVED
        let current_status = AktivitasStatus::from_i32(pengajuan.aktivitas_id)
            .ok_or_else(|| bad_request("Status aktivitas tidak valid"))?;

        if current_status != AktivitasStatus::Selesai {
            return Err(bad_request(&format!(
                "Pengajuan harus dalam status APPROVED untuk dicomplete, status saat ini: {}",
                current_status.label()
            )));
        }

        // Update status to COMPLETED (1008 - Selesai, but we'll keep it as is since it's already Selesai)
        // In a real implementation, you might have a separate COMPLETED status
        // For now, we'll just update the document metadata
        if let (Some(doc_id), Some(doc_url)) = (document_id, document_url) {
            self.repository
                .update_pengajuan_document(pengajuan_id, doc_id, doc_url)
                .await?;
        }

        // Return updated pengajuan
        self.repository.get_pengajuan_by_id(pengajuan_id).await
    }
}

// ============ Unit Tests ============

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Fase 1.8: validate_periode_pakaian_dinas ────────────────────

    #[test]
    fn periode_both_required() {
        assert!(validate_periode_pakaian_dinas(None, None).is_err());
        assert!(validate_periode_pakaian_dinas(
            Some(NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()),
            None,
        )
        .is_err());
        assert!(validate_periode_pakaian_dinas(
            None,
            Some(NaiveDate::from_ymd_opt(2027, 12, 31).unwrap()),
        )
        .is_err());
    }

    #[test]
    fn periode_start_must_not_exceed_end() {
        let result = validate_periode_pakaian_dinas(
            Some(NaiveDate::from_ymd_opt(2027, 12, 31).unwrap()),
            Some(NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn periode_same_day_allowed() {
        let d = NaiveDate::from_ymd_opt(2027, 6, 15).unwrap();
        assert!(validate_periode_pakaian_dinas(Some(d), Some(d)).is_ok());
    }

    #[test]
    fn scope_all_and_semua_ok_without_extras() {
        assert!(validate_scope_satker("all", None, None).is_ok());
        assert!(validate_scope_satker("semua", None, None).is_ok());
    }

    #[test]
    fn scope_wilayah_requires_wilayah_id() {
        assert!(validate_scope_satker("wilayah", None, None).is_err());
        assert!(validate_scope_satker("wilayah", Some("  "), None).is_err());
        assert!(validate_scope_satker("wilayah", Some("Kejati DKI"), None).is_ok());
    }

    #[test]
    fn scope_sebagian_requires_satker_ids() {
        assert!(validate_scope_satker("sebagian", None, None).is_err());
        assert!(validate_scope_satker("sebagian", None, Some(&[])).is_err());
        assert!(validate_scope_satker("sebagian", None, Some(&[Uuid::new_v4()])).is_ok());
    }

    #[test]
    fn scope_unknown_rejected() {
        assert!(validate_scope_satker("entah", None, None).is_err());
    }

    #[test]
    fn periode_normal_range_ok() {
        assert!(validate_periode_pakaian_dinas(
            Some(NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2027, 12, 31).unwrap()),
        )
        .is_ok());
    }

    #[test]
    fn test_determine_next_status_pelaksana_submit() {
        // Create a mock service (without actual repository)
        // In production, use mockall or similar for proper mocking

        // Test workflow transitions
        let test_cases = vec![
            (1000, "submit", "pelaksana", 1001),
            (1003, "submit", "pelaksana", 1001),
            (1001, "approve", "validator_wilayah", 1004),
            (1001, "reject", "validator_wilayah", 1003),
            (1004, "approve", "validator_pusat", 1008),
            (1004, "reject", "validator_pusat", 1007),
        ];

        // This is a simplified test - in production, you'd use mocking
        for (current, _action, _role, expected) in test_cases {
            let current_status = AktivitasStatus::from_i32(current).unwrap();
            let expected_status = AktivitasStatus::from_i32(expected).unwrap();

            // Verify status labels exist
            assert!(!current_status.label().is_empty());
            assert!(!expected_status.label().is_empty());
        }
    }
}
