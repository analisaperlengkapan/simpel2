//! # Kebutuhan BMN HTTP Handlers
//!
//! Request handlers for BMN needs analysis REST API.
//! All endpoints require authentication via JWT middleware.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use tracing::{debug, info};
use uuid::Uuid;

use crate::models::{ApiResponse, PaginatedResponse};
use crate::shared::error::{AppError, bad_request};
use crate::shared::middleware::{Claims, ClientIp};

use super::models::*;
use super::repository::UserInfo;
use super::services::KebutuhanBmnService;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

#[derive(Debug, Deserialize)]
pub struct PengajuanQueryParams {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
    pub tahun: Option<i32>,
    pub status_kode: Option<i32>,
    pub satker_id: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BarangQueryParams {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
    pub kode_barang: Option<String>,
    pub prioritas_min: Option<i32>,
    pub search: Option<String>,
}

fn default_page() -> i32 {
    1
}
fn default_per_page() -> i32 {
    20
}

impl PaginationQuery {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.page < 1 {
            return Err(bad_request("Page must be greater than 0"));
        }
        if self.page > 100_000 {
            return Err(bad_request("Page must be less than or equal to 100,000"));
        }
        if self.per_page < 1 {
            return Err(bad_request("Per page must be greater than 0"));
        }
        if self.per_page > 1000 {
            return Err(bad_request("Per page must be less than or equal to 1000"));
        }
        Ok(())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn extract_user_info(claims: &Claims) -> UserInfo {
    UserInfo {
        nip: claims.nip.clone(),
        nama: claims.nama.clone(),
        pangkat: None,
        jabatan: claims.jabatan.clone(),
        role: Some(claims.role.clone()),
    }
}

// ============================================================================
// Dashboard Handlers
// ============================================================================

/// GET /kebutuhan-bmn/dashboard
/// Get dashboard statistics for kebutuhan BMN
pub async fn get_dashboard_stats(
    State(service): State<KebutuhanBmnService>,
    _claims: Claims,
) -> Result<Json<ApiResponse<KebutuhanBmnDashboardStats>>, AppError> {
    debug!("Getting kebutuhan BMN dashboard stats");

    let stats = service.get_dashboard_stats().await?;

    Ok(Json(ApiResponse::success(
        stats,
        "Dashboard statistics retrieved successfully".to_string(),
    )))
}

// ============================================================================
// Pengajuan Handlers
// ============================================================================

/// POST /kebutuhan-bmn/pengajuan
/// Create a new BMN needs request
pub async fn create_pengajuan(
    State(service): State<KebutuhanBmnService>,
    claims: Claims,
    Json(request): Json<CreatePengajuanRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengajuanDetailResponse>>), AppError> {
    // RBAC: hanya Validator Pusat yg menetapkan periode RKBMN (lihat plan §3.1).
    claims.require_role("validator_pusat")?;
    info!("Creating pengajuan kebutuhan BMN: {}", request.nama);

    let user_id = Some(claims.user_id);
    let response = service.create_pengajuan(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            response,
            "Pengajuan created successfully".to_string(),
        )),
    ))
}

/// GET /kebutuhan-bmn/pengajuan
/// Get paginated list of pengajuan
pub async fn get_all_pengajuan(
    State(service): State<KebutuhanBmnService>,
    Query(params): Query<PengajuanQueryParams>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<KebutuhanBmnSummary>>, AppError> {
    let pagination = PaginationQuery {
        page: params.page,
        per_page: params.per_page,
    };
    pagination.validate()?;

    let filter = if params.tahun.is_some()
        || params.status_kode.is_some()
        || params.satker_id.is_some()
        || params.search.is_some()
    {
        Some(PengajuanFilter {
            tahun: params.tahun,
            status_kode: params.status_kode,
            satker_id: params.satker_id,
            search: params.search,
        })
    } else {
        None
    };

    let (data, total) = service
        .get_all_pengajuan(params.page, params.per_page, filter)
        .await?;

    Ok(Json(PaginatedResponse::new(
        data,
        total,
        params.page,
        params.per_page,
        "Pengajuan retrieved successfully".to_string(),
    )))
}

/// GET /kebutuhan-bmn/pengajuan/:id
/// Get a single pengajuan with full details
pub async fn get_pengajuan_by_id(
    State(service): State<KebutuhanBmnService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<PengajuanDetailResponse>>, AppError> {
    let response = service.get_pengajuan_detail(id).await?;

    Ok(Json(ApiResponse::success(
        response,
        "Pengajuan retrieved successfully".to_string(),
    )))
}

/// PUT /kebutuhan-bmn/pengajuan/:id
/// Update a pengajuan
pub async fn update_pengajuan(
    State(service): State<KebutuhanBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<UpdatePengajuanRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmn>>, AppError> {
    info!("Updating pengajuan kebutuhan BMN: {}", id);

    let user_id = Some(claims.user_id);
    let pengajuan = service.update_pengajuan(id, request, user_id).await?;

    Ok(Json(ApiResponse::success(
        pengajuan,
        "Pengajuan updated successfully".to_string(),
    )))
}

/// DELETE /kebutuhan-bmn/pengajuan/:id
/// Delete a pengajuan (draft only)
pub async fn delete_pengajuan(
    State(service): State<KebutuhanBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<()>>, AppError> {
    info!(
        "Deleting pengajuan kebutuhan BMN: {} by user {}",
        id, claims.user_id
    );

    service.delete_pengajuan(id).await?;

    Ok(Json(ApiResponse::success(
        (),
        "Pengajuan deleted successfully".to_string(),
    )))
}

/// POST /kebutuhan-bmn/pengajuan/:id/transition
/// Transition pengajuan to a new workflow status
pub async fn transition_pengajuan_status(
    State(service): State<KebutuhanBmnService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Json(request): Json<WorkflowTransitionRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmn>>, AppError> {
    info!(
        "Transitioning pengajuan {} to status {}",
        id, request.target_status
    );

    let user_id = Some(claims.user_id);
    let user_info = Some(extract_user_info(&claims));

    let pengajuan = service
        .transition_pengajuan_status(id, request, user_id, user_info, ip)
        .await?;

    Ok(Json(ApiResponse::success(
        pengajuan,
        "Status updated successfully".to_string(),
    )))
}

// ============================================================================
// Satker Handlers
// ============================================================================

/// GET /kebutuhan-bmn/pengajuan/:id/satker
/// Get all satkers for a pengajuan
pub async fn get_pengajuan_satkers(
    State(service): State<KebutuhanBmnService>,
    Path(pengajuan_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengajuanKebutuhanBmnSatker>>>, AppError> {
    let satkers = service.get_pengajuan_satkers(pengajuan_id).await?;

    Ok(Json(ApiResponse::success(
        satkers,
        "Satkers retrieved successfully".to_string(),
    )))
}

/// GET /kebutuhan-bmn/pengajuan/:id/bmn-referensi
///
/// V029 (Fase 1.6): Daftar allowed-list BMN utk pengajuan ini. Dipakai
/// FE saat Operator Satker input barang sbg dropdown pilihan kode_barang
/// — operator hanya boleh input dari daftar ini (validate di service).
pub async fn list_bmn_referensi_handler(
    State(service): State<KebutuhanBmnService>,
    Path(pengajuan_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengajuanBmnReferensi>>>, AppError> {
    let items = service.list_bmn_referensi(pengajuan_id).await?;
    Ok(Json(ApiResponse::success(
        items,
        "Daftar BMN referensi (allowed-list) berhasil diambil".to_string(),
    )))
}

/// POST /kebutuhan-bmn/pengajuan/:id/satker
/// Add a satker to pengajuan
#[derive(Debug, Deserialize)]
pub struct AddSatkerRequest {
    pub satker_id: String,
    pub satker_name: Option<String>,
}

pub async fn add_satker_to_pengajuan(
    State(service): State<KebutuhanBmnService>,
    Path(pengajuan_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<AddSatkerRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengajuanKebutuhanBmnSatker>>), AppError> {
    info!(
        "Adding satker {} to pengajuan {}",
        request.satker_id, pengajuan_id
    );

    let user_id = Some(claims.user_id);
    let satker = service
        .add_satker_to_pengajuan(
            pengajuan_id,
            &request.satker_id,
            request.satker_name,
            user_id,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            satker,
            "Satker added successfully".to_string(),
        )),
    ))
}

/// GET /kebutuhan-bmn/satker/:id
/// Get satker detail with barang list
pub async fn get_satker_detail(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<SatkerWithBarangResponse>>, AppError> {
    pagination.validate()?;

    let response = service
        .get_satker_with_barang(satker_id, pagination.page, pagination.per_page)
        .await?;

    Ok(Json(ApiResponse::success(
        response,
        "Satker detail retrieved successfully".to_string(),
    )))
}

/// POST /kebutuhan-bmn/satker/:id/transition
/// Transition satker to a new workflow status
pub async fn transition_satker_status(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<WorkflowTransitionRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmnSatker>>, AppError> {
    info!(
        "Transitioning satker {} to status {}",
        satker_id, request.target_status
    );

    let user_id = Some(claims.user_id);
    let user_info = Some(extract_user_info(&claims));

    let satker = service
        .transition_satker_status(satker_id, request, user_id, user_info)
        .await?;

    Ok(Json(ApiResponse::success(
        satker,
        "Satker status updated successfully".to_string(),
    )))
}

/// GET /kebutuhan-bmn/satker/:id/aktivitas
/// Get workflow history for a satker
pub async fn get_satker_aktivitas(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengajuanKebutuhanBmnAktivitas>>>, AppError> {
    let aktivitas = service.get_satker_aktivitas(satker_id).await?;

    Ok(Json(ApiResponse::success(
        aktivitas,
        "Aktivitas retrieved successfully".to_string(),
    )))
}

// ============================================================================
// Barang Handlers
// ============================================================================

/// POST /kebutuhan-bmn/satker/:id/barang
/// Add a barang to a satker
pub async fn create_barang(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<CreateBarangRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengajuanKebutuhanBmnBarang>>), AppError> {
    info!("Creating barang for satker {}: {}", satker_id, request.nama);

    let user_id = Some(claims.user_id);
    let barang = service.create_barang(satker_id, request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            barang,
            "Barang created successfully".to_string(),
        )),
    ))
}

/// PUT /kebutuhan-bmn/barang/:id/approval
/// Update barang approval (jml_setuju)
pub async fn update_barang_approval(
    State(service): State<KebutuhanBmnService>,
    Path(barang_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<UpdateBarangApprovalRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmnBarang>>, AppError> {
    info!("Updating barang approval: {}", barang_id);

    let user_id = Some(claims.user_id);
    let barang = service
        .update_barang_approval(barang_id, request, user_id)
        .await?;

    Ok(Json(ApiResponse::success(
        barang,
        "Barang approval updated successfully".to_string(),
    )))
}

/// DELETE /kebutuhan-bmn/barang/:id
/// Delete a barang
pub async fn delete_barang(
    State(service): State<KebutuhanBmnService>,
    Path(barang_id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<()>>, AppError> {
    info!("Deleting barang: {}", barang_id);

    let user_id = Some(claims.user_id);
    service.delete_barang(barang_id, user_id).await?;

    Ok(Json(ApiResponse::success(
        (),
        "Barang deleted successfully".to_string(),
    )))
}

/// POST /kebutuhan-bmn/prioritas
/// Set priorities for multiple barang
pub async fn set_barang_prioritas(
    State(service): State<KebutuhanBmnService>,
    claims: Claims,
    Json(request): Json<SetPrioritasRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    info!(
        "Setting prioritas for {} items by user {}",
        request.items.len(),
        claims.user_id
    );

    service.set_barang_prioritas(request).await?;

    Ok(Json(ApiResponse::success(
        (),
        "Prioritas updated successfully".to_string(),
    )))
}

// ============================================================================
// Analisis Kelayakan Handlers
// ============================================================================

/// GET /kebutuhan-bmn/satker/:id/analisis
/// Get feasibility analysis for a satker
pub async fn get_analisis_kelayakan(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<AnalisisKelayakanResponse>>, AppError> {
    let response = service.get_analisis_kelayakan(satker_id).await?;

    Ok(Json(ApiResponse::success(
        response,
        "Analisis kelayakan retrieved successfully".to_string(),
    )))
}

/// GET /kebutuhan-bmn/satker/:id/laporan/preview?format=pdf
///
/// Stream Laporan Hasil Analisis Kebutuhan BMN sbg `application/pdf` dgn
/// `Content-Disposition: inline` — FE dapat me-render di `<iframe>` tanpa
/// memicu unduhan. Hanya format `pdf` yg didukung untuk preview saat ini.
pub async fn preview_laporan_analisis(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    Query(query): Query<LaporanFormatQuery>,
    _claims: Claims,
) -> Result<axum::response::Response, AppError> {
    use axum::http::header;
    use axum::response::IntoResponse;
    let format = query.format.as_deref().unwrap_or("pdf").to_ascii_lowercase();
    if format != "pdf" {
        return Err(bad_request(
            "Hanya format=pdf yg didukung utk preview inline. Gunakan endpoint download utk format lain.",
        ));
    }
    let bytes = super::pdf_laporan::generate_laporan_analisis_pdf(&service, satker_id).await?;
    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "inline; filename=\"Laporan_Analisis_Kebutuhan_BMN.pdf\"",
        ),
    ];
    Ok((headers, bytes).into_response())
}

/// GET /kebutuhan-bmn/satker/:id/laporan/download?format=pdf|docx
///
/// Stream Laporan Hasil Analisis Kebutuhan BMN sbg attachment (force
/// download). Format `pdf` saat ini didukung; `docx` ditandai sebagai
/// follow-up (memerlukan template DOCX yg masih dirancang dgn Biro Hukum).
pub async fn download_laporan_analisis(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    Query(query): Query<LaporanFormatQuery>,
    _claims: Claims,
) -> Result<axum::response::Response, AppError> {
    use axum::http::header;
    use axum::response::IntoResponse;
    let format = query.format.as_deref().unwrap_or("pdf").to_ascii_lowercase();
    match format.as_str() {
        "pdf" => {
            let bytes =
                super::pdf_laporan::generate_laporan_analisis_pdf(&service, satker_id).await?;
            let headers = [
                (header::CONTENT_TYPE, "application/pdf"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"Laporan_Analisis_Kebutuhan_BMN.pdf\"",
                ),
            ];
            Ok((headers, bytes).into_response())
        }
        "docx" => Err(bad_request(
            "Export DOCX masih dlm perancangan template (Biro Hukum). Gunakan format=pdf utk sementara.",
        )),
        _ => Err(bad_request(
            "Format tidak didukung. Gunakan format=pdf (atau format=docx setelah template selesai).",
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct LaporanFormatQuery {
    pub format: Option<String>,
}

// ============================================================================
// Satker Workflow Handlers (Validator Wilayah & Pusat)
// ============================================================================

/// POST /kebutuhan-bmn/satker/:id/submit-wilayah
/// Operator Satker submits pengajuan to Validator Wilayah
pub async fn submit_satker_to_wilayah(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<SubmitKebutuhanSatkerRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmnSatker>>, AppError> {
    claims.require_role("operator_satker")?;
    let user_info = extract_user_info(&claims);
    let satker = service
        .submit_satker_to_wilayah(satker_id, request, Some(claims.user_id), Some(user_info))
        .await?;

    Ok(Json(ApiResponse::success(
        satker,
        "Pengajuan berhasil dikirim ke Validator Wilayah".to_string(),
    )))
}

/// POST /kebutuhan-bmn/satker/:id/validator-wilayah
/// Validator Wilayah forwards to Pusat or returns to Operator
pub async fn validator_wilayah_action(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<ValidatorWilayahActionRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmnSatker>>, AppError> {
    claims.require_role("validator_wilayah")?;
    let user_info = extract_user_info(&claims);
    let satker = service
        .validator_wilayah_action(satker_id, request, Some(claims.user_id), Some(user_info))
        .await?;

    Ok(Json(ApiResponse::success(
        satker,
        "Aksi Validator Wilayah berhasil".to_string(),
    )))
}

/// POST /kebutuhan-bmn/satker/:id/keputusan-pusat
/// Validator Pusat makes final decision: approve or reject
pub async fn validator_pusat_keputusan(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<ValidatorPusatKeputusanRequest>,
) -> Result<Json<ApiResponse<PengajuanKebutuhanBmnSatker>>, AppError> {
    claims.require_role("validator_pusat")?;
    let user_info = extract_user_info(&claims);
    let satker = service
        .validator_pusat_keputusan(satker_id, request, Some(claims.user_id), Some(user_info))
        .await?;

    let message = if satker.status == KebutuhanBmnStatus::Approved {
        "Kebutuhan BMN disetujui oleh Validator Pusat"
    } else {
        "Kebutuhan BMN ditolak oleh Validator Pusat"
    };

    Ok(Json(ApiResponse::success(satker, message.to_string())))
}

// ============================================================================
// SIMAN Integration Handlers
// ============================================================================

/// Query parameters for SIMAN asset search
#[derive(Debug, Deserialize)]
pub struct SimanSearchQuery {
    pub search: String,
    pub kategori: Option<String>,
    #[serde(default = "default_siman_limit")]
    pub limit: usize,
}

fn default_siman_limit() -> usize {
    20
}

/// GET /kebutuhan-bmn/siman/search
/// Search for existing assets from SIMAN inventory
pub async fn search_siman_assets(
    State(service): State<KebutuhanBmnService>,
    Query(params): Query<SimanSearchQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<super::siman_integration::SimanAsset>>>, AppError> {
    if params.search.len() < 2 {
        return Err(bad_request("Search term must be at least 2 characters"));
    }

    let kategori_ref = params.kategori.as_deref();
    let assets = service
        .search_siman_assets(&params.search, kategori_ref, params.limit)
        .await?;

    let count = assets.len();
    Ok(Json(ApiResponse::success(
        assets,
        format!("Found {} matching assets", count),
    )))
}

/// GET /kebutuhan-bmn/siman/summary/:satker_id
/// Get asset summary from SIMAN for a specific satker
pub async fn get_siman_satker_summary(
    State(service): State<KebutuhanBmnService>,
    Path(satker_id): Path<String>,
    _claims: Claims,
) -> Result<Json<ApiResponse<super::siman_integration::SatkerAssetSummary>>, AppError> {
    let summary = service.get_satker_siman_summary(&satker_id).await?;

    Ok(Json(ApiResponse::success(
        summary,
        "SIMAN asset summary retrieved successfully".to_string(),
    )))
}

// ============================================================================
// Search Handlers
// ============================================================================

/// Query parameters for advanced search
#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    /// Search term (required)
    pub q: String,

    /// Pagination
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,

    /// Filters
    pub satker_id: Option<Uuid>,
    pub tahun_anggaran: Option<i32>,
    pub status: Option<String>, // Comma-separated list
    pub kode_barang: Option<String>,
    pub is_sbsk: Option<bool>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,

    /// Sort options
    #[serde(default = "default_sort_field")]
    pub sort_by: String,
    #[serde(default = "default_sort_direction")]
    pub sort_dir: String,
}

fn default_sort_field() -> String {
    "relevance".to_string()
}

fn default_sort_direction() -> String {
    "desc".to_string()
}

/// GET /kebutuhan-bmn/search
/// Advanced full-text search with filters and pagination
pub async fn search_kebutuhan(
    State(service): State<KebutuhanBmnService>,
    Query(params): Query<SearchQueryParams>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<KebutuhanBmnSummary>>, AppError> {
    use lib_perlengkapan::search::{
        Pagination, SearchFilters, SearchQuery, SortDirection, SortField, SortOptions,
    };

    // Validate search query
    if params.q.trim().is_empty() {
        return Err(bad_request("Search query cannot be empty"));
    }
    if params.q.len() < 2 {
        return Err(bad_request("Search query must be at least 2 characters"));
    }

    // Validate pagination
    let pagination_check = PaginationQuery {
        page: params.page,
        per_page: params.per_page,
    };
    pagination_check.validate()?;

    // Parse status list
    let status_list = params.status.as_ref().map(|s| {
        s.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    });

    // Build search query
    let search_query = SearchQuery {
        query: params.q.clone(),
        filters: SearchFilters {
            satker_id: params.satker_id,
            tahun_anggaran: params.tahun_anggaran,
            status: status_list,
            kode_barang: params.kode_barang.clone(),
            priority_level: None,
            date_from: params.date_from.clone(),
            date_to: params.date_to.clone(),
            is_sbsk: params.is_sbsk,
        },
        pagination: Pagination {
            page: params.page,
            per_page: params.per_page,
        },
        sort: SortOptions {
            field: SortField::parse_str(&params.sort_by).unwrap_or(SortField::Relevance),
            direction: SortDirection::parse_str(&params.sort_dir)
                .unwrap_or(SortDirection::Descending),
        },
    };

    // Execute search
    let results = service.search_kebutuhan(search_query).await?;

    Ok(Json(PaginatedResponse::new(
        results.results.into_iter().map(|r| r.item).collect(),
        results.total,
        results.page,
        results.per_page,
        format!("Found {} results for '{}'", results.total, params.q),
    )))
}

/// GET /kebutuhan-bmn/search/suggestions
/// Get search suggestions based on partial query
#[derive(Debug, Deserialize)]
pub struct SuggestionsQuery {
    pub q: String,
    #[serde(default = "default_suggestions_limit")]
    pub limit: i32,
}

fn default_suggestions_limit() -> i32 {
    10
}

pub async fn get_search_suggestions(
    State(service): State<KebutuhanBmnService>,
    Query(params): Query<SuggestionsQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    if params.q.len() < 2 {
        return Err(bad_request("Query must be at least 2 characters"));
    }

    let suggestions = service
        .get_search_suggestions(&params.q, params.limit)
        .await?;

    Ok(Json(ApiResponse::success(
        suggestions,
        "Suggestions retrieved successfully".to_string(),
    )))
}

// ============================================================================
// Batch Operations Handlers
// ============================================================================

/// POST /kebutuhan-bmn/batch/approve
/// Batch approve multiple kebutuhan
pub async fn batch_approve_kebutuhan(
    State(service): State<KebutuhanBmnService>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Json(request): Json<super::models::BatchApproveRequest>,
) -> Result<Json<ApiResponse<super::models::BatchOperationResponse>>, AppError> {
    use validator::Validate;

    // Validate request
    request
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    info!(
        "Batch approve requested for {} items by user {}",
        request.kebutuhan_ids.len(),
        claims.user_id
    );

    let user_id = Some(claims.user_id);
    let user_info = Some(extract_user_info(&claims));

    let response = service
        .batch_approve_kebutuhan(request, user_id, user_info, ip)
        .await?;

    Ok(Json(ApiResponse::success(
        response,
        "Batch approval completed".to_string(),
    )))
}

/// POST /kebutuhan-bmn/batch/reject
/// Batch reject multiple kebutuhan
pub async fn batch_reject_kebutuhan(
    State(service): State<KebutuhanBmnService>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Json(request): Json<super::models::BatchRejectRequest>,
) -> Result<Json<ApiResponse<super::models::BatchOperationResponse>>, AppError> {
    use validator::Validate;

    // Validate request
    request
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    info!(
        "Batch reject requested for {} items by user {}",
        request.kebutuhan_ids.len(),
        claims.user_id
    );

    let user_id = Some(claims.user_id);
    let user_info = Some(extract_user_info(&claims));

    let response = service
        .batch_reject_kebutuhan(request, user_id, user_info, ip)
        .await?;

    Ok(Json(ApiResponse::success(
        response,
        "Batch rejection completed".to_string(),
    )))
}

/// POST /kebutuhan-bmn/batch/update-status
/// Batch update status for multiple kebutuhan
pub async fn batch_update_status(
    State(service): State<KebutuhanBmnService>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Json(request): Json<super::models::BatchUpdateStatusRequest>,
) -> Result<Json<ApiResponse<super::models::BatchOperationResponse>>, AppError> {
    use validator::Validate;

    // Validate request
    request
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    info!(
        "Batch update status requested for {} items to status {} by user {}",
        request.kebutuhan_ids.len(),
        request.target_status,
        claims.user_id
    );

    let user_id = Some(claims.user_id);
    let user_info = Some(extract_user_info(&claims));

    let response = service
        .batch_update_status(request, user_id, user_info, ip)
        .await?;

    Ok(Json(ApiResponse::success(
        response,
        "Batch status update completed".to_string(),
    )))
}

// ============================================================================
// Report/Export Handlers (Placeholder)
// ============================================================================

/// GET /kebutuhan-bmn/pengajuan/:id/export
/// Export pengajuan data to PDF/Excel
pub async fn export_pengajuan(
    State(_service): State<KebutuhanBmnService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<String>>, AppError> {
    // TODO(kebutuhan-export): generate the kebutuhan BMN PDF / Excel
    // through `state.docs.preview(DocumentRequest{...})` — the
    // DocumentGenerator port is already wired into AppState. Awaiting a
    // production template id in `dokumen.document_templates`
    // (template_type = `rekapitulasi_kebutuhan`) and the env var
    // KEBUTUHAN_REKAP_TEMPLATE_ID, after which this handler shrinks to
    // a `state.docs.preview(...)` + stream-bytes-back call.
    info!("Export requested for pengajuan: {}", id);

    Ok(Json(ApiResponse::success(
        format!("Export for pengajuan {} is being generated", id),
        "Export initiated".to_string(),
    )))
}

/// GET /kebutuhan-bmn/wilayah
///
/// V029 (Fase 1.7): Daftar nama wilayah Kejaksaan Tinggi distinct dari
/// `integrasi.mysimkari_satker`. Dipakai FE saat user pilih
/// `pilihan_satker = wilayah` untuk dropdown wilayah.
pub async fn list_wilayah_kejati(
    State(service): State<KebutuhanBmnService>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let wilayah = service.list_wilayah().await?;
    Ok(Json(ApiResponse::success(
        wilayah,
        "Daftar wilayah Kejaksaan Tinggi berhasil diambil".to_string(),
    )))
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_validation() {
        let valid = PaginationQuery {
            page: 1,
            per_page: 20,
        };
        assert!(valid.validate().is_ok());

        let invalid_page = PaginationQuery {
            page: 0,
            per_page: 20,
        };
        assert!(invalid_page.validate().is_err());

        let invalid_per_page = PaginationQuery {
            page: 1,
            per_page: 0,
        };
        assert!(invalid_per_page.validate().is_err());

        let too_large = PaginationQuery {
            page: 1,
            per_page: 2000,
        };
        assert!(too_large.validate().is_err());
    }
}
