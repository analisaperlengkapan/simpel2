//! # Request Handlers
//!
//! HTTP request handlers for the Perlengkapan service

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{errors::*, middleware::Claims, models::*, services::PerlengkapanService};

// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
    pub category: Option<String>,
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

fn default_page() -> i32 {
    1
}
fn default_per_page() -> i32 {
    20
}

// Health check handler
pub async fn health_check() -> Result<Json<ApiResponse<String>>, AppError> {
    Ok(Json(ApiResponse::success(
        "Service is healthy".to_string(),
        "Health check passed".to_string(),
    )))
}

// Dashboard handlers
pub async fn get_dashboard_stats(
    State(service): State<PerlengkapanService>,
    _claims: Claims,
) -> Result<Json<ApiResponse<DashboardStats>>, AppError> {
    let stats = service.get_dashboard_stats().await?;

    Ok(Json(ApiResponse::success(
        stats,
        "Dashboard statistics retrieved successfully".to_string(),
    )))
}

// Asset handlers (Read-Only)
pub async fn get_all_assets(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Asset>>, AppError> {
    pagination.validate()?;
    let (assets, total) = service
        .get_all_assets(pagination.page, pagination.per_page, pagination.category)
        .await?;

    Ok(Json(PaginatedResponse::new(
        assets,
        total,
        pagination.page,
        pagination.per_page,
        "Assets retrieved successfully".to_string(),
    )))
}

pub async fn get_asset_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Asset>>, AppError> {
    let asset = service.get_asset_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        asset,
        "Asset retrieved successfully".to_string(),
    )))
}

// Pengadaan handlers
pub async fn get_all_pengadaan(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Pengadaan>>, AppError> {
    pagination.validate()?;
    let (pengadaan, total) = service
        .get_all_pengadaan(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        pengadaan,
        total,
        pagination.page,
        pagination.per_page,
        "Pengadaan retrieved successfully".to_string(),
    )))
}

pub async fn get_pengadaan_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Pengadaan>>, AppError> {
    let pengadaan = service.get_pengadaan_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        pengadaan,
        "Pengadaan retrieved successfully".to_string(),
    )))
}

pub async fn create_pengadaan(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreatePengadaanRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Pengadaan>>), AppError> {
    let user_id = Some(claims.user_id);
    let pengadaan = service.create_pengadaan(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            pengadaan,
            "Pengadaan created successfully".to_string(),
        )),
    ))
}

// Pengadaan Sub-Document Handlers

pub async fn create_pengadaan_hps(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
    Json(mut request): Json<CreatePengadaanHpsRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengadaanHps>>), AppError> {
    if request.pengadaan_id != pengadaan_id {
        return Err(bad_request("Path ID and Body ID mismatch"));
    }
    request.pengadaan_id = pengadaan_id;
    let hps = service.create_pengadaan_hps(request).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            hps,
            "HPS created successfully".to_string(),
        )),
    ))
}

pub async fn get_pengadaan_hps(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengadaanHps>>>, AppError> {
    let hps_list = service.get_pengadaan_hps(pengadaan_id).await?;
    Ok(Json(ApiResponse::success(
        hps_list,
        "HPS list retrieved successfully".to_string(),
    )))
}

pub async fn create_pengadaan_skppbj(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
    Json(mut request): Json<CreatePengadaanSkppbjRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengadaanSkppbj>>), AppError> {
    if request.pengadaan_id != pengadaan_id {
        return Err(bad_request("Path ID and Body ID mismatch"));
    }
    request.pengadaan_id = pengadaan_id;
    let item = service.create_pengadaan_skppbj(request).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            item,
            "SKPPBJ created successfully".to_string(),
        )),
    ))
}

pub async fn get_pengadaan_skppbj(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengadaanSkppbj>>>, AppError> {
    let items = service.get_pengadaan_skppbj(pengadaan_id).await?;
    Ok(Json(ApiResponse::success(
        items,
        "SKPPBJ list retrieved successfully".to_string(),
    )))
}

pub async fn create_pengadaan_spk(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
    Json(mut request): Json<CreatePengadaanSpkRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengadaanSpk>>), AppError> {
    if request.pengadaan_id != pengadaan_id {
        return Err(bad_request("Path ID and Body ID mismatch"));
    }
    request.pengadaan_id = pengadaan_id;
    let item = service.create_pengadaan_spk(request).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            item,
            "SPK created successfully".to_string(),
        )),
    ))
}

pub async fn get_pengadaan_spk(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengadaanSpk>>>, AppError> {
    let items = service.get_pengadaan_spk(pengadaan_id).await?;
    Ok(Json(ApiResponse::success(
        items,
        "SPK list retrieved successfully".to_string(),
    )))
}

pub async fn create_pengadaan_ringkasan(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
    Json(mut request): Json<CreatePengadaanRingkasanRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengadaanRingkasan>>), AppError> {
    if request.pengadaan_id != pengadaan_id {
        return Err(bad_request("Path ID and Body ID mismatch"));
    }
    request.pengadaan_id = pengadaan_id;
    let item = service.create_pengadaan_ringkasan(request).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            item,
            "Ringkasan created successfully".to_string(),
        )),
    ))
}

pub async fn get_pengadaan_ringkasan(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengadaanRingkasan>>>, AppError> {
    let items = service.get_pengadaan_ringkasan(pengadaan_id).await?;
    Ok(Json(ApiResponse::success(
        items,
        "Ringkasan list retrieved successfully".to_string(),
    )))
}

pub async fn create_pengadaan_kontrak(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
    Json(mut request): Json<CreatePengadaanKontrakRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengadaanKontrak>>), AppError> {
    if request.pengadaan_id != pengadaan_id {
        return Err(bad_request("Path ID and Body ID mismatch"));
    }
    request.pengadaan_id = pengadaan_id;
    let item = service.create_pengadaan_kontrak(request).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            item,
            "Kontrak created successfully".to_string(),
        )),
    ))
}

pub async fn get_pengadaan_kontrak(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengadaanKontrak>>>, AppError> {
    let items = service.get_pengadaan_kontrak(pengadaan_id).await?;
    Ok(Json(ApiResponse::success(
        items,
        "Kontrak list retrieved successfully".to_string(),
    )))
}

pub async fn create_pengadaan_bast(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
    Json(mut request): Json<CreatePengadaanBastRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengadaanBast>>), AppError> {
    if request.pengadaan_id != pengadaan_id {
        return Err(bad_request("Path ID and Body ID mismatch"));
    }
    request.pengadaan_id = pengadaan_id;
    let item = service.create_pengadaan_bast(request).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            item,
            "BAST created successfully".to_string(),
        )),
    ))
}

pub async fn get_pengadaan_bast(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengadaanBast>>>, AppError> {
    let items = service.get_pengadaan_bast(pengadaan_id).await?;
    Ok(Json(ApiResponse::success(
        items,
        "BAST list retrieved successfully".to_string(),
    )))
}

pub async fn create_pengadaan_nodis(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
    Json(mut request): Json<CreatePengadaanNodisRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengadaanNodis>>), AppError> {
    if request.pengadaan_id != pengadaan_id {
        return Err(bad_request("Path ID and Body ID mismatch"));
    }
    request.pengadaan_id = pengadaan_id;
    let item = service.create_pengadaan_nodis(request).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            item,
            "Nodis created successfully".to_string(),
        )),
    ))
}

pub async fn get_pengadaan_nodis(
    State(service): State<PerlengkapanService>,
    Path(pengadaan_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengadaanNodis>>>, AppError> {
    let items = service.get_pengadaan_nodis(pengadaan_id).await?;
    Ok(Json(ApiResponse::success(
        items,
        "Nodis list retrieved successfully".to_string(),
    )))
}

// Pengalihan handlers
pub async fn get_all_pengalihan(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Pengalihan>>, AppError> {
    pagination.validate()?;
    let (pengalihan, total) = service
        .get_all_pengalihan(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        pengalihan,
        total,
        pagination.page,
        pagination.per_page,
        "Pengalihan retrieved successfully".to_string(),
    )))
}

pub async fn create_pengalihan(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreatePengalihanRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Pengalihan>>), AppError> {
    let user_id = Some(claims.user_id);
    // Validate asset exists
    service.get_asset_by_id(request.asset_id).await?;

    let pengalihan = service.create_pengalihan(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            pengalihan,
            "Pengalihan recorded successfully".to_string(),
        )),
    ))
}

// Pemeliharaan handlers
pub async fn get_all_pemeliharaan(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Pemeliharaan>>, AppError> {
    pagination.validate()?;
    let (pemeliharaan, total) = service
        .get_all_pemeliharaan(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        pemeliharaan,
        total,
        pagination.page,
        pagination.per_page,
        "Pemeliharaan retrieved successfully".to_string(),
    )))
}

pub async fn get_pemeliharaan_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Pemeliharaan>>, AppError> {
    let pemeliharaan = service.get_pemeliharaan_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        pemeliharaan,
        "Pemeliharaan retrieved successfully".to_string(),
    )))
}

pub async fn create_pemeliharaan(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreatePemeliharaanRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Pemeliharaan>>), AppError> {
    let user_id = Some(claims.user_id);
    // Validate asset exists
    service.get_asset_by_id(request.asset_id).await?;

    let pemeliharaan = service.create_pemeliharaan(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            pemeliharaan,
            "Pemeliharaan recorded successfully".to_string(),
        )),
    ))
}

// Penghapusan handlers
pub async fn get_all_penghapusan(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Penghapusan>>, AppError> {
    pagination.validate()?;
    let (penghapusan, total) = service
        .get_all_penghapusan(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        penghapusan,
        total,
        pagination.page,
        pagination.per_page,
        "Penghapusan retrieved successfully".to_string(),
    )))
}

pub async fn create_penghapusan(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreatePenghapusanRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Penghapusan>>), AppError> {
    let user_id = Some(claims.user_id);
    // Validate asset exists
    service.get_asset_by_id(request.asset_id).await?;

    let penghapusan = service.create_penghapusan(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            penghapusan,
            "Penghapusan recorded successfully".to_string(),
        )),
    ))
}

// Mutasi handlers
pub async fn get_all_mutasi(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Mutasi>>, AppError> {
    pagination.validate()?;
    let (mutasi, total) = service
        .get_all_mutasi(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        mutasi,
        total,
        pagination.page,
        pagination.per_page,
        "Mutasi retrieved successfully".to_string(),
    )))
}

pub async fn get_mutasi_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Mutasi>>, AppError> {
    let mutasi = service.get_mutasi_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        mutasi,
        "Mutasi retrieved successfully".to_string(),
    )))
}

pub async fn create_mutasi(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreateMutasiRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Mutasi>>), AppError> {
    let user_id = Some(claims.user_id);
    // Validate asset exists
    service.get_asset_by_id(request.asset_id).await?;

    let mutasi = service.create_mutasi(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            mutasi,
            "Mutasi recorded successfully".to_string(),
        )),
    ))
}

// Hibah handlers
pub async fn get_all_hibah(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Hibah>>, AppError> {
    pagination.validate()?;
    let (hibah, total) = service
        .get_all_hibah(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        hibah,
        total,
        pagination.page,
        pagination.per_page,
        "Hibah retrieved successfully".to_string(),
    )))
}

pub async fn get_hibah_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Hibah>>, AppError> {
    let hibah = service.get_hibah_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        hibah,
        "Hibah retrieved successfully".to_string(),
    )))
}

pub async fn create_hibah(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreateHibahRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Hibah>>), AppError> {
    let user_id = Some(claims.user_id);
    // Validate asset exists
    service.get_asset_by_id(request.asset_id).await?;

    let hibah = service.create_hibah(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            hibah,
            "Hibah recorded successfully".to_string(),
        )),
    ))
}

// Pemakaian handlers
pub async fn get_all_pemakaian(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<Pemakaian>>, AppError> {
    pagination.validate()?;
    let (pemakaian, total) = service
        .get_all_pemakaian(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        pemakaian,
        total,
        pagination.page,
        pagination.per_page,
        "Pemakaian retrieved successfully".to_string(),
    )))
}

pub async fn get_pemakaian_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Pemakaian>>, AppError> {
    let pemakaian = service.get_pemakaian_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        pemakaian,
        "Pemakaian retrieved successfully".to_string(),
    )))
}

pub async fn get_penghapusan_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Penghapusan>>, AppError> {
    let penghapusan = service.get_penghapusan_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        penghapusan,
        "Penghapusan retrieved successfully".to_string(),
    )))
}

pub async fn create_pemakaian(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreatePemakaianRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Pemakaian>>), AppError> {
    let user_id = Some(claims.user_id);
    // Validate asset exists
    service.get_asset_by_id(request.asset_id).await?;

    let pemakaian = service.create_pemakaian(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            pemakaian,
            "Pemakaian recorded successfully".to_string(),
        )),
    ))
}

// Analisis Kebutuhan handlers
pub async fn get_all_analisis(
    State(service): State<PerlengkapanService>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<AnalisisKebutuhan>>, AppError> {
    pagination.validate()?;
    let (analisis, total) = service
        .get_all_analisis(pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        analisis,
        total,
        pagination.page,
        pagination.per_page,
        "Analisis kebutuhan retrieved successfully".to_string(),
    )))
}

pub async fn get_pengalihan_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Pengalihan>>, AppError> {
    let pengalihan = service.get_pengalihan_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        pengalihan,
        "Pengalihan retrieved successfully".to_string(),
    )))
}

pub async fn get_analisis_by_id(
    State(service): State<PerlengkapanService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<AnalisisKebutuhan>>, AppError> {
    let analisis = service.get_analisis_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        analisis,
        "Analisis kebutuhan retrieved successfully".to_string(),
    )))
}

pub async fn create_analisis(
    State(service): State<PerlengkapanService>,
    claims: Claims,
    Json(request): Json<CreateAnalisisRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AnalisisKebutuhan>>), AppError> {
    let user_id = Some(claims.user_id);
    let analisis = service.create_analisis(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            analisis,
            "Analisis kebutuhan created successfully".to_string(),
        )),
    ))
}

// ============================================================================
// Export Handlers
// ============================================================================

use axum::response::{IntoResponse, Response};

/// Export query parameters
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    /// Entity type to export (kebutuhan_bmn, pakaian_dinas, etc.)
    pub entity_type: String,
    /// JSON-encoded filters
    pub filters: Option<String>,
    /// Maximum number of rows to export
    pub limit: Option<u32>,
    /// Tahun anggaran filter
    pub tahun_anggaran: Option<i32>,
    /// Satker ID filter
    pub satker_id: Option<Uuid>,
    /// Status filter
    pub status: Option<String>,
}

/// Export job response
#[derive(Debug, serde::Serialize)]
pub struct ExportJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub message: String,
}

/// Export job status response
#[derive(Debug, serde::Serialize)]
pub struct ExportJobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub progress: Option<f32>,
    pub document_id: Option<Uuid>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Export to Excel handler
///
/// For small datasets (<1000 rows), returns Excel file synchronously.
/// For large datasets (>=1000 rows), queues an async export job.
pub async fn export_to_excel(
    State(service): State<PerlengkapanService>,
    Query(query): Query<ExportQuery>,
    _claims: Claims,
) -> Result<Response, AppError> {
    // Validate entity type
    validate_entity_type(&query.entity_type)?;

    // Limit export size to 50,000 rows maximum
    let limit = query.limit.unwrap_or(50000).min(50000);

    // Check if async export needed (>1000 rows)
    if limit > 1000 {
        // Queue async export job
        let job_id = service.queue_export_job(query).await?;

        return Ok((
            StatusCode::ACCEPTED,
            Json(ExportJobResponse {
                job_id,
                status: "queued".to_string(),
                message: "Export job queued. You will be notified when ready.".to_string(),
            }),
        )
            .into_response());
    }

    // Synchronous export for small datasets
    let excel_data = service.export_to_excel_sync(query).await?;

    Ok((
        StatusCode::OK,
        [
            (
                axum::http::header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                &format!(
                    "attachment; filename=\"export_{}.xlsx\"",
                    chrono::Utc::now().format("%Y%m%d_%H%M%S")
                ),
            ),
        ],
        excel_data,
    )
        .into_response())
}

/// Get export job status
pub async fn get_export_job_status(
    State(service): State<PerlengkapanService>,
    Path(job_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<ExportJobStatusResponse>>, AppError> {
    let status = service.get_export_job_status(job_id).await?;

    Ok(Json(ApiResponse::success(
        status,
        "Export job status retrieved successfully".to_string(),
    )))
}

/// Download completed export job
pub async fn download_export_job(
    State(service): State<PerlengkapanService>,
    Path(job_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Response, AppError> {
    let (filename, data) = service.download_export_job(job_id).await?;

    Ok((
        StatusCode::OK,
        [
            (
                axum::http::header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        data,
    )
        .into_response())
}

/// Validate entity type
fn validate_entity_type(entity_type: &str) -> Result<(), AppError> {
    match entity_type {
        "kebutuhan_bmn" | "pakaian_dinas" | "roadmap_sarpras" | "riwayat_pemenuhan" => Ok(()),
        _ => Err(bad_request(&format!(
            "Invalid entity type: {}. Supported types: kebutuhan_bmn, pakaian_dinas, roadmap_sarpras, riwayat_pemenuhan",
            entity_type
        ))),
    }
}
