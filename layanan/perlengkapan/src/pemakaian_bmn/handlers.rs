//! # Pemakaian BMN HTTP Handlers
//!
//! Request handlers for BMN usage permit REST API.
//! All endpoints require authentication via JWT middleware.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tracing::info;
use uuid::Uuid;

use crate::models::ApiResponse;
use crate::shared::error::AppError;
use crate::shared::middleware::{Claims, ClientIp};

use super::models::*;
use super::services::PemakaianBmnService;

/// GET /pemakaian-bmn/:id
/// Get a single permit with full details
pub async fn get_permit_by_id(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<IzinPemakaianDetailResponse>>, AppError> {
    let response = service.get_permit_detail(id).await?;

    Ok(Json(ApiResponse::success(
        response,
        "Permit retrieved successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn
/// Create a new permit request
pub async fn create_permit(
    State(service): State<PemakaianBmnService>,
    claims: Claims,
    Json(request): Json<CreateIzinPemakaianRequest>,
) -> Result<(StatusCode, Json<ApiResponse<IzinPemakaianBmn>>), AppError> {
    info!("Creating new permit for BMN {}", request.bmn_nup);

    let permit = service
        .create_permit(request, claims.user_id, claims.username.clone())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            permit,
            "Permit created successfully".to_string(),
        )),
    ))
}

/// PUT /pemakaian-bmn/:id
/// Update a permit (only in DRAFT status)
pub async fn update_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<UpdateIzinPemakaianRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    info!("Updating permit {}", id);

    let permit = service
        .update_permit(id, request, claims.user_id, claims.username.clone())
        .await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Permit updated successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn
/// List permits with pagination and filters
pub async fn list_permits(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<ListPermitsQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<PaginatedPermitsResponse>>, AppError> {
    let response = service.list_permits(query).await?;

    Ok(Json(ApiResponse::success(
        response,
        "Permits retrieved successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/transition
/// Transition permit to a new workflow status
pub async fn transition_permit_status(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Json(request): Json<WorkflowTransitionRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    info!(
        "Transitioning permit {} to status {}",
        id, request.target_status
    );

    let permit = service
        .transition_permit_status(id, request, claims.user_id, ip)
        .await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Status updated successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/activate
/// Activate a permit (generate permit number and set to ACTIVE)
pub async fn activate_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    info!("Activating permit {}", id);

    let permit = service.activate_permit(id, claims.user_id).await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Permit activated successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/:id/document
/// Download permit document
///
/// Requirements: REQ-P006, REQ-D005
pub async fn get_permit_document(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<axum::response::Redirect, AppError> {
    info!("Fetching document for permit {}", id);

    let permit = service.get_permit_detail(id).await?;

    // Creator or cross-satker role (pusat/admin) may access; everyone else is
    // rejected until authenc exposes per-user satker codes (commit 20 lands a
    // richer ValidateTokenResponse).
    if permit.izin.created_by != claims.user_id && !claims.is_cross_satker_role() {
        return Err(AppError::Authorization(
            "Anda tidak memiliki akses ke dokumen izin ini".to_string(),
        ));
    }

    // Redirect to the DOCX konsep (editable). Use `/konsep-surat.pdf` to grab
    // the PDF variant directly. `document_url` is the post-signature artifact
    // and is the right pick once the user has uploaded the signed PDF.
    let document_url = permit
        .izin
        .konsep_surat_url
        .or(permit.izin.signed_pdf_url)
        .or(permit.izin.document_url)
        .ok_or_else(|| AppError::NotFound("Document not found for this permit".to_string()))?;

    // Redirect to document URL
    Ok(axum::response::Redirect::temporary(&document_url))
}

/// POST /pemakaian-bmn/:id/generate-konsep-surat
/// Generate konsep surat izin pemakaian BMN — produces BOTH the editable
/// DOCX and the final PDF side-by-side.
pub async fn generate_konsep_surat(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<crate::models::ApiResponse<IzinPemakaianBmn>>, AppError> {
    let permit = service.generate_konsep_surat(id, claims.user_id).await?;

    Ok(Json(crate::models::ApiResponse::success(
        permit,
        "Konsep surat izin pemakaian BMN berhasil digenerate (DOCX + PDF)".to_string(),
    )))
}

/// GET /pemakaian-bmn/:id/konsep-surat.docx
/// GET /pemakaian-bmn/:id/konsep-surat.pdf
///
/// Stream the on-disk konsep surat artifact. `format` is "docx" or "pdf".
/// Path is read from the permit row written by `generate_konsep_surat`.
pub async fn serve_konsep_surat(
    State(service): State<PemakaianBmnService>,
    Path((id, format)): Path<(Uuid, String)>,
    _claims: Claims,
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
        .konsep_surat_path(id, format)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Konsep surat ({}) belum digenerate untuk permit {}",
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
    let filename = format!("konsep-surat-{}.{}", id, format);

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

/// GET /pemakaian-bmn/:id/sk-izin.pdf
///
/// Fase 1.10: SK Izin Pemakaian BMN format 2-halaman dgn struktur:
/// - Hal 1: info pegawai (NIP, nama, pangkat, jabatan, satker, foto)
/// - Hal 2: daftar BMN (kode, nama, NUP, merk, tipe, mulai, berakhir)
///
/// Berbeda dgn `/konsep-surat.pdf` yg generic — endpoint ini struktur
/// spesifik sesuai spesifikasi stakeholder (plan §5.1). Stream PDF
/// inline (Content-Disposition: inline) sehingga FE dapat
/// menampilkannya di iframe / new tab.
pub async fn serve_sk_izin_pdf(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<axum::response::Response, AppError> {
    use axum::http::header;
    use axum::response::IntoResponse;
    let bytes = super::sk_izin_pdf::generate_sk_izin_pdf(&service, id).await?;
    let filename = format!("SK-Izin-Pemakaian-BMN-{}.pdf", id);
    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            &format!("inline; filename=\"{}\"", filename),
        ),
    ];
    Ok((headers, bytes).into_response())
}

/// POST /pemakaian-bmn/:id/upload-signed-pdf
/// Upload signed PDF izin pemakaian and mark as completed
pub async fn upload_signed_pdf(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<UploadSignedPdfRequest>,
) -> Result<Json<crate::models::ApiResponse<IzinPemakaianBmn>>, AppError> {
    let permit = service
        .upload_signed_pdf(id, body.signed_pdf_url, claims.user_id)
        .await?;

    Ok(Json(crate::models::ApiResponse::success(
        permit,
        "PDF izin pemakaian BMN yang ditandatangani berhasil diupload. Proses selesai.".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/revoke
/// Revoke a permit. Hanya Approver Satker yg boleh; Admin secara eksplisit
/// DIBLOKIR (stakeholder mandate) lewat `enforce_no_admin_revoke`.
pub async fn revoke_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<RevokePermitRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{enforce_no_admin_revoke, PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    // Admin tidak boleh — guard ini di-cek SEBELUM policy.authorize() agar
    // admin bypass di policy.authorize() tidak overwrite stakeholder mandate.
    enforce_no_admin_revoke(&claims)?;
    let permit_now = service.get_permit_detail(id).await?.izin;
    PemakaianBmnPolicy.authorize(
        &claims,
        PemakaianBmnAction::Revoke,
        Some(permit_now.status.as_str()),
    )?;
    info!("Revoking permit {}", id);

    let permit = service
        .revoke_permit(id, request, claims.user_id, claims.username.clone())
        .await?;

    Ok(Json(ApiResponse::success(
        permit,
        "Permit revoked successfully".to_string(),
    )))
}

/// POST /pemakaian-bmn/:id/renew
/// Renew a permit (create new permit based on existing one)
pub async fn renew_permit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<RenewPermitRequest>,
) -> Result<(StatusCode, Json<ApiResponse<IzinPemakaianBmn>>), AppError> {
    info!("Renewing permit {}", id);

    let permit = service
        .renew_permit(id, request, claims.user_id, claims.username.clone())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            permit,
            "Permit renewed successfully".to_string(),
        )),
    ))
}

/// GET /pemakaian-bmn/bmn/:bmn_nup/availability
/// Check if a BMN is available for new permit
pub async fn check_bmn_availability(
    State(service): State<PemakaianBmnService>,
    Path(bmn_nup): Path<String>,
    _claims: Claims,
) -> Result<Json<ApiResponse<BmnAvailabilityResponse>>, AppError> {
    let response = service.check_bmn_availability(&bmn_nup).await?;

    let message = if response.is_available {
        "BMN is available".to_string()
    } else {
        "BMN is currently in use".to_string()
    };

    Ok(Json(ApiResponse::success(response, message)))
}

// ─── Fase 1.11: cek-pegawai + cek-bmn ────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct CekPegawaiQuery {
    pub satker_id: String,
}

/// GET /pemakaian-bmn/cek-pegawai/{nip}?satker_id=...
///
/// Fase 1.11: Validate pegawai berada di satker pemohon (lookup MySIMKARI
/// cache), lalu return info pegawai + pemakaian aktif + histori.
/// 422 dgn pesan "Pegawai tidak ditemukan / tidak berada di satker
/// bersangkutan" jika mismatch.
pub async fn cek_pegawai(
    State(service): State<PemakaianBmnService>,
    Path(nip): Path<String>,
    axum::extract::Query(query): axum::extract::Query<CekPegawaiQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<CekPegawaiResponse>>, AppError> {
    let resp = service
        .cek_pegawai_in_satker(&nip, &query.satker_id)
        .await?;
    Ok(Json(ApiResponse::success(
        resp,
        "Pegawai terverifikasi".to_string(),
    )))
}

#[derive(Debug, serde::Deserialize)]
pub struct CekBmnQuery {
    pub nup: String,
    #[serde(default)]
    pub satker_id: Option<String>,
    pub tgl_mulai: chrono::NaiveDate,
    pub tgl_selesai: chrono::NaiveDate,
}

/// GET /pemakaian-bmn/cek-bmn?nup=...&tgl_mulai=YYYY-MM-DD&tgl_selesai=YYYY-MM-DD
///
/// Fase 1.11: Validate BMN existence + cek availability per periode.
/// Mendukung pemakaian berurutan (existing berakhir sebelum usulan
/// mulai → diizinkan). 422 "BMN tidak ditemukan" jika NUP tidak ada di
/// referensi SIMAN. Status response: Available | PemakaianBerurutan |
/// Overlap.
pub async fn cek_bmn(
    State(service): State<PemakaianBmnService>,
    State(pool): State<deadpool_postgres::Pool>,
    axum::extract::Query(query): axum::extract::Query<CekBmnQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<CekBmnResponse>>, AppError> {
    let resp = service
        .cek_bmn_availability_for_period(&pool, &query.nup, query.tgl_mulai, query.tgl_selesai)
        .await?;
    Ok(Json(ApiResponse::success(resp, "Cek BMN selesai".to_string())))
}

/// GET /pemakaian-bmn/bmn/:bmn_nup/history
/// Get BMN usage history
pub async fn get_bmn_usage_history(
    State(service): State<PemakaianBmnService>,
    Path(bmn_nup): Path<String>,
    _claims: Claims,
) -> Result<Json<ApiResponse<BmnUsageStats>>, AppError> {
    let stats = service.get_bmn_usage_history(&bmn_nup).await?;

    Ok(Json(ApiResponse::success(
        stats,
        "BMN usage history retrieved successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/pegawai/:pegawai_nip/history
/// Get pegawai usage history
pub async fn get_pegawai_usage_history(
    State(service): State<PemakaianBmnService>,
    Path(pegawai_nip): Path<String>,
    _claims: Claims,
) -> Result<Json<ApiResponse<PegawaiUsageStats>>, AppError> {
    let stats = service.get_pegawai_usage_history(&pegawai_nip).await?;

    Ok(Json(ApiResponse::success(
        stats,
        "Pegawai usage history retrieved successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/expiring
/// Get permits expiring soon (for notifications)
pub async fn get_expiring_permits(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<IzinPemakaianBmn>>>, AppError> {
    let days_threshold = params
        .get("days")
        .and_then(|d| d.parse::<i32>().ok())
        .unwrap_or(30);

    let permits = service.get_expiring_permits(days_threshold).await?;
    let count = permits.len();

    Ok(Json(ApiResponse::success(
        permits,
        format!(
            "Found {} permits expiring within {} days",
            count, days_threshold
        ),
    )))
}

/// POST /pemakaian-bmn/auto-expire
/// Auto-expire permits (admin/scheduler endpoint)
pub async fn auto_expire_permits(
    State(service): State<PemakaianBmnService>,
    _claims: Claims,
) -> Result<Json<ApiResponse<usize>>, AppError> {
    let count = service.auto_expire_permits().await?;

    Ok(Json(ApiResponse::success(
        count,
        format!("Auto-expired {} permits", count),
    )))
}

// ============================================================================
// Monitoring Dashboard Handlers
// ============================================================================

/// GET /pemakaian-bmn/monitoring/summary
/// Tiga kartu agregat headline: sedang dipakai / tidak dipakai / akan expired.
/// Read-only — audiens Validator Wilayah & Pusat (Fase 2.6).
pub async fn get_monitoring_summary(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<MonitoringDashboardQuery>,
    claims: Claims,
) -> Result<Json<ApiResponse<MonitoringSummaryCards>>, AppError> {
    crate::shared::policy::enforce_monitoring_read(&claims)?;
    let summary = service.get_monitoring_summary(query).await?;

    Ok(Json(ApiResponse::success(
        summary,
        "Ringkasan monitoring pemakaian BMN".to_string(),
    )))
}

/// GET /pemakaian-bmn/monitoring/active-usage
/// Get active usage monitoring dashboard
/// Requirements: REQ-P011
pub async fn get_active_usage_dashboard(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<MonitoringDashboardQuery>,
    claims: Claims,
) -> Result<Json<ApiResponse<ActiveUsageMonitoringDashboard>>, AppError> {
    crate::shared::policy::enforce_monitoring_read(&claims)?;
    let dashboard = service.get_active_usage_dashboard(query).await?;

    Ok(Json(ApiResponse::success(
        dashboard,
        "Active usage dashboard retrieved successfully".to_string(),
    )))
}

/// GET /pemakaian-bmn/monitoring/utilization-report
/// Get BMN utilization report
/// Requirements: REQ-P013
pub async fn get_bmn_utilization_report(
    State(service): State<PemakaianBmnService>,
    axum::extract::Query(query): axum::extract::Query<MonitoringDashboardQuery>,
    claims: Claims,
) -> Result<Json<ApiResponse<BmnUtilizationReport>>, AppError> {
    crate::shared::policy::enforce_monitoring_read(&claims)?;
    let report = service.get_bmn_utilization_report(query).await?;

    Ok(Json(ApiResponse::success(
        report,
        "BMN utilization report generated successfully".to_string(),
    )))
}

// ============================================================================
// V035 (Fase 1.5): Endpoints alur internal-satker 3-step.
//
// RBAC ditegakkan dgn `Claims::require_any_role`. Admin/superadmin bypass
// untuk kebutuhan recovery, bukan untuk operasi harian.
// ============================================================================

use validator::Validate as _SatkerValidate;

/// POST /pemakaian-bmn/{id}/validator-satker-action
/// Body: { "action": "forward"|"return", "expected_version": i32, "catatan": "..." }
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum ValidatorSatkerActionRequest {
    Forward(SatkerForwardRequest),
    Return(SatkerReturnRequest),
}

pub async fn validator_satker_action(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<ValidatorSatkerActionRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    let permit_now = service.get_permit_detail(id).await?.izin;
    let state = Some(permit_now.status.as_str());
    let action = match &request {
        ValidatorSatkerActionRequest::Forward(_) => PemakaianBmnAction::ValidatorSatkerForward,
        ValidatorSatkerActionRequest::Return(_) => PemakaianBmnAction::ValidatorSatkerReturn,
    };
    PemakaianBmnPolicy.authorize(&claims, action, state)?;
    let permit = match request {
        ValidatorSatkerActionRequest::Forward(req) => {
            service
                .validator_satker_forward(
                    id,
                    claims.user_id,
                    claims.username.clone(),
                    req.expected_version,
                    req.catatan,
                )
                .await?
        }
        ValidatorSatkerActionRequest::Return(req) => {
            req.validate()?;
            service
                .validator_satker_return(
                    id,
                    claims.user_id,
                    claims.username.clone(),
                    req.expected_version,
                    req.catatan,
                )
                .await?
        }
    };
    Ok(Json(ApiResponse::success(
        permit,
        "Aksi Validator Satker berhasil".to_string(),
    )))
}

/// POST /pemakaian-bmn/{id}/approver-satker-action
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum ApproverSatkerActionRequest {
    Approve(SatkerForwardRequest),
    Return(SatkerReturnRequest),
}

pub async fn approver_satker_action(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<ApproverSatkerActionRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    let permit_now = service.get_permit_detail(id).await?.izin;
    let state = Some(permit_now.status.as_str());
    let action = match &request {
        ApproverSatkerActionRequest::Approve(_) => PemakaianBmnAction::ApproverSatkerApprove,
        ApproverSatkerActionRequest::Return(_) => PemakaianBmnAction::ApproverSatkerReturn,
    };
    PemakaianBmnPolicy.authorize(&claims, action, state)?;
    let permit = match request {
        ApproverSatkerActionRequest::Approve(req) => {
            service
                .approver_satker_approve(
                    id,
                    claims.user_id,
                    claims.username.clone(),
                    req.expected_version,
                    req.catatan,
                )
                .await?
        }
        ApproverSatkerActionRequest::Return(req) => {
            req.validate()?;
            service
                .approver_satker_return(
                    id,
                    claims.user_id,
                    claims.username.clone(),
                    req.expected_version,
                    req.catatan,
                )
                .await?
        }
    };
    Ok(Json(ApiResponse::success(
        permit,
        "Aksi Approver Satker berhasil".to_string(),
    )))
}

/// POST /pemakaian-bmn/{id}/resubmit
pub async fn operator_resubmit(
    State(service): State<PemakaianBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<OperatorResubmitRequest>,
) -> Result<Json<ApiResponse<IzinPemakaianBmn>>, AppError> {
    use crate::shared::policy::{PemakaianBmnAction, PemakaianBmnPolicy, WorkflowPolicy};
    let permit_now = service.get_permit_detail(id).await?.izin;
    PemakaianBmnPolicy.authorize(
        &claims,
        PemakaianBmnAction::Resubmit,
        Some(permit_now.status.as_str()),
    )?;
    let permit = service
        .operator_resubmit(
            id,
            claims.user_id,
            claims.username.clone(),
            request.expected_version,
        )
        .await?;
    Ok(Json(ApiResponse::success(
        permit,
        "Usulan berhasil di-resubmit ke Validator Satker".to_string(),
    )))
}
