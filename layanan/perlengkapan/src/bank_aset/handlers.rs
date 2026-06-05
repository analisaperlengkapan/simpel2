use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    models::*,
    repository::{BankAsetRepository, ListFilter},
};
use crate::shared::error::{AppError, AppResult, bad_request};
use crate::shared::middleware::Claims;
use crate::state::AppState;
use lib_perlengkapan::response::{ApiResponse, PaginatedResponse};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
    pub kategori: Option<String>,
    pub kondisi: Option<String>,
    pub satker: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
}

fn default_page() -> i32 {
    1
}
fn default_per_page() -> i32 {
    25
}

fn validate_query(q: &ListQuery) -> AppResult<()> {
    if q.page < 1 {
        return Err(bad_request("page must be >= 1"));
    }
    if q.per_page < 1 || q.per_page > 200 {
        return Err(bad_request("per_page must be between 1 and 200"));
    }
    Ok(())
}

pub async fn list_bank_aset(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
    _claims: Claims,
) -> Result<Json<PaginatedResponse<BankAsetItem>>, AppError> {
    validate_query(&query)?;
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let filter = ListFilter {
        page: query.page,
        per_page: query.per_page,
        kategori: query.kategori,
        kondisi: query.kondisi,
        satker: query.satker,
        search: query.search,
        sort: query.sort,
    };
    let (items, total) = repo.list(filter).await?;
    Ok(Json(PaginatedResponse::new(
        items,
        total,
        query.page,
        query.per_page,
        "Bank aset retrieved successfully".to_string(),
    )))
}

pub async fn get_bank_aset_detail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<Json<ApiResponse<BankAsetDetail>>, AppError> {
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let item = repo.get(id).await?;
    let detail = BankAsetDetail {
        item,
        riwayat_pemakaian: Vec::new(),
        riwayat_penghapusan: Vec::new(),
        riwayat_kebutuhan: Vec::new(),
    };
    Ok(Json(ApiResponse::success(
        detail,
        "Detail aset retrieved successfully".to_string(),
    )))
}

#[derive(Debug, Deserialize)]
pub struct LookupQuery {
    pub nup: String,
}

/// GET /bank-aset/lookup?nup={nup}
///
/// Slim lookup used by the pemakaian-bmn form to auto-fill `kode_barang` +
/// `nama_barang` (and a few display extras) the moment the operator types
/// a NUP. Returns 404 if the NUP isn't present in `integrasi.siman_aset`.
pub async fn lookup_bank_aset(
    State(state): State<AppState>,
    Query(q): Query<LookupQuery>,
    _claims: Claims,
) -> Result<Json<ApiResponse<BankAsetLookup>>, AppError> {
    if q.nup.trim().is_empty() {
        return Err(bad_request("nup query parameter is required"));
    }
    let repo = BankAsetRepository::new(state.db_pool.clone());
    match repo.find_lookup_by_nup(q.nup.trim()).await? {
        Some(item) => Ok(Json(ApiResponse::success(
            item,
            "BMN lookup retrieved successfully".to_string(),
        ))),
        None => Err(AppError::NotFound(format!(
            "BMN dengan NUP {} tidak ditemukan",
            q.nup
        ))),
    }
}

pub async fn get_bank_aset_dashboard(
    State(state): State<AppState>,
    _claims: Claims,
) -> Result<Json<ApiResponse<BankAsetDashboard>>, AppError> {
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let data = repo.dashboard().await?;
    Ok(Json(ApiResponse::success(
        data,
        "Dashboard bank aset retrieved successfully".to_string(),
    )))
}

pub async fn get_bank_aset_sebaran(
    State(state): State<AppState>,
    _claims: Claims,
) -> Result<Json<ApiResponse<BankAsetSebaran>>, AppError> {
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let data = repo.sebaran().await?;
    Ok(Json(ApiResponse::success(
        data,
        "Sebaran bank aset retrieved successfully".to_string(),
    )))
}

pub async fn get_bank_aset_last_sync(
    State(state): State<AppState>,
    _claims: Claims,
) -> Result<Json<ApiResponse<LastSyncInfo>>, AppError> {
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let data = repo.last_sync().await?;
    Ok(Json(ApiResponse::success(
        data,
        "Last sync info retrieved successfully".to_string(),
    )))
}
