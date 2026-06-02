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
use crate::handlers::PaginationQuery;
use crate::models::{ApiResponse, PaginatedResponse};
use crate::shared::error::*;
use crate::shared::middleware::Claims;

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
    pub eselon: Option<String>,
    pub jenis: Option<String>,
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
    claims: Claims,
    Json(request): Json<CreateJenisPakaianDinasRequest>,
) -> Result<(StatusCode, Json<ApiResponse<JenisPakaianDinas>>), AppError> {
    // RBAC: master jenis pakaian hanya boleh diubah admin (plan §4.1).
    claims.require_admin()?;
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
    claims: Claims,
    Json(request): Json<CreateJenisPakaianDinasRequest>,
) -> Result<Json<ApiResponse<JenisPakaianDinas>>, AppError> {
    claims.require_admin()?;
    let item = service.update_jenis(id, request).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Jenis pakaian dinas berhasil diperbarui".to_string(),
    )))
}

pub async fn delete_jenis_pakaian(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<()>>, AppError> {
    claims.require_admin()?;
    service.delete_jenis(id).await?;

    Ok(Json(ApiResponse::success(
        (),
        "Jenis pakaian dinas berhasil dihapus".to_string(),
    )))
}

// ============ Master: Spesifikasi ============

/// Sub-resource: list spesifikasi for a given jenis_id.
/// Used by the `/pakaian-dinas/jenis/:id/spesifikasi` frontend page.
pub async fn get_spesifikasi_by_jenis(
    State(service): State<PakaianDinasService>,
    Path(jenis_id): Path<Uuid>,
    Query(query): Query<PaginationQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<SpesifikasiPakaianDinas>>, AppError> {
    query.validate()?;
    let (items, total) = service
        .get_all_spesifikasi(query.page, query.per_page, Some(jenis_id))
        .await?;

    Ok(Json(PaginatedResponse::new(
        items,
        total,
        query.page,
        query.per_page,
        "Spesifikasi jenis pakaian dinas berhasil diambil".to_string(),
    )))
}

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
    claims: Claims,
    Json(request): Json<CreateSpesifikasiRequest>,
) -> Result<(StatusCode, Json<ApiResponse<SpesifikasiPakaianDinas>>), AppError> {
    claims.require_admin()?;
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
    claims: Claims,
    Json(request): Json<CreateSpesifikasiRequest>,
) -> Result<Json<ApiResponse<SpesifikasiPakaianDinas>>, AppError> {
    claims.require_admin()?;
    let item = service.update_spesifikasi(id, request).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Spesifikasi pakaian dinas berhasil diperbarui".to_string(),
    )))
}

pub async fn delete_spesifikasi_handler(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> Result<Json<ApiResponse<()>>, AppError> {
    claims.require_admin()?;
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
    claims: Claims,
    Json(request): Json<CreateSubSpesifikasiRequest>,
) -> Result<(StatusCode, Json<ApiResponse<SubSpesifikasiPakaianDinas>>), AppError> {
    claims.require_admin()?;
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
    claims: Claims,
) -> Result<Json<ApiResponse<()>>, AppError> {
    claims.require_admin()?;
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

/// GET /pakaian-dinas/satker/{id}/aktivitas — per-satker workflow history (#40).
pub async fn get_pengajuan_satker_aktivitas(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<PengajuanSatkerAktivitas>>>, AppError> {
    let items = service.list_satker_aktivitas(id).await?;
    Ok(Json(ApiResponse::success(
        items,
        "Riwayat aktivitas satker berhasil diambil".to_string(),
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

// ============ Profile Upsert (reporting fields + sizes) ============

/// Upsert a single pegawai profile — sets sizes AND reporting fields
/// (eselon, jenis_kelamin, jenis_pegawai, with_hijab, mapped_unit_kerja).
pub async fn upsert_pegawai_profile(
    State(service): State<PakaianDinasService>,
    _claims: Claims,
    Json(request): Json<UpsertPegawaiProfileRequest>,
) -> Result<Json<ApiResponse<PegawaiPakaianDinas>>, AppError> {
    let item = service.repository.upsert_pegawai_profile(&request).await?;

    Ok(Json(ApiResponse::success(
        item,
        "Profil pegawai berhasil disimpan".to_string(),
    )))
}

/// Bulk upsert pegawai profiles — used by the wizard to submit an
/// entire satker roster in one request.
pub async fn bulk_upsert_pegawai_profiles(
    State(service): State<PakaianDinasService>,
    _claims: Claims,
    Json(requests): Json<Vec<UpsertPegawaiProfileRequest>>,
) -> Result<Json<ApiResponse<usize>>, AppError> {
    let count = service
        .repository
        .bulk_upsert_pegawai_profiles(&requests)
        .await?;

    Ok(Json(ApiResponse::success(
        count,
        format!("{} profil pegawai berhasil disimpan", count),
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

/// Roster pegawai satker + info kesegaran sinkronisasi MySIMKARI (Fase 2.4).
/// Dipakai wizard ukuran utk menampilkan `last_sync_at` + banner data basi.
pub async fn get_pegawai_roster_with_sync(
    State(service): State<PakaianDinasService>,
    Path(satker_id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<PegawaiRosterWithSync>>, AppError> {
    let roster = service.get_pegawai_roster_with_sync(satker_id).await?;

    Ok(Json(ApiResponse::success(
        roster,
        "Roster pegawai + status sinkronisasi berhasil diambil".to_string(),
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
        eselon: query.eselon,
        jenis: query.jenis,
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

// ============ Workflow Transition Handlers ============

/// Submit pengajuan for approval
pub async fn submit_pengajuan_handler(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<PengajuanPakaianDinas>>, AppError> {
    let catatan = request
        .get("catatan")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let item = service
        .submit_pengajuan(id, claims.user_id, catatan)
        .await?;

    Ok(Json(ApiResponse::success(
        item,
        "Pengajuan berhasil disubmit".to_string(),
    )))
}

/// Approve pengajuan
pub async fn approve_pengajuan_handler(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<PengajuanPakaianDinas>>, AppError> {
    let catatan = request
        .get("catatan")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let item = service
        .approve_pengajuan(id, claims.user_id, catatan)
        .await?;

    Ok(Json(ApiResponse::success(
        item,
        "Pengajuan berhasil diapprove".to_string(),
    )))
}

/// Reject pengajuan
pub async fn reject_pengajuan_handler(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    claims: Claims,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<PengajuanPakaianDinas>>, AppError> {
    let catatan = request
        .get("catatan")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let item = service
        .reject_pengajuan(id, claims.user_id, catatan)
        .await?;

    Ok(Json(ApiResponse::success(
        item,
        "Pengajuan berhasil direject".to_string(),
    )))
}

/// Download rekapitulasi document
pub async fn download_rekapitulasi_handler(
    State(service): State<PakaianDinasService>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    // Get pengajuan to check document URL
    let pengajuan = service.get_pengajuan_by_id(id).await?;

    // Check if document exists
    let document_url = pengajuan
        .aktivitas_label
        .ok_or_else(|| bad_request("Rekapitulasi belum tersedia"))?;

    Ok(Json(ApiResponse::success(
        serde_json::json!({
            "document_url": document_url,
            "pengajuan_id": id,
        }),
        "URL rekapitulasi berhasil diambil".to_string(),
    )))
}
// ============ Export / Cetak Handlers ============

#[derive(Debug, Deserialize)]
pub struct CetakQuery {
    pub jenis_laporan: String, // "rekap" or "daftar"
    pub jenis_file: String,    // "excel" or "pdf"
    pub pengajuan_id: Uuid,
    pub satker_id: Option<Uuid>,
    pub jenis_kelamin: Option<String>,
    pub eselon: Option<String>,
    pub jenis: Option<String>,
}

/// GET /pakaian-dinas/laporan/cetak
/// Export report as PDF or Excel — matching simpel_web-main LaporanController::cetak()
pub async fn cetak_laporan(
    State(service): State<PakaianDinasService>,
    Query(query): Query<CetakQuery>,
    _claims: Claims,
) -> Result<axum::response::Response, AppError> {
    use axum::http::header;
    use axum::response::IntoResponse;

    let filter = LaporanFilter {
        pengajuan_id: Some(query.pengajuan_id),
        satker_id: query.satker_id,
        jenis_kelamin: query.jenis_kelamin,
        eselon: query.eselon,
        jenis: query.jenis,
        ..Default::default()
    };

    match (query.jenis_laporan.as_str(), query.jenis_file.as_str()) {
        ("rekap", "excel") => {
            let buffer =
                super::export::generate_rekap_xlsx(&service, query.pengajuan_id, &filter).await?;
            let headers = [
                (
                    header::CONTENT_TYPE,
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"Laporan_Rekap.xlsx\"",
                ),
            ];
            Ok((headers, buffer).into_response())
        }
        ("daftar", "excel") => {
            let buffer =
                super::export::generate_daftar_xlsx(&service, query.pengajuan_id, &filter).await?;
            let headers = [
                (
                    header::CONTENT_TYPE,
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"Laporan_Daftar.xlsx\"",
                ),
            ];
            Ok((headers, buffer).into_response())
        }
        ("rekap", "pdf") => {
            let buffer =
                super::pdf_export::generate_rekap_pdf(&service, query.pengajuan_id, &filter)
                    .await?;
            let headers = [
                (header::CONTENT_TYPE, "application/pdf"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"Laporan_Rekap.pdf\"",
                ),
            ];
            Ok((headers, buffer).into_response())
        }
        ("daftar", "pdf") => {
            let buffer =
                super::pdf_export::generate_daftar_pdf(&service, query.pengajuan_id, &filter)
                    .await?;
            let headers = [
                (header::CONTENT_TYPE, "application/pdf"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"Laporan_Daftar.pdf\"",
                ),
            ];
            Ok((headers, buffer).into_response())
        }
        _ => Err(bad_request(
            "Jenis laporan atau file tidak valid. Gunakan jenis_laporan=rekap|daftar dan jenis_file=excel|pdf",
        )),
    }
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
