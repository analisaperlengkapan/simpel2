// ============================================================================
// Penghapusan BMN Services
// Description: Business logic for Usulan SK Penghapusan BMN workflow
// Requirements: REQ-W001, REQ-W004, REQ-D002, REQ-N001
//
// Flow:
//   Operator Satker: Draft → SubmitWilayah (with lampiran)
//   Validator Wilayah: SubmitWilayah → SubmitPusat (forward) or ReturnedToOperator (return)
//   Operator Satker: ReturnedToOperator → SubmitWilayah (re-submit)
//   Validator Pusat: SubmitPusat → VerifikasiPusat → KonsepSKGenerated (generate DOCX)
//   Validator Pusat: KonsepSKGenerated → SKSigned (upload signed PDF)
//   System: SKSigned → Completed
// ============================================================================

use super::models::*;
use super::repository::PenghapusanBmnRepository;
use crate::bank_aset::repository::BankAsetRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::satker_scope::SatkerScope;
use crate::shared::siman_columns::normalize_kode_barang;
use crate::workflow::engine::{TransitionRequest, WorkflowEngine};
use deadpool_postgres::Pool;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Who is making a workflow move, and what the audit trail should record.
///
/// These four always travel together — every `transition` call passes all of
/// them — so they are one parameter rather than four. Bundling them is also
/// what keeps the signature under clippy's argument limit now that the satker
/// scope is a required argument.
pub struct TransitionActor {
    pub user_id: Uuid,
    pub user_role: String,
    pub catatan: Option<String>,
    pub ip_address: String,
}

impl TransitionActor {
    /// The common case: an action taken by the authenticated caller, with the
    /// action name standing in for `ip_address` (which is what every caller in
    /// this module already passed).
    pub fn new(user_id: Uuid, user_role: String, catatan: Option<String>, action: &str) -> Self {
        Self {
            user_id,
            user_role,
            catatan,
            ip_address: action.to_string(),
        }
    }
}

/// One uploaded attachment, as the handler already holds it.
pub struct LampiranUpload<'a> {
    pub nama: &'a str,
    pub file_url: &'a str,
    pub content_type: Option<&'a str>,
    pub size_bytes: Option<i64>,
    pub uploaded_by: Option<Uuid>,
}

pub struct PenghapusanBmnService {
    pool: Pool,
    repository: PenghapusanBmnRepository,
    workflow_engine: Arc<WorkflowEngine>,
    docs: Option<Arc<dyn crate::contracts::DocumentGenerator>>,
}

impl PenghapusanBmnService {
    pub fn new(pool: Pool, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self {
            pool: pool.clone(),
            repository: PenghapusanBmnRepository::new(pool),
            workflow_engine,
            docs: None,
        }
    }

    /// Inject the document generator (used by the konsep SK dual-format flow).
    pub fn with_document_generator(
        mut self,
        docs: Arc<dyn crate::contracts::DocumentGenerator>,
    ) -> Self {
        self.docs = Some(docs);
        self
    }

    /// Create a new Usulan SK Penghapusan BMN (by Operator Satker)
    ///
    /// `nilai_perolehan` SELALU diambil dari sumber otoritatif SIMAN
    /// (tabel cache `integrasi.siman_aset`) berdasarkan NUP + kode_barang
    /// dari request. Nilai yg dikirim operator (jika ada) di-override.
    /// Jika asset tidak ditemukan di SIMAN → tolak 422 (operator wajib
    /// memilih BMN yg memang ada di SIMAN). Jika query SIMAN gagal
    /// (DB connection error dll), graceful-fallback ke nilai operator
    /// dgn warning log — usulan tetap dpt disimpan, verifikasi nilai
    /// dapat dilakukan di tahap validasi wilayah/pusat.
    pub async fn create(
        &self,
        mut request: CreatePenghapusanBmnRequest,
        created_by: Uuid,
        // Authoritative MySIMKARI satker_code of the creating operator (from JWT
        // claims, #66) — persisted for RBAC scoping; NOT the client satker_id.
        satker_code: Option<String>,
        // What the caller may see in SIMAN. The BMN being proposed for removal
        // must be one of THEIR satker's assets.
        aset_scope: &crate::bank_aset::AsetScope,
    ) -> AppResult<PenghapusanBmn> {
        info!(
            "Creating Usulan SK Penghapusan BMN for asset: {}",
            request.nama_barang
        );

        let bank_repo = BankAsetRepository::new(self.pool.clone());
        // The asset's value is SIMAN's, never the operator's: it is what decides
        // whether the record is within the Wilayah's authority, and an operator
        // who could type it could simply type a small one.
        //
        // The lookup used to run with `AsetScope::All` ("satker ownership is
        // enforced by the workflow" — it was not: nothing compared the asset's
        // satker to the caller's), so an operator could open a removal for any
        // satker's asset. And a failed lookup fell back to the operator's own
        // number. Both are closed: the lookup is scoped to the caller, and a SIMAN
        // failure refuses the request instead of trusting the client.
        match bank_repo
            .find_nilai_perolehan(&request.nup, Some(&request.kode_barang), aset_scope)
            .await
        {
            Ok(Some(nilai)) => {
                request.nilai_perolehan = Some(nilai);
            }
            Ok(None) => {
                return Err(AppError::BadRequest(format!(
                    "BMN dgn NUP {} (kode_barang {}) tidak ditemukan di SIMAN untuk satker Anda. \
                     Pastikan kode_barang & NUP cocok dgn data SIMAN.",
                    request.nup, request.kode_barang
                )));
            }
            Err(e) => {
                warn!(
                    "find_nilai_perolehan failed for NUP {}: {} — refusing rather than \
                     trusting the operator-supplied value",
                    request.nup, e
                );
                return Err(AppError::Internal(
                    "Data SIMAN belum dapat diverifikasi; usulan tidak dapat dibuat sekarang. \
                     Coba lagi beberapa saat."
                        .to_string(),
                ));
            }
        }

        // Authority follows value. When a ceiling is configured, a value above it
        // cannot be routed through the Wilayah, whatever the operator selected.
        enforce_wilayah_value_ceiling(
            &request.kewenangan_penetap_sk,
            request.nilai_perolehan,
            wilayah_max_nilai_from_env(),
        )?;

        self.repository
            .create(request, created_by, satker_code)
            .await
    }

    /// Get penghapusan BMN by ID, within the caller's satker scope.
    pub async fn get_by_id(&self, id: Uuid, scope: &SatkerScope) -> AppResult<PenghapusanBmn> {
        self.repository.get_by_id(id, scope).await
    }

    /// Get penghapusan BMN detail with allowed transitions
    pub async fn get_detail(
        &self,
        id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmnDetailResponse> {
        let penghapusan = self.repository.get_by_id(id, scope).await?;
        let status = PenghapusanBmnStatus::from_state_name(&penghapusan.status).unwrap_or_default();

        let transitions: Vec<PenghapusanTransitionInfo> = status
            .allowed_transitions()
            .into_iter()
            .map(|s| PenghapusanTransitionInfo {
                status_kode: s.to_code(),
                status_nama: s.label().to_string(),
                action_label: s.action_label().to_string(),
                to_state: s.to_state_name().to_string(),
                requires_comment: matches!(
                    s,
                    PenghapusanBmnStatus::ReturnedToOperator | PenghapusanBmnStatus::Rejected
                ),
            })
            .collect();

        let can_generate_sk = matches!(status, PenghapusanBmnStatus::VerifikasiPusat);
        let can_upload_signed_sk = matches!(status, PenghapusanBmnStatus::KonsepSKGenerated);

        // V036 (Fase 2.8): sertakan daftar item BMN multi-item.
        let items = self.repository.list_items(id).await?;

        Ok(PenghapusanBmnDetailResponse {
            penghapusan,
            allowed_transitions: transitions,
            can_generate_sk,
            can_upload_signed_sk,
            items,
        })
    }

    /// Verifikasi aset usulan ke SIMAN (Fase 2.3).
    ///
    /// Dipakai validator (Wilayah/Pusat) saat menelaah usulan: memastikan
    /// NUP masih terdaftar di SIMAN, kode_barang konsisten, dan menampilkan
    /// kondisi terkini (BAIK/RR/RB) + nilai perolehan. Tujuan: mencegah
    /// penerbitan SK penghapusan atas aset yg sudah tidak ada / tidak cocok.
    /// Bersifat read-only & best-effort — sumber: replika `integrasi.siman_aset`.
    pub async fn verify_asset_siman(
        &self,
        id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<SimanAssetVerification> {
        let record = self.repository.get_by_id(id, scope).await?;
        let bank_repo = BankAsetRepository::new(self.pool.clone());
        // Authoritative SIMAN cross-check — unscoped on purpose; the penghapusan
        // record's satker ownership was validated when it was created.
        //
        // Two lookups, because the three outcomes this endpoint reports are
        // genuinely different questions and one query cannot answer both:
        //
        //   1. full identity (satker + kode barang + NUP) -> the asset exists
        //      and matches the usulan;
        //   2. satker + NUP only -> an asset with that NUP exists at this
        //      satker but under a DIFFERENT barang code, which is the
        //      "verifikasi manual" case;
        //   3. neither -> not in SIMAN.
        //
        // It used to be one lookup keyed on NUP alone. NUP repeats across the
        // whole country (14 142 distinct values over 624 533 assets), so the
        // arbitrary row it picked carried the usulan's barang code for an
        // expected 10% of assets — meaning branch 2 fired, and `layak_lanjut`
        // came back false, for roughly nine of every ten valid usulan.
        let scope = crate::bank_aset::AsetScope::All;
        let satker_code = record.satker_code.as_deref();
        let exact = bank_repo
            .find_lookup_by_identity(
                crate::bank_aset::AsetIdentity {
                    nup: &record.nup,
                    kode_barang: Some(&record.kode_barang),
                    satker_code,
                },
                &scope,
            )
            .await?;
        let lookup = match exact {
            Some(asset) => Some(asset),
            None => {
                bank_repo
                    .find_lookup_by_identity(
                        crate::bank_aset::AsetIdentity {
                            nup: &record.nup,
                            kode_barang: None,
                            satker_code,
                        },
                        &scope,
                    )
                    .await?
            }
        };

        let verification = match lookup {
            Some(asset) => {
                let kode_barang_siman = asset.kode_barang.clone();
                let kode_barang_cocok = kode_barang_siman
                    .as_deref()
                    .map(|k| normalize_kode_barang(k) == normalize_kode_barang(&record.kode_barang))
                    .unwrap_or(false);
                // Every message names all three parts of the identity, because
                // "NUP 677" on its own is ambiguous to the validator reading it
                // just as it was to the query that produced it.
                let aset_disebut = format!(
                    "satker {} / kode barang {} / NUP {}",
                    record.satker_code.as_deref().unwrap_or("-"),
                    record.kode_barang,
                    record.nup
                );
                let pesan = if kode_barang_cocok {
                    format!(
                        "Aset {aset_disebut} terdaftar di SIMAN dengan kondisi {}.",
                        asset.kondisi.as_deref().unwrap_or("tidak diketahui")
                    )
                } else {
                    format!(
                        "Di satker {} ada aset ber-NUP {}, namun kode barangnya di SIMAN ({}) berbeda dari usulan ({}). Mohon verifikasi manual.",
                        record.satker_code.as_deref().unwrap_or("-"),
                        record.nup,
                        kode_barang_siman.as_deref().unwrap_or("-"),
                        record.kode_barang
                    )
                };
                SimanAssetVerification {
                    nup: record.nup.clone(),
                    ditemukan: true,
                    kode_barang_diajukan: record.kode_barang.clone(),
                    kode_barang_siman,
                    kode_barang_cocok,
                    nama_barang_siman: asset.nama_barang,
                    merk: asset.merk,
                    kondisi: asset.kondisi,
                    nilai_perolehan_siman: asset.nilai_perolehan,
                    pesan,
                    layak_lanjut: kode_barang_cocok,
                }
            }
            None => SimanAssetVerification {
                nup: record.nup.clone(),
                ditemukan: false,
                kode_barang_diajukan: record.kode_barang.clone(),
                kode_barang_siman: None,
                kode_barang_cocok: false,
                nama_barang_siman: None,
                merk: None,
                kondisi: None,
                nilai_perolehan_siman: None,
                pesan: format!(
                    "Aset satker {} / kode barang {} / NUP {} TIDAK ditemukan di SIMAN. Aset mungkin sudah dihapus/dipindahkan — penerbitan SK perlu kehati-hatian.",
                    record.satker_code.as_deref().unwrap_or("-"),
                    record.kode_barang,
                    record.nup
                ),
                layak_lanjut: false,
            },
        };

        Ok(verification)
    }

    /// List penghapusan BMN with filters and pagination.
    ///
    /// `scope` enforces tiered RBAC data visibility (#66).
    pub async fn list(
        &self,
        filters: PenghapusanBmnFilters,
        page: i32,
        per_page: i32,
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<(Vec<PenghapusanBmn>, i64)> {
        self.repository.list(filters, page, per_page, scope).await
    }

    // ========================================================================
    // V030: File upload — Surat Usulan + Lampiran[]
    // ========================================================================

    /// Set Surat Usulan file URL setelah file di-upload via DocumentStorage.
    /// Hanya entity yg sudah ada yg boleh — caller wajib pastikan ID valid.
    pub async fn set_surat_usulan_url(
        &self,
        id: Uuid,
        file_url: &str,
        scope: &SatkerScope,
    ) -> AppResult<()> {
        self.repository.get_by_id(id, scope).await?;
        self.repository.set_surat_usulan_url(id, file_url).await
    }

    /// Insert satu entry lampiran pendukung. `nama` biasanya adalah
    /// nama file asli; `file_url` adalah URL hasil
    /// `DocumentStorage::presigned_url`.
    pub async fn add_lampiran(
        &self,
        penghapusan_id: Uuid,
        upload: LampiranUpload<'_>,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmnLampiran> {
        self.repository.get_by_id(penghapusan_id, scope).await?;
        self.repository
            .insert_lampiran(
                penghapusan_id,
                upload.nama,
                upload.file_url,
                upload.content_type,
                upload.size_bytes,
                upload.uploaded_by,
            )
            .await
    }

    pub async fn list_lampiran(
        &self,
        penghapusan_id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<Vec<PenghapusanBmnLampiran>> {
        self.repository.get_by_id(penghapusan_id, scope).await?;
        self.repository.list_lampiran(penghapusan_id).await
    }

    /// Update penghapusan BMN (only in Draft/ReturnedToOperator status)
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdatePenghapusanBmnRequest,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmn> {
        let current = self.repository.get_by_id(id, scope).await?;
        let status = PenghapusanBmnStatus::from_state_name(&current.status).unwrap_or_default();

        if !matches!(
            status,
            PenghapusanBmnStatus::Draft | PenghapusanBmnStatus::ReturnedToOperator
        ) {
            return Err(crate::shared::error::AppError::WorkflowError(
                "Hanya bisa diubah saat status Draft atau Dikembalikan ke Operator".into(),
            ));
        }

        self.repository.update(id, request, scope).await
    }

    /// Delete penghapusan BMN (only in Draft status)
    pub async fn delete(&self, id: Uuid, scope: &SatkerScope) -> AppResult<()> {
        let current = self.repository.get_by_id(id, scope).await?;
        let status = PenghapusanBmnStatus::from_state_name(&current.status).unwrap_or_default();

        if !matches!(status, PenghapusanBmnStatus::Draft) {
            return Err(crate::shared::error::AppError::WorkflowError(
                "Hanya bisa dihapus saat status Draft".into(),
            ));
        }

        self.repository.delete(id, scope).await
    }

    /// Operator Satker submits to Validator Wilayah
    pub async fn submit_to_wilayah(
        &self,
        id: Uuid,
        user_id: Uuid,
        user_role: String,
        catatan: Option<String>,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmn> {
        self.transition(
            id,
            PenghapusanBmnStatus::SubmitWilayah
                .to_state_name()
                .to_string(),
            TransitionActor::new(user_id, user_role, catatan, "submit_to_wilayah"),
            scope,
        )
        .await
    }

    /// Validator Wilayah forwards to Validator Pusat
    pub async fn forward_to_pusat(
        &self,
        id: Uuid,
        validator_id: Uuid,
        user_role: String,
        catatan: Option<String>,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmn> {
        // Scope BEFORE the write, not just before the transition: this stamps
        // the record with a validator identity, so an out-of-scope caller must
        // not reach it even if the transition would later be refused.
        self.repository.get_by_id(id, scope).await?;

        // Update validator wilayah info
        self.repository
            .update_validator_wilayah(id, validator_id, catatan.clone())
            .await?;

        self.transition(
            id,
            PenghapusanBmnStatus::SubmitPusat
                .to_state_name()
                .to_string(),
            TransitionActor::new(validator_id, user_role, catatan, "forward_to_pusat"),
            scope,
        )
        .await
    }

    /// Validator Pusat verifies the asset and advances SubmitPusat →
    /// VerifikasiPusat.
    ///
    /// This is the deliberate human gate the workflow describes: the Validator
    /// Pusat consults the read-only SIMAN verification (`verifikasi-siman` GET)
    /// and then confirms it here. The SIMAN match is advisory — the asset may
    /// legitimately be absent from the replica — so verification stamps the
    /// validator + timestamp and moves the status without hard-blocking on a
    /// SIMAN hit (the read-only endpoint already surfaces any caution to the
    /// reviewer).
    pub async fn verifikasi_pusat(
        &self,
        id: Uuid,
        validator_id: Uuid,
        user_role: String,
        catatan: Option<String>,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmn> {
        self.repository.get_by_id(id, scope).await?;

        self.repository
            .update_verifikasi_pusat(id, validator_id, catatan.clone())
            .await?;

        self.transition(
            id,
            PenghapusanBmnStatus::VerifikasiPusat
                .to_state_name()
                .to_string(),
            TransitionActor::new(validator_id, user_role, catatan, "verifikasi_pusat"),
            scope,
        )
        .await
    }

    /// Validator Wilayah returns to Operator Satker
    pub async fn return_to_operator(
        &self,
        id: Uuid,
        validator_id: Uuid,
        user_role: String,
        catatan: Option<String>,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmn> {
        self.repository.get_by_id(id, scope).await?;

        self.repository
            .update_validator_wilayah(id, validator_id, catatan.clone())
            .await?;

        self.transition(
            id,
            PenghapusanBmnStatus::ReturnedToOperator
                .to_state_name()
                .to_string(),
            TransitionActor::new(validator_id, user_role, catatan, "return_to_operator"),
            scope,
        )
        .await
    }

    /// Filesystem path the konsep-sk route handler streams from. `format` is
    /// "docx" or "pdf".
    pub async fn konsep_sk_path(
        &self,
        id: Uuid,
        format: &str,
        scope: &SatkerScope,
    ) -> AppResult<Option<String>> {
        self.repository.get_by_id(id, scope).await?;
        self.repository.konsep_sk_path(id, format).await
    }

    /// Validator Pusat generates konsep SK in BOTH DOCX (editable) and PDF
    /// (final) formats. Files land under
    /// `${DOCUMENT_STORAGE_PATH}/penghapusan-bmn/{id}/konsep-sk.{ext}`; the
    /// public download URLs are persisted in `konsep_sk_url` (DOCX) and
    /// `konsep_sk_pdf_url` (PDF).
    pub async fn generate_konsep_sk(
        &self,
        id: Uuid,
        validator_id: Uuid,
        user_role: String,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmn> {
        let penghapusan = self.repository.get_by_id(id, scope).await?;
        let status = PenghapusanBmnStatus::from_state_name(&penghapusan.status).unwrap_or_default();

        if !matches!(status, PenghapusanBmnStatus::VerifikasiPusat) {
            return Err(crate::shared::error::AppError::WorkflowError(
                "Konsep SK hanya bisa digenerate saat status Verifikasi Pusat".into(),
            ));
        }

        let docs = self.docs.as_ref().ok_or_else(|| {
            crate::shared::error::AppError::Internal(
                "DocumentGenerator port not wired into PenghapusanBmnService".into(),
            )
        })?;

        // V036 (Fase 2.8): sertakan seluruh item BMN ke konteks template agar
        // SK dapat mencetak N item (tabel). `nama_barang` tunggal tetap ada
        // untuk kompatibilitas template lama.
        let items = self.repository.list_items(id).await?;
        let items_json: Vec<serde_json::Value> = items
            .iter()
            .map(|it| {
                serde_json::json!({
                    "urutan": it.urutan,
                    "kode_barang": it.kode_barang,
                    "nama_barang": it.nama_barang,
                    "nup": it.nup,
                    "nilai_perolehan": it.nilai_perolehan,
                    "kondisi": it.kondisi,
                })
            })
            .collect();

        let template_id = std::env::var("KONSEP_SK_TEMPLATE_ID")
            .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000002".to_string());
        let data = serde_json::json!({
            "id": id.to_string(),
            // The authoritative MySIMKARI code. This was `satker_id`, the UUID
            // V010 dropped — no template ever referenced it, so an SK could
            // never have shown it, but it was the value on offer.
            "satker_code": penghapusan.satker_code,
            "nama_barang": penghapusan.nama_barang,
            "alasan": penghapusan.alasan,
            "status": penghapusan.status,
            "approval_date": chrono::Utc::now().format("%d %B %Y").to_string(),
            "items": items_json,
            "jumlah_item": items.len(),
        });

        let storage_root = std::env::var("DOCUMENT_STORAGE_PATH")
            .unwrap_or_else(|_| "/tmp/perlengkapan/docs".to_string());
        let dir = format!("{}/penghapusan-bmn/{}", storage_root, id);
        tokio::fs::create_dir_all(&dir).await.map_err(|e| {
            crate::shared::error::AppError::Internal(format!("mkdir {}: {}", dir, e))
        })?;
        let docx_path = format!("{}/konsep-sk.docx", dir);
        let pdf_path = format!("{}/konsep-sk.pdf", dir);

        let docx_request = crate::contracts::DocumentRequest {
            template_id: template_id.clone(),
            format: crate::contracts::DocumentFormat::Docx,
            data: data.clone(),
            locale: None,
            requested_by: None,
        };
        let pdf_request = crate::contracts::DocumentRequest {
            template_id,
            format: crate::contracts::DocumentFormat::Pdf,
            data,
            locale: None,
            requested_by: None,
        };

        let docx_bytes = docs.preview(docx_request).await.map_err(|e| {
            crate::shared::error::AppError::Internal(format!("konsep SK DOCX render failed: {}", e))
        })?;
        tokio::fs::write(&docx_path, &docx_bytes)
            .await
            .map_err(|e| {
                crate::shared::error::AppError::Internal(format!("write {}: {}", docx_path, e))
            })?;

        let pdf_bytes = docs.preview(pdf_request).await.map_err(|e| {
            crate::shared::error::AppError::Internal(format!("konsep SK PDF render failed: {}", e))
        })?;
        tokio::fs::write(&pdf_path, &pdf_bytes).await.map_err(|e| {
            crate::shared::error::AppError::Internal(format!("write {}: {}", pdf_path, e))
        })?;

        let docx_url = format!("/api/v1/perlengkapan/penghapusan-bmn/{}/konsep-sk.docx", id);
        let pdf_url = format!("/api/v1/perlengkapan/penghapusan-bmn/{}/konsep-sk.pdf", id);

        self.repository
            .update_konsep_sk(id, &docx_url, &docx_path, &pdf_url, &pdf_path)
            .await?;

        // Transition to KonsepSKGenerated
        self.transition(
            id,
            PenghapusanBmnStatus::KonsepSKGenerated
                .to_state_name()
                .to_string(),
            TransitionActor::new(
                validator_id,
                user_role,
                Some(
                    "Konsep Usulan SK Penghapusan BMN berhasil digenerate (DOCX + PDF)".to_string(),
                ),
                "generate_konsep_sk",
            ),
            scope,
        )
        .await
    }

    /// Validator Pusat uploads signed SK PDF
    pub async fn upload_signed_sk(
        &self,
        id: Uuid,
        validator_id: Uuid,
        user_role: String,
        signed_sk_pdf_url: String,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmn> {
        let penghapusan = self.repository.get_by_id(id, scope).await?;
        let status = PenghapusanBmnStatus::from_state_name(&penghapusan.status).unwrap_or_default();

        if !matches!(status, PenghapusanBmnStatus::KonsepSKGenerated) {
            return Err(crate::shared::error::AppError::WorkflowError(
                "SK hanya bisa diupload setelah konsep SK digenerate".into(),
            ));
        }

        // Update signed SK PDF URL
        self.repository
            .update_signed_sk(id, &signed_sk_pdf_url)
            .await?;

        // The validator's signing action transitions to SK_SIGNED under their own
        // role (config: SK_SIGNED → validator_pusat).
        self.transition(
            id,
            PenghapusanBmnStatus::SKSigned.to_state_name().to_string(),
            TransitionActor::new(
                validator_id,
                user_role,
                Some("Usulan SK Penghapusan BMN telah ditandatangani".to_string()),
                "upload_signed_sk",
            ),
            scope,
        )
        .await?;

        // Auto-complete is a SYSTEM-initiated continuation of the decision the
        // validator just took (signing the SK), not a second decision, so it
        // runs as the engine's internal actor. The audit row still names the
        // validator (`validator_id`) as the user.
        self.transition(
            id,
            PenghapusanBmnStatus::Completed.to_state_name().to_string(),
            TransitionActor::new(
                validator_id,
                "system".to_string(),
                Some("Proses Usulan SK Penghapusan BMN selesai".to_string()),
                "complete",
            ),
            scope,
        )
        .await
    }

    // ========================================================================
    // V029 (Fase 1.9): SK Wilayah workflow
    // ========================================================================

    /// Generate konsep SK jalur WILAYAH. Hanya valid jika entity
    /// kewenangan_penetap_sk='WILAYAH' DAN status saat ini = SubmitWilayah.
    /// Skip jalur Pusat sepenuhnya: SubmitWilayah → KonsepSKWilayahGenerated.
    pub async fn generate_konsep_sk_wilayah(
        &self,
        id: Uuid,
        validator_id: Uuid,
        user_role: String,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmn> {
        let penghapusan = self.repository.get_by_id(id, scope).await?;
        if penghapusan.kewenangan_penetap_sk.to_uppercase() != "WILAYAH" {
            return Err(crate::shared::error::AppError::BadRequest(
                "Endpoint ini hanya utk kewenangan WILAYAH. Gunakan /generate-sk utk PUSAT.".into(),
            ));
        }
        let status = PenghapusanBmnStatus::from_state_name(&penghapusan.status).unwrap_or_default();
        if !matches!(status, PenghapusanBmnStatus::SubmitWilayah) {
            return Err(crate::shared::error::AppError::WorkflowError(
                "Konsep SK Wilayah hanya bisa digenerate dari status SubmitWilayah".into(),
            ));
        }

        // Generate konsep SK URLs (DOCX + PDF). Untuk MVP gunakan path
        // placeholder dgn template yg sama (template SK Wilayah final
        // pending Biro Hukum). Service nyata pakai `self.docs` jika
        // ter-inject — sama dgn generate_konsep_sk PUSAT.
        // Untuk PR ini, isi URL sementara berdasarkan storage convention.
        let docx_url = format!("/storage/penghapusan-bmn/{}/konsep-sk-wilayah.docx", id);
        let pdf_url = format!("/storage/penghapusan-bmn/{}/konsep-sk-wilayah.pdf", id);

        self.repository
            .update_konsep_sk_wilayah(id, &docx_url, Some(&pdf_url))
            .await?;

        // Audit aktivitas
        self.transition(
            id,
            PenghapusanBmnStatus::KonsepSKWilayahGenerated
                .to_state_name()
                .to_string(),
            TransitionActor::new(
                validator_id,
                user_role,
                Some("Konsep SK Wilayah digenerate (mewakili Kepala Kejaksaan Tinggi)".into()),
                "generate_konsep_sk_wilayah",
            ),
            scope,
        )
        .await
    }

    /// Upload signed SK PDF jalur WILAYAH. KonsepSKWilayahGenerated →
    /// SKSignedWilayah → Completed (auto).
    pub async fn upload_signed_sk_wilayah(
        &self,
        id: Uuid,
        validator_id: Uuid,
        user_role: String,
        signed_sk_pdf_url: String,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmn> {
        let penghapusan = self.repository.get_by_id(id, scope).await?;
        let status = PenghapusanBmnStatus::from_state_name(&penghapusan.status).unwrap_or_default();
        if !matches!(status, PenghapusanBmnStatus::KonsepSKWilayahGenerated) {
            return Err(crate::shared::error::AppError::WorkflowError(
                "SK Wilayah hanya bisa diupload setelah konsep SK Wilayah digenerate".into(),
            ));
        }

        self.repository
            .update_signed_sk_wilayah(id, &signed_sk_pdf_url)
            .await?;

        // Kepala Kejati's signing action transitions to SK_SIGNED_WILAYAH under
        // their own role (config: SK_SIGNED_WILAYAH → validator_wilayah).
        self.transition(
            id,
            PenghapusanBmnStatus::SKSignedWilayah
                .to_state_name()
                .to_string(),
            TransitionActor::new(
                validator_id,
                user_role,
                Some("SK Wilayah ditandatangani Kepala Kejaksaan Tinggi".into()),
                "upload_signed_sk_wilayah",
            ),
            scope,
        )
        .await?;
        // Auto-complete = SYSTEM continuation (see upload_signed_sk): run as the
        // engine's internal actor; the Wilayah validator who signed is not the
        // role COMPLETED names (validator_pusat), and should not need to be.
        self.transition(
            id,
            PenghapusanBmnStatus::Completed.to_state_name().to_string(),
            TransitionActor::new(
                validator_id,
                "system".to_string(),
                Some("Proses Usulan SK Penghapusan BMN (jalur Wilayah) selesai".into()),
                "complete_wilayah",
            ),
            scope,
        )
        .await
    }

    /// Perform workflow transition, within the caller's satker scope.
    ///
    /// Role gating answers "may this role ever do this"; the scope answers "may
    /// this caller do it to THIS record". Without the second, a validator
    /// wilayah in one Kejati could forward a usulan belonging to another.
    pub async fn transition(
        &self,
        id: Uuid,
        to_state: String,
        actor: TransitionActor,
        scope: &SatkerScope,
    ) -> AppResult<PenghapusanBmn> {
        let penghapusan = self.repository.get_by_id(id, scope).await?;

        let transition_request = TransitionRequest {
            entity_id: id,
            from_state: penghapusan.status.clone(),
            to_state: to_state.clone(),
            user_id: actor.user_id,
            user_role: actor.user_role,
            catatan: actor.catatan,
            ip_address: actor.ip_address,
        };

        self.workflow_engine
            .transition(transition_request)
            .await
            .map_err(|e| crate::shared::error::AppError::WorkflowError(e.to_string()))?;

        self.repository.update_status(id, &to_state).await?;
        self.repository.get_by_id(id, scope).await
    }
}

/// The highest asset value (rupiah) the Kejaksaan Tinggi may decide on its own,
/// from `PENGHAPUSAN_WILAYAH_MAX_NILAI`. `None` when unset or unparseable.
///
/// Configuration rather than a constant on purpose: the threshold is a
/// regulatory figure (PMK 83/PMK.06/2016 and its amendments) that changes by
/// regulation, not by release, and this codebase does not hard-code a number
/// nobody has confirmed. Unset = no ceiling is enforced (the pre-existing
/// behaviour); set = enforced for every new usulan.
pub fn wilayah_max_nilai_from_env() -> Option<f64> {
    std::env::var("PENGHAPUSAN_WILAYAH_MAX_NILAI")
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
}

/// Refuse the WILAYAH route for a value above `ceiling`.
///
/// With a ceiling configured the rule fails closed: an unknown value
/// (`nilai = None`) cannot be shown to be within it, so it is refused too.
/// Without one, nothing is enforced. The PUSAT route is always allowed —
/// sending a matter to the higher authority is never the violation.
pub fn enforce_wilayah_value_ceiling(
    kewenangan: &str,
    nilai: Option<f64>,
    ceiling: Option<f64>,
) -> AppResult<()> {
    let Some(max) = ceiling else {
        return Ok(());
    };
    if !kewenangan.eq_ignore_ascii_case("WILAYAH") {
        return Ok(());
    }
    match nilai {
        Some(v) if v <= max => Ok(()),
        Some(v) => Err(AppError::BadRequest(format!(
            "Nilai perolehan Rp {v:.0} melebihi batas kewenangan Wilayah (Rp {max:.0}); \
             usulan harus melalui kewenangan PUSAT"
        ))),
        None => Err(AppError::BadRequest(
            "Nilai perolehan tidak diketahui, sehingga tidak dapat dipastikan berada dalam batas \
             kewenangan Wilayah; gunakan kewenangan PUSAT"
                .to_string(),
        )),
    }
}

#[cfg(test)]
mod ceiling_tests {
    use super::*;

    #[test]
    fn without_a_ceiling_nothing_is_enforced() {
        assert!(enforce_wilayah_value_ceiling("WILAYAH", Some(1e12), None).is_ok());
        assert!(enforce_wilayah_value_ceiling("WILAYAH", None, None).is_ok());
    }

    #[test]
    fn a_value_above_the_ceiling_cannot_use_the_wilayah_route() {
        assert!(enforce_wilayah_value_ceiling("WILAYAH", Some(999.0), Some(1000.0)).is_ok());
        assert!(enforce_wilayah_value_ceiling("WILAYAH", Some(1000.0), Some(1000.0)).is_ok());
        assert!(enforce_wilayah_value_ceiling("WILAYAH", Some(1000.01), Some(1000.0)).is_err());
        assert!(enforce_wilayah_value_ceiling("wilayah", Some(5000.0), Some(1000.0)).is_err());
    }

    #[test]
    fn an_unknown_value_fails_closed_once_a_ceiling_exists() {
        assert!(enforce_wilayah_value_ceiling("WILAYAH", None, Some(1000.0)).is_err());
    }

    #[test]
    fn the_pusat_route_is_never_the_violation() {
        assert!(enforce_wilayah_value_ceiling("PUSAT", Some(1e12), Some(1000.0)).is_ok());
        assert!(enforce_wilayah_value_ceiling("PUSAT", None, Some(1000.0)).is_ok());
    }
}
