use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

use super::{
    models::*,
    repository::{AsetFilter, BankAsetRepository, ListFilter},
    scope::AsetScope,
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
    pub jenis: Option<String>,
    pub kategori: Option<String>,
    pub kondisi: Option<String>,
    pub satker: Option<String>,
    pub satker_kode: Option<String>,
    pub wilayah: Option<String>,
    pub tgl_from: Option<String>,
    pub tgl_to: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
}

/// The filter half of [`ListQuery`], for surfaces that aggregate rather than
/// paginate.
///
/// Deliberately a separate struct rather than `#[serde(flatten)]` on a wrapper:
/// flatten forces serde to deserialize every field through an untyped
/// intermediate, which turns a query integer into a string and rejects the
/// request with 400 BEFORE the handler runs — and the 200 it returns when the
/// parameter is absent is exactly what let that ship (#817).
#[derive(Debug, Deserialize)]
pub struct AsetFilterQuery {
    pub jenis: Option<String>,
    pub kategori: Option<String>,
    pub kondisi: Option<String>,
    pub satker: Option<String>,
    pub satker_kode: Option<String>,
    pub wilayah: Option<String>,
    pub tgl_from: Option<String>,
    pub tgl_to: Option<String>,
    pub search: Option<String>,
}

impl AsetFilterQuery {
    fn into_filter(self) -> AsetFilter {
        AsetFilter {
            jenis: self.jenis,
            kategori: self.kategori,
            kondisi: self.kondisi,
            satker: self.satker,
            satker_kode: self.satker_kode,
            wilayah: self.wilayah,
            tgl_from: self.tgl_from,
            tgl_to: self.tgl_to,
            search: self.search,
        }
    }
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
    claims: Claims,
) -> Result<Json<PaginatedResponse<BankAsetItem>>, AppError> {
    validate_query(&query)?;
    let scope = AsetScope::from_claims(&claims);
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let filter = ListFilter {
        page: query.page,
        per_page: query.per_page,
        sort: query.sort,
        f: AsetFilter {
            jenis: query.jenis,
            kategori: query.kategori,
            kondisi: query.kondisi,
            satker: query.satker,
            satker_kode: query.satker_kode,
            wilayah: query.wilayah,
            tgl_from: query.tgl_from,
            tgl_to: query.tgl_to,
            search: query.search,
        },
    };
    let (items, total) = repo.list(filter, &scope).await?;
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
    // `integrasi.siman_aset.id` is BIGSERIAL, not uuid — a Path<Uuid> here can
    // never match a real asset id.
    Path(id): Path<i64>,
    claims: Claims,
) -> Result<Json<ApiResponse<BankAsetDetail>>, AppError> {
    let scope = AsetScope::from_claims(&claims);
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let item = repo.get(id, &scope).await?;
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
    claims: Claims,
) -> Result<Json<ApiResponse<BankAsetLookup>>, AppError> {
    if q.nup.trim().is_empty() {
        return Err(bad_request("nup query parameter is required"));
    }
    let scope = AsetScope::from_claims(&claims);
    let repo = BankAsetRepository::new(state.db_pool.clone());
    match repo.find_lookup_by_nup(q.nup.trim(), &scope).await? {
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

/// GET /bank-aset/dashboard
///
/// Accepts the SAME filter parameters as the list, so a drill-down narrows the
/// summary and the table it sits above together. Before this the summary
/// ignored every filter and described the whole visible population while the
/// table below it showed one satker.
///
/// The filters can only narrow: [`AsetScope`] is pushed before them (see
/// `BankAsetRepository::dashboard`), so `?satker_kode=` naming a satker outside
/// the caller's tier answers with zeros rather than with that satker.
pub async fn get_bank_aset_dashboard(
    State(state): State<AppState>,
    Query(query): Query<AsetFilterQuery>,
    claims: Claims,
) -> Result<Json<ApiResponse<BankAsetDashboard>>, AppError> {
    let scope = AsetScope::from_claims(&claims);
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let data = repo.dashboard(&query.into_filter(), &scope).await?;
    Ok(Json(ApiResponse::success(
        data,
        "Dashboard bank aset retrieved successfully".to_string(),
    )))
}

pub async fn get_bank_aset_sebaran(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<ApiResponse<BankAsetSebaran>>, AppError> {
    let scope = AsetScope::from_claims(&claims);
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let data = repo.sebaran(&scope).await?;
    Ok(Json(ApiResponse::success(
        data,
        "Sebaran bank aset retrieved successfully".to_string(),
    )))
}

pub async fn get_bank_aset_last_sync(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<ApiResponse<LastSyncInfo>>, AppError> {
    let scope = AsetScope::from_claims(&claims);
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let data = repo.last_sync(&scope).await?;
    Ok(Json(ApiResponse::success(
        data,
        "Last sync info retrieved successfully".to_string(),
    )))
}

/// Distinct filter values (jenis BMN, kategori, kondisi, satker) for populating
/// the FE filter dropdowns dynamically from real data.
pub async fn get_bank_aset_filter_options(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<ApiResponse<BankAsetFilterOptions>>, AppError> {
    let scope = AsetScope::from_claims(&claims);
    let repo = BankAsetRepository::new(state.db_pool.clone());
    let data = repo.filter_options(&scope).await?;
    Ok(Json(ApiResponse::success(
        data,
        "Filter options retrieved successfully".to_string(),
    )))
}
