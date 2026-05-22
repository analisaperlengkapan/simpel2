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
//   POST   /penghapusan-bmn/:id/return-operator   - Return to Operator
//   POST   /penghapusan-bmn/:id/generate-sk       - Generate konsep SK DOCX
//   POST   /penghapusan-bmn/:id/upload-signed-sk  - Upload signed SK PDF
//   POST   /penghapusan-bmn/:id/transition        - Generic transition (legacy)
// ============================================================================

use super::models::*;
use super::services::PenghapusanBmnService;
use crate::shared::error::AppError;
use crate::shared::middleware::Claims;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Pagination query parameters
#[derive(Debug, Deserialize, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 1))]
    pub page: Option<i32>,
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<i32>,
}

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: String) -> Self {
        Self {
            success: true,
            data,
            message,
        }
    }
}

/// Paginated response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub message: String,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i32, per_page: i32, message: String) -> Self {
        Self {
            success: true,
            data,
            total,
            page,
            per_page,
            message,
        }
    }
}

/// Create penghapusan BMN
pub async fn create_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    claims: Claims,
    Json(request): Json<CreatePenghapusanBmnRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PenghapusanBmn>>), AppError> {
    request.validate()?;

    let penghapusan = service.create(request, claims.user_id).await?;

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
    _claims: Claims,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    let penghapusan = service.get_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Usulan SK Penghapusan BMN retrieved successfully".to_string(),
    )))
}

/// Get penghapusan BMN detail with allowed transitions
pub async fn get_penghapusan_bmn_detail(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<PenghapusanBmnDetailResponse>>, AppError> {
    let detail = service.get_detail(id).await?;

    Ok(Json(ApiResponse::success(
        detail,
        "Detail Usulan SK Penghapusan BMN berhasil diambil".to_string(),
    )))
}

/// List penghapusan BMN with filters
#[derive(Debug, Deserialize)]
pub struct ListPenghapusanQuery {
    pub satker_id: Option<Uuid>,
    pub status: Option<String>,
    pub metode_penghapusan: Option<String>,
    pub tahun: Option<i32>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

pub async fn list_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Query(query): Query<ListPenghapusanQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<PenghapusanBmn>>, AppError> {
    let filters = PenghapusanBmnFilters {
        satker_id: query.satker_id,
        status: query.status,
        status_kode: None,
        metode_penghapusan: query.metode_penghapusan,
        tahun: query.tahun,
    };

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let (penghapusan, total) = service.list(filters, page, per_page).await?;

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
    _claims: Claims,
    Json(request): Json<UpdatePenghapusanBmnRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    request.validate()?;

    let penghapusan = service.update(id, request).await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Usulan SK Penghapusan BMN berhasil diperbarui".to_string(),
    )))
}

/// Delete penghapusan BMN
pub async fn delete_penghapusan_bmn(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    service.delete(id).await?;

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
    let penghapusan = service
        .submit_to_wilayah(id, claims.user_id, body.catatan)
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
    if body.aksi != "forward" {
        return Err(AppError::BadRequest("Action harus 'forward'".to_string()));
    }

    let penghapusan = service
        .forward_to_pusat(id, claims.user_id, body.catatan)
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Pengajuan berhasil diteruskan ke Validator Pusat".to_string(),
    )))
}

/// Validator Wilayah returns to Operator Satker
pub async fn return_to_operator(
    State(service): State<Arc<PenghapusanBmnService>>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(body): Json<ValidatorWilayahActionRequest>,
) -> Result<Json<ApiResponse<PenghapusanBmn>>, AppError> {
    if body.aksi != "return" {
        return Err(AppError::BadRequest("Action harus 'return'".to_string()));
    }

    if body.catatan.is_none() {
        return Err(AppError::BadRequest(
            "Catatan diperlukan saat mengembalikan ke Operator".to_string(),
        ));
    }

    let penghapusan = service
        .return_to_operator(id, claims.user_id, body.catatan)
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
    match body.aksi.as_str() {
        "forward" => {
            let penghapusan = service
                .forward_to_pusat(id, claims.user_id, body.catatan)
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
                .return_to_operator(id, claims.user_id, body.catatan)
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
    let penghapusan = service.generate_konsep_sk(id, claims.user_id).await?;

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

    let path = service.konsep_sk_path(id, format).await?.ok_or_else(|| {
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
    let penghapusan = service
        .upload_signed_sk(id, claims.user_id, body.signed_sk_pdf_url)
        .await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Usulan SK Penghapusan BMN yang ditandatangani berhasil diupload. Proses selesai."
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
    request.validate()?;

    let ip_address = "127.0.0.1".to_string();

    let penghapusan = service
        .transition(
            id,
            request.to_state,
            claims.user_id,
            request.catatan,
            ip_address,
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
    _claims: Claims,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let penghapusan = service.get_by_id(id).await?;

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
