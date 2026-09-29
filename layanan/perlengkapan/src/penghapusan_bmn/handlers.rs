// ============================================================================
// Penghapusan BMN Handlers
// Description: HTTP request handlers for Usulan SK Penghapusan BMN workflow
// Requirements: REQ-W001, REQ-W004
//
// Endpoints:
//   POST   /penghapusan-bmn               - Create new Usulan SK Penghapusan BMN
//   GET    /penghapusan-bmn               - List with filters
//   GET    /penghapusan-bmn/:id           - Get by ID
//   GET    /penghapusan-bmn/:id/detail    - Get detail with transitions
//   PUT    /penghapusan-bmn/:id           - Update (Draft only)
//   DELETE /penghapusan-bmn/:id           - Delete (Draft only)
//   POST   /penghapusan-bmn/:id/submit-wilayah    - Submit to Validator Wilayah
//   POST   /penghapusan-bmn/:id/forward-pusat     - Forward to Validator Pusat
//   POST   /penghapusan-bmn/:id/verifikasi-pusat  - Validator Pusat verifies (→ VerifikasiPusat)
//   POST   /penghapusan-bmn/:id/return-operator   - Return to Operator
//   POST   /penghapusan-bmn/:id/generate-sk       - Generate konsep SK DOCX
//   POST   /penghapusan-bmn/:id/upload-signed-sk  - Upload signed SK PDF
//   POST   /penghapusan-bmn/:id/transition        - Generic transition (legacy)
// ============================================================================

use super::models::*;
use super::services::{LampiranUpload, PenghapusanBmnService, TransitionActor};
use crate::contracts::DocumentStorage;
use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use crate::shared::satker_scope::SatkerScope;
use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use validator::Validate;

/// Sanitize a user-supplied filename so it cannot escape its storage key
/// segment. Strips path separators & control chars, collapses spaces.
fn sanitize_filename(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let cleaned = cleaned.trim_matches('_').to_string();
    if cleaned.is_empty() {
        "file".to_string()
    } else {
        cleaned
    }
}

/// Pagination query parameters
#[derive(Debug, Deserialize, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 1))]
    pub page: Option<i32>,
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<i32>,
}

/// API response wrapper
///
/// Re-exported from `lib_perlengkapan` rather than redeclared. A local copy
/// lived here and in `workflow::handlers` / `workflow::definition_handlers`;
/// the `PaginatedResponse` sibling below had already drifted from the shared
/// one by dropping `total_pages`, which made serde reject the whole
/// `/penghapusan-bmn` list response — the page rendered "Gagal memuat data"
/// over a perfectly healthy 200. One type, one shape.
pub use lib_perlengkapan::response::ApiResponse;

/// Paginated response
pub use lib_perlengkapan::response::PaginatedResponse;

/// Create penghapusan BMN
pub async fn create_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    claims: Claims,
    Json(request): Json<CreatePenghapusanBmnRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PenghapusanBmn>>), AppError> {
    request.validate()?;

    // Only an operator opens a usulan. This had no role check, so any
    // authenticated caller — a validator, a caller from another satker — could
    // create one; the policy's `Create` action existed and was never consulted.
    claims.require_any_role(&["operator_satker"])?;
    crate::shared::upload::validate_document_url(
        "lampiran_persyaratan",
        &request.lampiran_persyaratan,
    )?;

    // Derive the authoritative satker from identity (#66), not from client input,
    // and confine the SIMAN lookup to the caller's own satker's assets.
    let penghapusan = service
        .create(
            request,
            claims.user_id,
            claims.satker_code.clone(),
            &crate::bank_aset::AsetScope::from_claims(&claims),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            penghapusan,
            "Usulan SK Penghapusan BMN berhasil dibuat".to_string(),
        )),
    ))
}

/// Get penghapusan BMN by ID
pub async fn get_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    let penghapusan = service
        .get_by_id(id, &SatkerScope::from_claims(&claims))
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Usulan SK Penghapusan BMN retrieved successfully".to_string(),
    )))
}

/// Verifikasi aset usulan ke SIMAN (Fase 2.3). Read-only — dipakai validator
/// (Wilayah/Pusat) saat menelaah usulan sebelum menerbitkan SK.
pub async fn verify_penghapusan_asset_siman(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<SimanAssetVerification>>, AppError> {
    let verification = service
        .verify_asset_siman(id, &SatkerScope::from_claims(&claims))
        .await?;

    Ok(Json(ApiResponse::success(
        verification,
        "Verifikasi aset SIMAN selesai".to_string(),
    )))
}

/// Get penghapusan BMN detail with allowed transitions
pub async fn get_penghapusan_bmn_detail(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<PenghapusanBmnDetailResponse>>, AppError> {
    let mut detail = service
        .get_detail(id, &SatkerScope::from_claims(&claims))
        .await?;

    // `get_detail` answers "what may happen next" from the STATE alone. Narrow
    // it to "what may THIS caller do", so the FE can render the list verbatim
    // and never offer a button that would come back 403. Enforcement still
    // happens on each action handler below — this only shapes what is offered.
    detail.allowed_transitions.retain(|t| {
        PenghapusanBmnStatus::from_code(t.status_kode)
            .is_some_and(|s| claims.has_any_role(s.actor_roles()))
    });

    Ok(Json(ApiResponse::success(
        detail,
        "Detail Usulan SK Penghapusan BMN berhasil diambil".to_string(),
    )))
}

/// List penghapusan BMN with filters
#[derive(Debug, Deserialize)]
pub struct ListPenghapusanQuery {
    pub status: Option<String>,
    pub metode_penghapusan: Option<String>,
    pub tahun: Option<i32>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

pub async fn list_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Query(query): Query<ListPenghapusanQuery>,
    claims: Claims,
) -> Result<Json<PaginatedResponse<PenghapusanBmn>>, AppError> {
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
    let filters = PenghapusanBmnFilters {
        status: query.status,
        status_kode: None,
        metode_penghapusan: query.metode_penghapusan,
        tahun: query.tahun,
    };

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let (penghapusan, total) = service.list(filters, page, per_page, &scope).await?;

    Ok(Json(PaginatedResponse::new(
        penghapusan,
        total,
        page,
        per_page,
        "Daftar Usulan SK Penghapusan BMN berhasil diambil".to_string(),
    )))
}

/// Update penghapusan BMN
pub async fn update_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<UpdatePenghapusanBmnRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    // Editing a usulan is the operator's move, the same one `delete` below
    // already gated. This handler used to take `_claims` and check neither role
    // nor satker, so any authenticated caller could rewrite any satker's draft.
    claims.require_any_role(&["operator_satker"])?;
    request.validate()?;

    let penghapusan = service
        .update(id, request, &SatkerScope::from_claims(&claims))
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Usulan SK Penghapusan BMN berhasil diperbarui".to_string(),
    )))
}

/// Delete penghapusan BMN
pub async fn delete_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    // RBAC (Fase 0.3): hanya operator_satker atau admin yg boleh delete
    // usulan (Draft saja per logika service); admin sbg escape hatch.
    claims.require_any_role(&["operator_satker"])?;
    service
        .delete(id, &SatkerScope::from_claims(&claims))
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            (),
            "Usulan SK Penghapusan BMN berhasil dihapus".to_string(),
        )),
    ))
}

// ============================================================================
// Workflow Action Handlers
// ============================================================================

/// Request body for submitting to Validator Wilayah
#[derive(Debug, Deserialize)]
pub struct SubmitWilayahBody {
    pub catatan: Option<String>,
}

/// Operator Satker submits to Validator Wilayah
pub async fn submit_to_wilayah(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<SubmitWilayahBody>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    claims.require_any_role(PenghapusanBmnStatus::SubmitWilayah.actor_roles())?;
    let penghapusan = service
        .submit_to_wilayah(
            id,
            claims.user_id,
            claims.acting_role(PenghapusanBmnStatus::SubmitWilayah.actor_roles()),
            body.catatan,
            &SatkerScope::from_claims(&claims),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Pengajuan berhasil dikirim ke Validator Wilayah".to_string(),
    )))
}

/// Validator Wilayah forwards to Validator Pusat
pub async fn forward_to_pusat(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<ValidatorWilayahActionRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    claims.require_any_role(PenghapusanBmnStatus::SubmitPusat.actor_roles())?;
    if body.aksi != "forward" {
        return Err(AppError::BadRequest("Action harus 'forward'".to_string()));
    }

    let penghapusan = service
        .forward_to_pusat(
            id,
            claims.user_id,
            claims.acting_role(PenghapusanBmnStatus::SubmitPusat.actor_roles()),
            body.catatan,
            &SatkerScope::from_claims(&claims),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Pengajuan berhasil diteruskan ke Validator Pusat".to_string(),
    )))
}

/// Request body for Validator Pusat verification
#[derive(Debug, Deserialize)]
pub struct VerifikasiPusatBody {
    pub catatan: Option<String>,
}

/// Validator Pusat verifies the asset (against SIMAN) and advances the usulan
/// SubmitPusat → VerifikasiPusat, unlocking konsep-SK generation.
pub async fn verifikasi_pusat(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<VerifikasiPusatBody>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    claims.require_any_role(PenghapusanBmnStatus::VerifikasiPusat.actor_roles())?;
    let penghapusan = service
        .verifikasi_pusat(
            id,
            claims.user_id,
            claims.acting_role(PenghapusanBmnStatus::VerifikasiPusat.actor_roles()),
            body.catatan,
            &SatkerScope::from_claims(&claims),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Usulan berhasil diverifikasi Validator Pusat".to_string(),
    )))
}

/// Validator Wilayah returns to Operator Satker
pub async fn return_to_operator(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<ValidatorWilayahActionRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    claims.require_any_role(PenghapusanBmnStatus::ReturnedToOperator.actor_roles())?;
    if body.aksi != "return" {
        return Err(AppError::BadRequest("Action harus 'return'".to_string()));
    }

    if body.catatan.is_none() {
        return Err(AppError::BadRequest(
            "Catatan diperlukan saat mengembalikan ke Operator".to_string(),
        ));
    }

    let penghapusan = service
        .return_to_operator(
            id,
            claims.user_id,
            claims.acting_role(PenghapusanBmnStatus::ReturnedToOperator.actor_roles()),
            body.catatan,
            &SatkerScope::from_claims(&claims),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Pengajuan dikembalikan ke Operator Satker".to_string(),
    )))
}

/// Validator Wilayah action (combined forward/return)
pub async fn validator_wilayah_action(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<ValidatorWilayahActionRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    // Both branches are validator_wilayah moves; the state machine says so.
    claims.require_any_role(PenghapusanBmnStatus::SubmitPusat.actor_roles())?;
    match body.aksi.as_str() {
        "forward" => {
            let penghapusan = service
                .forward_to_pusat(
                    id,
                    claims.user_id,
                    claims.acting_role(PenghapusanBmnStatus::SubmitPusat.actor_roles()),
                    body.catatan,
                    &SatkerScope::from_claims(&claims),
                )
                .await?;
            Ok(Json(ApiResponse::success(
                penghapusan,
                "Pengajuan berhasil diteruskan ke Validator Pusat".to_string(),
            )))
        }
        "return" => {
            if body.catatan.is_none() {
                return Err(AppError::BadRequest(
                    "Catatan diperlukan saat mengembalikan ke Operator".to_string(),
                ));
            }
            let penghapusan = service
                .return_to_operator(
                    id,
                    claims.user_id,
                    claims.acting_role(PenghapusanBmnStatus::SubmitPusat.actor_roles()),
                    body.catatan,
                    &SatkerScope::from_claims(&claims),
                )
                .await?;
            Ok(Json(ApiResponse::success(
                penghapusan,
                "Pengajuan dikembalikan ke Operator Satker".to_string(),
            )))
        }
        _ => Err(AppError::BadRequest(
            "Action harus 'forward' atau 'return'".to_string(),
        )),
    }
}

/// Validator Pusat generates konsep SK in BOTH DOCX (editable) and PDF
/// (final) formats side-by-side.
pub async fn generate_konsep_sk(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    // RBAC: SK generation = otoritas hukum. Validator Wilayah utk
    // kewenangan WILAYAH (mewakili Kepala Kejati), Validator Pusat utk
    // kewenangan PUSAT (mewakili Jaksa Agung Muda Pembinaan). Field
    // kewenangan_penetap_sk seharusnya divalidasi di service layer
    // (lihat plan §6.3); di sini cukup pastikan caller adalah salah
    // satu dari kedua role.
    claims.require_any_role(PenghapusanBmnStatus::KonsepSKGenerated.actor_roles())?;
    let penghapusan = service
        .generate_konsep_sk(
            id,
            claims.user_id,
            claims.acting_role(PenghapusanBmnStatus::KonsepSKGenerated.actor_roles()),
            &SatkerScope::from_claims(&claims),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Konsep Usulan SK Penghapusan BMN berhasil digenerate (DOCX + PDF)".to_string(),
    )))
}

/// GET /penghapusan-bmn/:id/konsep-sk.docx
/// GET /penghapusan-bmn/:id/konsep-sk.pdf
///
/// Stream the on-disk konsep SK artifact. `format` is "docx" or "pdf".
pub async fn serve_konsep_sk(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path((id, format)): Path<(Uuid, String)>,
    claims: Claims,
) -> Result<axum::response::Response, AppError> {
    use axum::body::Body;
    use axum::http::header;
    use axum::response::IntoResponse;

    let format = format.as_str();
    if format != "docx" && format != "pdf" {
        return Err(AppError::BadRequest(format!(
            "format harus 'docx' atau 'pdf', dapat: '{}'",
            format
        )));
    }

    let path = service
        .konsep_sk_path(id, format, &SatkerScope::from_claims(&claims))
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Konsep SK ({}) belum digenerate untuk penghapusan {}",
                format, id
            ))
        })?;

    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| AppError::Internal(format!("read {}: {}", path, e)))?;

    let mime = match format {
        "pdf" => "application/pdf",
        _ => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    };
    let filename = format!("konsep-sk-{}.{}", id, format);

    let resp = (
        [
            (header::CONTENT_TYPE, mime),
            (
                header::CONTENT_DISPOSITION,
                &format!("inline; filename=\"{}\"", filename),
            ),
        ],
        Body::from(bytes),
    )
        .into_response();
    Ok(resp)
}

/// Validator Pusat uploads signed SK PDF
pub async fn upload_signed_sk(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<UploadSignedSKRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    claims.require_any_role(PenghapusanBmnStatus::SKSigned.actor_roles())?;
    // Stored and later rendered as a link for other users: web URLs only.
    crate::shared::upload::validate_document_url("signed_sk_pdf_url", &body.signed_sk_pdf_url)?;
    let penghapusan = service
        .upload_signed_sk(
            id,
            claims.user_id,
            claims.acting_role(PenghapusanBmnStatus::SKSigned.actor_roles()),
            body.signed_sk_pdf_url,
            &SatkerScope::from_claims(&claims),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Usulan SK Penghapusan BMN yang ditandatangani berhasil diupload. Proses selesai."
            .to_string(),
    )))
}

/// Upload Surat Usulan (1 file) dan/atau Lampiran pendukung (multi-file)
/// untuk Usulan SK Penghapusan BMN. Multipart fields:
/// - `surat_usulan` (opsional, 1x): file Surat Usulan
/// - `lampiran` (opsional, 0..N): file lampiran pendukung
///
/// Minimal satu dari kedua field wajib ada. URL hasil upload disimpan di
/// `surat_usulan_file_url` (entity) dan rows di `penghapusan_bmn_lampiran`.
pub async fn upload_lampiran(
    State(service): State<Arc<PenghapusanBmnService>>,
    State(storage): State<Arc<dyn DocumentStorage>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<UploadLampiranResponse>>, AppError> {
    // Verifikasi entity ada DAN dalam scope pemanggil — lookup ini juga
    // memastikan FK constraint nantinya tidak gagal di insert.
    let scope = SatkerScope::from_claims(&claims);
    let existing = service.get_by_id(id, &scope).await?;

    // Lampiran adalah bagian dari usulan yang disusun OPERATOR satker, dan hanya
    // selama usulan masih di tangannya. Sebelumnya tak ada cek peran maupun state:
    // validator (scope wilayah/pusat) — atau siapa pun dalam scope — bisa
    // menambah berkas ke usulan yang sedang diperiksa.
    claims.require_role("operator_satker")?;
    if !matches!(existing.status.as_str(), "DRAFT" | "RETURNED_TO_OPERATOR") {
        return Err(AppError::Conflict(format!(
            "Lampiran hanya dapat ditambahkan saat usulan berstatus Draft atau Dikembalikan (status saat ini: {})",
            existing.status
        )));
    }
    let mut files_seen = 0usize;

    // Presigned URL TTL: 1 tahun. FilesystemStorage abaikan TTL & emit
    // static URL; S3 adapter akan rotate sendiri.
    let url_ttl = Duration::from_secs(60 * 60 * 24 * 365);

    let mut surat_usulan_url: Option<String> = None;
    let mut lampirans: Vec<PenghapusanBmnLampiran> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        let original_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "file".to_string());
        let content_type = field.content_type().map(|s| s.to_string());
        let bytes = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Gagal membaca file: {}", e)))?;
        let size = bytes.len() as i64;
        let safe_name = sanitize_filename(&original_name);

        // Content decides what a file is, not the client's label (see
        // `shared::upload`): allowlisted extension, matching magic bytes, a size
        // ceiling, a per-request file cap — and the stored `Content-Type` is the
        // one WE derive. Unknown form fields are skipped, not validated.
        let validated = if matches!(field_name.as_str(), "surat_usulan" | "lampiran") {
            files_seen += 1;
            if files_seen > crate::shared::upload::MAX_FILES_PER_REQUEST {
                return Err(AppError::BadRequest(format!(
                    "Maksimal {} file per unggahan",
                    crate::shared::upload::MAX_FILES_PER_REQUEST
                )));
            }
            Some(crate::shared::upload::validate_upload(
                &original_name,
                content_type.as_deref(),
                &bytes,
            )?)
        } else {
            None
        };

        match field_name.as_str() {
            "surat_usulan" => {
                let key = format!(
                    "penghapusan-bmn/{}/surat-usulan/{}-{}",
                    id,
                    Uuid::new_v4(),
                    safe_name
                );
                let stored_type = validated
                    .as_ref()
                    .map(|v| v.content_type)
                    .unwrap_or("application/octet-stream");
                let handle = storage
                    .put(&key, bytes, stored_type)
                    .await
                    .map_err(|e| AppError::Internal(format!("Storage put: {}", e)))?;
                let url = storage
                    .presigned_url(&handle.key, url_ttl)
                    .await
                    .map_err(|e| AppError::Internal(format!("Presigned URL: {}", e)))?;
                service.set_surat_usulan_url(id, &url, &scope).await?;
                surat_usulan_url = Some(url);
            }
            "lampiran" => {
                let key = format!(
                    "penghapusan-bmn/{}/lampiran/{}-{}",
                    id,
                    Uuid::new_v4(),
                    safe_name
                );
                let stored_type = validated
                    .as_ref()
                    .map(|v| v.content_type)
                    .unwrap_or("application/octet-stream");
                let handle = storage
                    .put(&key, bytes, stored_type)
                    .await
                    .map_err(|e| AppError::Internal(format!("Storage put: {}", e)))?;
                let url = storage
                    .presigned_url(&handle.key, url_ttl)
                    .await
                    .map_err(|e| AppError::Internal(format!("Presigned URL: {}", e)))?;
                let lampiran = service
                    .add_lampiran(
                        id,
                        LampiranUpload {
                            nama: &original_name,
                            file_url: &url,
                            content_type: validated.as_ref().map(|v| v.content_type),
                            size_bytes: Some(size),
                            uploaded_by: Some(claims.user_id),
                        },
                        &scope,
                    )
                    .await?;
                lampirans.push(lampiran);
            }
            other => {
                // Unknown field — log silently and skip rather than fail
                // the whole upload (forward-compat dgn front-end yg add
                // metadata fields).
                tracing::debug!("upload_lampiran: ignored unknown field '{}'", other);
            }
        }
    }

    if surat_usulan_url.is_none() && lampirans.is_empty() {
        return Err(AppError::BadRequest(
            "Tidak ada file di-upload. Sertakan field 'surat_usulan' atau 'lampiran'.".into(),
        ));
    }

    Ok(Json(ApiResponse::success(
        UploadLampiranResponse {
            surat_usulan_file_url: surat_usulan_url,
            lampiran: lampirans,
        },
        "File berhasil di-upload".to_string(),
    )))
}

/// List lampiran pendukung utk satu penghapusan.
pub async fn list_lampiran(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<Vec<PenghapusanBmnLampiran>>>, AppError> {
    let items = service
        .list_lampiran(id, &SatkerScope::from_claims(&claims))
        .await?;
    Ok(Json(ApiResponse::success(
        items,
        "Lampiran retrieved successfully".to_string(),
    )))
}

// ─── V029 (Fase 1.9): SK Wilayah handlers ───────────────────────────────

/// RBAC: validator_wilayah only. No admin bypass — the SK Wilayah is signed on
/// behalf of the Kepala Kejaksaan Tinggi, which an IT administrator is not.
fn require_validator_wilayah(claims: &Claims) -> Result<(), AppError> {
    claims
        .require_role("validator_wilayah")
        .map_err(|_| {
            AppError::Authorization(format!(
                "Akses ditolak: role '{}' tidak diizinkan utk aksi SK Wilayah (perlu validator_wilayah)",
                claims.role
            ))
        })
}

/// Validator Wilayah generate konsep SK (jalur kewenangan WILAYAH).
/// Mewakili Kepala Kejaksaan Tinggi.
pub async fn generate_konsep_sk_wilayah(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    require_validator_wilayah(&claims)?;
    let penghapusan = service
        .generate_konsep_sk_wilayah(
            id,
            claims.user_id,
            claims.acting_role(&["validator_wilayah"]),
            &SatkerScope::from_claims(&claims),
        )
        .await?;
    Ok(Json(ApiResponse::success(
        penghapusan,
        "Konsep SK Wilayah berhasil digenerate (mewakili Kepala Kejaksaan Tinggi)".to_string(),
    )))
}

/// Validator Wilayah upload signed SK PDF jalur WILAYAH.
pub async fn upload_signed_sk_wilayah(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<UploadSignedSKRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    require_validator_wilayah(&claims)?;
    crate::shared::upload::validate_document_url("signed_sk_pdf_url", &body.signed_sk_pdf_url)?;
    let penghapusan = service
        .upload_signed_sk_wilayah(
            id,
            claims.user_id,
            claims.acting_role(&["validator_wilayah"]),
            body.signed_sk_pdf_url,
            &SatkerScope::from_claims(&claims),
        )
        .await?;
    Ok(Json(ApiResponse::success(
        penghapusan,
        "SK Wilayah yang ditandatangani Kepala Kejaksaan Tinggi berhasil diupload. Proses selesai."
            .to_string(),
    )))
}

/// Legacy: Generic workflow transition
#[derive(Debug, Deserialize, Validate)]
pub struct TransitionRequest {
    #[validate(length(min = 1))]
    pub to_state: String,
    pub catatan: Option<String>,
}

pub async fn transition_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<TransitionRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    // RBAC: legacy generic transition endpoint dipakai utk Reject di Pusat.
    // Validator Wilayah & Pusat boleh trigger; operator tidak.
    claims.require_any_role(PenghapusanBmnStatus::Rejected.actor_roles())?;
    request.validate()?;

    let penghapusan = service
        .transition(
            id,
            request.to_state,
            TransitionActor::new(
                claims.user_id,
                claims.acting_role(PenghapusanBmnStatus::Rejected.actor_roles()),
                request.catatan,
                "127.0.0.1",
            ),
            &SatkerScope::from_claims(&claims),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Workflow transition completed successfully".to_string(),
    )))
}

/// Get document for penghapusan BMN
pub async fn get_penghapusan_document(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let penghapusan = service
        .get_by_id(id, &SatkerScope::from_claims(&claims))
        .await?;

    // First check new konsep_sk_url, then fallback to legacy document_url
    let document_url = penghapusan
        .konsep_sk_url
        .or(penghapusan.document_url)
        .ok_or_else(|| AppError::NotFound("Dokumen tidak ditemukan".to_string()))?;

    Ok(Json(ApiResponse::success(
        document_url,
        "Document URL retrieved successfully".to_string(),
    )))
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;

    #[test]
    fn sanitize_filename_strips_path_separators() {
        assert_eq!(sanitize_filename("../../etc/passwd"), ".._.._etc_passwd");
        assert_eq!(sanitize_filename("foo bar.pdf"), "foo_bar.pdf");
        assert_eq!(sanitize_filename("a/b/c.txt"), "a_b_c.txt");
    }

    #[test]
    fn sanitize_filename_preserves_safe_chars() {
        assert_eq!(
            sanitize_filename("Surat_Usulan-001.pdf"),
            "Surat_Usulan-001.pdf"
        );
        assert_eq!(sanitize_filename("file.DOCX"), "file.DOCX");
    }

    #[test]
    fn sanitize_filename_handles_empty() {
        assert_eq!(sanitize_filename(""), "file");
        assert_eq!(sanitize_filename("///"), "file");
    }
}
