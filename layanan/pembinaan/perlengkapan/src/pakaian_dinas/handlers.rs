//! # Pakaian Dinas HTTP Handlers
//!
//! Request handlers for Pakaian Dinas API endpoints.
//! Follows Axum 0.8.x patterns with proper error handling.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use super::models::*;
use super::services::PakaianDinasService;
use crate::errors::*;
use crate::handlers::PaginationQuery;
use crate::middleware::Claims;
use crate::models::{ApiResponse, PaginatedResponse};

// ============ Query Parameters ============

#[derive(Debug, Deserialize)]
pub struct JenisFilterQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

#[derive(Debug, Deserialize)]
pub struct SpesifikasiFilterQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub jenis_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct SubSpesifikasiFilterQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub spesifikasi_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UkuranFilterQuery {
    pub group: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PengajuanFilterQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub tahun: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct LaporanRekapQuery {
    pub pengajuan_id: Uuid,
    pub jenis_kelamin: Option<String>,
    pub eselon: Option<String>,
    pub jenis: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LaporanDaftarQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub pengajuan_id: Uuid,
    pub satker_id: Option<Uuid>,
    pub jenis_kelamin: Option<String>,
}

// ============ Master: Jenis Pakaian Dinas ============

pub async fn get_all_jenis_pakaian(
    State(service): State<PakaianDinasService>,
    Query(query): Query<JenisFilterQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<JenisPakaianDinas>>, AppError> {
    query.pagination.validate()?;
    let (items, total) = service
        .get_all_jenis(query.pagination.page, query.pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        items,
        total,
        query.pagination.page,
        query.pagination.per_page,
        "Jenis pakaian dinas berhasil diambil".to_string(),
    )))
}

pub async fn get_jenis_pakaian_by_id(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<JenisPakaianDinas>>, AppError> {
    let item = service.get_jenis_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Jenis pakaian dinas berhasil diambil".to_string(),
    )))
}

pub async fn create_jenis_pakaian(
    State(service): State<PakaianDinasService>,
    _claims: Claims,
    Json(request): Json<CreateJenisPakaianDinasRequest>,
) -> Result<(StatusCode, Json<ApiResponse<JenisPakaianDinas>>), AppError> {
    let item = service.create_jenis(request).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            item,
            "Jenis pakaian dinas berhasil dibuat".to_string(),
        )),
    ))
}

pub async fn update_jenis_pakaian(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
    Json(request): Json<CreateJenisPakaianDinasRequest>,
) -> Result<Json<ApiResponse<JenisPakaianDinas>>, AppError> {
    let item = service.update_jenis(id, request).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Jenis pakaian dinas berhasil diperbarui".to_string(),
    )))
}

pub async fn delete_jenis_pakaian(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<()>>, AppError> {
    service.delete_jenis(id).await?;

    Ok(Json(ApiResponse::success(
        (),
        "Jenis pakaian dinas berhasil dihapus".to_string(),
    )))
}

// ============ Master: Spesifikasi ============

pub async fn get_all_spesifikasi(
    State(service): State<PakaianDinasService>,
    Query(query): Query<SpesifikasiFilterQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<SpesifikasiPakaianDinas>>, AppError> {
    query.pagination.validate()?;
    let (items, total) = service
        .get_all_spesifikasi(
            query.pagination.page,
            query.pagination.per_page,
            query.jenis_id,
        )
        .await?;

    Ok(Json(PaginatedResponse::new(
        items,
        total,
        query.pagination.page,
        query.pagination.per_page,
        "Spesifikasi pakaian dinas berhasil diambil".to_string(),
    )))
}

pub async fn get_spesifikasi_by_id(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<SpesifikasiPakaianDinas>>, AppError> {
    let item = service.get_spesifikasi_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Spesifikasi pakaian dinas berhasil diambil".to_string(),
    )))
}

pub async fn create_spesifikasi(
    State(service): State<PakaianDinasService>,
    _claims: Claims,
    Json(request): Json<CreateSpesifikasiRequest>,
) -> Result<(StatusCode, Json<ApiResponse<SpesifikasiPakaianDinas>>), AppError> {
    let item = service.create_spesifikasi(request).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            item,
            "Spesifikasi pakaian dinas berhasil dibuat".to_string(),
        )),
    ))
}

pub async fn update_spesifikasi(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
    Json(request): Json<CreateSpesifikasiRequest>,
) -> Result<Json<ApiResponse<SpesifikasiPakaianDinas>>, AppError> {
    let item = service.update_spesifikasi(id, request).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Spesifikasi pakaian dinas berhasil diperbarui".to_string(),
    )))
}

pub async fn delete_spesifikasi_handler(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<()>>, AppError> {
    service.delete_spesifikasi(id).await?;

    Ok(Json(ApiResponse::success(
        (),
        "Spesifikasi pakaian dinas berhasil dihapus".to_string(),
    )))
}

// ============ Master: SubSpesifikasi ============

pub async fn get_all_subspesifikasi(
    State(service): State<PakaianDinasService>,
    Query(query): Query<SubSpesifikasiFilterQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<SubSpesifikasiPakaianDinas>>, AppError> {
    query.pagination.validate()?;
    let (items, total) = service
        .get_all_subspesifikasi(
            query.pagination.page,
            query.pagination.per_page,
            query.spesifikasi_id,
        )
        .await?;

    Ok(Json(PaginatedResponse::new(
        items,
        total,
        query.pagination.page,
        query.pagination.per_page,
        "Sub-spesifikasi pakaian dinas berhasil diambil".to_string(),
    )))
}

pub async fn get_subspesifikasi_by_id(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<SubSpesifikasiPakaianDinas>>, AppError> {
    let item = service.get_subspesifikasi_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Sub-spesifikasi pakaian dinas berhasil diambil".to_string(),
    )))
}

pub async fn create_subspesifikasi(
    State(service): State<PakaianDinasService>,
    _claims: Claims,
    Json(request): Json<CreateSubSpesifikasiRequest>,
) -> Result<(StatusCode, Json<ApiResponse<SubSpesifikasiPakaianDinas>>), AppError> {
    let item = service.create_subspesifikasi(request).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            item,
            "Sub-spesifikasi pakaian dinas berhasil dibuat".to_string(),
        )),
    ))
}

pub async fn delete_subspesifikasi_handler(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<()>>, AppError> {
    service.delete_subspesifikasi(id).await?;

    Ok(Json(ApiResponse::success(
        (),
        "Sub-spesifikasi pakaian dinas berhasil dihapus".to_string(),
    )))
}

// ============ Master: Ukuran ============

pub async fn get_all_ukuran(
    State(service): State<PakaianDinasService>,
    Query(query): Query<UkuranFilterQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<Ukuran>>>, AppError> {
    let items = service.get_all_ukuran(query.group).await?;

    Ok(Json(ApiResponse::success(
        items,
        "Daftar ukuran berhasil diambil".to_string(),
    )))
}

// ============ Pengajuan ============

pub async fn get_all_pengajuan_pakaian(
    State(service): State<PakaianDinasService>,
    Query(query): Query<PengajuanFilterQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<PengajuanPakaianDinas>>, AppError> {
    query.pagination.validate()?;
    let (items, total) = service
        .get_all_pengajuan(
            query.pagination.page,
            query.pagination.per_page,
            query.tahun,
        )
        .await?;

    Ok(Json(PaginatedResponse::new(
        items,
        total,
        query.pagination.page,
        query.pagination.per_page,
        "Daftar pengajuan pakaian dinas berhasil diambil".to_string(),
    )))
}

pub async fn get_pengajuan_pakaian_by_id(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<PengajuanPakaianDinas>>, AppError> {
    let item = service.get_pengajuan_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Detail pengajuan pakaian dinas berhasil diambil".to_string(),
    )))
}

pub async fn create_pengajuan_pakaian(
    State(service): State<PakaianDinasService>,
    claims: Claims,
    Json(request): Json<CreatePengajuanRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PengajuanPakaianDinas>>), AppError> {
    let user_id = Some(claims.user_id);
    let item = service.create_pengajuan(request, user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            item,
            "Pengajuan pakaian dinas berhasil dibuat".to_string(),
        )),
    ))
}

pub async fn delete_pengajuan_pakaian(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<()>>, AppError> {
    service.delete_pengajuan(id).await?;

    Ok(Json(ApiResponse::success(
        (),
        "Pengajuan pakaian dinas berhasil dihapus".to_string(),
    )))
}

// ============ Pengajuan Satker ============

pub async fn get_pengajuan_satker_list(
    State(service): State<PakaianDinasService>,
    Path(pengajuan_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<PengajuanSatker>>, AppError> {
    pagination.validate()?;
    let (items, total) = service
        .get_pengajuan_satker_list(pengajuan_id, pagination.page, pagination.per_page)
        .await?;

    Ok(Json(PaginatedResponse::new(
        items,
        total,
        pagination.page,
        pagination.per_page,
        "Daftar satker pengajuan berhasil diambil".to_string(),
    )))
}

pub async fn get_pengajuan_satker_by_id(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<PengajuanSatker>>, AppError> {
    let item = service.get_pengajuan_satker_by_id(id).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Detail pengajuan satker berhasil diambil".to_string(),
    )))
}

// ============ Workflow Actions ============

pub async fn process_validator_action(
    State(service): State<PakaianDinasService>,
    claims: Claims,
    Json(request): Json<ValidatorActionRequest>,
) -> Result<Json<ApiResponse<PengajuanSatker>>, AppError> {
    let user_nip = claims.nip.as_deref().unwrap_or("unknown");
    let user_nama = claims.name.as_deref().unwrap_or("unknown");
    let user_role = &claims.role;

    let item = service
        .process_validator_action(request, user_nip, user_nama, user_role)
        .await?;

    Ok(Json(ApiResponse::success(
        item,
        "Aksi validator berhasil diproses".to_string(),
    )))
}

// ============ Personal Uniform Sizes ============

pub async fn get_personal_ukuran(
    State(service): State<PakaianDinasService>,
    claims: Claims,
) -> Result<Json<ApiResponse<Option<PegawaiPakaianDinas>>>, AppError> {
    let nip = claims
        .nip
        .as_deref()
        .ok_or_else(|| bad_request("NIP tidak ditemukan"))?;
    let item = service.get_personal_ukuran(nip).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Data ukuran pakaian berhasil diambil".to_string(),
    )))
}

pub async fn update_personal_ukuran(
    State(service): State<PakaianDinasService>,
    claims: Claims,
    Json(request): Json<UpdatePersonalUkuranRequest>,
) -> Result<Json<ApiResponse<PegawaiPakaianDinas>>, AppError> {
    let nip = claims
        .nip
        .as_deref()
        .ok_or_else(|| bad_request("NIP tidak ditemukan"))?;
    let nama = claims.name.as_deref();

    let item = service.update_personal_ukuran(request, nip, nama).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Data ukuran pakaian berhasil disimpan".to_string(),
    )))
}

// ============ MySIMKARI Integration ============

pub async fn get_pegawai_by_satker(
    State(service): State<PakaianDinasService>,
    Path(satker_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<MysimkariPegawai>>>, AppError> {
    let items = service.get_pegawai_by_satker(satker_id).await?;

    Ok(Json(ApiResponse::success(
        items,
        "Daftar pegawai satker berhasil diambil".to_string(),
    )))
}

/// Response type for pegawai with existing sizes
#[derive(serde::Serialize)]
pub struct PegawaiWithSizes {
    pub pegawai: MysimkariPegawai,
    pub existing_sizes: Option<PegawaiPakaianDinas>,
}

pub async fn get_pegawai_with_sizes(
    State(service): State<PakaianDinasService>,
    Path(satker_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PegawaiWithSizes>>>, AppError> {
    let items = service.get_pegawai_with_sizes(satker_id).await?;

    let response: Vec<PegawaiWithSizes> = items
        .into_iter()
        .map(|(pegawai, sizes)| PegawaiWithSizes {
            pegawai,
            existing_sizes: sizes,
        })
        .collect();

    Ok(Json(ApiResponse::success(
        response,
        "Daftar pegawai dengan ukuran berhasil diambil".to_string(),
    )))
}

// ============ Reports ============

pub async fn get_laporan_rekap_ukuran(
    State(service): State<PakaianDinasService>,
    Query(query): Query<LaporanRekapQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<LaporanRekapUkuran>>>, AppError> {
    let filter = LaporanFilter {
        pengajuan_id: Some(query.pengajuan_id),
        jenis_kelamin: query.jenis_kelamin,
        eselon: query.eselon,
        jenis: query.jenis,
        ..Default::default()
    };

    let items = service
        .get_laporan_rekap_ukuran(query.pengajuan_id, filter)
        .await?;

    Ok(Json(ApiResponse::success(
        items,
        "Laporan rekap ukuran berhasil diambil".to_string(),
    )))
}

pub async fn get_laporan_daftar_pegawai(
    State(service): State<PakaianDinasService>,
    Query(query): Query<LaporanDaftarQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<LaporanDaftarPegawai>>, AppError> {
    query.pagination.validate()?;

    let filter = LaporanFilter {
        pengajuan_id: Some(query.pengajuan_id),
        satker_id: query.satker_id,
        jenis_kelamin: query.jenis_kelamin,
        ..Default::default()
    };

    let (items, total) = service
        .get_laporan_daftar_pegawai(
            query.pengajuan_id,
            filter,
            query.pagination.page,
            query.pagination.per_page,
        )
        .await?;

    Ok(Json(PaginatedResponse::new(
        items,
        total,
        query.pagination.page,
        query.pagination.per_page,
        "Laporan daftar pegawai berhasil diambil".to_string(),
    )))
}

// ============ Unit Tests ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_params_default() {
        let query: PengajuanFilterQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.pagination.page, 1);
        assert_eq!(query.pagination.per_page, 20);
        assert!(query.tahun.is_none());
    }

    #[test]
    fn test_laporan_filter_query() {
        let json =
            r#"{"pengajuan_id": "550e8400-e29b-41d4-a716-446655440000", "jenis_kelamin": "L"}"#;
        let query: LaporanRekapQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.jenis_kelamin, Some("L".to_string()));
    }
}
