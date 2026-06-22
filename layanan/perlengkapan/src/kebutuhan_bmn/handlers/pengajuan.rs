use super::params::*;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use tracing::info;
use uuid::Uuid;

use crate::shared::error::AppError;
use crate::shared::middleware::{Claims, ClientIp};
use lib_perlengkapan::response::{ApiResponse, PaginatedResponse};

use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::services::KebutuhanBmnService;

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
    claims: Claims,
) -> Result<Json<PaginatedResponse<KebutuhanBmnSummary>>, AppError> {
    let scope = crate::shared::satker_scope::SatkerScope::from_claims(&claims);
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
        .get_all_pengajuan(params.page, params.per_page, filter, &scope)
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
    // Coarse-grained gate; the workflow engine enforces the per-state required role.
    claims.require_any_role(&[
        "operator_satker",
        "validator_wilayah",
        "validator_pusat",
        "admin",
    ])?;
    info!(
        "Transitioning pengajuan {} to status {}",
        id, request.target_status
    );

    let user_id = Some(claims.user_id);
    let user_info = Some(extract_user_info(&claims));

    let pengajuan = service
        .transition_pengajuan_status(id, request, user_id, user_info, claims.role.clone(), ip)
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
